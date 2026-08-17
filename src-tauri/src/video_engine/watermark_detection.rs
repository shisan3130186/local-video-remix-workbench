use crate::video_engine::probe::{probe_video_dimensions, probe_video_duration_seconds};
use crate::video_engine::tool_paths::{background_command, ffmpeg_program};
use serde::Serialize;
use std::cmp::Ordering;
use std::process::Stdio;

const SAMPLE_WIDTH: usize = 160;
const REGION_WIDTH_RATIO: f64 = 0.22;
const REGION_HEIGHT_RATIO: f64 = 0.14;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedWatermarkRegion {
    pub x_ratio: f64,
    pub y_ratio: f64,
    pub width_ratio: f64,
    pub height_ratio: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WatermarkDetectionResult {
    pub regions: Vec<DetectedWatermarkRegion>,
    pub confidence: f64,
    pub message: String,
}

#[derive(Debug, Clone, Copy)]
struct Candidate {
    x_ratio: f64,
    y_ratio: f64,
    score: f64,
}

pub fn detect_watermark_regions(
    input_file_path: String,
    max_regions: u32,
) -> Result<WatermarkDetectionResult, String> {
    if max_regions == 0 || max_regions > 8 {
        return Err("自动检测区域数必须在 1 到 8 之间。".to_string());
    }
    let (source_width, source_height) = probe_video_dimensions(&input_file_path)?;
    let duration_seconds = probe_video_duration_seconds(&input_file_path)?;
    if source_width < 16 || source_height < 16 {
        return Err("视频画面太小，无法进行自动检测。".to_string());
    }

    let sample_height = ((SAMPLE_WIDTH as f64 * f64::from(source_height) / f64::from(source_width))
        .round() as usize)
        .max(2);
    let sample_times = [0.12, 0.5, 0.88];
    let frames = sample_times
        .iter()
        .map(|ratio| {
            read_gray_frame(
                &input_file_path,
                (duration_seconds * ratio).min((duration_seconds - 0.02).max(0.0)),
                sample_height,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let candidates = build_candidates(&frames, SAMPLE_WIDTH, sample_height);
    let best = candidates
        .first()
        .copied()
        .ok_or_else(|| "没有生成可分析的画面区域。".to_string())?;
    let selected = select_non_overlapping_candidates(&candidates, max_regions as usize);
    let best_score = best.score;
    let confidence = (1.0 - best_score / 96.0).clamp(0.05, 0.98);
    let regions = selected
        .into_iter()
        .map(|candidate| DetectedWatermarkRegion {
            x_ratio: candidate.x_ratio,
            y_ratio: candidate.y_ratio,
            width_ratio: REGION_WIDTH_RATIO,
            height_ratio: REGION_HEIGHT_RATIO,
            confidence: (1.0 - candidate.score / 96.0).clamp(0.05, 0.98),
        })
        .collect::<Vec<_>>();

    Ok(WatermarkDetectionResult {
        regions,
        confidence,
        message: "已根据三帧画面的静态标记特征生成候选区域，可继续拖动红框微调。".to_string(),
    })
}

fn read_gray_frame(path: &str, time_ratio: f64, sample_height: usize) -> Result<Vec<u8>, String> {
    let time = format!("{:.3}", time_ratio.clamp(0.0, 0.99));
    let filter = format!("scale={SAMPLE_WIDTH}:{sample_height},format=gray");
    let output = background_command(ffmpeg_program())
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-ss",
            &time,
            "-i",
            path,
            "-frames:v",
            "1",
            "-vf",
            &filter,
            "-f",
            "rawvideo",
            "-pix_fmt",
            "gray",
            "pipe:1",
        ])
        .stdout(Stdio::piped())
        .output()
        .map_err(|error| format!("无法启动 FFmpeg 进行水印检测：{error}"))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if detail.is_empty() {
            "FFmpeg 读取检测画面失败。".to_string()
        } else {
            format!("FFmpeg 读取检测画面失败：{detail}")
        });
    }
    let expected = SAMPLE_WIDTH * sample_height;
    if output.stdout.len() != expected {
        return Err("检测画面数据不完整，无法判断水印区域。".to_string());
    }
    Ok(output.stdout)
}

fn build_candidates(frames: &[Vec<u8>], width: usize, height: usize) -> Vec<Candidate> {
    if frames.len() < 2 || frames.iter().any(|frame| frame.len() != width * height) {
        return Vec::new();
    }
    let region_width = ((width as f64 * REGION_WIDTH_RATIO).round() as usize).max(4);
    let region_height = ((height as f64 * REGION_HEIGHT_RATIO).round() as usize).max(4);
    let x_ratios = [0.0, 0.39, 0.78];
    let y_ratios = [0.0, 0.43, 0.86];
    let mut candidates = x_ratios
        .into_iter()
        .flat_map(|x_ratio| {
            y_ratios.into_iter().map(move |y_ratio| Candidate {
                x_ratio,
                y_ratio,
                score: region_score(
                    frames,
                    width,
                    height,
                    region_width,
                    region_height,
                    x_ratio,
                    y_ratio,
                ),
            })
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        left.score
            .partial_cmp(&right.score)
            .unwrap_or(Ordering::Equal)
    });
    candidates
}

