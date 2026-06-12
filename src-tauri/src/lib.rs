mod video_engine;

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
fn export_current_video(
    input_file_path: String,
    output_directory: String,
) -> Result<RenderVideoResult, String> {
    export_basic_video(input_file_path, output_directory)
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
) -> Result<MixVideoResult, String> {
    concat_video_segments(
        segment_paths,
        output_directory,
        apply_horizontal_mirror,
        playback_speed,
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
            read_video_metadata,
            split_current_video
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri app");
}
