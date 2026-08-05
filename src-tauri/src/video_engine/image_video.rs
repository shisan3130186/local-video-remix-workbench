use crate::task_runtime::{run_ffmpeg, TaskProgressContext};
use crate::video_engine::canvas::CanvasAspectRatio;
use crate::video_engine::output::{append_final_output_args, OutputSettings};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageVideoItemResult {
    source_path: String,
    output_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageVideoFailure {
    source_path: String,
    message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageVideoBatchResult {
    items: Vec<ImageVideoItemResult>,
    failures: Vec<ImageVideoFailure>,
}

pub fn convert_images_to_videos(
    image_paths: Vec<String>,
    output_directory: String,
    duration_seconds: f64,
    aspect_ratio: CanvasAspectRatio,
    output_settings: OutputSettings,
    task_context: Option<TaskProgressContext>,
) -> Result<ImageVideoBatchResult, String> {
    if image_paths.is_empty() {
        return Err("请先导入图片。".to_string());
    }
    if !(0.5..=120.0).contains(&duration_seconds) {
        return Err("单张图片时长需要在0.5到120秒之间。".to_string());
    }
    let output_dir = Path::new(&output_directory);
    if !output_dir.is_dir() {
        return Err("请选择有效的输出目录。".to_string());
    }

    let mut items = Vec::new();
    let mut failures = Vec::new();
    let total = image_paths.len().max(1) as f64;
    for (index, source_text) in image_paths.into_iter().enumerate() {
        let source = Path::new(&source_text);
        if !source.is_file() {
            failures.push(ImageVideoFailure {
                source_path: source_text,
                message: "图片不存在。".to_string(),
            });
            continue;
        }
        let output_path = unique_output_path(source, output_dir);
        let output_text = output_path.to_string_lossy().to_string();
        let filter = build_image_filter(aspect_ratio);
        let mut args = vec![
            "-y".to_string(),
            "-loop".to_string(),
            "1".to_string(),
            "-framerate".to_string(),
            "30".to_string(),
            "-i".to_string(),
            source_text.clone(),
            "-t".to_string(),
            format!("{duration_seconds:.3}"),
            "-vf".to_string(),
            filter,
        ];
        if let Err(error) = append_final_output_args(&mut args, output_settings, aspect_ratio) {
            failures.push(ImageVideoFailure {
                source_path: source_text,
                message: error,
            });
            continue;
        }
        args.push(output_text.clone());
        let child_context = task_context.as_ref().map(|context| {
            context.child(
                index as f64 / total,
                (index + 1) as f64 / total,
                format!("正在处理第{}张图片", index + 1),
            )
        });
        match run_ffmpeg(
            args,
            child_context.as_ref(),
            Some(duration_seconds),
            "图片转视频失败。",
        ) {
            Ok(()) if output_path.is_file() => items.push(ImageVideoItemResult {
                source_path: source_text,
                output_path: output_text,
            }),
            Ok(()) => failures.push(ImageVideoFailure {
                source_path: source_text,
                message: "处理结束，但没有找到输出视频。".to_string(),
            }),
            Err(error) => {
                let _ = fs::remove_file(&output_path);
                failures.push(ImageVideoFailure {
                    source_path: source_text,
                    message: error,
                });
            }
        }
    }

    Ok(ImageVideoBatchResult { items, failures })
}

fn build_image_filter(aspect_ratio: CanvasAspectRatio) -> String {
    match aspect_ratio {
        CanvasAspectRatio::Portrait916 => scale_pad_filter(1080, 1920),
        CanvasAspectRatio::Square11 => scale_pad_filter(1080, 1080),
        CanvasAspectRatio::Landscape169 => scale_pad_filter(1920, 1080),
        CanvasAspectRatio::Original => {
            "scale=trunc(iw/2)*2:trunc(ih/2)*2,format=yuv420p".to_string()
        }
    }
}

fn scale_pad_filter(width: u32, height: u32) -> String {
    format!(
        "scale={width}:{height}:force_original_aspect_ratio=decrease,pad={width}:{height}:(ow-iw)/2:(oh-ih)/2:black,format=yuv420p"
    )
}

fn unique_output_path(source: &Path, output_dir: &Path) -> PathBuf {
    let stem = source
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("image");
    let base = output_dir.join(format!("{stem}_视频.mp4"));
    if !base.exists() {
        return base;
    }
    for index in 2..=9999 {
        let candidate = output_dir.join(format!("{stem}_视频_{index}.mp4"));
        if !candidate.exists() {
            return candidate;
        }
    }
    output_dir.join(format!("{stem}_视频_{}.mp4", uuid::Uuid::new_v4()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_engine::output::{OutputFrameRate, VideoEncoder};
    use uuid::Uuid;

    #[test]
    fn portrait_filter_keeps_target_canvas() {
        let filter = build_image_filter(CanvasAspectRatio::Portrait916);
        assert!(filter.contains("1080:1920"));
        assert!(filter.contains("pad=1080:1920"));
    }

    #[test]
    fn converts_a_real_image_with_video_engine() {
        let root = std::env::temp_dir().join(format!("smartcut-image-video-{}", Uuid::new_v4()));
        let output = root.join("output");
        fs::create_dir_all(&output).unwrap();
        let source = root.join("sample.ppm");
        fs::write(
            &source,
            b"P3\n2 2\n255\n255 0 0  0 255 0\n0 0 255  255 255 255\n",
        )
        .unwrap();

        let result = convert_images_to_videos(
            vec![source.to_string_lossy().to_string()],
            output.to_string_lossy().to_string(),
            0.5,
            CanvasAspectRatio::Original,
            OutputSettings {
                frame_rate: OutputFrameRate::Fps30,
                encoder: VideoEncoder::Cpu,
                ..OutputSettings::default()
            },
            None,
        )
        .unwrap();

        assert!(result.failures.is_empty());
        assert_eq!(result.items.len(), 1);
        let output_path = Path::new(&result.items[0].output_path);
        assert!(output_path.is_file());
        assert!(fs::metadata(output_path).unwrap().len() > 0);
        fs::remove_dir_all(root).unwrap();
    }
}
