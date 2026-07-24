use crate::video_engine::canvas::CanvasAspectRatio;
use crate::video_engine::tool_paths::{background_command, ffmpeg_program};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OutputFormat {
    Mp4,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OutputResolution {
    FollowCanvas,
    Hd720,
    FullHd1080,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OutputFrameRate {
    Source,
    Fps24,
    Fps25,
    Fps30,
    Fps50,
    Fps60,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OutputQuality {
    Compact,
    Standard,
    High,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum VideoEncoder {
    Auto,
    Cpu,
    Nvidia,
    Intel,
    Amd,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct OutputSettings {
    pub format: OutputFormat,
    pub resolution: OutputResolution,
    pub frame_rate: OutputFrameRate,
    pub quality: OutputQuality,
    pub encoder: VideoEncoder,
}

impl Default for OutputSettings {
    fn default() -> Self {
        Self {
            format: OutputFormat::Mp4,
            resolution: OutputResolution::FollowCanvas,
            frame_rate: OutputFrameRate::Source,
            quality: OutputQuality::Standard,
            encoder: VideoEncoder::Auto,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncoderCapability {
    encoder: VideoEncoder,
    label: String,
    available: bool,
    detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncoderCapabilities {
    encoders: Vec<EncoderCapability>,
    recommended_encoder: VideoEncoder,
    message: String,
}

#[derive(Debug, Clone)]
pub struct AppliedOutputSettings {
    pub encoder_label: String,
    pub resolution_label: String,
    pub frame_rate_label: String,
    pub quality_label: String,
    pub video_bitrate_kbps: u32,
}

static ENCODER_CAPABILITIES: OnceLock<Mutex<Option<EncoderCapabilities>>> = OnceLock::new();

pub fn detect_video_encoder_capabilities(force_refresh: bool) -> EncoderCapabilities {
    let cache = ENCODER_CAPABILITIES.get_or_init(|| Mutex::new(None));
    if let Ok(mut cached) = cache.lock() {
        if !force_refresh {
            if let Some(capabilities) = cached.as_ref() {
                return capabilities.clone();
            }
        }
        let capabilities = detect_video_encoder_capabilities_uncached();
        *cached = Some(capabilities.clone());
        return capabilities;
    }
    detect_video_encoder_capabilities_uncached()
}

pub fn build_output_video_filters(
    settings: OutputSettings,
    canvas_aspect_ratio: CanvasAspectRatio,
) -> Vec<String> {
    if canvas_aspect_ratio != CanvasAspectRatio::Original {
        return Vec::new();
    }

    match settings.resolution {
        OutputResolution::FollowCanvas => Vec::new(),
        OutputResolution::Hd720 => vec![build_original_scale_filter(1280, 720)],
        OutputResolution::FullHd1080 => vec![build_original_scale_filter(1920, 1080)],
    }
}

pub fn output_canvas_dimensions(
    settings: OutputSettings,
    canvas_aspect_ratio: CanvasAspectRatio,
) -> Option<(u32, u32)> {
    if canvas_aspect_ratio == CanvasAspectRatio::Original {
        return None;
    }

    match settings.resolution {
        OutputResolution::FollowCanvas | OutputResolution::FullHd1080 => {
            canvas_aspect_ratio.dimensions()
        }
        OutputResolution::Hd720 => match canvas_aspect_ratio {
            CanvasAspectRatio::Portrait916 => Some((720, 1280)),
            CanvasAspectRatio::Square11 => Some((720, 720)),
            CanvasAspectRatio::Landscape169 => Some((1280, 720)),
            CanvasAspectRatio::Original => None,
        },
    }
}

pub fn resolve_output_video_dimensions(
    settings: OutputSettings,
    canvas_aspect_ratio: CanvasAspectRatio,
    source_dimensions: (u32, u32),
) -> (u32, u32) {
    if let Some(dimensions) = output_canvas_dimensions(settings, canvas_aspect_ratio) {
        return dimensions;
    }

    match settings.resolution {
        OutputResolution::FollowCanvas => source_dimensions,
        OutputResolution::Hd720 => scale_dimensions_to_fit(source_dimensions, 1280, 720),
        OutputResolution::FullHd1080 => scale_dimensions_to_fit(source_dimensions, 1920, 1080),
    }
}

fn scale_dimensions_to_fit(
    source_dimensions: (u32, u32),
    landscape_width: u32,
    landscape_height: u32,
) -> (u32, u32) {
    let (source_width, source_height) = source_dimensions;
    let (target_width, target_height) = if source_width >= source_height {
        (landscape_width, landscape_height)
    } else {
        (landscape_height, landscape_width)
    };
    let scale = (f64::from(target_width) / f64::from(source_width))
        .min(f64::from(target_height) / f64::from(source_height));
    let width = ((f64::from(source_width) * scale / 2.0).floor() as u32 * 2).max(2);
    let height = ((f64::from(source_height) * scale / 2.0).floor() as u32 * 2).max(2);
    (width, height)
}

pub fn append_final_output_args(
    ffmpeg_args: &mut Vec<String>,
    settings: OutputSettings,
    canvas_aspect_ratio: CanvasAspectRatio,
) -> Result<AppliedOutputSettings, String> {
    let capabilities = detect_video_encoder_capabilities(false);
    let encoder = resolve_encoder(settings.encoder, &capabilities)?;
    let bitrate_kbps = target_bitrate_kbps(settings.quality, settings.resolution);

    ffmpeg_args.extend(["-c:v".to_string(), encoder.ffmpeg_name().to_string()]);
    append_encoder_preset(ffmpeg_args, encoder);
    ffmpeg_args.extend([
        "-b:v".to_string(),
        format!("{bitrate_kbps}k"),
        "-maxrate".to_string(),
        format!("{}k", bitrate_kbps * 5 / 4),
        "-bufsize".to_string(),
        format!("{}k", bitrate_kbps * 2),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
    ]);

    if let Some(frame_rate) = settings.frame_rate.value() {
        ffmpeg_args.extend(["-r".to_string(), frame_rate.to_string()]);
    }

    ffmpeg_args.extend([
        "-c:a".to_string(),
        "aac".to_string(),
        "-b:a".to_string(),
        "192k".to_string(),
        "-movflags".to_string(),
        "+faststart".to_string(),
    ]);

    Ok(AppliedOutputSettings {
        encoder_label: encoder.label().to_string(),
        resolution_label: resolution_label(settings.resolution, canvas_aspect_ratio),
        frame_rate_label: settings.frame_rate.label().to_string(),
        quality_label: settings.quality.label().to_string(),
        video_bitrate_kbps: bitrate_kbps,
    })
}

fn detect_video_encoder_capabilities_uncached() -> EncoderCapabilities {
    let encoders = [
        VideoEncoder::Cpu,
        VideoEncoder::Nvidia,
        VideoEncoder::Intel,
        VideoEncoder::Amd,
    ]
    .into_iter()
    .map(test_encoder_capability)
    .collect::<Vec<_>>();
    let recommended_encoder = [VideoEncoder::Nvidia, VideoEncoder::Intel, VideoEncoder::Amd]
        .into_iter()
        .find(|encoder| capability_available(&encoders, *encoder))
        .unwrap_or(VideoEncoder::Cpu);
    let gpu_count = encoders
        .iter()
        .filter(|capability| capability.encoder != VideoEncoder::Cpu && capability.available)
        .count();

    EncoderCapabilities {
        encoders,
        recommended_encoder,
        message: if gpu_count > 0 {
            format!(
                "检测到 {gpu_count} 个可用GPU编码器，自动模式优先使用{}。",
                recommended_encoder.label()
            )
        } else {
            "未检测到可实际使用的GPU编码器，自动模式将使用CPU。".to_string()
        },
    }
}

fn test_encoder_capability(encoder: VideoEncoder) -> EncoderCapability {
    let output = background_command(ffmpeg_program())
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=c=black:s=320x240:r=10:d=0.1",
            "-frames:v",
            "1",
            "-an",
            "-c:v",
            encoder.ffmpeg_name(),
            "-f",
            "null",
            "-",
        ])
        .output();

    match output {
        Ok(output) if output.status.success() => EncoderCapability {
            encoder,
            label: encoder.label().to_string(),
            available: true,
            detail: "实际编码测试通过".to_string(),
        },
        Ok(output) => EncoderCapability {
            encoder,
            label: encoder.label().to_string(),
            available: false,
            detail: summarize_encoder_error(&String::from_utf8_lossy(&output.stderr)),
        },
        Err(error) => EncoderCapability {
            encoder,
            label: encoder.label().to_string(),
            available: false,
            detail: format!("无法启动FFmpeg：{error}"),
        },
    }
}

fn summarize_encoder_error(stderr: &str) -> String {
    let normalized = stderr.to_ascii_lowercase();
    if normalized.contains("unknown encoder") || normalized.contains("not found") {
        return "当前FFmpeg不包含这个编码器".to_string();
    }
    if normalized.contains("no capable devices")
        || normalized.contains("no device")
        || normalized.contains("cannot load")
        || normalized.contains("device setup failed")
        || normalized.contains("unsupported device")
    {
        return "未检测到对应显卡，或显卡驱动暂不可用".to_string();
    }
    "实际编码测试未通过".to_string()
}

fn resolve_encoder(
    requested: VideoEncoder,
    capabilities: &EncoderCapabilities,
) -> Result<VideoEncoder, String> {
    let resolved = if requested == VideoEncoder::Auto {
        capabilities.recommended_encoder
    } else {
        requested
    };

    if capability_available(&capabilities.encoders, resolved) {
        Ok(resolved)
    } else {
        Err(format!(
            "当前电脑无法使用{}，请重新检测或改用自动/CPU编码。",
            resolved.label()
        ))
    }
}

fn capability_available(capabilities: &[EncoderCapability], encoder: VideoEncoder) -> bool {
    capabilities
        .iter()
        .any(|capability| capability.encoder == encoder && capability.available)
}

fn append_encoder_preset(ffmpeg_args: &mut Vec<String>, encoder: VideoEncoder) {
    match encoder {
        VideoEncoder::Cpu => {
            ffmpeg_args.extend(["-preset".to_string(), "veryfast".to_string()]);
        }
        VideoEncoder::Nvidia => {
            ffmpeg_args.extend(["-preset".to_string(), "p4".to_string()]);
        }
        VideoEncoder::Intel => {
            ffmpeg_args.extend(["-preset".to_string(), "veryfast".to_string()]);
        }
        VideoEncoder::Amd => {
            ffmpeg_args.extend(["-quality".to_string(), "speed".to_string()]);
        }
        VideoEncoder::Auto => {}
    }
}

fn target_bitrate_kbps(quality: OutputQuality, resolution: OutputResolution) -> u32 {
    let is_720 = resolution == OutputResolution::Hd720;
    match (quality, is_720) {
        (OutputQuality::Compact, true) => 2_500,
        (OutputQuality::Standard, true) => 4_500,
        (OutputQuality::High, true) => 6_500,
        (OutputQuality::Compact, false) => 5_000,
        (OutputQuality::Standard, false) => 8_000,
        (OutputQuality::High, false) => 12_000,
    }
}

fn build_original_scale_filter(landscape_width: u32, landscape_height: u32) -> String {
    format!(
        "scale=w='if(gte(iw\\,ih)\\,{landscape_width}\\,{landscape_height})':h='if(gte(iw\\,ih)\\,{landscape_height}\\,{landscape_width})':force_original_aspect_ratio=decrease:force_divisible_by=2"
    )
}

fn resolution_label(
    resolution: OutputResolution,
    canvas_aspect_ratio: CanvasAspectRatio,
) -> String {
    match resolution {
        OutputResolution::FollowCanvas => canvas_aspect_ratio
            .dimensions()
            .map(|(width, height)| format!("{width}x{height}"))
            .unwrap_or_else(|| "跟随原画".to_string()),
        OutputResolution::Hd720 => output_canvas_dimensions(
            OutputSettings {
                resolution,
                ..OutputSettings::default()
            },
            canvas_aspect_ratio,
        )
        .map(|(width, height)| format!("{width}x{height}"))
        .unwrap_or_else(|| "720p（保持原比例）".to_string()),
        OutputResolution::FullHd1080 => output_canvas_dimensions(
            OutputSettings {
                resolution,
                ..OutputSettings::default()
            },
            canvas_aspect_ratio,
        )
        .map(|(width, height)| format!("{width}x{height}"))
        .unwrap_or_else(|| "1080p（保持原比例）".to_string()),
    }
}

impl OutputFrameRate {
    fn value(self) -> Option<u32> {
        match self {
            Self::Source => None,
            Self::Fps24 => Some(24),
            Self::Fps25 => Some(25),
            Self::Fps30 => Some(30),
            Self::Fps50 => Some(50),
            Self::Fps60 => Some(60),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Source => "跟随源视频",
            Self::Fps24 => "24 fps",
            Self::Fps25 => "25 fps",
            Self::Fps30 => "30 fps",
            Self::Fps50 => "50 fps",
            Self::Fps60 => "60 fps",
        }
    }
}

impl OutputQuality {
    fn label(self) -> &'static str {
        match self {
            Self::Compact => "省空间",
            Self::Standard => "标准",
            Self::High => "高清",
        }
    }
}

impl VideoEncoder {
    fn ffmpeg_name(self) -> &'static str {
        match self {
            Self::Auto | Self::Cpu => "libx264",
            Self::Nvidia => "h264_nvenc",
            Self::Intel => "h264_qsv",
            Self::Amd => "h264_amf",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Auto => "自动选择",
            Self::Cpu => "CPU（兼容性最好）",
            Self::Nvidia => "NVIDIA GPU",
            Self::Intel => "Intel GPU",
            Self::Amd => "AMD GPU",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_output_settings_keep_existing_behavior() {
        let settings = OutputSettings::default();
        assert_eq!(settings.format, OutputFormat::Mp4);
        assert_eq!(settings.resolution, OutputResolution::FollowCanvas);
        assert_eq!(settings.frame_rate, OutputFrameRate::Source);
        assert_eq!(settings.quality, OutputQuality::Standard);
        assert_eq!(settings.encoder, VideoEncoder::Auto);
    }

    #[test]
    fn missing_saved_output_fields_use_safe_defaults() {
        let settings: OutputSettings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings, OutputSettings::default());
    }

    #[test]
    fn maps_720p_to_each_fixed_canvas_ratio() {
        let settings = OutputSettings {
            resolution: OutputResolution::Hd720,
            ..OutputSettings::default()
        };
        assert_eq!(
            output_canvas_dimensions(settings, CanvasAspectRatio::Portrait916),
            Some((720, 1280))
        );
        assert_eq!(
            output_canvas_dimensions(settings, CanvasAspectRatio::Square11),
            Some((720, 720))
        );
        assert_eq!(
            output_canvas_dimensions(settings, CanvasAspectRatio::Landscape169),
            Some((1280, 720))
        );
    }

    #[test]
    fn quality_profiles_use_higher_bitrate_for_higher_quality() {
        let compact = target_bitrate_kbps(OutputQuality::Compact, OutputResolution::FullHd1080);
        let standard = target_bitrate_kbps(OutputQuality::Standard, OutputResolution::FullHd1080);
        let high = target_bitrate_kbps(OutputQuality::High, OutputResolution::FullHd1080);
        assert!(compact < standard && standard < high);
    }

    #[test]
    fn original_resolution_filter_preserves_orientation() {
        let filters = build_output_video_filters(
            OutputSettings {
                resolution: OutputResolution::Hd720,
                ..OutputSettings::default()
            },
            CanvasAspectRatio::Original,
        );
        assert_eq!(filters.len(), 1);
        assert!(filters[0].contains("gte(iw\\,ih)"));
        assert!(filters[0].contains("1280"));
        assert!(filters[0].contains("720"));
    }

    #[test]
    fn resolves_final_dimensions_for_original_and_fixed_canvas() {
        let hd = OutputSettings {
            resolution: OutputResolution::Hd720,
            ..OutputSettings::default()
        };
        assert_eq!(
            resolve_output_video_dimensions(hd, CanvasAspectRatio::Original, (1920, 1080)),
            (1280, 720)
        );
        assert_eq!(
            resolve_output_video_dimensions(hd, CanvasAspectRatio::Original, (1080, 1920)),
            (720, 1280)
        );
        assert_eq!(
            resolve_output_video_dimensions(hd, CanvasAspectRatio::Square11, (1920, 1080)),
            (720, 720)
        );
    }

    #[test]
    fn converts_common_gpu_errors_to_readable_messages() {
        assert_eq!(
            summarize_encoder_error("Unknown encoder 'h264_nvenc'"),
            "当前FFmpeg不包含这个编码器"
        );
        assert_eq!(
            summarize_encoder_error("No capable devices found"),
            "未检测到对应显卡，或显卡驱动暂不可用"
        );
    }
}
