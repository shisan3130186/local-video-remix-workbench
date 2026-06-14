use crate::video_engine::canvas::{
    build_canvas_filter, build_plain_video_filter, CanvasAspectRatio, CanvasBackgroundMode,
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

pub fn concat_video_segments(
    segment_paths: Vec<String>,
    output_directory: String,
    apply_horizontal_mirror: bool,
    playback_speed: f64,
    canvas_aspect_ratio: CanvasAspectRatio,
    canvas_background_mode: CanvasBackgroundMode,
    smooth_remix_enabled: bool,
    video_effect_settings: VideoEffectSettings,
) -> Result<MixVideoResult, String> {
    let output_dir = Path::new(&output_directory);
    let normalized_playback_speed = normalize_playback_speed(playback_speed)?;
    let normalized_effect_settings = normalize_video_effect_settings(video_effect_settings)?;

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
    let list_path = output_dir.join(format!("concat_list_{timestamp}.txt"));
    let output_path = output_dir.join(format!("remix_{timestamp}.mp4"));
    let prepared_segments = if smooth_remix_enabled {
        prepare_smooth_segments(&segment_paths, output_dir, timestamp)?
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

    let concat_list_content = build_concat_list_content(&prepared_segments.segment_paths);
    let output_dimensions = canvas_aspect_ratio.dimensions();

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

    let mut video_filters = Vec::new();

    if apply_horizontal_mirror {
        video_filters.push("hflip".to_string());
    }

    video_filters.extend(build_video_effect_filters(normalized_effect_settings));

    if should_apply_speed_filter(normalized_playback_speed) {
        video_filters.push(format!("setpts=PTS/{normalized_playback_speed:.3}"));
    }

    let video_filter =
        build_canvas_filter(canvas_aspect_ratio, canvas_background_mode, &video_filters)
            .or_else(|| build_plain_video_filter(&video_filters));

    if let Some(video_filter) = video_filter {
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

    if should_apply_speed_filter(normalized_playback_speed) {
        ffmpeg_args.push("-af".to_string());
        ffmpeg_args.push(format!("atempo={normalized_playback_speed:.3}"));
    }

    ffmpeg_args.extend([
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "veryfast".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
        "-c:a".to_string(),
        "aac".to_string(),
        "-movflags".to_string(),
        "+faststart".to_string(),
        output_path_text.to_string(),
    ]);

    let output = Command::new("ffmpeg")
        .args(ffmpeg_args)
        .output()
        .map_err(|error| format!("无法调用 ffmpeg：{error}"));

    let _ = fs::remove_file(&list_path);
    cleanup_files(&prepared_segments.temporary_paths);

    let output = output?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "片段拼接失败。".to_string()
        } else {
            stderr
        });
    }

    if !output_path.is_file() {
        return Err("拼接命令已结束，但没有找到输出文件。".to_string());
    }

    Ok(MixVideoResult {
        output_path: output_path.to_string_lossy().to_string(),
        input_count: prepared_segments.segment_paths.len(),
        message: "片段拼接完成。".to_string(),
        output_aspect_ratio: canvas_aspect_ratio_label(canvas_aspect_ratio).to_string(),
        output_resolution: output_dimensions
            .map(|(width, height)| format!("{width}x{height}"))
            .unwrap_or_else(|| "原画".to_string()),
        background_mode: canvas_background_mode_label(canvas_background_mode).to_string(),
        applied_to_remix_export: true,
        smooth_remix_enabled,
        skipped_short_segment_count: prepared_segments.skipped_short_segment_count,
    })
}

fn normalize_video_effect_settings(
    settings: VideoEffectSettings,
) -> Result<VideoEffectSettings, String> {
    if !settings.brightness.is_finite() || settings.brightness < -1.0 || settings.brightness > 1.0
    {
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

fn should_apply_eq_filter(settings: VideoEffectSettings) -> bool {
    settings.brightness.abs() > 0.001
        || (settings.contrast - 1.0).abs() > 0.001
        || (settings.saturation - 1.0).abs() > 0.001
}

fn should_apply_scale_filter(scale: f64) -> bool {
    (scale - 1.0).abs() > 0.001
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
) -> Result<PreparedSegments, String> {
    let mut prepared_paths = Vec::new();
    let mut temporary_paths = Vec::new();
    let mut skipped_short_segment_count = 0;

    for (index, segment_path) in segment_paths.iter().enumerate() {
        let info = probe_segment_info(segment_path)?;

        if info.duration_seconds < 1.5 {
            skipped_short_segment_count += 1;
            continue;
        }

        let temp_path = output_dir.join(format!("smooth_segment_{timestamp}_{index}.mp4"));
        create_smooth_segment(segment_path, &temp_path, &info)?;
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
    let duration_output = Command::new("ffprobe")
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

    let audio_output = Command::new("ffprobe")
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
) -> Result<(), String> {
    let fade_duration = 0.2;
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

    let output = Command::new("ffmpeg")
        .args(ffmpeg_args)
        .output()
        .map_err(|error| format!("无法调用 ffmpeg 生成平滑片段：{error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "生成平滑片段失败。".to_string()
        } else {
            stderr
        });
    }

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
