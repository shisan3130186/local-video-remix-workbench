use crate::task_runtime::{run_ffmpeg, TaskProgressContext};
use crate::video_engine::tool_paths::{background_command, ffprobe_program};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_ASR_DURATION_SECONDS: f64 = 2.0 * 60.0 * 60.0;
const MAX_NORMALIZED_AUDIO_BYTES: u64 = 24 * 1024 * 1024;

#[derive(Debug)]
pub struct NormalizedAsrAudio {
    pub path: PathBuf,
    pub duration_seconds: Option<f64>,
    pub byte_count: u64,
}

#[derive(Debug, Deserialize)]
struct MediaProbeOutput {
    #[serde(default)]
    streams: Vec<MediaProbeStream>,
    format: Option<MediaProbeFormat>,
}

#[derive(Debug, Deserialize)]
struct MediaProbeStream {
    codec_type: Option<String>,
    duration: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MediaProbeFormat {
    duration: Option<String>,
}

pub fn normalize_audio_for_asr(
    source_path: &Path,
    output_path: &Path,
    task_context: Option<&TaskProgressContext>,
) -> Result<NormalizedAsrAudio, String> {
    let source_metadata =
        fs::metadata(source_path).map_err(|error| format!("无法读取待识别文件：{error}"))?;
    if !source_metadata.is_file() {
        return Err("选择的路径不是音频或视频文件。".to_string());
    }

    let probe = probe_audio_source(source_path)?;
    if !probe.has_audio {
        return Err("这个视频没有检测到声音，无法进行语音识别。".to_string());
    }
    if probe
        .duration_seconds
        .is_some_and(|duration| duration > MAX_ASR_DURATION_SECONDS)
    {
        return Err("单个音频或视频不能超过2小时，请先分段后再识别。".to_string());
    }

    let source_text = source_path.to_string_lossy().to_string();
    let output_text = output_path.to_string_lossy().to_string();
    let ffmpeg_args = vec![
        "-y".to_string(),
        "-i".to_string(),
        source_text,
        "-map".to_string(),
        "0:a:0".to_string(),
        "-vn".to_string(),
        "-ac".to_string(),
        "1".to_string(),
        "-ar".to_string(),
        "16000".to_string(),
        "-c:a".to_string(),
        "libmp3lame".to_string(),
        "-b:a".to_string(),
        "24k".to_string(),
        output_text,
    ];

    run_ffmpeg(
        ffmpeg_args,
        task_context,
        probe.duration_seconds,
        "音频标准化失败，请确认文件可以正常播放。",
    )?;

    let output_metadata =
        fs::metadata(output_path).map_err(|error| format!("无法读取标准化音频：{error}"))?;
    if output_metadata.len() == 0 {
        return Err("音频转换完成，但没有生成有效声音数据。".to_string());
    }
    if output_metadata.len() > MAX_NORMALIZED_AUDIO_BYTES {
        return Err(
            "音频压缩后仍然过大，暂时无法直接上传识别。请先把文件分成更短的片段。".to_string(),
        );
    }

    Ok(NormalizedAsrAudio {
        path: output_path.to_path_buf(),
        duration_seconds: probe.duration_seconds,
        byte_count: output_metadata.len(),
    })
}

struct AudioProbeResult {
    has_audio: bool,
    duration_seconds: Option<f64>,
}

fn probe_audio_source(source_path: &Path) -> Result<AudioProbeResult, String> {
    let output = background_command(ffprobe_program())
        .args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(source_path)
        .output()
        .map_err(|error| format!("无法调用ffprobe读取声音信息：{error}"))?;

    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if detail.is_empty() {
            "无法读取这个音频或视频，请确认文件没有损坏。".to_string()
        } else {
            detail
        });
    }

    let parsed: MediaProbeOutput = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("声音信息解析失败：{error}"))?;
    let audio_stream = parsed
        .streams
        .iter()
        .find(|stream| stream.codec_type.as_deref() == Some("audio"));
    let duration_seconds = parsed
        .format
        .as_ref()
        .and_then(|format| parse_duration(format.duration.as_deref()))
        .or_else(|| audio_stream.and_then(|stream| parse_duration(stream.duration.as_deref())));

    Ok(AudioProbeResult {
        has_audio: audio_stream.is_some(),
        duration_seconds,
    })
}

fn parse_duration(value: Option<&str>) -> Option<f64> {
    value
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value >= 0.0)
}

#[cfg(test)]
mod tests {
    use super::parse_duration;

    #[test]
    fn parses_valid_media_duration() {
        assert_eq!(parse_duration(Some("12.5")), Some(12.5));
        assert_eq!(parse_duration(Some("-1")), None);
        assert_eq!(parse_duration(Some("unknown")), None);
    }
}
