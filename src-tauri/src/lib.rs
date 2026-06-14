mod video_engine;

use std::path::Path;
use std::process::Command;
use video_engine::canvas::{CanvasAspectRatio, CanvasBackgroundMode};
use video_engine::import::list_supported_videos_in_folder;
use video_engine::mix::{concat_video_segments, MixVideoResult};
use video_engine::probe::{
    check_environment, probe_video_metadata, FfmpegEnvironmentResult, VideoMetadata,
};
use video_engine::render::{export_basic_video, RenderVideoResult};
use video_engine::split::{split_video_by_duration, SplitVideoResult};

#[tauri::command]
fn check_ffmpeg_environment() -> FfmpegEnvironmentResult {
    check_environment()
}

#[tauri::command]
fn read_video_metadata(file_path: String) -> Result<VideoMetadata, String> {
    probe_video_metadata(file_path)
}

#[tauri::command]
fn list_video_files_in_folder(folder_path: String) -> Result<Vec<String>, String> {
    list_supported_videos_in_folder(folder_path)
}

#[tauri::command]
fn open_path_in_file_manager(path: String) -> Result<(), String> {
    let target_path = Path::new(&path);

    if !target_path.exists() {
        return Err("路径不存在，无法打开。".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        let mut command = Command::new("explorer");

        if target_path.is_file() {
            command.arg(format!("/select,{}", target_path.to_string_lossy()));
        } else {
            command.arg(target_path);
        }

        command
            .spawn()
            .map_err(|error| format!("无法打开资源管理器：{error}"))?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        Command::new("open")
            .arg(target_path)
            .spawn()
            .map_err(|error| format!("无法打开文件管理器：{error}"))?;
    }

    Ok(())
}

#[tauri::command]
fn export_current_video(
    input_file_path: String,
    output_directory: String,
    canvas_aspect_ratio: CanvasAspectRatio,
    canvas_background_mode: CanvasBackgroundMode,
) -> Result<RenderVideoResult, String> {
    export_basic_video(
        input_file_path,
        output_directory,
        canvas_aspect_ratio,
        canvas_background_mode,
    )
}

#[tauri::command]
fn split_current_video(
    input_file_path: String,
    output_directory: String,
    segment_duration_seconds: f64,
) -> Result<SplitVideoResult, String> {
    split_video_by_duration(input_file_path, output_directory, segment_duration_seconds)
}

#[tauri::command]
fn concat_selected_segments(
    segment_paths: Vec<String>,
    output_directory: String,
    apply_horizontal_mirror: bool,
    playback_speed: f64,
    canvas_aspect_ratio: CanvasAspectRatio,
    canvas_background_mode: CanvasBackgroundMode,
    smooth_remix_enabled: bool,
) -> Result<MixVideoResult, String> {
    concat_video_segments(
        segment_paths,
        output_directory,
        apply_horizontal_mirror,
        playback_speed,
        canvas_aspect_ratio,
        canvas_background_mode,
        smooth_remix_enabled,
    )
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            check_ffmpeg_environment,
            concat_selected_segments,
            export_current_video,
            list_video_files_in_folder,
            open_path_in_file_manager,
            read_video_metadata,
            split_current_video
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri app");
}
