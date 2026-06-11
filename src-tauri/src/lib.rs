mod video_engine;

use video_engine::probe::{check_environment, FfmpegEnvironmentResult};

#[tauri::command]
fn check_ffmpeg_environment() -> FfmpegEnvironmentResult {
    check_environment()
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![check_ffmpeg_environment])
        .run(tauri::generate_context!())
        .expect("failed to run tauri app");
}
