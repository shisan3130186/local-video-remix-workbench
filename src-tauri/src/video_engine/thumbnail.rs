use crate::video_engine::tool_paths::{background_command, ffmpeg_program};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoThumbnailResult {
    thumbnail_path: String,
    source_path: String,
    time_seconds: f64,
    message: String,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ThumbnailFitMode {
    Source,
    Contain,
}

pub fn generate_video_thumbnail(
    input_file_path: String,
    output_directory: Option<String>,
    time_seconds: f64,
    label: String,
    fit_mode: ThumbnailFitMode,
) -> Result<VideoThumbnailResult, String> {
    let input_path = Path::new(&input_file_path);

    if !input_path.is_file() {
        return Err("请选择一个有效的视频文件。".to_string());
    }

    if !time_seconds.is_finite() || time_seconds < 0.0 {
        return Err("抽帧时间必须大于或等于 0。".to_string());
    }

    let thumbnail_dir = build_thumbnail_directory(output_directory)?;
    fs::create_dir_all(&thumbnail_dir).map_err(|error| format!("无法创建预览图目录：{error}"))?;

    let file_stem = input_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("video");
    let timestamp = current_timestamp_millis()?;
    let safe_stem = sanitize_file_name(file_stem);
    let safe_label = sanitize_file_name(&label);
    let thumbnail_path = thumbnail_dir.join(format!("{safe_stem}_{safe_label}_{timestamp}.jpg"));
    let thumbnail_path_text = thumbnail_path
        .to_str()
        .ok_or_else(|| "预览图输出路径包含无法识别的字符。".to_string())?;
    let time_text = format!("{time_seconds:.3}");
    let video_filter = thumbnail_video_filter(fit_mode);

    let output = background_command(ffmpeg_program())
        .args([
            "-y",
            "-ss",
            &time_text,
            "-i",
            &input_file_path,
            "-frames:v",
            "1",
            "-vf",
            video_filter,
            "-q:v",
            "3",
            thumbnail_path_text,
        ])
        .output()
        .map_err(|error| format!("无法调用 ffmpeg：{error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "生成预览图失败。".to_string()
        } else {
            stderr
        });
    }

    if !thumbnail_path.is_file() {
        return Err("抽帧命令已结束，但没有找到预览图文件。".to_string());
    }

    Ok(VideoThumbnailResult {
        thumbnail_path: thumbnail_path.to_string_lossy().to_string(),
        source_path: input_file_path,
        time_seconds,
        message: "预览图生成完成。".to_string(),
    })
}

fn thumbnail_video_filter(fit_mode: ThumbnailFitMode) -> &'static str {
    match fit_mode {
        ThumbnailFitMode::Source => "scale=320:-2",
        ThumbnailFitMode::Contain => {
            "scale=320:240:force_original_aspect_ratio=decrease,pad=320:240:(ow-iw)/2:(oh-ih)/2:color=0x101312"
        }
    }
}

fn build_thumbnail_directory(output_directory: Option<String>) -> Result<PathBuf, String> {
    if let Some(output_directory) = output_directory {
        let output_dir = Path::new(&output_directory);

        if output_dir.is_dir() {
            return Ok(output_dir.join("thumbnails"));
        }
    }

    Ok(std::env::temp_dir().join("local_video_remix_thumbnails"))
}

fn sanitize_file_name(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();

    let trimmed = sanitized.trim_matches('_');

    if trimmed.is_empty() {
        "thumbnail".to_string()
    } else {
        trimmed.to_string()
    }
}

fn current_timestamp_millis() -> Result<u128, String> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("无法生成预览图文件名：{error}"))?
        .as_millis())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_source_aspect_ratio_for_ai_thumbnails() {
        assert_eq!(
            thumbnail_video_filter(ThumbnailFitMode::Source),
            "scale=320:-2"
        );
    }

    #[test]
    fn pads_material_covers_without_cropping_portrait_video() {
        let filter = thumbnail_video_filter(ThumbnailFitMode::Contain);

        assert!(filter.contains("force_original_aspect_ratio=decrease"));
        assert!(filter.contains("pad=320:240"));
    }
}
