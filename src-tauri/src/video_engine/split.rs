use crate::task_runtime::{run_ffmpeg, run_ffmpeg_capture_stderr, TaskProgressContext};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitVideoResult {
    output_directory: String,
    segment_paths: Vec<String>,
    segment_count: usize,
    detected_scene_count: usize,
    split_mode: SplitMode,
    message: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SplitMode {
    Duration,
    Scene,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SceneSensitivity {
    Stable,
    Balanced,
    Sensitive,
}

impl SceneSensitivity {
    fn threshold(self) -> f64 {
        match self {
            Self::Stable => 0.45,
            Self::Balanced => 0.32,
            Self::Sensitive => 0.22,
        }
    }
}

pub fn split_video_by_duration(
    input_file_path: String,
    output_directory: String,
    segment_duration_seconds: f64,
    input_duration_seconds: Option<f64>,
    task_context: Option<TaskProgressContext>,
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

    if let Err(error) = run_ffmpeg(
        [
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
            "-segment_time_delta",
            "0.05",
            "-reset_timestamps",
            "1",
            segment_pattern_text,
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        task_context.as_ref(),
        input_duration_seconds,
        "视频切片失败。",
    ) {
        let _ = fs::remove_dir_all(&segment_dir);
        return Err(error);
    }

    let segment_paths = list_generated_segments(&segment_dir)?;

    if segment_paths.is_empty() {
        return Err("切片命令已结束，但没有找到生成的片段文件。".to_string());
    }

    Ok(SplitVideoResult {
        output_directory: segment_dir.to_string_lossy().to_string(),
        segment_count: segment_paths.len(),
        segment_paths,
        detected_scene_count: 0,
        split_mode: SplitMode::Duration,
        message: "切片完成。".to_string(),
    })
}

pub fn split_video_by_scene(
    input_file_path: String,
    output_directory: String,
    sensitivity: SceneSensitivity,
    minimum_segment_seconds: f64,
    maximum_segment_seconds: f64,
    input_duration_seconds: Option<f64>,
    task_context: Option<TaskProgressContext>,
) -> Result<SplitVideoResult, String> {
    let input_path = Path::new(&input_file_path);
    let output_dir = Path::new(&output_directory);

    if !input_path.is_file() {
        return Err("请选择一个已导入的视频文件。".to_string());
    }
    if !output_dir.is_dir() {
        return Err("请选择有效的输出目录。".to_string());
    }
    validate_scene_settings(minimum_segment_seconds, maximum_segment_seconds)?;
    let duration_seconds = input_duration_seconds
        .filter(|duration| duration.is_finite() && *duration > 0.0)
        .ok_or_else(|| "智能切片需要先读取到有效的视频时长。".to_string())?;

    let detection_context = task_context
        .as_ref()
        .map(|context| context.child(0.0, 0.25, "正在扫描画面变化"));
    let threshold = format!("{:.2}", sensitivity.threshold());
    let filter = format!("select='gt(scene,{threshold})',showinfo");
    let stderr = run_ffmpeg_capture_stderr(
        [
            "-i",
            &input_file_path,
            "-vf",
            &filter,
            "-an",
            "-f",
            "null",
            "-",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        detection_context.as_ref(),
        Some(duration_seconds),
        "扫描画面变化失败。",
    )?;
    let detected_timestamps = parse_scene_timestamps(&stderr);
    let cut_times = build_smart_cut_times(
        &detected_timestamps,
        duration_seconds,
        minimum_segment_seconds,
        maximum_segment_seconds,
    );

    let segment_dir = build_segment_directory(input_path, output_dir)?;
    fs::create_dir_all(&segment_dir).map_err(|error| format!("无法创建切片目录：{error}"))?;
    let segment_pattern = segment_dir.join("segment_%03d.mp4");
    let segment_pattern_text = segment_pattern
        .to_str()
        .ok_or_else(|| "切片输出路径包含无法识别的字符。".to_string())?;
    let cut_times_text = cut_times
        .iter()
        .map(|time| format!("{time:.3}"))
        .collect::<Vec<_>>()
        .join(",");
    let encoding_context = task_context
        .as_ref()
        .map(|context| context.child(0.25, 1.0, "正在生成智能切片"));
    let mut args = vec![
        "-y".to_string(),
        "-i".to_string(),
        input_file_path.clone(),
        "-map".to_string(),
        "0:v:0".to_string(),
        "-map".to_string(),
        "0:a?".to_string(),
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "veryfast".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
        "-c:a".to_string(),
        "aac".to_string(),
    ];
    if !cut_times_text.is_empty() {
        args.extend([
            "-force_key_frames".to_string(),
            cut_times_text.clone(),
            "-segment_times".to_string(),
            cut_times_text,
            "-segment_time_delta".to_string(),
            "0.05".to_string(),
        ]);
    }
    args.extend([
        "-f".to_string(),
        "segment".to_string(),
        "-reset_timestamps".to_string(),
        "1".to_string(),
        segment_pattern_text.to_string(),
    ]);

    if let Err(error) = run_ffmpeg(
        args,
        encoding_context.as_ref(),
        Some(duration_seconds),
        "智能切片失败。",
    ) {
        let _ = fs::remove_dir_all(&segment_dir);
        return Err(error);
    }

    let segment_paths = list_generated_segments(&segment_dir)?;
    if segment_paths.is_empty() {
        return Err("智能切片命令已结束，但没有找到生成的片段文件。".to_string());
    }

    Ok(SplitVideoResult {
        output_directory: segment_dir.to_string_lossy().to_string(),
        segment_count: segment_paths.len(),
        segment_paths,
        detected_scene_count: detected_timestamps.len(),
        split_mode: SplitMode::Scene,
        message: if detected_timestamps.is_empty() {
            "未发现明显转场，已按最长片段时长智能兜底。".to_string()
        } else {
            format!("识别到 {} 个画面变化候选点。", detected_timestamps.len())
        },
    })
}

fn validate_scene_settings(minimum: f64, maximum: f64) -> Result<(), String> {
    if !minimum.is_finite() || !(0.5..=10.0).contains(&minimum) {
        return Err("最短片段时长必须在 0.5 到 10 秒之间。".to_string());
    }
    if !maximum.is_finite() || !(2.0..=60.0).contains(&maximum) {
        return Err("最长片段时长必须在 2 到 60 秒之间。".to_string());
    }
    if maximum < minimum + 0.5 {
        return Err("最长片段时长需要至少比最短片段多 0.5 秒。".to_string());
    }
    Ok(())
}

fn parse_scene_timestamps(stderr: &str) -> Vec<f64> {
    let mut timestamps = stderr
        .lines()
        .filter_map(|line| {
            let (_, remainder) = line.split_once("pts_time:")?;
            let value = remainder.split_whitespace().next()?;
            value.parse::<f64>().ok()
        })
        .filter(|time| time.is_finite() && *time > 0.0)
        .collect::<Vec<_>>();
    timestamps.sort_by(f64::total_cmp);
    timestamps.dedup_by(|left, right| (*left - *right).abs() < 0.01);
    timestamps
}

fn build_smart_cut_times(detected: &[f64], duration: f64, minimum: f64, maximum: f64) -> Vec<f64> {
    let mut cuts = Vec::new();
    let mut segment_start = 0.0;

    for &candidate in detected {
        if candidate >= duration || candidate - segment_start < minimum {
            continue;
        }
        while candidate - segment_start > maximum {
            segment_start += maximum;
            if duration - segment_start >= minimum {
                cuts.push(segment_start);
            }
        }
        if candidate - segment_start >= minimum && duration - candidate >= minimum {
            cuts.push(candidate);
            segment_start = candidate;
        }
    }

    while duration - segment_start > maximum {
        let next = segment_start + maximum;
        if duration - next < minimum {
            break;
        }
        cuts.push(next);
        segment_start = next;
    }
    if cuts.is_empty() && duration >= minimum * 2.0 {
        cuts.push(duration / 2.0);
    }
    cuts.sort_by(f64::total_cmp);
    cuts.dedup_by(|left, right| (*left - *right).abs() < 0.05);
    cuts
}

fn build_segment_directory(input_path: &Path, output_dir: &Path) -> Result<PathBuf, String> {
    let file_stem = input_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("video");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("无法生成切片目录名：{error}"))?
        .as_millis();
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

#[cfg(test)]
mod tests {
    use super::{build_smart_cut_times, parse_scene_timestamps};

    #[test]
    fn parses_scene_timestamps_from_showinfo() {
        let output = "[Parsed_showinfo_1] n: 0 pts: 90 pts_time:3.000 pos: 1\n[Parsed_showinfo_1] n: 1 pts: 240 pts_time:8.000 pos: 2";
        assert_eq!(parse_scene_timestamps(output), vec![3.0, 8.0]);
    }

    #[test]
    fn removes_scene_cuts_that_would_create_tiny_segments() {
        let cuts = build_smart_cut_times(&[0.4, 2.2, 2.8, 7.0], 10.0, 2.0, 8.0);
        assert_eq!(cuts, vec![2.2, 7.0]);
    }

    #[test]
    fn inserts_fallback_cuts_for_long_continuous_scenes() {
        let cuts = build_smart_cut_times(&[], 31.0, 2.0, 10.0);
        assert_eq!(cuts, vec![10.0, 20.0]);
    }

    #[test]
    fn splits_short_continuous_video_into_two_usable_segments() {
        let cuts = build_smart_cut_times(&[], 6.0, 2.0, 10.0);
        assert_eq!(cuts, vec![3.0]);
    }
}