fn region_score(
    frames: &[Vec<u8>],
    width: usize,
    height: usize,
    region_width: usize,
    region_height: usize,
    x_ratio: f64,
    y_ratio: f64,
) -> f64 {
    let x = ((x_ratio * width as f64).round() as usize).min(width.saturating_sub(region_width));
    let y = ((y_ratio * height as f64).round() as usize).min(height.saturating_sub(region_height));
    let mut temporal_delta = 0.0;
    let mut brightness = 0.0;
    let mut samples = 0usize;
    for row in 0..region_height {
        for column in 0..region_width {
            let offset = (y + row) * width + x + column;
            brightness += f64::from(frames[0][offset]);
            for pair in frames.windows(2) {
                temporal_delta += f64::from(pair[1][offset].abs_diff(pair[0][offset]));
            }
            samples += 1;
        }
    }
    if samples == 0 {
        return f64::MAX;
    }
    let average_brightness = brightness / samples as f64;
    let black_bar_penalty = if average_brightness < 8.0 { 48.0 } else { 0.0 };
    temporal_delta / samples as f64 + black_bar_penalty
}

fn select_non_overlapping_candidates(
    candidates: &[Candidate],
    max_regions: usize,
) -> Vec<Candidate> {
    let mut selected = Vec::new();
    for candidate in candidates {
        let overlaps = selected.iter().any(|item: &Candidate| {
            (candidate.x_ratio - item.x_ratio).abs() < REGION_WIDTH_RATIO
                && (candidate.y_ratio - item.y_ratio).abs() < REGION_HEIGHT_RATIO
        });
        if !overlaps {
            selected.push(*candidate);
        }
        if selected.len() >= max_regions {
            break;
        }
    }
    selected
}

#[cfg(test)]
mod tests {
    use super::{
        build_candidates, detect_watermark_regions, region_score,
        select_non_overlapping_candidates, Candidate,
    };

    #[test]
    fn static_corner_is_ranked_before_moving_corner() {
        let width = 20;
        let height = 10;
        let mut first = vec![120u8; width * height];
        let mut second = first.clone();
        let mut third = first.clone();
        for frame in [&mut first, &mut second, &mut third] {
            for row in 0..2 {
                for column in 0..4 {
                    frame[row * width + column] = 220;
                }
            }
        }
        for row in 6..8 {
            for column in 16..20 {
                first[row * width + column] = 0;
                second[row * width + column] = 255;
                third[row * width + column] = 0;
            }
        }
        let candidates = build_candidates(&[first, second, third], width, height);
        assert_eq!(candidates.first().map(|item| item.x_ratio), Some(0.0));
        assert_eq!(candidates.first().map(|item| item.y_ratio), Some(0.0));
    }

    #[test]
    fn black_bars_are_penalized() {
        let frames = vec![vec![0u8; 20 * 10], vec![0u8; 20 * 10], vec![0u8; 20 * 10]];
        let score = region_score(&frames, 20, 10, 4, 2, 0.0, 0.0);
        assert!(score >= 48.0);
    }

    #[test]
    fn selected_candidates_do_not_overlap() {
        let candidates = [
            Candidate {
                x_ratio: 0.0,
                y_ratio: 0.0,
                score: 1.0,
            },
            Candidate {
                x_ratio: 0.05,
                y_ratio: 0.02,
                score: 2.0,
            },
            Candidate {
                x_ratio: 0.78,
                y_ratio: 0.86,
                score: 3.0,
            },
        ];
        let selected = select_non_overlapping_candidates(&candidates, 3);
        assert_eq!(selected.len(), 2);
    }

    #[test]
    fn detects_a_static_corner_mark_in_a_real_video() {
        use crate::video_engine::tool_paths::ffmpeg_program;
        use std::process::Command;
        use std::time::{SystemTime, UNIX_EPOCH};

        let root = std::env::temp_dir().join(format!(
            "smartcut-watermark-detection-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let source = root.join("source.mp4");
        let source_text = source.to_string_lossy().to_string();
        let generated = Command::new(ffmpeg_program())
            .args([
                "-y",
                "-f",
                "lavfi",
                "-i",
                "color=c=black:s=320x180:rate=10",
                "-vf",
                "drawbox=x=0:y=0:w=70:h=24:color=white:t=fill,drawbox=x='mod(t*20\\,250)':y=100:w=20:h=20:color=red:t=fill",
                "-t",
                "3",
                "-an",
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
                &source_text,
            ])
            .output()
            .unwrap();
        assert!(
            generated.status.success(),
            "{}",
            String::from_utf8_lossy(&generated.stderr)
        );

        let result = detect_watermark_regions(source_text, 1).unwrap();
        assert_eq!(result.regions.len(), 1);
        assert_eq!(result.regions[0].x_ratio, 0.0);
        assert_eq!(result.regions[0].y_ratio, 0.0);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn detects_configured_acceptance_fixture_when_present() {
        let Ok(path) = std::env::var("SMARTCUT_WATERMARK_FIXTURE") else {
            return;
        };
        if !std::path::Path::new(&path).is_file() {
            panic!("configured watermark fixture does not exist: {path}");
        }
        let result = detect_watermark_regions(path, 2).expect("fixture detection should succeed");
        assert!(!result.regions.is_empty());
        assert!(result.regions.iter().all(|region| {
            region.x_ratio >= 0.0
                && region.y_ratio >= 0.0
                && region.x_ratio + region.width_ratio <= 1.0
                && region.y_ratio + region.height_ratio <= 1.0
        }));
    }
}
