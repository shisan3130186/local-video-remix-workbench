use crate::video_engine::mix::{concat_narrated_prepared_segments, MixVideoResult, RemixSettings};
use crate::video_engine::tool_paths::{ffmpeg_program, ffprobe_program};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NarratedSegmentInput {
    video_path: String,
    narration_path: String,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NarratedAudioSettings {
    keep_original_audio: bool,
    original_audio_volume: f64,
}

pub fn concat_narrated_segments(
    segments: Vec<NarratedSegmentInput>,
    output_directory: String,
    settings: RemixSettings,
    audio_settings: NarratedAudioSettings,
) -> Result<MixVideoResult, String> {
    if segments.len() < 2 {
        return Err("至少需要 2 个带配音分镜才能生成视频。".to_string());
    }

    let output_dir = Path::new(&output_directory);
    if !output_dir.is_dir() {
        return Err("请选择有效的输出目录。".to_string());
    }

    let audio_settings = normalize_narrated_audio_settings(audio_settings)?;

    for segment in &segments {
        if !Path::new(&segment.video_path).is_file() {
            return Err(format!("分镜视频不存在：{}", segment.video_path));
        }

        if !Path::new(&segment.narration_path).is_file() {
            return Err("分镜配音文件不存在，请重新生成配音。".to_string());
        }
    }

    let session_dir = narrated_session_directory();
    fs::create_dir_all(&session_dir)
        .map_err(|error| format!("无法创建配音混剪临时目录：{error}"))?;

    let result = prepare_and_concat_narrated_segments(
        &segments,
        &session_dir,
        output_directory,
        settings,
        audio_settings,
    );
    let _ = fs::remove_dir_all(session_dir);
    result
}

fn prepare_and_concat_narrated_segments(
    segments: &[NarratedSegmentInput],
    session_dir: &Path,
    output_directory: String,
    settings: RemixSettings,
    audio_settings: NarratedAudioSettings,
) -> Result<MixVideoResult, String> {
    let mut prepared_paths = Vec::with_capacity(segments.len());

    for (index, segment) in segments.iter().enumerate() {
        let output_path = session_dir.join(format!("narrated_{index:03}.mp4"));
        create_narrated_segment(segment, &output_path, audio_settings)?;
        prepared_paths.push(output_path.to_string_lossy().to_string());
    }

    concat_narrated_prepared_segments(prepared_paths, output_directory, settings)
}

fn create_narrated_segment(
    segment: &NarratedSegmentInput,
    output_path: &Path,
    audio_settings: NarratedAudioSettings,
) -> Result<(), String> {
    let video_duration = probe_media_duration(&segment.video_path, "分镜视频")?;
    let narration_duration = probe_media_duration(&segment.narration_path, "分镜配音")?;
    let video_filter = build_narrated_video_filter(video_duration, narration_duration)?;
    let has_original_audio = audio_settings.keep_original_audio
        && probe_media_has_audio(&segment.video_path, "分镜视频")?;
    let audio_filter =
        build_narrated_audio_filter(narration_duration, audio_settings, has_original_audio)?;
    let filter_complex = format!("[0:v]{video_filter}[v];{audio_filter}");
    let output_path_text = output_path
        .to_str()
        .ok_or_else(|| "配音分镜临时路径包含无法识别的字符。".to_string())?;
    let output = Command::new(ffmpeg_program())
        .args([
            "-y",
            "-i",
            &segment.video_path,
            "-i",
            &segment.narration_path,
            "-filter_complex",
            &filter_complex,
            "-map",
            "[v]",
            "-map",
            "[a]",
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-shortest",
            output_path_text,
        ])
        .output()
        .map_err(|error| format!("无法调用 ffmpeg 生成配音分镜：{error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "生成配音分镜失败。".to_string()
        } else {
            stderr
        });
    }

    if !output_path.is_file() {
        return Err("配音分镜命令已结束，但没有找到临时视频。".to_string());
    }

    Ok(())
}

fn normalize_narrated_audio_settings(
    settings: NarratedAudioSettings,
) -> Result<NarratedAudioSettings, String> {
    if !settings.original_audio_volume.is_finite()
        || settings.original_audio_volume < 0.0
        || settings.original_audio_volume > 1.0
    {
        return Err("原视频声音音量必须在 0% 到 100% 之间。".to_string());
    }

    Ok(settings)
}

fn build_narrated_audio_filter(
    narration_duration: f64,
    settings: NarratedAudioSettings,
    has_original_audio: bool,
) -> Result<String, String> {
    validate_duration(narration_duration, "分镜配音")?;
    let settings = normalize_narrated_audio_settings(settings)?;
    let narration_filter =
        format!("[1:a]atrim=duration={narration_duration:.3},asetpts=PTS-STARTPTS");

    if !settings.keep_original_audio || !has_original_audio {
        return Ok(format!("{narration_filter}[a]"));
    }

    Ok(format!(
        "[0:a]atrim=duration={narration_duration:.3},asetpts=PTS-STARTPTS,volume={:.3},apad,atrim=duration={narration_duration:.3}[original];\
         {narration_filter}[narration];\
         [original][narration]amix=inputs=2:duration=longest:dropout_transition=0:normalize=0,alimiter=limit=0.950[a]",
        settings.original_audio_volume
    ))
}

