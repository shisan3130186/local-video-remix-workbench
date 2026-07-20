use crate::task_runtime::{run_ffmpeg, TaskProgressContext};
use crate::temp_storage::TaskTempDirectory;
use crate::video_engine::canvas::{
    build_canvas_filter_with_dimensions, CanvasAspectRatio, CanvasBackgroundMode, CanvasFilter,
};
use crate::video_engine::output::{
    append_final_output_args, build_output_video_filters, output_canvas_dimensions,
    resolve_output_video_dimensions, OutputSettings,
};
use crate::video_engine::probe::probe_video_dimensions;
use crate::video_engine::watermark::{
    append_watermark_input_args, build_image_watermark_layer, build_text_watermark_layer,
    normalize_watermark_settings, prepare_text_watermark_file, uses_image_input, WatermarkKind,
    WatermarkSettings,
};
use crate::video_engine::watermark_removal::{
    build_watermark_removal_layer, normalize_watermark_removal_settings,
    validate_tracking_duration, WatermarkRemovalSettings,
};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasicExportSettings {
    canvas_aspect_ratio: CanvasAspectRatio,
    canvas_background_mode: CanvasBackgroundMode,
    #[serde(default)]
    output_settings: OutputSettings,
    #[serde(default)]
    watermark_settings: WatermarkSettings,
    #[serde(default)]
    watermark_removal_settings: WatermarkRemovalSettings,
}

pub fn export_basic_video(
    input_file_path: String,
    output_directory: String,
    duration_seconds: Option<f64>,
    task_context: Option<TaskProgressContext>,
    settings: BasicExportSettings,
) -> Result<RenderVideoResult, String> {
    let BasicExportSettings {
        canvas_aspect_ratio,
        canvas_background_mode,
        output_settings,
        watermark_settings,
        watermark_removal_settings,
    } = settings;
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
    let normalized_watermark_settings = normalize_watermark_settings(watermark_settings)?;
    let normalized_watermark_removal_settings =
        normalize_watermark_removal_settings(watermark_removal_settings)?;
    if normalized_watermark_removal_settings
        .as_ref()
        .is_some_and(|settings| settings.tracking_enabled)
        && canvas_aspect_ratio != CanvasAspectRatio::Original
    {
        return Err("移动水印关键帧跟踪第一版需要把画布比例设置为“原画”。".to_string());
    }
    validate_tracking_duration(
        normalized_watermark_removal_settings.as_ref(),
        duration_seconds,
    )?;
    let temp_directory = normalized_watermark_settings
        .as_ref()
        .map(|_| TaskTempDirectory::create("watermark"))
        .transpose()?;
    let watermark_text_file = normalized_watermark_settings
        .as_ref()
        .zip(temp_directory.as_ref())
        .map(|(settings, directory)| prepare_text_watermark_file(settings, directory.path()))
        .transpose()?
        .flatten();

    let mut ffmpeg_args = vec!["-y".to_string(), "-i".to_string(), input_file_path.clone()];
    let watermark_image_input_index = normalized_watermark_settings
        .as_ref()
        .filter(|settings| uses_image_input(settings))
        .map(|settings| {
            append_watermark_input_args(&mut ffmpeg_args, settings)?;
            Ok::<usize, String>(1)
        })
        .transpose()?;
    let output_dimensions = output_canvas_dimensions(output_settings, canvas_aspect_ratio);
    let watermark_removal_frame_dimensions = normalized_watermark_removal_settings
        .as_ref()
        .map(|_| probe_video_dimensions(input_path.to_string_lossy().as_ref()))
        .transpose()?
        .map(|source_dimensions| {
            resolve_output_video_dimensions(output_settings, canvas_aspect_ratio, source_dimensions)
        });
    let video_filters = build_output_video_filters(output_settings, canvas_aspect_ratio);

    let base_filter = build_canvas_filter_with_dimensions(
        canvas_aspect_ratio,
        canvas_background_mode,
        &video_filters,
        output_dimensions,
    );
    let video_filter = build_processed_video_filter(
        base_filter,
        normalized_watermark_removal_settings.as_ref(),
        watermark_removal_frame_dimensions,
        normalized_watermark_settings.as_ref(),
        watermark_image_input_index,
        watermark_text_file.as_deref(),
    )?;

    if let Some(canvas_filter) = video_filter {
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

    if normalized_watermark_settings
        .as_ref()
        .is_some_and(uses_image_input)
    {
        ffmpeg_args.push("-shortest".to_string());
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

fn build_processed_video_filter(
    base_filter: Option<CanvasFilter>,
    watermark_removal_settings: Option<&WatermarkRemovalSettings>,
    watermark_removal_frame_dimensions: Option<(u32, u32)>,
    watermark_settings: Option<&WatermarkSettings>,
    watermark_image_input_index: Option<usize>,
    watermark_text_file: Option<&Path>,
) -> Result<Option<CanvasFilter>, String> {
    if watermark_removal_settings.is_none() && watermark_settings.is_none() {
        return Ok(base_filter);
    }

    let mut filters = Vec::new();
    let base_label = "[processbase]";
    if let Some(base_filter) = base_filter {
        if base_filter.is_complex {
            filters.push(base_filter.filter.replace("[v]", base_label));
        } else {
            filters.push(format!("[0:v]{}{base_label}", base_filter.filter));
        }
    } else {
        filters.push(format!("[0:v]format=yuv420p{base_label}"));
    }

    let mut current_label = base_label.to_string();
    let mut layer_number = 1usize;
    if let Some(settings) = watermark_removal_settings {
        let output_label = format!("[process{layer_number}]");
        filters.push(build_watermark_removal_layer(
            &current_label,
            &output_label,
            settings,
            layer_number,
            watermark_removal_frame_dimensions
                .ok_or_else(|| "没有读取到原水印处理所需的画面尺寸。".to_string())?,
        ));
        current_label = output_label;
        layer_number += 1;
    }

    if let Some(settings) = watermark_settings {
        let output_label = format!("[process{layer_number}]");
        let filter = match settings.kind {
            WatermarkKind::Text => build_text_watermark_layer(
                &current_label,
                &output_label,
                settings,
                watermark_text_file.ok_or_else(|| "没有找到临时文字水印文件。".to_string())?,
            )?,
            WatermarkKind::Image => build_image_watermark_layer(
                &current_label,
                &output_label,
                watermark_image_input_index
                    .ok_or_else(|| "没有找到图片水印输入流。".to_string())?,
                settings,
                layer_number,
            ),
        };
        filters.push(filter);
        current_label = output_label;
    }

    filters.push(format!("{current_label}format=yuv420p[v]"));
    Ok(Some(CanvasFilter {
        filter: filters.join(";"),
        is_complex: true,
    }))
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
