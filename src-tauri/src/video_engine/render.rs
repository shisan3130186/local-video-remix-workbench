use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderVideoResult {
    output_path: String,
    message: String,
}

pub fn export_basic_video(
    input_file_path: String,
    output_directory: String,
) -> Result<RenderVideoResult, String> {
    let input_path = Path::new(&input_file_path);
    let output_dir = Path::new(&output_directory);

    if !input_path.is_file() {
        return Err("请选择一个已导入的视频文件。".to_string());
    }

    if !output_dir.is_dir() {
        return Err("请选择有效的输出目录。".to_string());
    }

    let output_path = build_output_path(input_path, output_dir)?;

    let output = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            &input_file_path,
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-movflags",
            "+faststart",
            output_path
                .to_str()
                .ok_or_else(|| "输出路径包含无法识别的字符。".to_string())?,
        ])
        .output()
        .map_err(|error| format!("无法调用 ffmpeg：{error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "视频导出失败。".to_string()
        } else {
            stderr
        });
    }

    if !output_path.is_file() {
        return Err("导出命令已结束，但没有找到输出文件。".to_string());
    }

    Ok(RenderVideoResult {
        output_path: output_path.to_string_lossy().to_string(),
        message: "导出完成。".to_string(),
    })
}

fn build_output_path(input_path: &Path, output_dir: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(output_dir).map_err(|error| format!("无法创建输出目录：{error}"))?;

    let file_stem = input_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("video");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("无法生成导出文件名：{error}"))?
        .as_secs();
    let file_name = format!("{file_stem}_export_{timestamp}.mp4");

    Ok(output_dir.join(file_name))
}
