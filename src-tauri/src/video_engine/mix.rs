use crate::video_engine::canvas::{
    build_canvas_filter, build_plain_video_filter, CanvasAspectRatio, CanvasBackgroundMode,
};
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
    apply_horizontal_mirror: bool,
    playback_speed: f64,
    canvas_aspect_ratio: CanvasAspectRatio,
    canvas_background_mode: CanvasBackgroundMode,
) -> Result<MixVideoResult, String> {
    let output_dir = Path::new(&output_directory);
    let normalized_playback_speed = normalize_playback_speed(playback_speed)?;

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

    let mut ffmpeg_args = vec![
        "-y".to_string(),
        "-f".to_string(),
        "concat".to_string(),
        "-safe".to_string(),
        "0".to_string(),
        "-i".to_string(),
        list_path_text.to_string(),
    ];

    let mut video_filters = Vec::new();

    if apply_horizontal_mirror {
        video_filters.push("hflip".to_string());
    }

    if should_apply_speed_filter(normalized_playback_speed) {
        video_filters.push(format!("setpts=PTS/{normalized_playback_speed:.3}"));
    }

    let video_filter =
        build_canvas_filter(canvas_aspect_ratio, canvas_background_mode, &video_filters)
            .or_else(|| build_plain_video_filter(&video_filters));

    if let Some(video_filter) = video_filter {
        if video_filter.is_complex {
            ffmpeg_args.push("-filter_complex".to_string());
            ffmpeg_args.push(video_filter.filter);
            ffmpeg_args.push("-map".to_string());
            ffmpeg_args.push("[v]".to_string());
            ffmpeg_args.push("-map".to_string());
            ffmpeg_args.push("0:a?".to_string());
        } else {
            ffmpeg_args.push("-vf".to_string());
            ffmpeg_args.push(video_filter.filter);
        }
    }

    if should_apply_speed_filter(normalized_playback_speed) {
        ffmpeg_args.push("-af".to_string());
        ffmpeg_args.push(format!("atempo={normalized_playback_speed:.3}"));
    }

    ffmpeg_args.extend([
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "veryfast".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
        "-c:a".to_string(),
        "aac".to_string(),
        "-movflags".to_string(),
        "+faststart".to_string(),
        output_path_text.to_string(),
    ]);

    let output = Command::new("ffmpeg")
        .args(ffmpeg_args)
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

fn normalize_playback_speed(playback_speed: f64) -> Result<f64, String> {
    if !playback_speed.is_finite() {
        return Err("变速倍数必须是有效数字。".to_string());
    }

    if !(0.5..=2.0).contains(&playback_speed) {
        return Err("变速倍数暂时只支持 0.5 到 2.0。".to_string());
    }

    Ok((playback_speed * 1000.0).round() / 1000.0)
}

fn should_apply_speed_filter(playback_speed: f64) -> bool {
    (playback_speed - 1.0).abs() > 0.001
}

fn current_timestamp() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("无法生成拼接文件名：{error}"))
        .map(|duration| duration.as_millis() as u64)
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
