use crate::task_runtime::{run_ffmpeg, TaskProgressContext};
use crate::temp_storage::TaskTempDirectory;
use crate::video_engine::mix::{concat_narrated_prepared_segments, MixVideoResult, RemixSettings};
use crate::video_engine::subtitle::{
    ensure_ass_filter_available, prepare_ass_subtitle, NarratedSubtitleSettings,
};
use crate::video_engine::tool_paths::{background_command, ffprobe_program};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NarratedSegmentInput {
    video_path: String,
    video_duration_seconds: f64,
    narration_path: String,
    subtitle_text: String,
    #[serde(default)]
    alternative_videos: Vec<NarratedVideoCandidateInput>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NarratedVideoCandidateInput {
    video_path: String,
    duration_seconds: f64,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NarratedAudioSettings {
    keep_original_audio: bool,
    original_audio_volume: f64,
}

#[derive(Debug)]
struct SelectedNarratedVideo {
    path: String,
    duration_seconds: f64,
}

#[derive(Debug, Clone, Copy)]
struct NarratedVideoTiming {
    playback_rate: f64,
    freeze_duration_seconds: f64,
}

const MIN_NARRATED_VIDEO_PLAYBACK_RATE: f64 = 0.92;
const MAX_NARRATED_FREEZE_SECONDS: f64 = 0.8;

pub fn concat_narrated_segments(
    segments: Vec<NarratedSegmentInput>,
    output_directory: String,
    settings: RemixSettings,
    audio_settings: NarratedAudioSettings,
    subtitle_settings: NarratedSubtitleSettings,
    task_context: Option<TaskProgressContext>,
) -> Result<MixVideoResult, String> {
    if segments.len() < 2 {
        return Err("至少需要 2 个带配音分镜才能生成视频。".to_string());
    }

    let output_dir = Path::new(&output_directory);
    if !output_dir.is_dir() {
        return Err("请选择有效的输出目录。".to_string());
    }

    let audio_settings = normalize_narrated_audio_settings(audio_settings)?;
    if subtitle_settings.enabled {
        ensure_ass_filter_available()?;
    }

    for segment in &segments {
        if !Path::new(&segment.video_path).is_file() {
            return Err(format!("分镜视频不存在：{}", segment.video_path));
        }

        validate_duration(segment.video_duration_seconds, "分镜视频")?;

        if !Path::new(&segment.narration_path).is_file() {
            return Err("分镜配音文件不存在，请重新生成配音。".to_string());
        }

        for alternative in &segment.alternative_videos {
            validate_duration(alternative.duration_seconds, "备选分镜视频")?;
            if !Path::new(&alternative.video_path).is_file() {
                return Err(format!("备选分镜视频不存在：{}", alternative.video_path));
            }
        }
    }

    let session_dir = TaskTempDirectory::create("narrated")?;
    prepare_and_concat_narrated_segments(
        &segments,
        session_dir.path(),
        output_directory,
        settings,
        audio_settings,
        subtitle_settings,
        task_context,
    )
}

fn prepare_and_concat_narrated_segments(
    segments: &[NarratedSegmentInput],
    session_dir: &Path,
    output_directory: String,
    settings: RemixSettings,
    audio_settings: NarratedAudioSettings,
    subtitle_settings: NarratedSubtitleSettings,
    task_context: Option<TaskProgressContext>,
) -> Result<MixVideoResult, String> {
    let mut prepared_paths = Vec::with_capacity(segments.len());
    let subtitle_output_directory = Path::new(&output_directory)
        .parent()
        .map(|parent| parent.join("配音与字幕"));
    if let Some(directory) = &subtitle_output_directory {
        fs::create_dir_all(directory)
            .map_err(|error| format!("无法创建配音与字幕目录：{error}"))?;
    }

    for (index, segment) in segments.iter().enumerate() {
        let output_path = session_dir.join(format!("narrated_{index:03}.mp4"));
        let segment_context = task_context.as_ref().map(|context| {
            context.child(
                0.7 * index as f64 / segments.len() as f64,
                0.7 * (index + 1) as f64 / segments.len() as f64,
                format!("正在处理配音分镜 {}/{}", index + 1, segments.len()),
            )
        });
        create_narrated_segment(
            segment,
            &output_path,
            audio_settings,
            subtitle_settings,
            subtitle_output_directory.as_deref(),
            segment_context.as_ref(),
        )
        .map_err(|error| format!("第 {} 个分镜处理失败：{error}", index + 1))?;
        prepared_paths.push(output_path.to_string_lossy().to_string());
    }

    let final_context = task_context
        .as_ref()
        .map(|context| context.child(0.7, 1.0, "正在合成带配音视频"));
    concat_narrated_prepared_segments(prepared_paths, output_directory, settings, final_context)
}

fn create_narrated_segment(
    segment: &NarratedSegmentInput,
    output_path: &Path,
    audio_settings: NarratedAudioSettings,
    subtitle_settings: NarratedSubtitleSettings,
    subtitle_output_directory: Option<&Path>,
    task_context: Option<&TaskProgressContext>,
) -> Result<(), String> {
    let narration_duration = probe_media_duration(&segment.narration_path, "分镜配音")?;
    let selected_video = select_narrated_video(segment, narration_duration)?;
    let timing =
        calculate_narrated_video_timing(selected_video.duration_seconds, narration_duration)?;
    let mut video_filter =
        build_narrated_video_filter(selected_video.duration_seconds, narration_duration, timing)?;
    let subtitle_path = output_path.with_extension("ass");
    if let Some(subtitle_filter) = prepare_ass_subtitle(
        &subtitle_path,
        &segment.subtitle_text,
        narration_duration,
        subtitle_settings,
    )? {
        if let Some(directory) = subtitle_output_directory {
            let archive_name = output_path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("segment");
            let archive_path = directory.join(format!("{archive_name}.ass"));
            let _ = fs::copy(&subtitle_path, archive_path);
        }
        video_filter.push(',');
        video_filter.push_str(&subtitle_filter);
    }
    let has_original_audio = audio_settings.keep_original_audio
        && probe_media_has_audio(&selected_video.path, "分镜视频")?;
    let audio_filter = build_narrated_audio_filter(
        selected_video.duration_seconds,
        narration_duration,
        timing.playback_rate,
        audio_settings,
        has_original_audio,
    )?;
    let filter_complex = format!("[0:v]{video_filter}[v];{audio_filter}");
    let output_path_text = output_path
        .to_str()
        .ok_or_else(|| "配音分镜临时路径包含无法识别的字符。".to_string())?;
    run_ffmpeg(
        [
            "-y",
            "-i",
            &selected_video.path,
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
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        task_context,
        Some(narration_duration),
        "生成配音分镜失败。",
    )?;

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
    video_duration: f64,
    narration_duration: f64,
    video_playback_rate: f64,
    settings: NarratedAudioSettings,
    has_original_audio: bool,
) -> Result<String, String> {
    validate_duration(video_duration, "分镜视频")?;
    validate_duration(narration_duration, "分镜配音")?;
    if !video_playback_rate.is_finite()
        || !(MIN_NARRATED_VIDEO_PLAYBACK_RATE..=1.0).contains(&video_playback_rate)
    {
        return Err("分镜画面减速比例无效。".to_string());
    }
    let settings = normalize_narrated_audio_settings(settings)?;
    let narration_filter =
        format!("[1:a]atrim=duration={narration_duration:.3},asetpts=PTS-STARTPTS");

    if !settings.keep_original_audio || !has_original_audio {
        return Ok(format!("{narration_filter}[a]"));
    }

    Ok(format!(
        "[0:a]atrim=duration={video_duration:.3},asetpts=PTS-STARTPTS,atempo={video_playback_rate:.6},volume={:.3},apad,atrim=duration={narration_duration:.3}[original];\
         {narration_filter}[narration];\
         [original][narration]amix=inputs=2:duration=longest:dropout_transition=0:normalize=0,alimiter=limit=0.950[a]",
        settings.original_audio_volume
    ))
}

fn build_narrated_video_filter(
    video_duration: f64,
    narration_duration: f64,
    timing: NarratedVideoTiming,
) -> Result<String, String> {
    validate_duration(video_duration, "分镜视频")?;
    validate_duration(narration_duration, "分镜配音")?;

    if video_duration + 0.001 >= narration_duration && timing.playback_rate >= 0.999 {
        return Ok(format!(
            "trim=duration={narration_duration:.3},setpts=PTS-STARTPTS,format=yuv420p"
        ));
    }

    let mut filter = format!(
        "trim=duration={video_duration:.3},setpts=(PTS-STARTPTS)/{:.6}",
        timing.playback_rate
    );
    if timing.freeze_duration_seconds > 0.001 {
        filter.push_str(&format!(
            ",tpad=stop_mode=clone:stop_duration={:.3}",
            timing.freeze_duration_seconds
        ));
    }
    filter.push_str(&format!(
        ",trim=duration={narration_duration:.3},format=yuv420p"
    ));
    Ok(filter)
}

fn calculate_narrated_video_timing(
    video_duration: f64,
    narration_duration: f64,
) -> Result<NarratedVideoTiming, String> {
    validate_duration(video_duration, "分镜视频")?;
    validate_duration(narration_duration, "分镜配音")?;

    if video_duration + 0.001 >= narration_duration {
        return Ok(NarratedVideoTiming {
            playback_rate: 1.0,
            freeze_duration_seconds: 0.0,
        });
    }

    let maximum_slowed_duration = video_duration / MIN_NARRATED_VIDEO_PLAYBACK_RATE;
    let slowed_duration = narration_duration.min(maximum_slowed_duration);
    let playback_rate = video_duration / slowed_duration;
    let freeze_duration_seconds = (narration_duration - slowed_duration).max(0.0);

    if freeze_duration_seconds > MAX_NARRATED_FREEZE_SECONDS + 0.001 {
        return Err(format!(
            "配音时长为 {narration_duration:.1} 秒，但可用画面最长只能安全适配到 {:.1} 秒。请缩短这句文案，或为它换一个更长的片段。",
            maximum_slowed_duration + MAX_NARRATED_FREEZE_SECONDS
        ));
    }

    Ok(NarratedVideoTiming {
        playback_rate,
        freeze_duration_seconds,
    })
}

fn select_narrated_video(
    segment: &NarratedSegmentInput,
    narration_duration: f64,
) -> Result<SelectedNarratedVideo, String> {
    let primary_duration = probe_media_duration(&segment.video_path, "分镜视频")?;
    let mut candidates = Vec::with_capacity(segment.alternative_videos.len() + 1);
    candidates.push(SelectedNarratedVideo {
        path: segment.video_path.clone(),
        duration_seconds: primary_duration,
    });
    for alternative in &segment.alternative_videos {
        let duration = probe_media_duration(&alternative.video_path, "备选分镜视频")?;
        candidates.push(SelectedNarratedVideo {
            path: alternative.video_path.clone(),
            duration_seconds: duration,
        });
    }

    let candidate_durations = candidates
        .iter()
        .map(|candidate| candidate.duration_seconds)
        .collect::<Vec<_>>();
    if let Some(candidate_index) =
        select_best_candidate_index(&candidate_durations, narration_duration)
    {
        return Ok(candidates.swap_remove(candidate_index));
    }

    let longest_duration = segment
        .alternative_videos
        .iter()
        .map(|candidate| candidate.duration_seconds)
        .chain(std::iter::once(segment.video_duration_seconds))
        .fold(0.0_f64, f64::max);
    Err(format!(
        "这句配音约 {narration_duration:.1} 秒，主画面和备选画面都不够长（最长约 {longest_duration:.1} 秒）。请缩短该句文案或重新生成分镜。"
    ))
}

fn select_best_candidate_index(
    candidate_durations: &[f64],
    narration_duration: f64,
) -> Option<usize> {
    if candidate_durations.first().is_some_and(|duration| {
        calculate_narrated_video_timing(*duration, narration_duration).is_ok()
    }) {
        return Some(0);
    }

    candidate_durations
        .iter()
        .enumerate()
        .skip(1)
        .filter(|(_, duration)| {
            calculate_narrated_video_timing(**duration, narration_duration).is_ok()
        })
        .min_by(|(_, left), (_, right)| {
            let left_distance = (**left - narration_duration).abs();
            let right_distance = (**right - narration_duration).abs();
            left_distance.total_cmp(&right_distance)
        })
        .map(|(index, _)| index)
}

fn probe_media_duration(path: &str, label: &str) -> Result<f64, String> {
    let output = background_command(ffprobe_program())
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
    let output = background_command(ffprobe_program())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_video_when_narration_is_shorter() {
        let timing = calculate_narrated_video_timing(5.0, 3.0).unwrap();
        let filter = build_narrated_video_filter(5.0, 3.0, timing).unwrap();

        assert!(filter.contains("trim=duration=3.000"));
        assert!(!filter.contains("tpad="));
    }

    #[test]
    fn gently_slows_video_before_using_a_short_freeze() {
        let timing = calculate_narrated_video_timing(3.0, 4.0).unwrap();
        let filter = build_narrated_video_filter(3.0, 4.0, timing).unwrap();

        assert!(filter.contains("setpts=(PTS-STARTPTS)/0.920000"));
        assert!(filter.contains("tpad=stop_mode=clone:stop_duration=0.739"));
        assert!(filter.contains("trim=duration=4.000"));
    }

    #[test]
    fn rejects_multi_second_last_frame_freeze() {
        let error = calculate_narrated_video_timing(2.0, 4.5).unwrap_err();

        assert!(error.contains("请缩短这句文案"));
    }

    #[test]
    fn selects_a_longer_alternative_when_primary_video_is_too_short() {
        let selected = select_best_candidate_index(&[2.0, 4.0, 3.5], 4.2);

        assert_eq!(selected, Some(1));
    }

    #[test]
    fn rejects_invalid_narration_duration() {
        assert_eq!(
            calculate_narrated_video_timing(2.0, 0.0).unwrap_err(),
            "分镜配音时长无效。"
        );
    }

    #[test]
    fn keeps_only_narration_when_original_audio_is_disabled() {
        let filter = build_narrated_audio_filter(
            3.0,
            3.0,
            1.0,
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
            3.0,
            0.95,
            NarratedAudioSettings {
                keep_original_audio: true,
                original_audio_volume: 0.15,
            },
            true,
        )
        .unwrap();

        assert!(filter.contains("[0:a]atrim=duration=3.000"));
        assert!(filter.contains("atempo=0.950000"));
        assert!(filter.contains("volume=0.150"));
        assert!(filter.contains("amix=inputs=2"));
        assert!(filter.contains("alimiter=limit=0.950"));
    }

    #[test]
    fn falls_back_to_narration_when_video_has_no_audio() {
        let filter = build_narrated_audio_filter(
            3.0,
            3.0,
            1.0,
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
