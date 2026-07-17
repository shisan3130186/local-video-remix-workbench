use crate::task_runtime::{run_ffmpeg, TaskProgressContext};
use crate::video_engine::canvas::{
    build_canvas_filter_with_dimensions, CanvasAspectRatio, CanvasBackgroundMode,
};
use crate::video_engine::output::{
    append_final_output_args, build_output_video_filters, output_canvas_dimensions, OutputSettings,
};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderVideoResult {
    output_path: String,
    message: String,
    output_resolution: String,
    output_encoder: String,
    output_frame_rate: String,
    output_quality: String,
    output_video_bitrate_kbps: u32,
}

pub fn export_basic_video(
    input_file_path: String,
    output_directory: String,
    canvas_aspect_ratio: CanvasAspectRatio,
    canvas_background_mode: CanvasBackgroundMode,
    duration_seconds: Option<f64>,
    task_context: Option<TaskProgressContext>,
    output_settings: OutputSettings,
) -> Result<RenderVideoResult, String> {
    let input_path = Path::new(&input_file_path);
    let output_dir = Path::new(&output_directory);

    if !input_path.is_file() {
        return Err("请选择一个已导入的视频文件。".to_string());
    }

    if !output_dir.is_dir() {
        return Err("请选择有效的输出目录。".to_string());
    }

    let output_path = build_output_path(input_path, output_dir)?;
    let output_path_text = output_path
        .to_str()
        .ok_or_else(|| "输出路径包含无法识别的字符。".to_string())?;
    let mut ffmpeg_args = vec!["-y".to_string(), "-i".to_string(), input_file_path];
    let output_dimensions = output_canvas_dimensions(output_settings, canvas_aspect_ratio);
    let video_filters = build_output_video_filters(output_settings, canvas_aspect_ratio);

    if let Some(canvas_filter) = build_canvas_filter_with_dimensions(
        canvas_aspect_ratio,
        canvas_background_mode,
        &video_filters,
        output_dimensions,
    ) {
        if canvas_filter.is_complex {
            ffmpeg_args.push("-filter_complex".to_string());
            ffmpeg_args.push(canvas_filter.filter);
            ffmpeg_args.push("-map".to_string());
            ffmpeg_args.push("[v]".to_string());
            ffmpeg_args.push("-map".to_string());
            ffmpeg_args.push("0:a?".to_string());
        } else {
            ffmpeg_args.push("-vf".to_string());
            ffmpeg_args.push(canvas_filter.filter);
        }
    }

    let applied_output_settings =
        append_final_output_args(&mut ffmpeg_args, output_settings, canvas_aspect_ratio)?;
    ffmpeg_args.push(output_path_text.to_string());

    if let Err(error) = run_ffmpeg(
        ffmpeg_args,
        task_context.as_ref(),
        duration_seconds,
        "视频导出失败。",
    ) {
        let _ = fs::remove_file(&output_path);
        return Err(error);
    }

    if !output_path.is_file() {
        return Err("导出命令已结束，但没有找到输出文件。".to_string());
    }

    Ok(RenderVideoResult {
        output_path: output_path.to_string_lossy().to_string(),
        message: "导出完成。".to_string(),
        output_resolution: applied_output_settings.resolution_label,
        output_encoder: applied_output_settings.encoder_label,
        output_frame_rate: applied_output_settings.frame_rate_label,
        output_quality: applied_output_settings.quality_label,
        output_video_bitrate_kbps: applied_output_settings.video_bitrate_kbps,
    })
}

fn build_output_path(input_path: &Path, output_dir: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(output_dir).map_err(|error| format!("无法创建输出目录：{error}"))?;

    let file_stem = input_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("video");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("无法生成导出文件名：{error}"))?
        .as_millis();
    let file_name = format!("{file_stem}_export_{timestamp}.mp4");

    Ok(output_dir.join(file_name))
}
