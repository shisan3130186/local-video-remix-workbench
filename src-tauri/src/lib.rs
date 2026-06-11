mod video_engine;

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

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            check_ffmpeg_environment,
            read_video_metadata
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri app");
}
