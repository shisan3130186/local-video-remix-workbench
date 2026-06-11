use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitVideoResult {
    output_directory: String,
    segment_paths: Vec<String>,
    segment_count: usize,
    message: String,
}

pub fn split_video_by_duration(
    input_file_path: String,
    output_directory: String,
    segment_duration_seconds: f64,
) -> Result<SplitVideoResult, String> {
    let input_path = Path::new(&input_file_path);
    let output_dir = Path::new(&output_directory);

    if !input_path.is_file() {
        return Err("请选择一个已导入的视频文件。".to_string());
    }

    if !output_dir.is_dir() {
        return Err("请选择有效的输出目录。".to_string());
    }

    if !segment_duration_seconds.is_finite() || segment_duration_seconds <= 0.0 {
        return Err("切片秒数必须大于 0。".to_string());
    }

    let segment_dir = build_segment_directory(input_path, output_dir)?;
    fs::create_dir_all(&segment_dir).map_err(|error| format!("无法创建切片目录：{error}"))?;

    let segment_pattern = segment_dir.join("segment_%03d.mp4");
    let segment_pattern_text = segment_pattern
        .to_str()
        .ok_or_else(|| "切片输出路径包含无法识别的字符。".to_string())?;
    let segment_duration_text = format!("{segment_duration_seconds:.3}");
    let force_key_frames = format!("expr:gte(t,n_forced*{segment_duration_text})");

    let output = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            &input_file_path,
            "-map",
            "0:v:0",
            "-map",
            "0:a?",
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-force_key_frames",
            &force_key_frames,
            "-f",
            "segment",
            "-segment_time",
            &segment_duration_text,
            "-reset_timestamps",
            "1",
            segment_pattern_text,
        ])
        .output()
        .map_err(|error| format!("无法调用 ffmpeg：{error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "视频切片失败。".to_string()
        } else {
            stderr
        });
    }

    let segment_paths = list_generated_segments(&segment_dir)?;

    if segment_paths.is_empty() {
        return Err("切片命令已结束，但没有找到生成的片段文件。".to_string());
    }

    Ok(SplitVideoResult {
        output_directory: segment_dir.to_string_lossy().to_string(),
        segment_count: segment_paths.len(),
        segment_paths,
        message: "切片完成。".to_string(),
    })
}

fn build_segment_directory(input_path: &Path, output_dir: &Path) -> Result<PathBuf, String> {
    let file_stem = input_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("video");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("无法生成切片目录名：{error}"))?
        .as_secs();
    let directory_name = format!("{file_stem}_segments_{timestamp}");

    Ok(output_dir.join(directory_name))
}

fn list_generated_segments(segment_dir: &Path) -> Result<Vec<String>, String> {
    let mut segment_paths = fs::read_dir(segment_dir)
        .map_err(|error| format!("无法读取切片目录：{error}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("mp4"))
        })
        .map(|path| path.to_string_lossy().to_string())
        .collect::<Vec<_>>();

    segment_paths.sort();

    Ok(segment_paths)
}
