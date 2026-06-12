use serde::Serialize;
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
}

pub fn concat_video_segments(
    segment_paths: Vec<String>,
    output_directory: String,
) -> Result<MixVideoResult, String> {
    let output_dir = Path::new(&output_directory);

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
    let concat_list_content = build_concat_list_content(&segment_paths);

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

    let output = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            list_path_text,
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
            output_path_text,
        ])
        .output()
        .map_err(|error| format!("无法调用 ffmpeg：{error}"));

    let _ = fs::remove_file(&list_path);

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
        input_count: segment_paths.len(),
        message: "片段拼接完成。".to_string(),
    })
}

fn current_timestamp() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("无法生成拼接文件名：{error}"))
        .map(|duration| duration.as_secs())
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
