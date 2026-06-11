mod video_engine;

use video_engine::import::list_supported_videos_in_folder;
use video_engine::probe::{
    check_environment, probe_video_metadata, FfmpegEnvironmentResult, VideoMetadata,
};

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

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            check_ffmpeg_environment,
            list_video_files_in_folder,
            read_video_metadata
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri app");
}
