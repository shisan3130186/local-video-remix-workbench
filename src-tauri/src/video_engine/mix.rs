use crate::task_runtime::{run_ffmpeg, TaskProgressContext};
use crate::temp_storage::TaskTempDirectory;
use crate::video_engine::canvas::{
    build_canvas_filter_with_dimensions, build_plain_video_filter, CanvasAspectRatio,
    CanvasBackgroundMode, CanvasFilter,
};
use crate::video_engine::output::{
    append_final_output_args, build_output_video_filters, output_canvas_dimensions,
    resolve_output_video_dimensions, OutputSettings,
};
use crate::video_engine::probe::probe_video_dimensions;
use crate::video_engine::tool_paths::ffprobe_program;
use crate::video_engine::watermark::{
    append_watermark_input_args, build_image_watermark_layer, build_text_watermark_layer,
    normalize_watermark_settings, prepare_text_watermark_file, uses_image_input, WatermarkKind,
    WatermarkSettings,
};
use crate::video_engine::watermark_removal::{
    build_watermark_removal_layer, normalize_watermark_removal_settings, WatermarkRemovalSettings,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MixVideoResult {
    output_path: String,
    input_count: usize,
    message: String,
    output_aspect_ratio: String,
    output_resolution: String,
    background_mode: String,
    applied_to_remix_export: bool,
    smooth_remix_enabled: bool,
    skipped_short_segment_count: usize,
    output_encoder: String,
    output_frame_rate: String,
    output_quality: String,
    output_video_bitrate_kbps: u32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoEffectSettings {
    vertical_mirror: bool,
    rotation: RotationMode,
    brightness: f64,
    contrast: f64,
    saturation: f64,
    scale: f64,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RotationMode {
    None,
    Clockwise90,
    Counterclockwise90,
    Rotate180,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PictureInPictureSettings {
    enabled: bool,
    overlay_file_path: Option<String>,
    position: PipPosition,
    size_ratio: f64,
    opacity: f64,
    margin: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BgmSettings {
    enabled: bool,
    audio_file_path: Option<String>,
    original_volume: f64,
    bgm_volume: f64,
    fade_in_seconds: f64,
    fade_out_seconds: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleSettings {
    enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemixSettings {
    apply_horizontal_mirror: bool,
    playback_speed: f64,
    canvas_aspect_ratio: CanvasAspectRatio,
    canvas_background_mode: CanvasBackgroundMode,
    smooth_remix_enabled: bool,
    video_effect_settings: VideoEffectSettings,
    picture_in_picture_settings: PictureInPictureSettings,
    bgm_settings: BgmSettings,
    #[serde(default)]
    watermark_settings: WatermarkSettings,
    #[serde(default)]
    watermark_removal_settings: WatermarkRemovalSettings,
    subtitle_settings: SubtitleSettings,
    #[serde(default)]
    output_settings: OutputSettings,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PipPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

pub fn concat_video_segments(
    segment_paths: Vec<String>,
    output_directory: String,
    settings: RemixSettings,
    task_context: Option<TaskProgressContext>,
) -> Result<MixVideoResult, String> {
    concat_video_segments_with_options(
        segment_paths,
        output_directory,
        settings,
        true,
        task_context,
    )
}

pub(crate) fn concat_narrated_prepared_segments(
    segment_paths: Vec<String>,
    output_directory: String,
    mut settings: RemixSettings,
    task_context: Option<TaskProgressContext>,
) -> Result<MixVideoResult, String> {
    if should_apply_speed_filter(settings.playback_speed) {
        return Err("AI配音视频暂时只支持 1.0x 速度，请把视频变速恢复为 1.0x。".to_string());
    }

    settings.bgm_settings.original_volume = 1.0;
    concat_video_segments_with_options(
        segment_paths,
        output_directory,
        settings,
        false,
        task_context,
    )
}

fn concat_video_segments_with_options(
    segment_paths: Vec<String>,
    output_directory: String,
    settings: RemixSettings,
    filter_short_smooth_segments: bool,
    task_context: Option<TaskProgressContext>,
) -> Result<MixVideoResult, String> {
    let RemixSettings {
        apply_horizontal_mirror,
        playback_speed,
        canvas_aspect_ratio,
        canvas_background_mode,
        smooth_remix_enabled,
        video_effect_settings,
        picture_in_picture_settings,
        bgm_settings,
        watermark_settings,
        watermark_removal_settings,
        subtitle_settings,
        output_settings,
    } = settings;

    if subtitle_settings.enabled {
        return Err("当前版本尚未支持字幕烧录，请先关闭字幕。".to_string());
    }

    let output_dir = Path::new(&output_directory);
    let normalized_playback_speed = normalize_playback_speed(playback_speed)?;
    let normalized_effect_settings = normalize_video_effect_settings(video_effect_settings)?;
    let normalized_pip_settings =
        normalize_picture_in_picture_settings(picture_in_picture_settings)?;
    let normalized_bgm_settings = normalize_bgm_settings(bgm_settings)?;
    let normalized_watermark_settings = normalize_watermark_settings(watermark_settings)?;
    let normalized_watermark_removal_settings =
        normalize_watermark_removal_settings(watermark_removal_settings)?;

    if normalized_watermark_removal_settings
        .as_ref()
        .is_some_and(|settings| settings.tracking_enabled)
    {
        return Err("移动水印关键帧跟踪第一版只支持“导出当前完整视频”。请先导出清理后的视频，再重新导入进行混剪。".to_string());
    }

    if segment_paths.len() < 2 {
        return Err("至少需要 2 个片段才能拼接。".to_string());
    }

    if !output_dir.is_dir() {
        return Err("请选择有效的输出目录。".to_string());
    }

    for segment_path in &segment_paths {
        if !Path::new(segment_path).is_file() {
            return Err(format!("片段文件不存在：{segment_path}"));
        }
    }

    let timestamp = current_timestamp()?;
    let temp_directory = TaskTempDirectory::create("mix")?;
    let watermark_text_file = normalized_watermark_settings
        .as_ref()
        .map(|settings| prepare_text_watermark_file(settings, temp_directory.path()))
        .transpose()?
        .flatten();
    let list_path = temp_directory
        .path()
        .join(format!("concat_list_{timestamp}.txt"));
    let output_path = output_dir.join(format!("remix_{timestamp}.mp4"));
    let smooth_context = task_context
        .as_ref()
        .map(|context| context.child(0.0, 0.45, "正在处理平滑片段"));
    let prepared_segments = if smooth_remix_enabled {
        prepare_smooth_segments(
            &segment_paths,
            temp_directory.path(),
            timestamp,
            filter_short_smooth_segments,
            smooth_context.as_ref(),
        )?
    } else {
        PreparedSegments {
            segment_paths,
            temporary_paths: Vec::new(),
            skipped_short_segment_count: 0,
        }
    };

    if prepared_segments.segment_paths.len() < 2 {
        cleanup_files(&prepared_segments.temporary_paths);
        return Err("过滤过短片段后，至少需要 2 个片段才能拼接。".to_string());
    }

    let prepared_segments_have_audio = probe_segment_info(&prepared_segments.segment_paths[0])
        .map(|info| info.has_audio)
        .unwrap_or(false);
    let output_duration_seconds = calculate_output_duration_seconds(
        &prepared_segments.segment_paths,
        normalized_playback_speed,
    )?;

    let concat_list_content = build_concat_list_content(&prepared_segments.segment_paths);
    let output_dimensions = output_canvas_dimensions(output_settings, canvas_aspect_ratio);
    let watermark_removal_frame_dimensions = normalized_watermark_removal_settings
        .as_ref()
        .map(|_| probe_video_dimensions(&prepared_segments.segment_paths[0]))
        .transpose()?
        .map(|source_dimensions| {
            let source_dimensions =
                dimensions_after_rotation(source_dimensions, normalized_effect_settings.rotation);
            resolve_output_video_dimensions(output_settings, canvas_aspect_ratio, source_dimensions)
        });

    fs::write(&list_path, concat_list_content).map_err(|error| {
        format!(
            "无法创建拼接列表文件 {}：{error}",
            list_path.to_string_lossy()
        )
    })?;

    let list_path_text = list_path
        .to_str()
        .ok_or_else(|| "拼接列表路径包含无法识别的字符。".to_string())?;
    let output_path_text = output_path
        .to_str()
        .ok_or_else(|| "拼接输出路径包含无法识别的字符。".to_string())?;

    let mut ffmpeg_args = vec![
        "-y".to_string(),
        "-f".to_string(),
        "concat".to_string(),
        "-safe".to_string(),
        "0".to_string(),
        "-i".to_string(),
        list_path_text.to_string(),
    ];

    let mut next_input_index = 1usize;
    let pip_input_index = normalized_pip_settings.as_ref().map(|pip_settings| {
        let index = next_input_index;
        next_input_index += 1;
        (index, pip_settings)
    });
    if let Some((_index, pip_settings)) = pip_input_index {
        append_pip_input_args(&mut ffmpeg_args, pip_settings)?;
    }

    let watermark_image_input_index = normalized_watermark_settings
        .as_ref()
        .filter(|settings| uses_image_input(settings))
        .map(|settings| {
            let index = next_input_index;
            next_input_index += 1;
            (index, settings)
        });
    if let Some((_index, settings)) = watermark_image_input_index {
        append_watermark_input_args(&mut ffmpeg_args, settings)?;
    }

    let bgm_input_index = next_input_index;
    if let Some(bgm_settings) = &normalized_bgm_settings {
        append_bgm_input_args(&mut ffmpeg_args, bgm_settings)?;
    }

    let mut video_filters = Vec::new();

    if apply_horizontal_mirror {
        video_filters.push("hflip".to_string());
    }

    video_filters.extend(build_video_effect_filters(normalized_effect_settings));

    if should_apply_speed_filter(normalized_playback_speed) {
        video_filters.push(format!("setpts=PTS/{normalized_playback_speed:.3}"));
    }

    video_filters.extend(build_output_video_filters(
        output_settings,
        canvas_aspect_ratio,
    ));

    let base_video_filter = build_canvas_filter_with_dimensions(
        canvas_aspect_ratio,
        canvas_background_mode,
        &video_filters,
        output_dimensions,
    )
    .or_else(|| build_plain_video_filter(&video_filters));
    let video_filter = build_composed_video_filter(
        base_video_filter,
        normalized_watermark_removal_settings.as_ref(),
        watermark_removal_frame_dimensions,
        pip_input_index,
        normalized_watermark_settings.as_ref(),
        watermark_image_input_index.map(|(index, _settings)| index),
        watermark_text_file.as_deref(),
    )?;

    if let Some(bgm_settings) = &normalized_bgm_settings {
        let audio_filter = build_bgm_audio_filter(
            bgm_settings,
            bgm_input_index,
            prepared_segments_have_audio,
            normalized_playback_speed,
            output_duration_seconds,
        );

        if let Some(video_filter) = video_filter {
            if video_filter.is_complex {
                ffmpeg_args.push("-filter_complex".to_string());
                ffmpeg_args.push(format!("{};{}", video_filter.filter, audio_filter));
                ffmpeg_args.push("-map".to_string());
                ffmpeg_args.push("[v]".to_string());
                ffmpeg_args.push("-map".to_string());
                ffmpeg_args.push("[a]".to_string());
            } else {
                ffmpeg_args.push("-vf".to_string());
                ffmpeg_args.push(video_filter.filter);
                ffmpeg_args.push("-filter_complex".to_string());
                ffmpeg_args.push(audio_filter);
                ffmpeg_args.push("-map".to_string());
                ffmpeg_args.push("0:v".to_string());
                ffmpeg_args.push("-map".to_string());
                ffmpeg_args.push("[a]".to_string());
            }
        } else {
            ffmpeg_args.push("-filter_complex".to_string());
            ffmpeg_args.push(audio_filter);
            ffmpeg_args.push("-map".to_string());
            ffmpeg_args.push("0:v".to_string());
            ffmpeg_args.push("-map".to_string());
            ffmpeg_args.push("[a]".to_string());
        }
    } else if let Some(video_filter) = video_filter {
        if video_filter.is_complex {
            ffmpeg_args.push("-filter_complex".to_string());
            ffmpeg_args.push(video_filter.filter);
            ffmpeg_args.push("-map".to_string());
            ffmpeg_args.push("[v]".to_string());
            ffmpeg_args.push("-map".to_string());
            ffmpeg_args.push("0:a?".to_string());
        } else {
            ffmpeg_args.push("-vf".to_string());
            ffmpeg_args.push(video_filter.filter);
        }
    }

    if normalized_bgm_settings.is_none() && should_apply_speed_filter(normalized_playback_speed) {
        ffmpeg_args.push("-af".to_string());
        ffmpeg_args.push(format!("atempo={normalized_playback_speed:.3}"));
    }

    if normalized_pip_settings.is_some()
        || normalized_bgm_settings.is_some()
        || normalized_watermark_settings
            .as_ref()
            .is_some_and(uses_image_input)
    {
        ffmpeg_args.push("-shortest".to_string());
    }

    let applied_output_settings =
        append_final_output_args(&mut ffmpeg_args, output_settings, canvas_aspect_ratio)?;
    ffmpeg_args.push(output_path_text.to_string());

    let final_context = task_context.as_ref().map(|context| {
        context.child(
            if smooth_remix_enabled { 0.45 } else { 0.0 },
            1.0,
            "正在合成最终视频",
        )
    });
    if let Err(error) = run_ffmpeg(
        ffmpeg_args,
        final_context.as_ref(),
        Some(output_duration_seconds),
        "片段拼接失败。",
    ) {
        let _ = fs::remove_file(&output_path);
        return Err(error);
    }

    if !output_path.is_file() {
        return Err("拼接命令已结束，但没有找到输出文件。".to_string());
    }

    Ok(MixVideoResult {
        output_path: output_path.to_string_lossy().to_string(),
        input_count: prepared_segments.segment_paths.len(),
        message: "片段拼接完成。".to_string(),
        output_aspect_ratio: canvas_aspect_ratio_label(canvas_aspect_ratio).to_string(),
        output_resolution: applied_output_settings.resolution_label,
        background_mode: canvas_background_mode_label(canvas_background_mode).to_string(),
        applied_to_remix_export: true,
        smooth_remix_enabled,
        skipped_short_segment_count: prepared_segments.skipped_short_segment_count,
        output_encoder: applied_output_settings.encoder_label,
        output_frame_rate: applied_output_settings.frame_rate_label,
        output_quality: applied_output_settings.quality_label,
        output_video_bitrate_kbps: applied_output_settings.video_bitrate_kbps,
    })
}

fn normalize_picture_in_picture_settings(
    settings: PictureInPictureSettings,
) -> Result<Option<PictureInPictureSettings>, String> {
    if !settings.enabled {
        return Ok(None);
    }

    let overlay_file_path = settings
        .overlay_file_path
        .as_deref()
        .ok_or_else(|| "请先选择画中画叠加视频或图片。".to_string())?;

    if !Path::new(overlay_file_path).is_file() {
        return Err(format!("画中画素材文件不存在：{overlay_file_path}"));
    }

    if !settings.size_ratio.is_finite() || settings.size_ratio < 0.2 || settings.size_ratio > 0.5 {
        return Err("画中画大小比例必须在 0.2 到 0.5 之间。".to_string());
    }

    if !settings.opacity.is_finite() || settings.opacity < 0.0 || settings.opacity > 1.0 {
        return Err("画中画透明度必须在 0 到 1 之间。".to_string());
    }

    if settings.margin > 240 {
        return Err("画中画边距不能超过 240。".to_string());
    }

    Ok(Some(settings))
}

fn normalize_bgm_settings(settings: BgmSettings) -> Result<Option<BgmSettings>, String> {
    if !settings.enabled {
        return Ok(None);
    }

    let audio_file_path = settings
        .audio_file_path
        .as_deref()
        .ok_or_else(|| "请先选择 BGM 音频文件。".to_string())?;

    if !Path::new(audio_file_path).is_file() {
        return Err(format!("BGM 音频文件不存在：{audio_file_path}"));
    }

    if !settings.original_volume.is_finite()
        || settings.original_volume < 0.0
        || settings.original_volume > 2.0
    {
        return Err("原视频音量必须在 0 到 2 之间。".to_string());
    }

    if !settings.bgm_volume.is_finite() || settings.bgm_volume < 0.0 || settings.bgm_volume > 2.0 {
        return Err("BGM 音量必须在 0 到 2 之间。".to_string());
    }

    if !settings.fade_in_seconds.is_finite()
        || settings.fade_in_seconds < 0.0
        || settings.fade_in_seconds > 10.0
    {
        return Err("BGM 淡入秒数必须在 0 到 10 之间。".to_string());
    }

    if !settings.fade_out_seconds.is_finite()
        || settings.fade_out_seconds < 0.0
        || settings.fade_out_seconds > 10.0
    {
        return Err("BGM 淡出秒数必须在 0 到 10 之间。".to_string());
    }

    Ok(Some(settings))
}

fn append_pip_input_args(
    ffmpeg_args: &mut Vec<String>,
    settings: &PictureInPictureSettings,
) -> Result<(), String> {
    let overlay_file_path = settings
        .overlay_file_path
        .as_deref()
        .ok_or_else(|| "请先选择画中画叠加视频或图片。".to_string())?;

    if is_image_file(overlay_file_path) {
        ffmpeg_args.push("-loop".to_string());
        ffmpeg_args.push("1".to_string());
    }

    ffmpeg_args.push("-i".to_string());
    ffmpeg_args.push(overlay_file_path.to_string());
    Ok(())
}

fn append_bgm_input_args(
    ffmpeg_args: &mut Vec<String>,
    settings: &BgmSettings,
) -> Result<(), String> {
    let audio_file_path = settings
        .audio_file_path
        .as_deref()
        .ok_or_else(|| "请先选择 BGM 音频文件。".to_string())?;

    ffmpeg_args.push("-stream_loop".to_string());
    ffmpeg_args.push("-1".to_string());
    ffmpeg_args.push("-i".to_string());
    ffmpeg_args.push(audio_file_path.to_string());
    Ok(())
}

fn build_bgm_audio_filter(
    settings: &BgmSettings,
    bgm_input_index: usize,
    has_original_audio: bool,
    playback_speed: f64,
    output_duration_seconds: f64,
) -> String {
    let bgm_filter = build_bgm_source_filter(settings, bgm_input_index, output_duration_seconds);

    if !has_original_audio {
        return format!("{bgm_filter};[bgm]anull[a]");
    }

    let original_filter = if should_apply_speed_filter(playback_speed) {
        format!(
            "[0:a]volume={:.3},atempo={playback_speed:.3}[original]",
            settings.original_volume
        )
    } else {
        format!("[0:a]volume={:.3}[original]", settings.original_volume)
    };

    format!(
        "{original_filter};{bgm_filter};[original][bgm]amix=inputs=2:duration=first:dropout_transition=0[a]"
    )
}

fn build_bgm_source_filter(
    settings: &BgmSettings,
    bgm_input_index: usize,
    output_duration_seconds: f64,
) -> String {
    let mut filters = vec![format!("volume={:.3}", settings.bgm_volume)];

    if settings.fade_in_seconds > 0.001 {
        let fade_in_duration = settings.fade_in_seconds.min(output_duration_seconds);
        filters.push(format!("afade=t=in:st=0:d={fade_in_duration:.3}"));
    }

    if settings.fade_out_seconds > 0.001 {
        let fade_out_duration = settings.fade_out_seconds.min(output_duration_seconds);
        let fade_out_start = (output_duration_seconds - fade_out_duration).max(0.0);
        filters.push(format!(
            "afade=t=out:st={fade_out_start:.3}:d={fade_out_duration:.3}"
        ));
    }

    format!("[{bgm_input_index}:a]{}[bgm]", filters.join(","))
}

fn build_composed_video_filter(
    base_filter: Option<CanvasFilter>,
    watermark_removal_settings: Option<&WatermarkRemovalSettings>,
    watermark_removal_frame_dimensions: Option<(u32, u32)>,
    pip_input: Option<(usize, &PictureInPictureSettings)>,
    watermark_settings: Option<&WatermarkSettings>,
    watermark_image_input_index: Option<usize>,
    watermark_text_file: Option<&Path>,
) -> Result<Option<CanvasFilter>, String> {
    if watermark_removal_settings.is_none() && pip_input.is_none() && watermark_settings.is_none() {
        return Ok(base_filter);
    }

    let mut filters = Vec::new();
    let mut current_label = "[layer0]".to_string();
    if let Some(base_filter) = base_filter {
        if base_filter.is_complex {
            filters.push(base_filter.filter.replace("[v]", &current_label));
        } else {
            filters.push(format!("[0:v]{}{}", base_filter.filter, current_label));
        }
    } else {
        filters.push(format!("[0:v]format=yuv420p{current_label}"));
    }

    let mut layer_number = 1usize;
    if let Some(settings) = watermark_removal_settings {
        let output_label = format!("[layer{layer_number}]");
        filters.push(build_watermark_removal_layer(
            &current_label,
            &output_label,
            settings,
            layer_number,
            watermark_removal_frame_dimensions
                .ok_or_else(|| "没有读取到原水印处理所需的画面尺寸。".to_string())?,
        ));
        current_label = output_label;
        layer_number += 1;
    }

    if let Some((input_index, settings)) = pip_input {
        let output_label = format!("[layer{layer_number}]");
        filters.push(build_pip_layer_filter(
            &current_label,
            &output_label,
            input_index,
            settings,
            layer_number,
        ));
        current_label = output_label;
        layer_number += 1;
    }

    if let Some(settings) = watermark_settings {
        let output_label = format!("[layer{layer_number}]");
        let filter = match settings.kind {
            WatermarkKind::Text => build_text_watermark_layer(
                &current_label,
                &output_label,
                settings,
                watermark_text_file.ok_or_else(|| "没有找到临时文字水印文件。".to_string())?,
            )?,
            WatermarkKind::Image => build_image_watermark_layer(
                &current_label,
                &output_label,
                watermark_image_input_index
                    .ok_or_else(|| "没有找到图片水印输入流。".to_string())?,
                settings,
                layer_number,
            ),
        };
        filters.push(filter);
        current_label = output_label;
    }

    filters.push(format!("{current_label}format=yuv420p[v]"));
    Ok(Some(CanvasFilter {
        filter: filters.join(";"),
        is_complex: true,
    }))
}

fn build_pip_layer_filter(
    input_label: &str,
    output_label: &str,
    input_index: usize,
    settings: &PictureInPictureSettings,
    layer_number: usize,
) -> String {
    let (x, y) = pip_position_expression(settings.position, settings.margin);

    format!(
        "[{input_index}:v]format=rgba,colorchannelmixer=aa={opacity:.3}[pipraw{layer_number}];\
         [pipraw{layer_number}]{input_label}scale2ref=w=main_w*{size_ratio:.3}:h=-1[pip{layer_number}][pipbase{layer_number}];\
         [pipbase{layer_number}][pip{layer_number}]overlay={x}:{y}:eof_action=pass:format=auto{output_label}",
        opacity = settings.opacity,
        size_ratio = settings.size_ratio,
    )
}

fn pip_position_expression(position: PipPosition, margin: u32) -> (String, String) {
    let margin_text = margin.to_string();

    match position {
        PipPosition::TopLeft => (margin_text.clone(), margin_text),
        PipPosition::TopRight => (format!("W-w-{margin}"), margin_text),
        PipPosition::BottomLeft => (margin_text, format!("H-h-{margin}")),
        PipPosition::BottomRight => (format!("W-w-{margin}"), format!("H-h-{margin}")),
        PipPosition::Center => ("(W-w)/2".to_string(), "(H-h)/2".to_string()),
    }
}

fn is_image_file(file_path: &str) -> bool {
    Path::new(file_path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "webp" | "bmp"
            )
        })
        .unwrap_or(false)
}

fn normalize_video_effect_settings(
    settings: VideoEffectSettings,
) -> Result<VideoEffectSettings, String> {
    if !settings.brightness.is_finite() || settings.brightness < -1.0 || settings.brightness > 1.0 {
        return Err("亮度调整范围必须在 -1.0 到 1.0 之间。".to_string());
    }

    if !settings.contrast.is_finite() || settings.contrast < 0.0 || settings.contrast > 3.0 {
        return Err("对比度调整范围必须在 0.0 到 3.0 之间。".to_string());
    }

    if !settings.saturation.is_finite() || settings.saturation < 0.0 || settings.saturation > 3.0 {
        return Err("饱和度调整范围必须在 0.0 到 3.0 之间。".to_string());
    }

    if !settings.scale.is_finite() || settings.scale < 1.0 || settings.scale > 1.2 {
        return Err("轻微缩放范围必须在 1.0 到 1.2 之间。".to_string());
    }

    Ok(settings)
}

fn build_video_effect_filters(settings: VideoEffectSettings) -> Vec<String> {
    let mut filters = Vec::new();

    if settings.vertical_mirror {
        filters.push("vflip".to_string());
    }

    match settings.rotation {
        RotationMode::None => {}
        RotationMode::Clockwise90 => filters.push("transpose=1".to_string()),
        RotationMode::Counterclockwise90 => filters.push("transpose=2".to_string()),
        RotationMode::Rotate180 => filters.push("hflip,vflip".to_string()),
    }

    if should_apply_eq_filter(settings) {
        filters.push(format!(
            "eq=brightness={:.3}:contrast={:.3}:saturation={:.3}",
            settings.brightness, settings.contrast, settings.saturation
        ));
    }

    if should_apply_scale_filter(settings.scale) {
        filters.push(format!(
            "scale=iw*{:.3}:ih*{:.3},crop=iw/{:.3}:ih/{:.3}",
            settings.scale, settings.scale, settings.scale, settings.scale
        ));
    }

    filters
}

fn dimensions_after_rotation(dimensions: (u32, u32), rotation: RotationMode) -> (u32, u32) {
    match rotation {
        RotationMode::Clockwise90 | RotationMode::Counterclockwise90 => {
            (dimensions.1, dimensions.0)
        }
        RotationMode::None | RotationMode::Rotate180 => dimensions,
    }
}

fn should_apply_eq_filter(settings: VideoEffectSettings) -> bool {
    settings.brightness.abs() > 0.001
        || (settings.contrast - 1.0).abs() > 0.001
        || (settings.saturation - 1.0).abs() > 0.001
}

fn should_apply_scale_filter(scale: f64) -> bool {
    (scale - 1.0).abs() > 0.001
}

fn calculate_output_duration_seconds(
    segment_paths: &[String],
    playback_speed: f64,
) -> Result<f64, String> {
    let mut total_duration_seconds = 0.0;

    for segment_path in segment_paths {
        total_duration_seconds += probe_segment_info(segment_path)?.duration_seconds;
    }

    Ok((total_duration_seconds / playback_speed).max(0.0))
}

struct PreparedSegments {
    segment_paths: Vec<String>,
    temporary_paths: Vec<PathBuf>,
    skipped_short_segment_count: usize,
}

struct SegmentInfo {
    duration_seconds: f64,
    has_audio: bool,
}

fn prepare_smooth_segments(
    segment_paths: &[String],
    output_dir: &Path,
    timestamp: u64,
    filter_short_segments: bool,
    task_context: Option<&TaskProgressContext>,
) -> Result<PreparedSegments, String> {
    let mut prepared_paths = Vec::new();
    let mut temporary_paths = Vec::new();
    let mut skipped_short_segment_count = 0;

    for (index, segment_path) in segment_paths.iter().enumerate() {
        let info = probe_segment_info(segment_path)?;

        if filter_short_segments && info.duration_seconds < 1.5 {
            skipped_short_segment_count += 1;
            continue;
        }

        let temp_path = output_dir.join(format!("smooth_segment_{timestamp}_{index}.mp4"));
        let segment_context = task_context.map(|context| {
            let start = index as f64 / segment_paths.len() as f64;
            let end = (index + 1) as f64 / segment_paths.len() as f64;
            context.child(
                start,
                end,
                format!("正在处理平滑片段 {}/{}", index + 1, segment_paths.len()),
            )
        });
        create_smooth_segment(segment_path, &temp_path, &info, segment_context.as_ref())?;
        prepared_paths.push(temp_path.to_string_lossy().to_string());
        temporary_paths.push(temp_path);
    }

    Ok(PreparedSegments {
        segment_paths: prepared_paths,
        temporary_paths,
        skipped_short_segment_count,
    })
}

fn probe_segment_info(segment_path: &str) -> Result<SegmentInfo, String> {
    let duration_output = Command::new(ffprobe_program())
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            segment_path,
        ])
        .output()
        .map_err(|error| format!("无法调用 ffprobe 读取片段时长：{error}"))?;

    if !duration_output.status.success() {
        let stderr = String::from_utf8_lossy(&duration_output.stderr)
            .trim()
            .to_string();
        return Err(if stderr.is_empty() {
            "读取片段时长失败。".to_string()
        } else {
            stderr
        });
    }

    let duration_text = String::from_utf8_lossy(&duration_output.stdout)
        .trim()
        .to_string();
    let duration_seconds = duration_text
        .parse::<f64>()
        .map_err(|_| format!("无法识别片段时长：{duration_text}"))?;

    let audio_output = Command::new(ffprobe_program())
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=index",
            "-of",
            "csv=p=0",
            segment_path,
        ])
        .output()
        .map_err(|error| format!("无法调用 ffprobe 读取音频信息：{error}"))?;

    Ok(SegmentInfo {
        duration_seconds,
        has_audio: audio_output.status.success() && !audio_output.stdout.is_empty(),
    })
}

fn create_smooth_segment(
    segment_path: &str,
    temp_path: &Path,
    info: &SegmentInfo,
    task_context: Option<&TaskProgressContext>,
) -> Result<(), String> {
    let fade_duration = 0.2_f64.min((info.duration_seconds / 2.0).max(0.01));
    let fade_out_start = (info.duration_seconds - fade_duration).max(0.0);
    let video_filter = format!(
        "fade=t=in:st=0:d={fade_duration},fade=t=out:st={fade_out_start:.3}:d={fade_duration},format=yuv420p"
    );
    let audio_filter = format!(
        "afade=t=in:st=0:d={fade_duration},afade=t=out:st={fade_out_start:.3}:d={fade_duration}"
    );
    let temp_path_text = temp_path
        .to_str()
        .ok_or_else(|| "平滑混剪临时文件路径包含无法识别的字符。".to_string())?;

    let mut ffmpeg_args = vec![
        "-y".to_string(),
        "-i".to_string(),
        segment_path.to_string(),
        "-vf".to_string(),
        video_filter,
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "veryfast".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
    ];

    if info.has_audio {
        ffmpeg_args.extend([
            "-af".to_string(),
            audio_filter,
            "-c:a".to_string(),
            "aac".to_string(),
        ]);
    } else {
        ffmpeg_args.push("-an".to_string());
    }

    ffmpeg_args.push(temp_path_text.to_string());

    run_ffmpeg(
        ffmpeg_args,
        task_context,
        Some(info.duration_seconds),
        "生成平滑片段失败。",
    )?;

    if !temp_path.is_file() {
        return Err("平滑片段命令已结束，但没有找到临时文件。".to_string());
    }

    Ok(())
}

fn cleanup_files(paths: &[PathBuf]) {
    for path in paths {
        let _ = fs::remove_file(path);
    }
}

fn normalize_playback_speed(playback_speed: f64) -> Result<f64, String> {
    if !playback_speed.is_finite() {
        return Err("变速倍数必须是有效数字。".to_string());
    }

    if !(0.5..=2.0).contains(&playback_speed) {
        return Err("变速倍数暂时只支持 0.5 到 2.0。".to_string());
    }

    Ok((playback_speed * 1000.0).round() / 1000.0)
}

fn should_apply_speed_filter(playback_speed: f64) -> bool {
    (playback_speed - 1.0).abs() > 0.001
}

fn canvas_aspect_ratio_label(aspect_ratio: CanvasAspectRatio) -> &'static str {
    match aspect_ratio {
        CanvasAspectRatio::Original => "原画",
        CanvasAspectRatio::Portrait916 => "9:16",
        CanvasAspectRatio::Square11 => "1:1",
        CanvasAspectRatio::Landscape169 => "16:9",
    }
}

fn canvas_background_mode_label(background_mode: CanvasBackgroundMode) -> &'static str {
    match background_mode {
        CanvasBackgroundMode::Black => "黑边背景",
        CanvasBackgroundMode::Blur => "模糊背景",
    }
}

