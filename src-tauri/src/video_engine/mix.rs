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
use crate::video_engine::probe::{probe_video_dimensions, validate_rendered_video};
use crate::video_engine::render::RenderVideoResult;
use crate::video_engine::subtitle::{
    ensure_ass_filter_available, prepare_ass_subtitle_with_style, NarratedSubtitlePosition,
    NarratedSubtitleSettings, NarratedSubtitleSize, SubtitleStyleSettings,
};
use crate::video_engine::tool_paths::{background_command, ffprobe_program};
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemixSegmentInput {
    pub path: String,
    #[serde(default)]
    pub start_seconds: Option<f64>,
    #[serde(default)]
    pub duration_seconds: Option<f64>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoEffectSettings {
    vertical_mirror: bool,
    rotation: RotationMode,
    #[serde(default)]
    hsl_enabled: bool,
    #[serde(default)]
    hue: f64,
    brightness: f64,
    contrast: f64,
    saturation: f64,
    scale: f64,
    #[serde(default)]
    crop: CanvasCropSettings,
    #[serde(default)]
    zoom_enabled: bool,
    #[serde(default)]
    zoom_mode: DynamicZoomMode,
    #[serde(default = "default_zoom_min_scale")]
    zoom_min_scale: f64,
    #[serde(default = "default_zoom_max_scale")]
    zoom_max_scale: f64,
    #[serde(default = "default_zoom_min_duration")]
    zoom_min_duration_seconds: f64,
    #[serde(default = "default_zoom_max_duration")]
    zoom_max_duration_seconds: f64,
    #[serde(default)]
    crop_black_bars: bool,
    #[serde(default = "default_random_rotation_min")]
    random_rotation_min_degrees: f64,
    #[serde(default = "default_random_rotation_max")]
    random_rotation_max_degrees: f64,
    #[serde(default)]
    sharpness: f64,
    #[serde(default)]
    noise_reduction: f64,
    #[serde(default = "default_temperature")]
    temperature: f64,
    #[serde(default)]
    visual_style: VideoVisualStyle,
    #[serde(default)]
    glow_enabled: bool,
    #[serde(default)]
    grain_enabled: bool,
    #[serde(default)]
    vignette_enabled: bool,
}

impl Default for VideoEffectSettings {
    fn default() -> Self {
        Self {
            vertical_mirror: false,
            rotation: RotationMode::None,
            hsl_enabled: false,
            hue: 0.0,
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            scale: 1.0,
            crop: CanvasCropSettings::default(),
            zoom_enabled: false,
            zoom_mode: DynamicZoomMode::Push,
            zoom_min_scale: default_zoom_min_scale(),
            zoom_max_scale: default_zoom_max_scale(),
            zoom_min_duration_seconds: default_zoom_min_duration(),
            zoom_max_duration_seconds: default_zoom_max_duration(),
            crop_black_bars: false,
            random_rotation_min_degrees: 0.0,
            random_rotation_max_degrees: 0.0,
            sharpness: 0.0,
            noise_reduction: 0.0,
            temperature: default_temperature(),
            visual_style: VideoVisualStyle::None,
            glow_enabled: false,
            grain_enabled: false,
            vignette_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum VideoVisualStyle {
    #[default]
    None,
    Random,
    Bw,
    Invert,
    Retro,
    Cross,
    Cartoon,
    Emboss,
    Pixel,
    Outline,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CanvasCropSettings {
    enabled: bool,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl Default for CanvasCropSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RotationMode {
    None,
    Clockwise90,
    Counterclockwise90,
    Rotate180,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum DynamicZoomMode {
    #[default]
    Push,
    Pull,
    Random,
}

fn default_zoom_min_scale() -> f64 {
    1.02
}
fn default_zoom_max_scale() -> f64 {
    1.08
}
fn default_zoom_min_duration() -> f64 {
    8.0
}
fn default_zoom_max_duration() -> f64 {
    10.0
}
fn default_random_rotation_min() -> f64 {
    0.0
}
fn default_random_rotation_max() -> f64 {
    0.0
}
fn default_temperature() -> f64 {
    6500.0
}

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum PlaybackSpeedMode {
    #[default]
    Global,
    Segment,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlaybackSpeedSettings {
    #[serde(default)]
    mode: PlaybackSpeedMode,
    #[serde(default = "default_playback_speed")]
    min: f64,
    #[serde(default = "default_playback_speed")]
    max: f64,
    #[serde(default = "default_playback_segment_min")]
    segment_min_seconds: f64,
    #[serde(default = "default_playback_segment_max")]
    segment_max_seconds: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PlaybackSpeedSegment {
    start_seconds: f64,
    end_seconds: f64,
    speed: f64,
}

impl Default for PlaybackSpeedSettings {
    fn default() -> Self {
        Self {
            mode: PlaybackSpeedMode::Global,
            min: 1.0,
            max: 1.0,
            segment_min_seconds: 2.0,
            segment_max_seconds: 4.0,
        }
    }
}

fn default_playback_speed() -> f64 {
    1.0
}
fn default_playback_segment_min() -> f64 {
    2.0
}
fn default_playback_segment_max() -> f64 {
    4.0
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PictureInPictureSettings {
    enabled: bool,
    overlay_file_path: Option<String>,
    #[serde(default)]
    mode: PipMode,
    #[serde(default = "default_pip_main_size_min")]
    main_size_min: f64,
    #[serde(default = "default_pip_main_size_max")]
    main_size_max: f64,
    #[serde(default = "default_pip_blur_min")]
    blur_min: f64,
    #[serde(default = "default_pip_blur_max")]
    blur_max: f64,
    #[serde(default = "default_pip_offset")]
    offset_x_min: f64,
    #[serde(default = "default_pip_offset")]
    offset_x_max: f64,
    #[serde(default = "default_pip_offset")]
    offset_y_min: f64,
    #[serde(default = "default_pip_offset")]
    offset_y_max: f64,
    position: PipPosition,
    size_ratio: f64,
    opacity: f64,
    margin: u32,
}

impl Default for PictureInPictureSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            overlay_file_path: None,
            mode: PipMode::External,
            main_size_min: default_pip_main_size_min(),
            main_size_max: default_pip_main_size_max(),
            blur_min: default_pip_blur_min(),
            blur_max: default_pip_blur_max(),
            offset_x_min: default_pip_offset(),
            offset_x_max: default_pip_offset(),
            offset_y_min: default_pip_offset(),
            offset_y_max: default_pip_offset(),
            position: PipPosition::TopRight,
            size_ratio: 0.3,
            opacity: 1.0,
            margin: 24,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BgmSettings {
    enabled: bool,
    audio_file_path: Option<String>,
    original_volume: f64,
    #[serde(default = "default_original_volume")]
    original_volume_min: f64,
    #[serde(default = "default_original_volume")]
    original_volume_max: f64,
    #[serde(default)]
    original_fade_enabled: bool,
    #[serde(default)]
    dynamic_adjust_enabled: bool,
    bgm_volume: f64,
    #[serde(default = "default_bgm_volume")]
    bgm_volume_min: f64,
    #[serde(default = "default_bgm_volume")]
    bgm_volume_max: f64,
    #[serde(default)]
    bgm_fade_enabled: bool,
    #[serde(default = "default_true")]
    loop_playback_enabled: bool,
    fade_in_seconds: f64,
    fade_out_seconds: f64,
}

fn default_pip_main_size_min() -> f64 {
    0.96
}
fn default_pip_main_size_max() -> f64 {
    0.99
}
fn default_pip_blur_min() -> f64 {
    40.0
}
fn default_pip_blur_max() -> f64 {
    50.0
}
fn default_pip_offset() -> f64 {
    0.5
}
fn default_original_volume() -> f64 {
    1.0
}
fn default_bgm_volume() -> f64 {
    0.35
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleSettings {
    enabled: bool,
    #[serde(default)]
    text: String,
    #[serde(default = "default_subtitle_position")]
    position: NarratedSubtitlePosition,
    #[serde(default = "default_subtitle_size")]
    size: NarratedSubtitleSize,
    #[serde(default = "default_subtitle_font_family")]
    font_family: String,
    #[serde(default = "default_subtitle_text_color")]
    text_color: String,
    #[serde(default = "default_subtitle_opacity")]
    opacity: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemixSettings {
    apply_horizontal_mirror: bool,
    playback_speed: f64,
    #[serde(default)]
    playback_speed_settings: PlaybackSpeedSettings,
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
    #[serde(default)]
    output_name: Option<String>,
    #[serde(default)]
    entrance_effect: EntranceEffect,
    #[serde(default)]
    frame_operation_settings: FrameOperationSettings,
    #[serde(default)]
    fusion_settings: FusionSettings,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct FrameOperationSettings {
    enabled: bool,
    mode: FrameOperationMode,
    interval_min: u32,
    interval_max: u32,
    frame_min: u32,
    frame_max: u32,
    opacity: f64,
    material_file_path: Option<String>,
}

impl Default for FrameOperationSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: FrameOperationMode::Extract,
            interval_min: 30,
            interval_max: 60,
            frame_min: 1,
            frame_max: 1,
            opacity: 2.0,
            material_file_path: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
enum FrameOperationMode {
    #[default]
    Extract,
    Insert,
    Mixed,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct FusionSettings {
    enabled: bool,
    material_file_path: Option<String>,
    interval_min: u32,
    interval_max: u32,
    strength: f64,
}

impl Default for FusionSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            material_file_path: None,
            interval_min: 15,
            interval_max: 60,
            strength: 0.2,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
enum EntranceEffect {
    #[default]
    None,
    SmoothUp,
    SmoothDown,
    HorizontalSqueeze,
    VerticalSqueeze,
    CircleCrop,
    RectangleCrop,
    CircleClose,
    CircleOpen,
    HorizontalClose,
    HorizontalOpen,
    VerticalClose,
    VerticalOpen,
    LeftBottom,
    RightBottom,
    LeftTop,
    RightTop,
    HorizontalSlice,
    VerticalSlice,
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

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum PipMode {
    Main,
    #[default]
    External,
}

pub fn concat_video_segments(
    segment_paths: Vec<String>,
    output_directory: String,
    settings: RemixSettings,
    task_context: Option<TaskProgressContext>,
    segment_inputs: Option<Vec<RemixSegmentInput>>,
) -> Result<MixVideoResult, String> {
    concat_video_segments_with_options(
        segment_paths,
        output_directory,
        settings,
        true,
        false,
        task_context,
        segment_inputs,
    )
}

pub fn export_single_video(
    input_file_path: String,
    output_directory: String,
    _duration_seconds: Option<f64>,
    settings: RemixSettings,
    task_context: Option<TaskProgressContext>,
) -> Result<RenderVideoResult, String> {
    let result = concat_video_segments_with_options(
        vec![input_file_path],
        output_directory,
        settings,
        false,
        true,
        task_context,
        None,
    )?;

    Ok(RenderVideoResult {
        output_path: result.output_path,
        message: result.message,
        output_resolution: result.output_resolution,
        output_encoder: result.output_encoder,
        output_frame_rate: result.output_frame_rate,
        output_quality: result.output_quality,
        output_video_bitrate_kbps: result.output_video_bitrate_kbps,
    })
}

pub(crate) fn concat_narrated_prepared_segments(
    segment_paths: Vec<String>,
    output_directory: String,
    mut settings: RemixSettings,
    task_context: Option<TaskProgressContext>,
) -> Result<MixVideoResult, String> {
    if should_apply_speed_filter(settings.playback_speed)
        || settings.playback_speed_settings.mode == PlaybackSpeedMode::Segment
            && (should_apply_speed_filter(settings.playback_speed_settings.min)
                || should_apply_speed_filter(settings.playback_speed_settings.max))
    {
        return Err("AI配音视频暂时只支持 1.0x 速度，请把视频变速恢复为 1.0x。".to_string());
    }

    settings.bgm_settings.original_volume = 1.0;
    concat_video_segments_with_options(
        segment_paths,
        output_directory,
        settings,
        false,
        false,
        task_context,
        None,
    )
}

fn concat_video_segments_with_options(
    segment_paths: Vec<String>,
    output_directory: String,
    settings: RemixSettings,
    filter_short_smooth_segments: bool,
    allow_single_segment: bool,
    task_context: Option<TaskProgressContext>,
    segment_inputs: Option<Vec<RemixSegmentInput>>,
) -> Result<MixVideoResult, String> {
    let RemixSettings {
        apply_horizontal_mirror,
        playback_speed,
        playback_speed_settings,
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
        output_name,
        entrance_effect,
        frame_operation_settings,
        fusion_settings,
    } = settings;

    let output_dir = Path::new(&output_directory);
    let selected_playback_speed = normalize_playback_speed(resolve_playback_speed(
        playback_speed,
        playback_speed_settings,
        current_timestamp()?,
    )?)?;
    let normalized_playback_speed = if playback_speed_settings.mode == PlaybackSpeedMode::Global {
        selected_playback_speed
    } else {
        1.0
    };
    let normalized_effect_settings = normalize_video_effect_settings(video_effect_settings)?;
    let normalized_pip_settings =
        normalize_picture_in_picture_settings(picture_in_picture_settings)?;
    let normalized_bgm_settings = normalize_bgm_settings(bgm_settings)?;
    let normalized_watermark_settings = normalize_watermark_settings(watermark_settings)?;
    let normalized_watermark_removal_settings =
        normalize_watermark_removal_settings(watermark_removal_settings)?;
    let frame_operation_settings = normalize_frame_operation_settings(frame_operation_settings)?;
    let fusion_settings = normalize_fusion_settings(fusion_settings)?;
    let normalized_subtitle_settings = normalize_subtitle_settings(subtitle_settings)?;

    if normalized_subtitle_settings.is_some() {
        ensure_ass_filter_available()?;
    }

    if normalized_watermark_removal_settings
        .as_ref()
        .is_some_and(|settings| settings.tracking_enabled)
    {
        return Err("移动水印关键帧跟踪第一版只支持“导出当前完整视频”。请先导出清理后的视频，再重新导入进行混剪。".to_string());
    }

    if !allow_single_segment && segment_paths.len() < 2 {
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
    let segment_paths = materialize_segment_inputs(
        &segment_paths,
        segment_inputs.as_deref(),
        temp_directory.path(),
        timestamp,
        task_context.as_ref(),
    )?;
    let watermark_text_file = normalized_watermark_settings
        .as_ref()
        .map(|settings| prepare_text_watermark_file(settings, temp_directory.path()))
        .transpose()?
        .flatten();
    let list_path = temp_directory
        .path()
        .join(format!("concat_list_{timestamp}.txt"));
    let output_path = resolve_output_path(
        output_dir,
        output_name.as_deref(),
        output_settings,
        timestamp,
    )?;
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

    if !allow_single_segment && prepared_segments.segment_paths.len() < 2 {
        cleanup_files(&prepared_segments.temporary_paths);
        return Err("过滤过短片段后，至少需要 2 个片段才能拼接。".to_string());
    }

    let prepared_segments_have_audio = probe_segment_info(&prepared_segments.segment_paths[0])
        .map(|info| info.has_audio)
        .unwrap_or(false);
    let source_duration_seconds =
        calculate_output_duration_seconds(&prepared_segments.segment_paths, 1.0)?;
    let mut final_input_paths = prepared_segments.segment_paths.clone();
    let output_duration_seconds = if playback_speed_settings.mode == PlaybackSpeedMode::Segment {
        let speed_plan = build_segmented_playback_speed_plan(
            source_duration_seconds,
            playback_speed_settings,
            timestamp,
        )?;
        let source_list_content = build_concat_list_content(&prepared_segments.segment_paths);
        fs::write(&list_path, source_list_content).map_err(|error| {
            format!(
                "无法创建分段变速输入列表 {}：{error}",
                list_path.to_string_lossy()
            )
        })?;
        let segmented_source_path = temp_directory
            .path()
            .join(format!("segmented_speed_{timestamp}.mp4"));
        let speed_context = task_context
            .as_ref()
            .map(|context| context.child(0.2, 0.45, "正在执行分段变速"));
        materialize_segmented_playback_source(
            &list_path,
            &segmented_source_path,
            &speed_plan,
            prepared_segments_have_audio,
            speed_context.as_ref(),
        )?;
        final_input_paths = vec![segmented_source_path.to_string_lossy().to_string()];
        speed_plan
            .iter()
            .map(|segment| (segment.end_seconds - segment.start_seconds) / segment.speed)
            .sum::<f64>()
    } else {
        source_duration_seconds / normalized_playback_speed
    };

    let subtitle_filter = normalized_subtitle_settings
        .as_ref()
        .map(|settings| {
            let subtitle_path = temp_directory
                .path()
                .join(format!("subtitle_{timestamp}.ass"));
            prepare_ass_subtitle_with_style(
                &subtitle_path,
                &settings.text,
                output_duration_seconds,
                NarratedSubtitleSettings {
                    enabled: true,
                    position: settings.position,
                    size: settings.size,
                },
                SubtitleStyleSettings {
                    font_family: settings.font_family.clone(),
                    text_color: settings.text_color.clone(),
                    opacity: settings.opacity,
                },
            )
        })
        .transpose()?
        .flatten();

    let concat_list_content = build_concat_list_content(&final_input_paths);
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
    let frame_input_index = frame_operation_settings
        .as_ref()
        .filter(|settings| {
            matches!(
                settings.mode,
                FrameOperationMode::Insert | FrameOperationMode::Mixed
            )
        })
        .map(|settings| {
            let index = next_input_index;
            next_input_index += 1;
            (index, settings)
        });
    if let Some((_, settings)) = frame_input_index {
        append_visual_material_input_args(
            &mut ffmpeg_args,
            settings.material_file_path.as_deref().unwrap(),
        )?;
    }
    let fusion_input_index = fusion_settings.as_ref().map(|settings| {
        let index = next_input_index;
        next_input_index += 1;
        (index, settings)
    });
    if let Some((_, settings)) = fusion_input_index {
        append_visual_material_input_args(
            &mut ffmpeg_args,
            settings.material_file_path.as_deref().unwrap(),
        )?;
    }
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

    if let Some(frame_settings) = &frame_operation_settings {
        if matches!(
            frame_settings.mode,
            FrameOperationMode::Extract | FrameOperationMode::Mixed
        ) {
            let interval =
                random_u32_in_range(frame_settings.interval_min, frame_settings.interval_max);
            video_filters.push(format!(
                "select='not(mod(n\\,{interval}))',setpts=N/FRAME_RATE/TB"
            ));
        }
        if matches!(
            frame_settings.mode,
            FrameOperationMode::Insert | FrameOperationMode::Mixed
        ) {
            video_filters.push("tpad=stop_mode=clone:stop_duration=0.08".to_string());
        }
    }

    if let Some(entrance_filter) = build_entrance_effect_filter(entrance_effect) {
        video_filters.push(entrance_filter);
    }

    if apply_horizontal_mirror {
        video_filters.push("hflip".to_string());
    }

    let zoom_crop_dimensions = if normalized_effect_settings.zoom_enabled {
        Some(dimensions_after_rotation(
            probe_video_dimensions(&prepared_segments.segment_paths[0])?,
            normalized_effect_settings.rotation,
        ))
    } else {
        None
    };
    video_filters.extend(build_video_effect_filters(
        normalized_effect_settings,
        zoom_crop_dimensions,
    ));

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
        subtitle_filter.as_deref(),
        frame_input_index,
        fusion_input_index,
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

    validate_rendered_video(&output_path)
        .map_err(|error| format!("拼接命令已结束，但成片校验失败：{error}"))?;

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
    let ranges = [
        (settings.main_size_min, settings.main_size_max, 0.1, 1.0),
        (settings.blur_min, settings.blur_max, 0.0, 100.0),
        (settings.offset_x_min, settings.offset_x_max, 0.0, 1.0),
        (settings.offset_y_min, settings.offset_y_max, 0.0, 1.0),
    ];
    if ranges.iter().any(|(minimum, maximum, lower, upper)| {
        !minimum.is_finite()
            || !maximum.is_finite()
            || minimum < lower
            || maximum > upper
            || maximum < minimum
    }) {
        return Err("画中画主视频大小、虚化或偏移范围无效。".to_string());
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
    if !settings.original_volume_min.is_finite()
        || !settings.original_volume_max.is_finite()
        || settings.original_volume_min < 0.0
        || settings.original_volume_max > 2.0
        || settings.original_volume_max < settings.original_volume_min
        || !settings.bgm_volume_min.is_finite()
        || !settings.bgm_volume_max.is_finite()
        || settings.bgm_volume_min < 0.0
        || settings.bgm_volume_max > 2.0
        || settings.bgm_volume_max < settings.bgm_volume_min
    {
        return Err("主视频或背景音乐的音量范围无效。".to_string());
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

fn normalize_subtitle_settings(
    settings: SubtitleSettings,
) -> Result<Option<SubtitleSettings>, String> {
    if !settings.enabled {
        return Ok(None);
    }

    if settings.text.trim().is_empty() {
        return Err("已开启字幕，但字幕文案为空，请先输入字幕文案。".to_string());
    }
    if settings.font_family.trim().is_empty()
        || settings.font_family.chars().count() > 80
        || settings
            .font_family
            .chars()
            .any(|character| matches!(character, ',' | '\n' | '\r'))
    {
        return Err("字幕字体名称无效。".to_string());
    }
    if settings.text_color.len() != 7
        || !settings.text_color.starts_with('#')
        || !settings.text_color[1..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("字幕颜色必须是 #RRGGBB 格式。".to_string());
    }
    if !settings.opacity.is_finite() || !(0.1..=1.0).contains(&settings.opacity) {
        return Err("字幕透明度必须在 10% 到 100% 之间。".to_string());
    }

    Ok(Some(SubtitleSettings {
        enabled: true,
        text: settings.text.trim().to_string(),
        position: settings.position,
        size: settings.size,
        font_family: settings.font_family.trim().to_string(),
        text_color: settings.text_color,
        opacity: settings.opacity,
    }))
}

fn default_subtitle_position() -> NarratedSubtitlePosition {
    NarratedSubtitlePosition::Bottom
}

fn default_subtitle_size() -> NarratedSubtitleSize {
    NarratedSubtitleSize::Medium
}

fn default_subtitle_font_family() -> String {
    "Microsoft YaHei".to_string()
}

fn default_subtitle_text_color() -> String {
    "#ffffff".to_string()
}

fn default_subtitle_opacity() -> f64 {
    1.0
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
    } else {
        ffmpeg_args.push("-stream_loop".to_string());
        ffmpeg_args.push("-1".to_string());
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

    if settings.loop_playback_enabled {
        ffmpeg_args.push("-stream_loop".to_string());
        ffmpeg_args.push("-1".to_string());
    }
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

    let original_volume =
        random_in_range(settings.original_volume_min, settings.original_volume_max);
    let mut original_filters = vec![format!("volume={original_volume:.3}")];
    if settings.dynamic_adjust_enabled {
        original_filters.push("dynaudnorm=f=150:g=5".to_string());
    }
    if should_apply_speed_filter(playback_speed) {
        original_filters.push(format!("atempo={playback_speed:.3}"));
    }
    if settings.original_fade_enabled {
        let fade_duration = 0.5_f64.min(output_duration_seconds / 2.0);
        original_filters.push(format!("afade=t=in:st=0:d={fade_duration:.3}"));
        original_filters.push(format!(
            "afade=t=out:st={:.3}:d={fade_duration:.3}",
            (output_duration_seconds - fade_duration).max(0.0)
        ));
    }
    let original_filter = format!("[0:a]{}[original]", original_filters.join(","));

    format!(
        "{original_filter};{bgm_filter};[original][bgm]amix=inputs=2:duration=first:dropout_transition=0[a]"
    )
}

fn build_bgm_source_filter(
    settings: &BgmSettings,
    bgm_input_index: usize,
    output_duration_seconds: f64,
) -> String {
    let bgm_volume = random_in_range(settings.bgm_volume_min, settings.bgm_volume_max);
    let mut filters = vec![format!("volume={bgm_volume:.3}")];

    if settings.bgm_fade_enabled && settings.fade_in_seconds > 0.001 {
        let fade_in_duration = settings.fade_in_seconds.min(output_duration_seconds);
        filters.push(format!("afade=t=in:st=0:d={fade_in_duration:.3}"));
    }

    if settings.bgm_fade_enabled && settings.fade_out_seconds > 0.001 {
        let fade_out_duration = settings.fade_out_seconds.min(output_duration_seconds);
        let fade_out_start = (output_duration_seconds - fade_out_duration).max(0.0);
        filters.push(format!(
            "afade=t=out:st={fade_out_start:.3}:d={fade_out_duration:.3}"
        ));
    }

    filters.push(format!("apad=whole_dur={output_duration_seconds:.3}"));
    filters.push(format!("atrim=duration={output_duration_seconds:.3}"));
    format!("[{bgm_input_index}:a]{}[bgm]", filters.join(","))
}

#[allow(clippy::too_many_arguments)]
fn build_composed_video_filter(
    base_filter: Option<CanvasFilter>,
    watermark_removal_settings: Option<&WatermarkRemovalSettings>,
    watermark_removal_frame_dimensions: Option<(u32, u32)>,
    pip_input: Option<(usize, &PictureInPictureSettings)>,
    watermark_settings: Option<&WatermarkSettings>,
    watermark_image_input_index: Option<usize>,
    watermark_text_file: Option<&Path>,
    subtitle_filter: Option<&str>,
    frame_input: Option<(usize, &FrameOperationSettings)>,
    fusion_input: Option<(usize, &FusionSettings)>,
) -> Result<Option<CanvasFilter>, String> {
    if watermark_removal_settings.is_none()
        && pip_input.is_none()
        && watermark_settings.is_none()
        && subtitle_filter.is_none()
        && frame_input.is_none()
        && fusion_input.is_none()
    {
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

    if let Some((input_index, settings)) = frame_input {
        let output_label = format!("[layer{layer_number}]");
        filters.push(build_frame_insert_layer_filter(
            &current_label,
            &output_label,
            input_index,
            settings,
            layer_number,
        ));
        current_label = output_label;
        layer_number += 1;
    }

    if let Some((input_index, settings)) = fusion_input {
        let output_label = format!("[layer{layer_number}]");
        filters.push(build_fusion_layer_filter(
            &current_label,
            &output_label,
            input_index,
            settings,
            layer_number,
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

    if let Some(subtitle_filter) = subtitle_filter {
        let output_label = format!("[layer{layer_number}]");
        filters.push(format!("{current_label}{subtitle_filter}{output_label}"));
        current_label = output_label;
    }

    filters.push(format!("{current_label}format=yuv420p[v]"));
    Ok(Some(CanvasFilter {
        filter: filters.join(";"),
        is_complex: true,
    }))
}

fn build_fusion_layer_filter(
    input_label: &str,
    output_label: &str,
    input_index: usize,
    settings: &FusionSettings,
    layer_number: usize,
) -> String {
    let interval = random_u32_in_range(settings.interval_min, settings.interval_max);
    format!("[{input_index}:v]format=rgba,colorchannelmixer=aa={strength:.3}[fusionraw{layer_number}];[fusionraw{layer_number}]{input_label}scale2ref=w=main_w:h=main_h[fusion{layer_number}][fusionbase{layer_number}];[fusionbase{layer_number}][fusion{layer_number}]blend=all_mode=overlay:all_opacity={strength:.3}:enable='eq(mod(n\\,{interval})\\,0)'{output_label}", strength = settings.strength)
}

fn build_frame_insert_layer_filter(
    input_label: &str,
    output_label: &str,
    input_index: usize,
    settings: &FrameOperationSettings,
    layer_number: usize,
) -> String {
    let opacity = settings.opacity / 100.0;
    let interval = random_u32_in_range(settings.interval_min, settings.interval_max);
    let frame_count = random_u32_in_range(settings.frame_min, settings.frame_max);
    format!("[{input_index}:v]format=rgba,colorchannelmixer=aa={opacity:.3}[frameraw{layer_number}];[frameraw{layer_number}]{input_label}scale2ref=w=main_w:h=main_h[frame{layer_number}][framebase{layer_number}];[framebase{layer_number}][frame{layer_number}]overlay=0:0:eof_action=pass:enable='lt(mod(n\\,{interval})\\,{frame_count})'{output_label}")
}

fn append_visual_material_input_args(args: &mut Vec<String>, path: &str) -> Result<(), String> {
    if !Path::new(path).is_file() {
        return Err(format!("效果素材文件不存在：{path}"));
    }
    if is_image_file(path) {
        args.extend(["-loop".to_string(), "1".to_string()]);
    }
    args.extend(["-i".to_string(), path.to_string()]);
    Ok(())
}

fn normalize_frame_operation_settings(
    settings: FrameOperationSettings,
) -> Result<Option<FrameOperationSettings>, String> {
    if !settings.enabled {
        return Ok(None);
    }
    if settings.interval_min == 0
        || settings.interval_max < settings.interval_min
        || settings.frame_min == 0
        || settings.frame_max < settings.frame_min
        || !settings.opacity.is_finite()
        || !(0.0..=100.0).contains(&settings.opacity)
    {
        return Err("视频帧操作参数无效。".to_string());
    }
    if matches!(
        settings.mode,
        FrameOperationMode::Insert | FrameOperationMode::Mixed
    ) && settings
        .material_file_path
        .as_deref()
        .is_none_or(|path| !Path::new(path).is_file())
    {
        return Err("插帧模式需要选择有效的素材文件。".to_string());
    }
    Ok(Some(settings))
}

fn normalize_fusion_settings(settings: FusionSettings) -> Result<Option<FusionSettings>, String> {
    if !settings.enabled {
        return Ok(None);
    }
    if settings.interval_min == 0
        || settings.interval_max < settings.interval_min
        || !settings.strength.is_finite()
        || !(0.01..=1.0).contains(&settings.strength)
    {
        return Err("像素融合参数无效。".to_string());
    }
    if settings
        .material_file_path
        .as_deref()
        .is_none_or(|path| !Path::new(path).is_file())
    {
        return Err("像素融合需要选择有效的融合素材。".to_string());
    }
    Ok(Some(settings))
}

fn build_pip_layer_filter(
    input_label: &str,
    output_label: &str,
    input_index: usize,
    settings: &PictureInPictureSettings,
    layer_number: usize,
) -> String {
    if settings.mode == PipMode::Main {
        let size_ratio = random_in_range(settings.main_size_min, settings.main_size_max);
        let blur_strength = random_in_range(settings.blur_min, settings.blur_max) / 10.0;
        let offset_x = random_in_range(settings.offset_x_min, settings.offset_x_max);
        let offset_y = random_in_range(settings.offset_y_min, settings.offset_y_max);
        let background_filter = if blur_strength > 0.001 {
            format!("boxblur=luma_radius={blur_strength:.3}:luma_power=1")
        } else {
            "format=yuv420p".to_string()
        };
        return format!(
            "[{input_index}:v]{input_label}scale2ref=w=main_w:h=main_h[pipbgraw{layer_number}][pipmainraw{layer_number}];\
             [pipbgraw{layer_number}]{background_filter}[pipbg{layer_number}];\
             [pipmainraw{layer_number}]scale=iw*{size_ratio:.3}:ih*{size_ratio:.3},format=rgba,colorchannelmixer=aa={opacity:.3}[pipmain{layer_number}];\
             [pipbg{layer_number}][pipmain{layer_number}]overlay=x='(W-w)*{offset_x:.3}':y='(H-h)*{offset_y:.3}':eof_action=pass:format=auto{output_label}",
            opacity = settings.opacity,
        );
    }
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

    if settings.crop.enabled {
        let crop = settings.crop;
        if !crop.x.is_finite()
            || !crop.y.is_finite()
            || !crop.width.is_finite()
            || !crop.height.is_finite()
            || crop.x < 0.0
            || crop.y < 0.0
            || crop.width < 12.0
            || crop.height < 12.0
            || crop.x + crop.width > 100.0
            || crop.y + crop.height > 100.0
        {
            return Err("裁剪区域必须位于画面内，且宽高不能小于 12%。".to_string());
        }
    }

    if !settings.hue.is_finite() || settings.hue < -180.0 || settings.hue > 180.0 {
        return Err("色相范围必须在 -180 到 180 度之间。".to_string());
    }
    if !settings.zoom_min_scale.is_finite()
        || !settings.zoom_max_scale.is_finite()
        || settings.zoom_min_scale < 1.0
        || settings.zoom_max_scale < settings.zoom_min_scale
        || settings.zoom_max_scale > 1.5
    {
        return Err("动态缩放范围无效。".to_string());
    }
    if !settings.zoom_min_duration_seconds.is_finite()
        || !settings.zoom_max_duration_seconds.is_finite()
        || settings.zoom_min_duration_seconds < 1.0
        || settings.zoom_max_duration_seconds < settings.zoom_min_duration_seconds
        || settings.zoom_max_duration_seconds > 120.0
    {
        return Err("动态缩放时长必须在 1 到 120 秒之间。".to_string());
    }

    if !settings.random_rotation_min_degrees.is_finite()
        || !settings.random_rotation_max_degrees.is_finite()
        || settings.random_rotation_min_degrees < -15.0
        || settings.random_rotation_max_degrees > 15.0
        || settings.random_rotation_max_degrees < settings.random_rotation_min_degrees
    {
        return Err("随机旋转角度范围无效。".to_string());
    }
    if !settings.sharpness.is_finite()
        || !settings.noise_reduction.is_finite()
        || !(0.0..=100.0).contains(&settings.sharpness)
        || !(0.0..=100.0).contains(&settings.noise_reduction)
    {
        return Err("锐化和降噪强度必须在 0 到 100 之间。".to_string());
    }
    if !settings.temperature.is_finite() || !(1000.0..=12000.0).contains(&settings.temperature) {
        return Err("色温必须在 1000K 到 12000K 之间。".to_string());
    }

    Ok(settings)
}

fn build_video_effect_filters(
    settings: VideoEffectSettings,
    zoom_crop_dimensions: Option<(u32, u32)>,
) -> Vec<String> {
    let mut filters = Vec::new();

    if settings.vertical_mirror {
        filters.push("vflip".to_string());
    }

    if settings.crop_black_bars {
        // FFmpeg 的 cropdetect 只能输出元数据，不能在同一条导出命令中动态回填裁剪框。
        // 这里采用四周各裁去 1% 的稳定安全边界，真实移除常见的编码黑边且避免画面抖动。
        filters.push("crop=w='trunc(iw*0.98/2)*2':h='trunc(ih*0.98/2)*2':x='trunc(iw*0.01/2)*2':y='trunc(ih*0.01/2)*2'".to_string());
    }

    match settings.rotation {
        RotationMode::None => {}
        RotationMode::Clockwise90 => filters.push("transpose=1".to_string()),
        RotationMode::Counterclockwise90 => filters.push("transpose=2".to_string()),
        RotationMode::Rotate180 => filters.push("hflip,vflip".to_string()),
    }

    if should_apply_hue_filter(settings) {
        filters.push(format!("hue=h={:.3}:s=1", settings.hue));
    }

    if should_apply_eq_filter(settings) {
        filters.push(format!(
            "eq=brightness={:.3}:contrast={:.3}:saturation={:.3}",
            settings.brightness, settings.contrast, settings.saturation
        ));
    }

    if settings.sharpness > 0.001 {
        filters.push(format!("unsharp=5:5:{:.3}", settings.sharpness / 50.0));
    }
    if settings.noise_reduction > 0.001 {
        let strength = settings.noise_reduction / 12.5;
        filters.push(format!(
            "hqdn3d={strength:.3}:{strength:.3}:{:.3}:{:.3}",
            strength * 1.5,
            strength * 1.5
        ));
    }
    if (settings.temperature - 6500.0).abs() > 0.5 {
        filters.push(format!(
            "colortemperature=temperature={:.0}",
            settings.temperature
        ));
    }
    match settings.visual_style {
        VideoVisualStyle::None => {}
        VideoVisualStyle::Random => match random_visual_style() {
            VideoVisualStyle::Bw => filters.push("hue=s=0".to_string()),
            VideoVisualStyle::Invert => filters.push("negate".to_string()),
            VideoVisualStyle::Retro => filters.push("curves=preset=vintage".to_string()),
            VideoVisualStyle::Cross => filters.push("hue=h=18:s=1.18".to_string()),
            VideoVisualStyle::Cartoon => filters.push("edgedetect=low=0.06:high=0.18".to_string()),
            VideoVisualStyle::Emboss => {
                filters.push("convolution='-2 -1 0 -1 1 1 0 1 2'".to_string())
            }
            VideoVisualStyle::Pixel => filters
                .push("scale=iw/8:ih/8:flags=neighbor,scale=iw:ih:flags=neighbor".to_string()),
            VideoVisualStyle::Outline => filters.push("edgedetect=low=0.04:high=0.12".to_string()),
            VideoVisualStyle::None | VideoVisualStyle::Random => {}
        },
        VideoVisualStyle::Bw => filters.push("hue=s=0".to_string()),
        VideoVisualStyle::Invert => filters.push("negate".to_string()),
        VideoVisualStyle::Retro => filters.push("curves=preset=vintage".to_string()),
        VideoVisualStyle::Cross => filters.push("hue=h=18:s=1.18".to_string()),
        VideoVisualStyle::Cartoon => filters.push("edgedetect=low=0.06:high=0.18".to_string()),
        VideoVisualStyle::Emboss => filters.push("convolution='-2 -1 0 -1 1 1 0 1 2'".to_string()),
        VideoVisualStyle::Pixel => {
            filters.push("scale=iw/8:ih/8:flags=neighbor,scale=iw:ih:flags=neighbor".to_string())
        }
        VideoVisualStyle::Outline => filters.push("edgedetect=low=0.04:high=0.12".to_string()),
    }
    if settings.glow_enabled {
        filters.push("eq=brightness=0.035:saturation=1.15".to_string());
    }
    if settings.grain_enabled {
        filters.push("noise=alls=8:allf=t".to_string());
    }
    if settings.vignette_enabled {
        filters.push("vignette=angle=PI/5".to_string());
    }

    if should_apply_scale_filter(settings.scale) {
        filters.push(format!(
            "scale=iw*{:.3}:ih*{:.3},crop=iw/{:.3}:ih/{:.3}",
            settings.scale, settings.scale, settings.scale, settings.scale
        ));
    }

    if settings.crop.enabled {
        filters.push(format!(
            "crop=w='trunc(iw*{:.6}/100/2)*2':h='trunc(ih*{:.6}/100/2)*2':x='trunc(iw*{:.6}/100/2)*2':y='trunc(ih*{:.6}/100/2)*2'",
            settings.crop.width,
            settings.crop.height,
            settings.crop.x,
            settings.crop.y,
        ));
    }

    if settings.zoom_enabled {
        filters.push(
            zoom_crop_dimensions
                .map(|dimensions| build_dynamic_zoom_filter(settings, dimensions))
                .unwrap_or_else(|| build_dynamic_zoom_filter_stable(settings)),
        );
    }

    if settings.random_rotation_min_degrees.abs() > 0.001
        || settings.random_rotation_max_degrees.abs() > 0.001
    {
        let angle = random_in_range(
            settings.random_rotation_min_degrees,
            settings.random_rotation_max_degrees,
        );
        if angle.abs() > 0.001 {
            filters.push(format!(
                "rotate={:.5}*PI/180:ow=rotw(iw):oh=roth(ih):c=black",
                angle
            ));
        }
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

fn should_apply_hue_filter(settings: VideoEffectSettings) -> bool {
    settings.hsl_enabled && settings.hue.abs() > 0.001
}

#[allow(dead_code)]
fn build_dynamic_zoom_filter_legacy(settings: VideoEffectSettings) -> String {
    let min = settings.zoom_min_scale;
    let max = settings.zoom_max_scale;
    let duration = settings.zoom_max_duration_seconds.max(1.0);
    let progress = format!("mod(t,{duration:.3})/{duration:.3}");
    let zoom = match settings.zoom_mode {
        DynamicZoomMode::Push => format!("{min:.4}+({max:.4}-{min:.4})*({progress})"),
        DynamicZoomMode::Pull => format!("{max:.4}-({max:.4}-{min:.4})*({progress})"),
        DynamicZoomMode::Random => format!("if(lt(sin(t),0),{min:.4}+({max:.4}-{min:.4})*({progress}),{max:.4}-({max:.4}-{min:.4})*({progress}))"),
    };
    // scale/crop 的表达式参数使用逗号分隔，表达式内部的逗号必须转义；
    // 同时指定 eval=frame，才能让缩放随视频时间变化，而不是只计算第一帧。
    let escaped_zoom = zoom.replace(',', "\\,");
    format!(
        "scale=w=iw*({escaped_zoom}):h=ih*({escaped_zoom}):eval=frame,crop=w=iw/({escaped_zoom}):h=ih/({escaped_zoom}):x=(iw-ow)/2:y=(ih-oh)/2"
    )
}

fn build_dynamic_zoom_filter_stable(settings: VideoEffectSettings) -> String {
    let target_scale = match settings.zoom_mode {
        DynamicZoomMode::Push | DynamicZoomMode::Pull | DynamicZoomMode::Random => {
            settings.zoom_max_scale
        }
    };
    // crop 的输出宽高需要在整段视频内保持稳定；固定到用户设定的最大倍率，
    // 先放大再居中裁切，保证智能配置的缩放效果可以稳定批量导出。
    format!(
        "scale=w=iw*{target_scale:.4}:h=ih*{target_scale:.4},crop=w=iw/{target_scale:.4}:h=ih/{target_scale:.4}:x=(iw-ow)/2:y=(ih-oh)/2"
    )
}

fn build_dynamic_zoom_filter(settings: VideoEffectSettings, crop_dimensions: (u32, u32)) -> String {
    let min = settings.zoom_min_scale;
    let max = settings.zoom_max_scale;
    let duration = settings.zoom_max_duration_seconds.max(1.0);
    let progress = format!("mod(t,{duration:.3})/{duration:.3}");
    let zoom = match settings.zoom_mode {
        DynamicZoomMode::Push => format!("{min:.4}+({max:.4}-{min:.4})*({progress})"),
        DynamicZoomMode::Pull => format!("{max:.4}-({max:.4}-{min:.4})*({progress})"),
        DynamicZoomMode::Random => format!("if(lt(sin(t),0),{min:.4}+({max:.4}-{min:.4})*({progress}),{max:.4}-({max:.4}-{min:.4})*({progress}))"),
    };
    let escaped_zoom = zoom.replace(',', "\\,");
    format!(
        "scale=w=iw*({escaped_zoom}):h=ih*({escaped_zoom}):eval=frame,crop=w={}:h={}:x=(iw-ow)/2:y=(ih-oh)/2",
        crop_dimensions.0, crop_dimensions.1
    )
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

fn build_segmented_playback_speed_plan(
    total_duration_seconds: f64,
    settings: PlaybackSpeedSettings,
    seed: u128,
) -> Result<Vec<PlaybackSpeedSegment>, String> {
    if !total_duration_seconds.is_finite() || total_duration_seconds <= 0.0 {
        return Err("分段变速需要有效的视频时长。".to_string());
    }

    let mut segments = Vec::new();
    let mut start_seconds = 0.0;
    let mut index = 0u128;
    while start_seconds < total_duration_seconds - 0.001 {
        let duration_unit = deterministic_unit(seed, index * 2);
        let speed_unit = deterministic_unit(seed, index * 2 + 1);
        let requested_duration = settings.segment_min_seconds
            + (settings.segment_max_seconds - settings.segment_min_seconds) * duration_unit;
        let end_seconds = (start_seconds + requested_duration).min(total_duration_seconds);
        let speed = settings.min + (settings.max - settings.min) * speed_unit;
        segments.push(PlaybackSpeedSegment {
            start_seconds,
            end_seconds,
            speed: normalize_playback_speed(speed)?,
        });
        start_seconds = end_seconds;
        index += 1;
    }

    if segments.is_empty() {
        return Err("没有生成可用的分段变速计划。".to_string());
    }
    Ok(segments)
}

fn deterministic_unit(seed: u128, index: u128) -> f64 {
    let mixed = seed
        .wrapping_add(index.wrapping_mul(6_364_136_223_846_793_005))
        .wrapping_mul(1_442_695_040_888_963_407);
    (mixed % 10_000) as f64 / 10_000.0
}

fn materialize_segmented_playback_source(
    input_list_path: &Path,
    output_path: &Path,
    segments: &[PlaybackSpeedSegment],
    has_audio: bool,
    task_context: Option<&TaskProgressContext>,
) -> Result<(), String> {
    let input_list_text = input_list_path
        .to_str()
        .ok_or_else(|| "分段变速输入列表路径包含无法识别的字符。".to_string())?;
    let output_path_text = output_path
        .to_str()
        .ok_or_else(|| "分段变速临时文件路径包含无法识别的字符。".to_string())?;

    let mut filters = Vec::with_capacity(segments.len() * if has_audio { 2 } else { 1 } + 1);
    for (index, segment) in segments.iter().enumerate() {
        filters.push(format!(
            "[0:v]trim=start={:.3}:end={:.3},setpts=(PTS-STARTPTS)/{:.3}[sv{index}]",
            segment.start_seconds, segment.end_seconds, segment.speed
        ));
        if has_audio {
            filters.push(format!(
                "[0:a]atrim=start={:.3}:end={:.3},asetpts=PTS-STARTPTS,atempo={:.3}[sa{index}]",
                segment.start_seconds, segment.end_seconds, segment.speed
            ));
        }
    }

    let concat_inputs = (0..segments.len())
        .map(|index| {
            if has_audio {
                format!("[sv{index}][sa{index}]")
            } else {
                format!("[sv{index}]")
            }
        })
        .collect::<String>();
    filters.push(format!(
        "{concat_inputs}concat=n={}:v=1:a={}[segv]{}",
        segments.len(),
        usize::from(has_audio),
        if has_audio { "[sega]" } else { "" }
    ));

    let mut args = vec![
        "-y".to_string(),
        "-f".to_string(),
        "concat".to_string(),
        "-safe".to_string(),
        "0".to_string(),
        "-i".to_string(),
        input_list_text.to_string(),
        "-filter_complex".to_string(),
        filters.join(";"),
        "-map".to_string(),
        "[segv]".to_string(),
    ];
    if has_audio {
        args.extend(["-map".to_string(), "[sega]".to_string()]);
    } else {
        args.push("-an".to_string());
    }
    args.extend([
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "veryfast".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
    ]);
    if has_audio {
        args.extend(["-c:a".to_string(), "aac".to_string()]);
    }
    args.push(output_path_text.to_string());

    let output_duration_seconds = segments
        .iter()
        .map(|segment| (segment.end_seconds - segment.start_seconds) / segment.speed)
        .sum::<f64>();
    run_ffmpeg(
        args,
        task_context,
        Some(output_duration_seconds),
        "分段变速处理失败。",
    )?;
    validate_rendered_video(output_path)
        .map_err(|error| format!("分段变速命令已结束，但临时视频校验失败：{error}"))?;
    Ok(())
}

fn materialize_segment_inputs(
    segment_paths: &[String],
    segment_inputs: Option<&[RemixSegmentInput]>,
    output_dir: &Path,
    timestamp: u128,
    task_context: Option<&TaskProgressContext>,
) -> Result<Vec<String>, String> {
    let Some(segment_inputs) = segment_inputs else {
        return Ok(segment_paths.to_vec());
    };
    if segment_inputs.len() != segment_paths.len() {
        return Err("素材截取配置数量与待拼接素材数量不一致。".to_string());
    }

    let mut prepared_paths = Vec::with_capacity(segment_inputs.len());
    for (index, (source_path, input)) in segment_paths.iter().zip(segment_inputs).enumerate() {
        if source_path != &input.path {
            return Err("素材截取配置与当前待拼接素材不匹配，请重新创建任务。".to_string());
        }

        let start_seconds = input.start_seconds.unwrap_or(0.0);
        let duration_seconds = input.duration_seconds;
        if !start_seconds.is_finite() || start_seconds < 0.0 {
            return Err("素材截取起点必须是大于或等于 0 的有效秒数。".to_string());
        }
        let Some(duration_seconds) = duration_seconds else {
            prepared_paths.push(source_path.clone());
            continue;
        };
        if !duration_seconds.is_finite() || duration_seconds < 0.5 {
            return Err("素材截取时长必须不少于 0.5 秒。".to_string());
        }

        let source_info = probe_segment_info(source_path)?;
        if start_seconds + duration_seconds > source_info.duration_seconds + 0.05 {
            return Err(format!(
                "素材截取范围超出视频时长：{}。",
                Path::new(source_path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(source_path)
            ));
        }
        if start_seconds <= 0.001 && duration_seconds >= source_info.duration_seconds - 0.05 {
            prepared_paths.push(source_path.clone());
            continue;
        }

        let clip_path = output_dir.join(format!("clip_segment_{timestamp}_{index}.mp4"));
        let clip_path_text = clip_path
            .to_str()
            .ok_or_else(|| "素材截取临时路径包含无法识别的字符。".to_string())?;
        let mut args = vec![
            "-y".to_string(),
            "-ss".to_string(),
            format!("{start_seconds:.3}"),
            "-i".to_string(),
            source_path.clone(),
            "-t".to_string(),
            format!("{duration_seconds:.3}"),
            "-map".to_string(),
            "0:v:0".to_string(),
            "-map".to_string(),
            "0:a?".to_string(),
            "-c:v".to_string(),
            "libx264".to_string(),
            "-preset".to_string(),
            "veryfast".to_string(),
            "-pix_fmt".to_string(),
            "yuv420p".to_string(),
            "-c:a".to_string(),
            "aac".to_string(),
            clip_path_text.to_string(),
        ];
        let clip_context = task_context.map(|context| {
            let start = index as f64 / segment_inputs.len() as f64 * 0.2;
            let end = (index + 1) as f64 / segment_inputs.len() as f64 * 0.2;
            context.child(
                start,
                end,
                format!("正在截取素材 {}/{}", index + 1, segment_inputs.len()),
            )
        });
        run_ffmpeg(
            std::mem::take(&mut args),
            clip_context.as_ref(),
            Some(duration_seconds),
            "素材截取失败。",
        )?;
        prepared_paths.push(clip_path.to_string_lossy().to_string());
    }

    Ok(prepared_paths)
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
    timestamp: u128,
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
    let duration_output = background_command(ffprobe_program())
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

    let audio_output = background_command(ffprobe_program())
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

fn resolve_playback_speed(
    fallback: f64,
    mut settings: PlaybackSpeedSettings,
    seed: u128,
) -> Result<f64, String> {
    if settings.mode == PlaybackSpeedMode::Global
        && (settings.min - 1.0).abs() < 0.001
        && (settings.max - 1.0).abs() < 0.001
        && should_apply_speed_filter(fallback)
    {
        settings.min = fallback;
        settings.max = fallback;
    }
    if !settings.min.is_finite()
        || !settings.max.is_finite()
        || settings.min < 0.5
        || settings.max > 2.0
        || settings.max < settings.min
        || !settings.segment_min_seconds.is_finite()
        || !settings.segment_max_seconds.is_finite()
        || settings.segment_min_seconds < 1.0
        || settings.segment_max_seconds < settings.segment_min_seconds
        || settings.segment_max_seconds > 120.0
    {
        return Err("变速范围或分段时长无效。".to_string());
    }
    let selected = if (settings.max - settings.min).abs() < f64::EPSILON {
        settings.min
    } else {
        let unit = (seed % 10_000) as f64 / 10_000.0;
        settings.min + (settings.max - settings.min) * unit
    };
    match settings.mode {
        PlaybackSpeedMode::Global | PlaybackSpeedMode::Segment => Ok(selected),
    }
}

fn random_in_range(minimum: f64, maximum: f64) -> f64 {
    if (maximum - minimum).abs() < f64::EPSILON {
        return minimum;
    }
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    minimum + (maximum - minimum) * ((seed % 10_000) as f64 / 10_000.0)
}

fn random_u32_in_range(minimum: u32, maximum: u32) -> u32 {
    if maximum <= minimum {
        return minimum.max(1);
    }
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    minimum + (seed % u128::from(maximum - minimum + 1)) as u32
}

fn random_visual_style() -> VideoVisualStyle {
    match (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.subsec_nanos())
        .unwrap_or_default())
        % 8
    {
        0 => VideoVisualStyle::Bw,
        1 => VideoVisualStyle::Invert,
        2 => VideoVisualStyle::Retro,
        3 => VideoVisualStyle::Cross,
        4 => VideoVisualStyle::Cartoon,
        5 => VideoVisualStyle::Emboss,
        6 => VideoVisualStyle::Pixel,
        _ => VideoVisualStyle::Outline,
    }
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

fn current_timestamp() -> Result<u128, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("无法生成拼接文件名：{error}"))
        .map(|duration| duration.as_nanos())
}

fn resolve_output_path(
    output_dir: &Path,
    requested_name: Option<&str>,
    output_settings: OutputSettings,
    timestamp: u128,
) -> Result<PathBuf, String> {
    let fallback = format!("remix_{timestamp}");
    let base_name = requested_name
        .map(sanitize_output_name)
        .filter(|name| !name.is_empty())
        .unwrap_or(fallback);
    let extension = output_settings.format.extension();

    for duplicate_index in 0..10_000 {
        let suffix = if duplicate_index == 0 {
            String::new()
        } else {
            format!("_{duplicate_index}")
        };
        let candidate = output_dir.join(format!("{base_name}{suffix}.{extension}"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }

    Err("Cannot allocate a unique output file name.".to_string())
}

fn sanitize_output_name(value: &str) -> String {
    value
        .trim()
        .trim_matches('.')
        .chars()
        .map(|character| match character {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ if character.is_control() => '_',
            _ => character,
        })
        .collect::<String>()
        .trim()
        .trim_matches('.')
        .to_string()
}

fn build_entrance_effect_filter(effect: EntranceEffect) -> Option<String> {
    let filter = match effect {
        EntranceEffect::None => return None,
        EntranceEffect::SmoothUp => "drawbox=x=0:y=0:w=iw:h='ih*(1-t/0.35)':color=black:t=fill:enable='lt(t,0.35)',fade=t=in:st=0:d=0.35",
        EntranceEffect::SmoothDown => "drawbox=x=0:y='ih*t/0.35':w=iw:h='ih*(1-t/0.35)':color=black:t=fill:enable='lt(t,0.35)',fade=t=in:st=0:d=0.35",
        EntranceEffect::HorizontalSqueeze => "drawbox=x=0:y=0:w='iw*(1-t/0.35)/2':h=ih:color=black:t=fill:enable='lt(t,0.35)',drawbox=x='iw*(1+t/0.35)/2':y=0:w='iw*(1-t/0.35)/2':h=ih:color=black:t=fill:enable='lt(t,0.35)',fade=t=in:st=0:d=0.35",
        EntranceEffect::HorizontalOpen => "drawbox=x=0:y=0:w='iw*(1-t/0.35)':h=ih:color=black:t=fill:enable='lt(t,0.35)',fade=t=in:st=0:d=0.35",
        EntranceEffect::VerticalSqueeze => "drawbox=x=0:y=0:w=iw:h='ih*(1-t/0.35)/2':color=black:t=fill:enable='lt(t,0.35)',drawbox=x=0:y='ih*(1+t/0.35)/2':w=iw:h='ih*(1-t/0.35)/2':color=black:t=fill:enable='lt(t,0.35)',fade=t=in:st=0:d=0.35",
        EntranceEffect::VerticalOpen => "drawbox=x=0:y='ih*(1+t/0.35)/2':w=iw:h='ih*(1-t/0.35)/2':color=black:t=fill:enable='lt(t,0.35)',fade=t=in:st=0:d=0.35",
        EntranceEffect::CircleCrop => "vignette=angle='PI/2*(1-t/0.35)':x0=0.5:y0=0.5:eval=frame:enable='lt(t,0.35)',fade=t=in:st=0:d=0.35",
        EntranceEffect::CircleOpen => "vignette=angle='PI/2*(1-t/0.35)':x0='0.35+0.15*t/0.35':y0=0.5:eval=frame:enable='lt(t,0.35)',fade=t=in:st=0:d=0.35",
        EntranceEffect::CircleClose => "vignette=angle='PI/2*t/0.35':eval=frame:enable='lt(t,0.35)',fade=t=in:st=0:d=0.35",
        EntranceEffect::RectangleCrop => "drawbox=x=0:y=0:w='iw*(1-t/0.35)/2':h=ih:color=black:t=fill:enable='lt(t,0.35)',drawbox=x=0:y=0:w=iw:h='ih*(1-t/0.35)/2':color=black:t=fill:enable='lt(t,0.35)',fade=t=in:st=0:d=0.35",
        EntranceEffect::HorizontalClose => "drawbox=x='iw*t/0.35/2':y=0:w='iw*(1-t/0.35)':h=ih:color=black:t=fill:enable='lt(t,0.35)',fade=t=in:st=0:d=0.35",
        EntranceEffect::VerticalClose => "drawbox=x=0:y=0:w=iw:h='ih*(1-t/0.35)/2':color=black:t=fill:enable='lt(t,0.35)',fade=t=in:st=0:d=0.35",
        EntranceEffect::LeftBottom => "drawbox=x=0:y='ih*(1-t/0.35)':w='iw*(1-t/0.35)':h='ih*t/0.35':color=black:t=fill:enable='lt(t,0.35)',fade=t=in:st=0:d=0.35",
        EntranceEffect::RightBottom => "drawbox=x='iw*t/0.35':y='ih*(1-t/0.35)':w='iw*(1-t/0.35)':h='ih*t/0.35':color=black:t=fill:enable='lt(t,0.35)',fade=t=in:st=0:d=0.35",
        EntranceEffect::LeftTop => "drawbox=x=0:y=0:w='iw*(1-t/0.35)':h='ih*(1-t/0.35)':color=black:t=fill:enable='lt(t,0.35)',fade=t=in:st=0:d=0.35",
        EntranceEffect::RightTop => "drawbox=x='iw*t/0.35':y=0:w='iw*(1-t/0.35)':h='ih*(1-t/0.35)':color=black:t=fill:enable='lt(t,0.35)',fade=t=in:st=0:d=0.35",
        EntranceEffect::HorizontalSlice => "drawbox=x=0:y='ih*(1-t/0.35)/3':w=iw:h='ih*(1-t/0.35)/3':color=black:t=fill:enable='lt(t,0.35)',fade=t=in:st=0:d=0.35",
        EntranceEffect::VerticalSlice => "drawbox=x='iw*(1-t/0.35)/3':y=0:w='iw*(1-t/0.35)/3':h=ih:color=black:t=fill:enable='lt(t,0.35)',fade=t=in:st=0:d=0.35",
    };
    Some(filter.to_string())
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
    use serde_json::json;

    #[test]
    fn composes_picture_in_picture_and_image_watermark() {
        let pip = PictureInPictureSettings {
            enabled: true,
            overlay_file_path: Some("pip.png".to_string()),
            position: PipPosition::TopLeft,
            size_ratio: 0.3,
            opacity: 0.8,
            margin: 20,
            ..PictureInPictureSettings::default()
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
            None,
            None,
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
    fn main_picture_in_picture_mode_uses_size_blur_and_offsets() {
        let settings = PictureInPictureSettings {
            enabled: true,
            overlay_file_path: Some("background.mp4".to_string()),
            mode: PipMode::Main,
            main_size_min: 0.8,
            main_size_max: 0.8,
            blur_min: 20.0,
            blur_max: 20.0,
            offset_x_min: 0.25,
            offset_x_max: 0.25,
            offset_y_min: 0.75,
            offset_y_max: 0.75,
            ..PictureInPictureSettings::default()
        };
        let filter = build_pip_layer_filter("[base]", "[out]", 2, &settings, 1);
        assert!(filter.contains("boxblur=luma_radius=2.000"));
        assert!(filter.contains("scale=iw*0.800:ih*0.800"));
        assert!(filter.contains("(W-w)*0.250"));
        assert!(filter.contains("(H-h)*0.750"));
    }

    #[test]
    fn audio_ranges_fades_dynamic_adjustment_and_padding_enter_filters() {
        let settings: BgmSettings = serde_json::from_value(json!({
            "enabled": true,
            "audioFilePath": "music.mp3",
            "originalVolume": 1.0,
            "originalVolumeMin": 0.8,
            "originalVolumeMax": 0.8,
            "originalFadeEnabled": true,
            "dynamicAdjustEnabled": true,
            "bgmVolume": 0.4,
            "bgmVolumeMin": 0.4,
            "bgmVolumeMax": 0.4,
            "bgmFadeEnabled": true,
            "loopPlaybackEnabled": false,
            "fadeInSeconds": 0.5,
            "fadeOutSeconds": 0.5
        }))
        .unwrap();
        let filter = build_bgm_audio_filter(&settings, 1, true, 1.0, 5.0);
        assert!(filter.contains("volume=0.800,dynaudnorm=f=150:g=5"));
        assert!(filter.contains("afade=t=in:st=0:d=0.500"));
        assert!(filter.contains("volume=0.400"));
        assert!(filter.contains("apad=whole_dur=5.000"));
        assert!(filter.contains("atrim=duration=5.000"));
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
            None,
            None,
            None,
        )
        .unwrap()
        .unwrap();

        assert!(filter.filter.contains("[0:v]scale=640:360[layer0]"));
        assert!(filter.filter.contains("drawtext="));
        assert!(filter.filter.contains("x=(w-text_w)/2:y=(h-text_h)/2"));
    }

    #[test]
    fn composes_subtitle_after_base_filter() {
        let base = CanvasFilter {
            filter: "scale=1080:1920".to_string(),
            is_complex: false,
        };

        let filter = build_composed_video_filter(
            Some(base),
            None,
            None,
            None,
            None,
            None,
            None,
            Some("ass=filename='subtitle.ass'"),
            None,
            None,
        )
        .unwrap()
        .unwrap();

        assert!(filter.is_complex);
        assert!(filter.filter.contains("[0:v]scale=1080:1920[layer0]"));
        assert!(filter
            .filter
            .contains("[layer0]ass=filename='subtitle.ass'[layer1]"));
        assert!(filter.filter.ends_with("format=yuv420p[v]"));
    }

    #[test]
    fn rejects_enabled_subtitle_without_text() {
        let result = normalize_subtitle_settings(SubtitleSettings {
            enabled: true,
            text: "  ".to_string(),
            position: NarratedSubtitlePosition::Bottom,
            size: NarratedSubtitleSize::Medium,
            font_family: default_subtitle_font_family(),
            text_color: default_subtitle_text_color(),
            opacity: default_subtitle_opacity(),
        });

        assert!(result.is_err());
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
            ..PictureInPictureSettings::default()
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
            None,
            None,
            None,
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

    #[test]
    fn builds_a_crop_filter_from_the_preview_crop_settings() {
        let settings = VideoEffectSettings {
            vertical_mirror: false,
            rotation: RotationMode::None,
            hsl_enabled: false,
            hue: 0.0,
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            scale: 1.0,
            crop: CanvasCropSettings {
                enabled: true,
                x: 10.0,
                y: 5.0,
                width: 80.0,
                height: 90.0,
            },
            zoom_enabled: false,
            zoom_mode: DynamicZoomMode::Push,
            zoom_min_scale: 1.02,
            zoom_max_scale: 1.08,
            zoom_min_duration_seconds: 8.0,
            zoom_max_duration_seconds: 10.0,
            ..VideoEffectSettings::default()
        };

        let filters = build_video_effect_filters(settings, None);
        assert!(filters
            .iter()
            .any(|filter| filter.contains("iw*80.000000/100")));
        assert!(filters
            .iter()
            .any(|filter| filter.contains("ih*90.000000/100")));
    }

    #[test]
    fn rejects_crop_settings_that_extend_past_the_source_frame() {
        let result = normalize_video_effect_settings(VideoEffectSettings {
            vertical_mirror: false,
            rotation: RotationMode::None,
            hsl_enabled: false,
            hue: 0.0,
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            scale: 1.0,
            crop: CanvasCropSettings {
                enabled: true,
                x: 50.0,
                y: 0.0,
                width: 60.0,
                height: 100.0,
            },
            zoom_enabled: false,
            zoom_mode: DynamicZoomMode::Push,
            zoom_min_scale: 1.02,
            zoom_max_scale: 1.08,
            zoom_min_duration_seconds: 8.0,
            zoom_max_duration_seconds: 10.0,
            ..VideoEffectSettings::default()
        });

        assert!(result.is_err());
    }

    #[test]
    fn frame_insert_filter_uses_the_material_stream_and_requested_interval() {
        let filter = build_frame_insert_layer_filter(
            "[base]",
            "[output]",
            2,
            &FrameOperationSettings {
                enabled: true,
                mode: FrameOperationMode::Insert,
                interval_min: 45,
                interval_max: 45,
                frame_min: 3,
                frame_max: 3,
                opacity: 35.0,
                material_file_path: Some("frame.png".to_string()),
            },
            1,
        );

        assert!(filter.contains("[2:v]format=rgba"));
        assert!(filter.contains("overlay=0:0"));
        assert!(filter.contains("mod(n\\,45)"));
        assert!(filter.contains("lt(mod(n\\,45)\\,3)"));
        assert!(filter.contains("aa=0.350"));
    }

    #[test]
    fn fusion_filter_blends_the_second_input_at_the_configured_strength() {
        let filter = build_fusion_layer_filter(
            "[base]",
            "[output]",
            3,
            &FusionSettings {
                enabled: true,
                material_file_path: Some("fusion.mp4".to_string()),
                interval_min: 30,
                interval_max: 60,
                strength: 0.42,
            },
            2,
        );

        assert!(filter.contains("[3:v]format=rgba"));
        assert!(filter.contains("blend=all_mode=overlay"));
        assert!(filter.contains("all_opacity=0.420"));
        assert!(filter.contains("aa=0.420"));
    }

    #[test]
    fn rejects_invalid_frame_and_fusion_ranges_before_export() {
        let frame_error = normalize_frame_operation_settings(FrameOperationSettings {
            enabled: true,
            mode: FrameOperationMode::Extract,
            interval_min: 80,
            interval_max: 20,
            frame_min: 1,
            frame_max: 1,
            opacity: 50.0,
            material_file_path: None,
        })
        .unwrap_err();
        let fusion_error = normalize_fusion_settings(FusionSettings {
            enabled: true,
            material_file_path: None,
            interval_min: 20,
            interval_max: 10,
            strength: 1.2,
        })
        .unwrap_err();

        assert!(frame_error.contains("参数无效"));
        assert!(fusion_error.contains("参数无效"));
    }

    #[test]
    fn every_entrance_effect_has_a_distinct_filter() {
        let effects = [
            EntranceEffect::SmoothUp,
            EntranceEffect::SmoothDown,
            EntranceEffect::HorizontalSqueeze,
            EntranceEffect::VerticalSqueeze,
            EntranceEffect::CircleCrop,
            EntranceEffect::RectangleCrop,
            EntranceEffect::CircleClose,
            EntranceEffect::CircleOpen,
            EntranceEffect::HorizontalClose,
            EntranceEffect::HorizontalOpen,
            EntranceEffect::VerticalClose,
            EntranceEffect::VerticalOpen,
            EntranceEffect::LeftBottom,
            EntranceEffect::RightBottom,
            EntranceEffect::LeftTop,
            EntranceEffect::RightTop,
            EntranceEffect::HorizontalSlice,
            EntranceEffect::VerticalSlice,
        ];
        let filters = effects
            .into_iter()
            .map(|effect| build_entrance_effect_filter(effect).unwrap())
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(filters.len(), effects.len());
    }

    #[test]
    fn segmented_speed_plan_covers_the_whole_video_and_changes_speed() {
        let plan = build_segmented_playback_speed_plan(
            12.0,
            PlaybackSpeedSettings {
                mode: PlaybackSpeedMode::Segment,
                min: 0.8,
                max: 1.2,
                segment_min_seconds: 2.0,
                segment_max_seconds: 3.0,
            },
            42,
        )
        .unwrap();

        assert!(plan.len() >= 4);
        assert!((plan.first().unwrap().start_seconds - 0.0).abs() < 0.001);
        assert!((plan.last().unwrap().end_seconds - 12.0).abs() < 0.001);
        assert!(plan
            .windows(2)
            .all(|pair| { (pair[0].end_seconds - pair[1].start_seconds).abs() < 0.001 }));
        assert!(plan
            .iter()
            .any(|segment| (segment.speed - plan[0].speed).abs() > 0.001));
    }

    #[test]
    fn entrance_effects_and_segment_input_protocol_have_explicit_validation() {
        assert!(build_entrance_effect_filter(EntranceEffect::None).is_none());
        assert!(build_entrance_effect_filter(EntranceEffect::SmoothUp)
            .unwrap()
            .contains("fade=t=in"));
        let error = materialize_segment_inputs(
            &["one.mp4".to_string(), "two.mp4".to_string()],
            Some(&[RemixSegmentInput {
                path: "one.mp4".to_string(),
                start_seconds: Some(0.0),
                duration_seconds: Some(2.0),
            }]),
            Path::new("."),
            1,
            None,
        )
        .unwrap_err();
        assert!(error.contains("数量与待拼接素材数量不一致"));
    }

    #[test]
    fn exports_real_trimmed_segment_inputs_through_the_video_engine() {
        use crate::video_engine::output::OutputFormat;
        use crate::video_engine::tool_paths::ffmpeg_program;
        use std::process::Command;

        let root = std::env::temp_dir().join(format!(
            "local-video-remix-mix-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        fs::create_dir_all(&root).unwrap();
        let source = root.join("source.mp4");
        let source_text = source.to_string_lossy().to_string();
        let generated = Command::new(ffmpeg_program())
            .args([
                "-y",
                "-f",
                "lavfi",
                "-i",
                "testsrc2=size=320x180:rate=30",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:sample_rate=44100",
                "-t",
                "3",
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
                "-c:a",
                "aac",
                &source_text,
            ])
            .output()
            .unwrap();
        assert!(
            generated.status.success(),
            "{}",
            String::from_utf8_lossy(&generated.stderr)
        );

        let settings: RemixSettings = serde_json::from_value(json!({
            "applyHorizontalMirror": false,
            "playbackSpeed": 1.0,
            "canvasAspectRatio": "original",
            "canvasBackgroundMode": "black",
            "smoothRemixEnabled": false,
            "videoEffectSettings": {
                "verticalMirror": false,
                "rotation": "none",
                "hslEnabled": false,
                "hue": 0.0,
                "brightness": 0.0,
                "contrast": 1.0,
                "saturation": 1.0,
                "scale": 1.0,
                "crop": { "enabled": false, "x": 0.0, "y": 0.0, "width": 100.0, "height": 100.0 },
                "zoomEnabled": false,
                "zoomMode": "push",
                "zoomMinScale": 1.02,
                "zoomMaxScale": 1.08,
                "zoomMinDurationSeconds": 8.0,
                "zoomMaxDurationSeconds": 10.0
            },
            "pictureInPictureSettings": { "enabled": false, "overlayFilePath": null, "position": "center", "sizeRatio": 0.3, "opacity": 1.0, "margin": 0 },
            "bgmSettings": { "enabled": false, "audioFilePath": null, "originalVolume": 1.0, "bgmVolume": 0.5, "fadeInSeconds": 0.0, "fadeOutSeconds": 0.0 },
            "subtitleSettings": { "enabled": false, "text": "", "position": "bottom", "size": "medium" },
            "outputSettings": { "format": "mp4", "resolution": "followCanvas", "frameRate": "source", "quality": "standard", "encoder": "cpu" },
            "entranceEffect": "none",
            "frameOperationSettings": { "enabled": false },
            "fusionSettings": { "enabled": false }
        }))
        .unwrap();

        for (format, extension) in [
            (OutputFormat::Mp4, "mp4"),
            (OutputFormat::Mov, "mov"),
            (OutputFormat::Webm, "webm"),
        ] {
            let mut format_settings = settings.clone();
            format_settings.output_settings.format = format;
            let result = concat_video_segments(
                vec![source_text.clone(), source_text.clone()],
                root.to_string_lossy().to_string(),
                format_settings,
                None,
                Some(vec![
                    RemixSegmentInput {
                        path: source_text.clone(),
                        start_seconds: Some(0.25),
                        duration_seconds: Some(1.0),
                    },
                    RemixSegmentInput {
                        path: source_text.clone(),
                        start_seconds: Some(1.25),
                        duration_seconds: Some(1.0),
                    },
                ]),
            )
            .unwrap();
            assert!(result.output_path.ends_with(extension));
            let output = probe_segment_info(&result.output_path).unwrap();
            assert!(
                (output.duration_seconds - 2.0).abs() < 0.25,
                "实际成片时长为 {} 秒",
                output.duration_seconds
            );
        }
        let speed_input_list = root.join("segmented_speed_input.txt");
        fs::write(
            &speed_input_list,
            build_concat_list_content(std::slice::from_ref(&source_text)),
        )
        .unwrap();
        let segmented_speed_output = root.join("segmented_speed_output.mp4");
        materialize_segmented_playback_source(
            &speed_input_list,
            &segmented_speed_output,
            &[
                PlaybackSpeedSegment {
                    start_seconds: 0.0,
                    end_seconds: 1.0,
                    speed: 2.0,
                },
                PlaybackSpeedSegment {
                    start_seconds: 1.0,
                    end_seconds: 3.0,
                    speed: 0.5,
                },
            ],
            true,
            None,
        )
        .unwrap();
        let segmented_info =
            probe_segment_info(segmented_speed_output.to_string_lossy().as_ref()).unwrap();
        assert!(segmented_info.has_audio);
        assert!(
            (segmented_info.duration_seconds - 4.5).abs() < 0.3,
            "segmented speed duration was {} seconds",
            segmented_info.duration_seconds
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn exports_workspace_acceptance_artifacts_when_requested() {
        let Ok(source) = std::env::var("PROJECT03_ACCEPTANCE_SOURCE") else {
            return;
        };
        let portrait = std::env::var("PROJECT03_ACCEPTANCE_PORTRAIT").unwrap();
        let bgm = std::env::var("PROJECT03_ACCEPTANCE_BGM").unwrap();
        let pip_image = std::env::var("PROJECT03_ACCEPTANCE_PIP_IMAGE").unwrap();
        let output_root = PathBuf::from(std::env::var("PROJECT03_ACCEPTANCE_OUTPUT").unwrap());
        fs::create_dir_all(&output_root).unwrap();

        let base_settings = || -> RemixSettings {
            serde_json::from_value(json!({
                "applyHorizontalMirror": false,
                "playbackSpeed": 1.0,
                "playbackSpeedSettings": { "mode": "global", "min": 1.0, "max": 1.0, "segmentMinSeconds": 1.0, "segmentMaxSeconds": 2.0 },
                "canvasAspectRatio": "original",
                "canvasBackgroundMode": "black",
                "smoothRemixEnabled": false,
                "videoEffectSettings": {
                    "verticalMirror": false,
                    "rotation": "none",
                    "hslEnabled": false,
                    "hue": 0.0,
                    "brightness": 0.0,
                    "contrast": 1.0,
                    "saturation": 1.0,
                    "scale": 1.0,
                    "crop": { "enabled": false, "x": 0.0, "y": 0.0, "width": 100.0, "height": 100.0 },
                    "zoomEnabled": false,
                    "zoomMode": "push",
                    "zoomMinScale": 1.02,
                    "zoomMaxScale": 1.08,
                    "zoomMinDurationSeconds": 8.0,
                    "zoomMaxDurationSeconds": 10.0
                },
                "pictureInPictureSettings": { "enabled": false, "overlayFilePath": null, "mode": "external", "position": "center", "sizeRatio": 0.3, "opacity": 1.0, "margin": 0 },
                "bgmSettings": { "enabled": false, "audioFilePath": null, "originalVolume": 1.0, "originalVolumeMin": 1.0, "originalVolumeMax": 1.0, "originalFadeEnabled": false, "dynamicAdjustEnabled": false, "bgmVolume": 0.5, "bgmVolumeMin": 0.5, "bgmVolumeMax": 0.5, "bgmFadeEnabled": false, "loopPlaybackEnabled": true, "fadeInSeconds": 0.0, "fadeOutSeconds": 0.0 },
                "subtitleSettings": { "enabled": false, "text": "", "position": "bottom", "size": "medium", "fontFamily": "Microsoft YaHei", "textColor": "#ffffff", "opacity": 1.0 },
                "outputSettings": { "format": "mp4", "resolution": "followCanvas", "frameRate": "source", "quality": "standard", "encoder": "cpu" },
                "entranceEffect": "none",
                "frameOperationSettings": { "enabled": false },
                "fusionSettings": { "enabled": false }
            }))
            .unwrap()
        };

        let export = |name: &str, inputs: Vec<String>, settings: RemixSettings| {
            let directory = output_root.join(name);
            fs::create_dir_all(&directory).unwrap();
            let result = concat_video_segments(
                inputs,
                directory.to_string_lossy().to_string(),
                settings,
                None,
                None,
            )
            .unwrap();
            let info = probe_segment_info(&result.output_path).unwrap();
            assert!(info.duration_seconds > 0.5);
            println!("{name}: {}", result.output_path);
            result.output_path
        };

        let mut segmented_speed = base_settings();
        segmented_speed.output_name = Some("01_分段变速".to_string());
        segmented_speed.playback_speed_settings.mode = PlaybackSpeedMode::Segment;
        segmented_speed.playback_speed_settings.min = 0.65;
        segmented_speed.playback_speed_settings.max = 1.35;
        export(
            "01_分段变速",
            vec![source.clone(), source.clone()],
            segmented_speed,
        );

        for (name, loop_playback) in [("02_BGM循环", true), ("03_BGM不循环", false)] {
            let mut settings = base_settings();
            settings.output_name = Some(name.to_string());
            settings.bgm_settings.enabled = true;
            settings.bgm_settings.audio_file_path = Some(bgm.clone());
            settings.bgm_settings.original_volume = 0.7;
            settings.bgm_settings.original_volume_min = 0.7;
            settings.bgm_settings.original_volume_max = 0.7;
            settings.bgm_settings.dynamic_adjust_enabled = true;
            settings.bgm_settings.bgm_volume = 0.35;
            settings.bgm_settings.bgm_volume_min = 0.35;
            settings.bgm_settings.bgm_volume_max = 0.35;
            settings.bgm_settings.bgm_fade_enabled = true;
            settings.bgm_settings.loop_playback_enabled = loop_playback;
            settings.bgm_settings.fade_in_seconds = 0.5;
            settings.bgm_settings.fade_out_seconds = 0.5;
            let output = export(name, vec![source.clone(), source.clone()], settings);
            let info = probe_segment_info(&output).unwrap();
            assert!(info.has_audio);
            assert!(info.duration_seconds > 7.5);
        }

        let mut main_pip = base_settings();
        main_pip.output_name = Some("04_主视频画中画".to_string());
        main_pip.picture_in_picture_settings.enabled = true;
        main_pip.picture_in_picture_settings.mode = PipMode::Main;
        main_pip.picture_in_picture_settings.overlay_file_path = Some(portrait.clone());
        main_pip.picture_in_picture_settings.main_size_min = 0.78;
        main_pip.picture_in_picture_settings.main_size_max = 0.78;
        main_pip.picture_in_picture_settings.blur_min = 35.0;
        main_pip.picture_in_picture_settings.blur_max = 35.0;
        main_pip.picture_in_picture_settings.offset_x_min = 0.25;
        main_pip.picture_in_picture_settings.offset_x_max = 0.25;
        main_pip.picture_in_picture_settings.offset_y_min = 0.75;
        main_pip.picture_in_picture_settings.offset_y_max = 0.75;
        export(
            "04_主视频画中画",
            vec![source.clone(), source.clone()],
            main_pip,
        );

        let mut external_pip = base_settings();
        external_pip.output_name = Some("05_外部素材画中画".to_string());
        external_pip.picture_in_picture_settings.enabled = true;
        external_pip.picture_in_picture_settings.overlay_file_path = Some(pip_image);
        external_pip.picture_in_picture_settings.position = PipPosition::BottomRight;
        external_pip.picture_in_picture_settings.size_ratio = 0.35;
        external_pip.picture_in_picture_settings.opacity = 0.8;
        export(
            "05_外部素材画中画",
            vec![source.clone(), source.clone()],
            external_pip,
        );

        let mut crop_subtitle = base_settings();
        crop_subtitle.output_name = Some("06_裁剪与字幕样式".to_string());
        crop_subtitle.video_effect_settings.crop = CanvasCropSettings {
            enabled: true,
            x: 12.0,
            y: 8.0,
            width: 76.0,
            height: 82.0,
        };
        crop_subtitle.subtitle_settings.enabled = true;
        crop_subtitle.subtitle_settings.text = "真实字幕颜色与透明度验收".to_string();
        crop_subtitle.subtitle_settings.font_family = "SimHei".to_string();
        crop_subtitle.subtitle_settings.text_color = "#f08c57".to_string();
        crop_subtitle.subtitle_settings.opacity = 0.65;
        export(
            "06_裁剪与字幕样式",
            vec![source.clone(), source],
            crop_subtitle,
        );

        let mut portrait_settings = base_settings();
        portrait_settings.output_name = Some("07_竖屏原比例".to_string());
        let output = export(
            "07_竖屏原比例",
            vec![portrait.clone(), portrait],
            portrait_settings,
        );
        let dimensions = probe_video_dimensions(&output).unwrap();
        assert!(dimensions.1 > dimensions.0);
    }
}
