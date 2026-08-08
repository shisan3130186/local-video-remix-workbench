use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatermarkSettings {
    pub enabled: bool,
    pub kind: WatermarkKind,
    #[serde(default)]
    pub asset_type: WatermarkAssetType,
    pub text: String,
    pub image_file_path: Option<String>,
    pub position: WatermarkPosition,
    pub opacity: f64,
    pub margin: u32,
    pub text_font_size: u32,
    pub text_color: String,
    pub image_size_ratio: f64,
    #[serde(default = "default_image_position_x_ratio")]
    pub image_position_x_ratio: f64,
    #[serde(default = "default_image_position_y_ratio")]
    pub image_position_y_ratio: f64,
    #[serde(default)]
    pub trajectory: WatermarkTrajectory,
}

impl Default for WatermarkSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            kind: WatermarkKind::Text,
            asset_type: WatermarkAssetType::Image,
            text: String::new(),
            image_file_path: None,
            position: WatermarkPosition::TopRight,
            opacity: 0.75,
            margin: 24,
            text_font_size: 36,
            text_color: "#ffffff".to_string(),
            image_size_ratio: 0.18,
            image_position_x_ratio: default_image_position_x_ratio(),
            image_position_y_ratio: default_image_position_y_ratio(),
            trajectory: WatermarkTrajectory::Static,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum WatermarkKind {
    Text,
    Image,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum WatermarkAssetType {
    #[default]
    Image,
    Video,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum WatermarkTrajectory {
    #[default]
    Static,
    Horizontal,
    Vertical,
    Diagonal,
    Random,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum WatermarkPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

pub fn normalize_watermark_settings(
    settings: WatermarkSettings,
) -> Result<Option<WatermarkSettings>, String> {
    if !settings.enabled {
        return Ok(None);
    }

    if !settings.opacity.is_finite() || settings.opacity < 0.1 || settings.opacity > 1.0 {
        return Err("水印透明度必须在 0.1 到 1 之间。".to_string());
    }
    if settings.margin > 240 {
        return Err("水印边距不能超过 240。".to_string());
    }

    match settings.kind {
        WatermarkKind::Text => {
            let text = settings.text.trim();
            if text.is_empty() {
                return Err("请输入文字水印内容。".to_string());
            }
            if text.chars().count() > 80 {
                return Err("文字水印最多支持 80 个字符。".to_string());
            }
            if !(16..=120).contains(&settings.text_font_size) {
                return Err("文字水印字号必须在 16 到 120 之间。".to_string());
            }
            if !is_hex_color(&settings.text_color) {
                return Err("文字水印颜色必须是有效的六位十六进制颜色。".to_string());
            }
            resolve_watermark_font()?;
        }
        WatermarkKind::Image => {
            let image_path = settings
                .image_file_path
                .as_deref()
                .ok_or_else(|| "请先选择图片水印文件。".to_string())?;
            if !Path::new(image_path).is_file() {
                return Err(format!("图片水印文件不存在：{image_path}"));
            }
            if settings.asset_type == WatermarkAssetType::Video && !is_supported_video(image_path) {
                return Err("视频水印只支持 MP4、MOV、WEBM 和 M4V。".to_string());
            }
            if settings.asset_type == WatermarkAssetType::Image && !is_supported_image(image_path) {
                return Err("图片水印只支持 PNG、JPG、JPEG、WEBP 和 BMP。".to_string());
            }
            if !settings.image_size_ratio.is_finite()
                || settings.image_size_ratio < 0.08
                || settings.image_size_ratio > 0.5
            {
                return Err("图片水印大小比例必须在 0.08 到 0.5 之间。".to_string());
            }
            if !settings.image_position_x_ratio.is_finite()
                || !(0.0..=1.0).contains(&settings.image_position_x_ratio)
                || !settings.image_position_y_ratio.is_finite()
                || !(0.0..=1.0).contains(&settings.image_position_y_ratio)
            {
                return Err("图片水印位置必须位于视频画面内部。".to_string());
            }
        }
    }

    Ok(Some(settings))
}

pub fn append_watermark_input_args(
    ffmpeg_args: &mut Vec<String>,
    settings: &WatermarkSettings,
) -> Result<(), String> {
    if settings.kind != WatermarkKind::Image {
        return Ok(());
    }
    let image_path = settings
        .image_file_path
        .as_deref()
        .ok_or_else(|| "请先选择图片水印文件。".to_string())?;
    if settings.asset_type == WatermarkAssetType::Video {
        ffmpeg_args.extend(["-stream_loop".to_string(), "-1".to_string()]);
    } else {
        ffmpeg_args.extend(["-loop".to_string(), "1".to_string()]);
    }
    ffmpeg_args.extend(["-i".to_string(), image_path.to_string()]);
    Ok(())
}

pub fn prepare_text_watermark_file(
    settings: &WatermarkSettings,
    temp_directory: &Path,
) -> Result<Option<PathBuf>, String> {
    if settings.kind != WatermarkKind::Text {
        return Ok(None);
    }
    let path = temp_directory.join("watermark_text.txt");
    fs::write(&path, settings.text.trim())
        .map_err(|error| format!("无法创建临时文字水印文件：{error}"))?;
    Ok(Some(path))
}

pub fn build_text_watermark_layer(
    input_label: &str,
    output_label: &str,
    settings: &WatermarkSettings,
    text_file_path: &Path,
) -> Result<String, String> {
    let font_path = resolve_watermark_font()?;
    let font_path = escape_filter_path(&font_path, "水印字体路径")?;
    let text_path = escape_filter_path(text_file_path, "临时文字水印路径")?;
    let color = settings
        .text_color
        .trim_start_matches('#')
        .to_ascii_uppercase();
    let (x, y) = text_position_expression(settings.position, settings.margin);

    Ok(format!(
        "{input_label}drawtext=fontfile='{font_path}':textfile='{text_path}':fontcolor=0x{color}@{opacity:.3}:fontsize={font_size}:x={x}:y={y}{output_label}",
        opacity = settings.opacity,
        font_size = settings.text_font_size,
    ))
}

pub fn build_image_watermark_layer(
    input_label: &str,
    output_label: &str,
    image_input_index: usize,
    settings: &WatermarkSettings,
    layer_number: usize,
) -> String {
    let (base_x, base_y) = (
        format!("W*{:.4}-w/2", settings.image_position_x_ratio),
        format!("H*{:.4}-h/2", settings.image_position_y_ratio),
    );
    let (x, y) = match settings.trajectory {
        WatermarkTrajectory::Static => (base_x, base_y),
        WatermarkTrajectory::Horizontal => ("(W-w)*mod(t\\,8)/8".to_string(), base_y),
        WatermarkTrajectory::Vertical => (base_x, "(H-h)*mod(t\\,8)/8".to_string()),
        WatermarkTrajectory::Diagonal => (
            "(W-w)*mod(t\\,8)/8".to_string(),
            "(H-h)*mod(t\\,8)/8".to_string(),
        ),
        WatermarkTrajectory::Random => (
            "(W-w)*(sin(t)*0.5+0.5)".to_string(),
            "(H-h)*(cos(t)*0.5+0.5)".to_string(),
        ),
    };
    format!(
        "[{image_input_index}:v]format=rgba,colorchannelmixer=aa={opacity:.3}[wmraw{layer_number}];\
         [wmraw{layer_number}]{input_label}scale2ref=w=main_w*{size_ratio:.3}:h=-1[wm{layer_number}][wmbase{layer_number}];\
         [wmbase{layer_number}][wm{layer_number}]overlay={x}:{y}:eof_action=pass:format=auto{output_label}",
        opacity = settings.opacity,
        size_ratio = settings.image_size_ratio,
    )
}

fn default_image_position_x_ratio() -> f64 {
    0.82
}

fn default_image_position_y_ratio() -> f64 {
    0.16
}

pub fn uses_image_input(settings: &WatermarkSettings) -> bool {
    settings.kind == WatermarkKind::Image
}

fn resolve_watermark_font() -> Result<PathBuf, String> {
    let windows_directory = std::env::var_os("WINDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    let fonts_directory = windows_directory.join("Fonts");
    for file_name in ["msyh.ttc", "msyhbd.ttc", "arial.ttf"] {
        let path = fonts_directory.join(file_name);
        if path.is_file() {
            return Ok(path);
        }
    }
    Err("没有找到可用于文字水印的系统字体，请确认Windows字体文件完整。".to_string())
}

fn escape_filter_path(path: &Path, label: &str) -> Result<String, String> {
    let text = path
        .to_str()
        .ok_or_else(|| format!("{label}包含无法识别的字符。"))?;
    Ok(text
        .replace('\\', "/")
        .replace(':', "\\:")
        .replace('\'', "\\'"))
}

fn text_position_expression(position: WatermarkPosition, margin: u32) -> (String, String) {
    let margin = margin.to_string();
    match position {
        WatermarkPosition::TopLeft => (margin.clone(), margin),
        WatermarkPosition::TopRight => (format!("w-text_w-{margin}"), margin),
        WatermarkPosition::BottomLeft => (margin.clone(), format!("h-text_h-{margin}")),
        WatermarkPosition::BottomRight => {
            (format!("w-text_w-{margin}"), format!("h-text_h-{margin}"))
        }
        WatermarkPosition::Center => ("(w-text_w)/2".to_string(), "(h-text_h)/2".to_string()),
    }
}

fn is_hex_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_supported_image(file_path: &str) -> bool {
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

fn is_supported_video(file_path: &str) -> bool {
    Path::new(file_path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "mp4" | "mov" | "webm" | "m4v"
            )
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_settings() -> WatermarkSettings {
        WatermarkSettings {
            enabled: true,
            text: "品牌水印".to_string(),
            ..WatermarkSettings::default()
        }
    }

    #[test]
    fn disabled_watermark_uses_no_filter() {
        assert!(normalize_watermark_settings(WatermarkSettings::default())
            .unwrap()
            .is_none());
    }

    #[test]
    fn rejects_empty_text_watermark() {
        let mut settings = text_settings();
        settings.text.clear();
        assert_eq!(
            normalize_watermark_settings(settings).unwrap_err(),
            "请输入文字水印内容。"
        );
    }

    #[test]
    fn text_position_keeps_margin() {
        assert_eq!(
            text_position_expression(WatermarkPosition::BottomRight, 24),
            ("w-text_w-24".to_string(), "h-text_h-24".to_string())
        );
    }

    #[test]
    fn image_watermark_uses_manual_video_position() {
        let settings = WatermarkSettings {
            kind: WatermarkKind::Image,
            image_position_x_ratio: 0.42,
            image_position_y_ratio: 0.68,
            ..WatermarkSettings::default()
        };
        let filter = build_image_watermark_layer("[base]", "[out]", 1, &settings, 2);
        assert!(filter.contains("W*0.4200-w/2"));
        assert!(filter.contains("H*0.6800-h/2"));
    }

    #[test]
    fn escapes_windows_filter_path() {
        assert_eq!(
            escape_filter_path(Path::new(r"C:\Temp Folder\水印.txt"), "测试").unwrap(),
            r"C\:/Temp Folder/水印.txt"
        );
    }
}
