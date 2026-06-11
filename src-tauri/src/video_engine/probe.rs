use serde::Serialize;
use std::process::Command;

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