fn current_timestamp() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("无法生成拼接文件名：{error}"))
        .map(|duration| duration.as_millis() as u64)
}

fn build_concat_list_content(segment_paths: &[String]) -> String {
    segment_paths
        .iter()
        .map(|segment_path| {
            let normalized_path = PathBuf::from(segment_path)
                .to_string_lossy()
                .replace('\\', "/");
            format!("file '{}'\n", normalized_path.replace('\'', "'\\''"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_engine::watermark::WatermarkPosition;

    #[test]
    fn composes_picture_in_picture_and_image_watermark() {
        let pip = PictureInPictureSettings {
            enabled: true,
            overlay_file_path: Some("pip.png".to_string()),
            position: PipPosition::TopLeft,
            size_ratio: 0.3,
            opacity: 0.8,
            margin: 20,
        };
        let watermark = WatermarkSettings {
            enabled: true,
            kind: WatermarkKind::Image,
            image_file_path: Some("watermark.png".to_string()),
            position: WatermarkPosition::BottomRight,
            ..WatermarkSettings::default()
        };

        let filter = build_composed_video_filter(
            None,
            None,
            None,
            Some((1, &pip)),
            Some(&watermark),
            Some(2),
            None,
        )
        .unwrap()
        .unwrap();

        assert!(filter.is_complex);
        assert!(filter.filter.contains("[1:v]format=rgba"));
        assert!(filter.filter.contains("[2:v]format=rgba"));
        assert_eq!(filter.filter.matches("overlay=").count(), 2);
        assert!(filter.filter.ends_with("format=yuv420p[v]"));
    }

    #[test]
    fn composes_text_watermark_after_base_filter() {
        let watermark = WatermarkSettings {
            enabled: true,
            text: "中文水印".to_string(),
            position: WatermarkPosition::Center,
            ..WatermarkSettings::default()
        };
        let base = CanvasFilter {
            filter: "scale=640:360".to_string(),
            is_complex: false,
        };

        let filter = build_composed_video_filter(
            Some(base),
            None,
            None,
            None,
            Some(&watermark),
            None,
            Some(Path::new(r"C:\Temp\watermark.txt")),
        )
        .unwrap()
        .unwrap();

        assert!(filter.filter.contains("[0:v]scale=640:360[layer0]"));
        assert!(filter.filter.contains("drawtext="));
        assert!(filter.filter.contains("x=(w-text_w)/2:y=(h-text_h)/2"));
    }

    #[test]
    fn processes_original_watermark_before_pip_and_new_watermark() {
        use crate::video_engine::watermark_removal::{
            WatermarkRemovalMode, WatermarkRemovalSettings,
        };

        let removal = WatermarkRemovalSettings {
            enabled: true,
            mode: WatermarkRemovalMode::Cover,
            ..WatermarkRemovalSettings::default()
        };
        let pip = PictureInPictureSettings {
            enabled: true,
            overlay_file_path: Some("pip.png".to_string()),
            position: PipPosition::TopLeft,
            size_ratio: 0.3,
            opacity: 0.8,
            margin: 20,
        };
        let watermark = WatermarkSettings {
            enabled: true,
            text: "新水印".to_string(),
            ..WatermarkSettings::default()
        };

        let filter = build_composed_video_filter(
            None,
            Some(&removal),
            Some((1920, 1080)),
            Some((1, &pip)),
            Some(&watermark),
            None,
            Some(Path::new(r"C:\Temp\watermark.txt")),
        )
        .unwrap()
        .unwrap();

        let removal_index = filter.filter.find("drawbox=").unwrap();
        let pip_index = filter.filter.find("[1:v]format=rgba").unwrap();
        let watermark_index = filter.filter.find("drawtext=").unwrap();
        assert!(removal_index < pip_index);
        assert!(pip_index < watermark_index);
    }

    #[test]
    fn swaps_removal_dimensions_for_quarter_turn_rotation() {
        assert_eq!(
            dimensions_after_rotation((1920, 1080), RotationMode::Clockwise90),
            (1080, 1920)
        );
        assert_eq!(
            dimensions_after_rotation((1920, 1080), RotationMode::Rotate180),
            (1920, 1080)
        );
    }
}