fn build_narrated_video_filter(
    video_duration: f64,
    narration_duration: f64,
) -> Result<String, String> {
    validate_duration(video_duration, "分镜视频")?;
    validate_duration(narration_duration, "分镜配音")?;

    if video_duration + 0.001 >= narration_duration {
        return Ok(format!(
            "trim=duration={narration_duration:.3},setpts=PTS-STARTPTS,format=yuv420p"
        ));
    }

    let freeze_duration = narration_duration - video_duration;
    Ok(format!(
        "trim=duration={video_duration:.3},setpts=PTS-STARTPTS,tpad=stop_mode=clone:stop_duration={freeze_duration:.3},trim=duration={narration_duration:.3},format=yuv420p"
    ))
}

fn probe_media_duration(path: &str, label: &str) -> Result<f64, String> {
    let output = Command::new(ffprobe_program())
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            path,
        ])
        .output()
        .map_err(|error| format!("无法调用 ffprobe 读取{label}时长：{error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("读取{label}时长失败。")
        } else {
            stderr
        });
    }

    let duration_text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let duration = duration_text
        .parse::<f64>()
        .map_err(|_| format!("无法识别{label}时长：{duration_text}"))?;
    validate_duration(duration, label)?;
    Ok(duration)
}

fn probe_media_has_audio(path: &str, label: &str) -> Result<bool, String> {
    let output = Command::new(ffprobe_program())
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=index",
            "-of",
            "csv=p=0",
            path,
        ])
        .output()
        .map_err(|error| format!("无法调用 ffprobe 检查{label}声音：{error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("检查{label}声音失败。")
        } else {
            stderr
        });
    }

    Ok(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
}

fn validate_duration(duration: f64, label: &str) -> Result<(), String> {
    if !duration.is_finite() || duration <= 0.0 {
        return Err(format!("{label}时长无效。"));
    }

    Ok(())
}

fn narrated_session_directory() -> PathBuf {
    env::temp_dir()
        .join("local-video-remix-workbench")
        .join("narrated-remix")
        .join(Uuid::new_v4().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_video_when_narration_is_shorter() {
        let filter = build_narrated_video_filter(5.0, 3.0).unwrap();

        assert!(filter.contains("trim=duration=3.000"));
        assert!(!filter.contains("tpad="));
    }

    #[test]
    fn freezes_last_frame_when_narration_is_longer() {
        let filter = build_narrated_video_filter(2.0, 4.5).unwrap();

        assert!(filter.contains("tpad=stop_mode=clone:stop_duration=2.500"));
        assert!(filter.contains("trim=duration=4.500"));
    }

    #[test]
    fn rejects_invalid_narration_duration() {
        assert_eq!(
            build_narrated_video_filter(2.0, 0.0).unwrap_err(),
            "分镜配音时长无效。"
        );
    }

    #[test]
    fn keeps_only_narration_when_original_audio_is_disabled() {
        let filter = build_narrated_audio_filter(
            3.0,
            NarratedAudioSettings {
                keep_original_audio: false,
                original_audio_volume: 0.15,
            },
            true,
        )
        .unwrap();

        assert_eq!(filter, "[1:a]atrim=duration=3.000,asetpts=PTS-STARTPTS[a]");
    }

    #[test]
    fn mixes_original_audio_at_selected_volume() {
        let filter = build_narrated_audio_filter(
            3.0,
            NarratedAudioSettings {
                keep_original_audio: true,
                original_audio_volume: 0.15,
            },
            true,
        )
        .unwrap();

        assert!(filter.contains("[0:a]atrim=duration=3.000"));
        assert!(filter.contains("volume=0.150"));
        assert!(filter.contains("amix=inputs=2"));
        assert!(filter.contains("alimiter=limit=0.950"));
    }

    #[test]
    fn falls_back_to_narration_when_video_has_no_audio() {
        let filter = build_narrated_audio_filter(
            3.0,
            NarratedAudioSettings {
                keep_original_audio: true,
                original_audio_volume: 0.15,
            },
            false,
        )
        .unwrap();

        assert_eq!(filter, "[1:a]atrim=duration=3.000,asetpts=PTS-STARTPTS[a]");
    }

    #[test]
    fn rejects_original_audio_volume_above_one_hundred_percent() {
        let error = normalize_narrated_audio_settings(NarratedAudioSettings {
            keep_original_audio: true,
            original_audio_volume: 1.01,
        })
        .unwrap_err();

        assert_eq!(error, "原视频声音音量必须在 0% 到 100% 之间。");
    }
}
