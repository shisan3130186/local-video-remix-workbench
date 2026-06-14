use serde::{Deserialize, Serialize};
use std::fs;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const FFPROBE_METADATA_TIMEOUT_SECONDS: u64 = 12;

#[derive(Debug, Serialize)]
pub struct ToolProbeResult {
    available: bool,
    version: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FfmpegEnvironmentResult {
    ffmpeg: ToolProbeResult,
    ffprobe: ToolProbeResult,
    available: bool,
    message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoMetadata {
    file_name: String,
    file_path: String,
    duration_seconds: Option<f64>,
    width: Option<u32>,
    height: Option<u32>,
    frame_rate: Option<f64>,
    has_audio: bool,
    file_size_bytes: u64,
}

#[derive(Debug, Deserialize)]
struct FfprobeOutput {
    streams: Vec<FfprobeStream>,
    format: Option<FfprobeFormat>,
}

#[derive(Debug, Deserialize)]
struct FfprobeStream {
    codec_type: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    avg_frame_rate: Option<String>,
    r_frame_rate: Option<String>,
    duration: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FfprobeFormat {
    duration: Option<String>,
}

pub fn check_environment() -> FfmpegEnvironmentResult {
    let ffmpeg = probe_tool("ffmpeg");
    let ffprobe = probe_tool("ffprobe");
    let available = ffmpeg.available && ffprobe.available;
    let message = if available {
        "FFmpeg 环境正常。".to_string()
    } else {
        "未检测到 FFmpeg，请配置路径。".to_string()
    };

    FfmpegEnvironmentResult {
        ffmpeg,
        ffprobe,
        available,
        message,
    }
}

pub fn probe_video_metadata(file_path: String) -> Result<VideoMetadata, String> {
    let metadata =
        fs::metadata(&file_path).map_err(|error| format!("无法读取视频文件：{error}"))?;

    if !metadata.is_file() {
        return Err("选择的路径不是视频文件。".to_string());
    }

    let output = run_ffprobe_with_timeout(
        &[
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            &file_path,
        ],
        FFPROBE_METADATA_TIMEOUT_SECONDS,
    )?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "ffprobe 读取视频信息失败。".to_string()
        } else {
            stderr
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let probe_output: FfprobeOutput =
        serde_json::from_str(&stdout).map_err(|error| format!("ffprobe 结果解析失败：{error}"))?;

    let video_stream = probe_output
        .streams
        .iter()
        .find(|stream| stream.codec_type.as_deref() == Some("video"));

    let has_audio = probe_output
        .streams
        .iter()
        .any(|stream| stream.codec_type.as_deref() == Some("audio"));

    let duration_seconds = probe_output
        .format
        .as_ref()
        .and_then(|format| parse_optional_f64(format.duration.as_deref()))
        .or_else(|| video_stream.and_then(|stream| parse_optional_f64(stream.duration.as_deref())));

    let frame_rate = video_stream.and_then(|stream| {
        parse_frame_rate(stream.avg_frame_rate.as_deref())
            .or_else(|| parse_frame_rate(stream.r_frame_rate.as_deref()))
    });

    Ok(VideoMetadata {
        file_name: file_name_from_path(&file_path),
        file_path,
        duration_seconds,
        width: video_stream.and_then(|stream| stream.width),
        height: video_stream.and_then(|stream| stream.height),
        frame_rate,
        has_audio,
        file_size_bytes: metadata.len(),
    })
}

fn probe_tool(binary_name: &str) -> ToolProbeResult {
    match Command::new(binary_name).arg("-version").output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let version = stdout.lines().next().map(|line| line.trim().to_string());

            ToolProbeResult {
                available: true,
                version,
                error: None,
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

            ToolProbeResult {
                available: false,
                version: None,
                error: Some(if stderr.is_empty() {
                    format!("{binary_name} 执行失败。")
                } else {
                    stderr
                }),
            }
        }
        Err(error) => ToolProbeResult {
            available: false,
            version: None,
            error: Some(error.to_string()),
        },
    }
}

fn run_ffprobe_with_timeout(args: &[&str], timeout_seconds: u64) -> Result<Output, String> {
    let mut child = Command::new("ffprobe")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("无法调用 ffprobe：{error}"))?;
    let started_at = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                return child
                    .wait_with_output()
                    .map_err(|error| format!("读取 ffprobe 输出失败：{error}"));
            }
            Ok(None) if started_at.elapsed() >= Duration::from_secs(timeout_seconds) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("ffprobe 读取视频信息超时，已跳过该文件。"));
            }
            Ok(None) => thread::sleep(Duration::from_millis(100)),
            Err(error) => {
                let _ = child.kill();
                return Err(format!("等待 ffprobe 结束失败：{error}"));
            }
        }
    }
}

fn parse_optional_f64(value: Option<&str>) -> Option<f64> {
    value.and_then(|value| value.parse::<f64>().ok())
}

fn parse_frame_rate(value: Option<&str>) -> Option<f64> {
    let value = value?;
    let (numerator, denominator) = value.split_once('/')?;
    let numerator = numerator.parse::<f64>().ok()?;
    let denominator = denominator.parse::<f64>().ok()?;

    if denominator == 0.0 {
        None
    } else {
        Some(numerator / denominator)
    }
}

fn file_name_from_path(file_path: &str) -> String {
    std::path::Path::new(file_path)
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or(file_path)
        .to_string()
}
