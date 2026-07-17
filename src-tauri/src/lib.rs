mod ai_remix;
mod api_config;
mod material_library;
mod project_snapshot;
mod task_runtime;
mod temp_storage;
mod tts;
mod video_engine;

use ai_remix::{
    analyze_ai_remix_segments as create_ai_remix_segment_analysis,
    build_ai_remix_variants as create_ai_remix_variants,
    extract_ai_remix_segment_content as create_ai_remix_segment_content_analysis,
    plan_ai_remix as create_ai_remix_plan, AiRemixPlanResult, AiRemixSegmentAnalysisResult,
    AiRemixSegmentContentAnalysisResult, AiRemixSegmentInput, AiRemixVariantPlanResult,
    AiRemixVariantShotInput, AiRemixVisualSegmentInput,
};
use api_config::{
    delete_api_credential as remove_api_credential,
    get_api_config_status as read_api_config_status, save_api_config as store_api_config,
    ApiConfigInput, ApiConfigStatus, ApiCredentialKind,
};
use material_library::{
    load_material_library as read_material_library,
    save_material_library as store_material_library, MaterialLibraryLoadResult,
    MaterialLibrarySnapshot,
};
use project_snapshot::{
    delete_project_snapshot as remove_project_snapshot,
    load_project_snapshot as read_project_snapshot,
    save_project_snapshot as store_project_snapshot, ProjectSnapshot, ProjectSnapshotLoadResult,
};
use std::path::Path;
use std::process::Command;
use task_runtime::{
    cancel_task as request_task_cancellation, create_task as register_task,
    finish_task as complete_task, get_task as read_task, update_task as report_task_progress,
    TaskProgressContext, TaskSnapshot, TaskStatus,
};
use temp_storage::{cleanup_workspace_temp_files, TempCleanupResult};
use tts::{
    cleanup_tts_session as remove_tts_session, get_tts_config_status as read_tts_config_status,
    synthesize_tts as create_tts_audio, synthesize_tts_shot as create_tts_shot_audio,
    TtsConfigStatus, TtsSynthesisResult,
};
use video_engine::canvas::{CanvasAspectRatio, CanvasBackgroundMode};
use video_engine::import::list_supported_videos_in_folder;
use video_engine::mix::{concat_video_segments, MixVideoResult, RemixSettings};
use video_engine::narrated_mix::{
    concat_narrated_segments as create_narrated_video, NarratedAudioSettings, NarratedSegmentInput,
};
use video_engine::output::{
    detect_video_encoder_capabilities, EncoderCapabilities, OutputSettings,
};
use video_engine::probe::{
    check_environment, probe_video_metadata, FfmpegEnvironmentResult, VideoMetadata,
};
use video_engine::render::{export_basic_video, RenderVideoResult};
use video_engine::split::{
    split_video_by_duration, split_video_by_scene, SceneSensitivity, SplitVideoResult,
};
use video_engine::subtitle::NarratedSubtitleSettings;
use video_engine::thumbnail::{generate_video_thumbnail, ThumbnailFitMode, VideoThumbnailResult};

#[tauri::command]
fn check_ffmpeg_environment() -> FfmpegEnvironmentResult {
    check_environment()
}

#[tauri::command]
fn get_video_encoder_capabilities(force_refresh: bool) -> EncoderCapabilities {
    detect_video_encoder_capabilities(force_refresh)
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
fn is_existing_directory(path: String) -> bool {
    Path::new(&path).is_dir()
}

#[tauri::command]
fn create_task(task_id: String, label: String) -> Result<TaskSnapshot, String> {
    register_task(task_id, label)
}

#[tauri::command]
fn get_task_progress(task_id: String) -> Result<TaskSnapshot, String> {
    read_task(&task_id)
}

#[tauri::command]
fn update_task_progress(
    task_id: String,
    progress_percent: f64,
    stage: String,
) -> Result<TaskSnapshot, String> {
    report_task_progress(&task_id, progress_percent, stage)
}

#[tauri::command]
fn cancel_task(task_id: String) -> Result<TaskSnapshot, String> {
    request_task_cancellation(&task_id)
}

#[tauri::command]
fn finish_task(
    task_id: String,
    status: TaskStatus,
    message: Option<String>,
) -> Result<TaskSnapshot, String> {
    complete_task(&task_id, status, message)
}

#[tauri::command]
fn cleanup_temp_files() -> Result<TempCleanupResult, String> {
    cleanup_workspace_temp_files()
}

#[tauri::command]
fn export_current_video(
    input_file_path: String,
    output_directory: String,
    canvas_aspect_ratio: CanvasAspectRatio,
    canvas_background_mode: CanvasBackgroundMode,
    duration_seconds: Option<f64>,
    task_context: Option<TaskProgressContext>,
    output_settings: OutputSettings,
) -> Result<RenderVideoResult, String> {
    export_basic_video(
        input_file_path,
        output_directory,
        canvas_aspect_ratio,
        canvas_background_mode,
        duration_seconds,
        task_context,
        output_settings,
    )
}

#[tauri::command]
fn split_current_video(
    input_file_path: String,
    output_directory: String,
    segment_duration_seconds: f64,
    input_duration_seconds: Option<f64>,
    task_context: Option<TaskProgressContext>,
) -> Result<SplitVideoResult, String> {
    split_video_by_duration(
        input_file_path,
        output_directory,
        segment_duration_seconds,
        input_duration_seconds,
        task_context,
    )
}

#[tauri::command]
fn split_video_by_scenes(
    input_file_path: String,
    output_directory: String,
    sensitivity: SceneSensitivity,
    minimum_segment_seconds: f64,
    maximum_segment_seconds: f64,
    input_duration_seconds: Option<f64>,
    task_context: Option<TaskProgressContext>,
) -> Result<SplitVideoResult, String> {
    split_video_by_scene(
        input_file_path,
        output_directory,
        sensitivity,
        minimum_segment_seconds,
        maximum_segment_seconds,
        input_duration_seconds,
        task_context,
    )
}

#[tauri::command]
fn generate_thumbnail(
    input_file_path: String,
    output_directory: Option<String>,
    time_seconds: f64,
    label: String,
    fit_mode: ThumbnailFitMode,
) -> Result<VideoThumbnailResult, String> {
    generate_video_thumbnail(
        input_file_path,
        output_directory,
        time_seconds,
        label,
        fit_mode,
    )
}

#[tauri::command]
async fn analyze_ai_remix_segments(
    segments: Vec<AiRemixVisualSegmentInput>,
) -> Result<AiRemixSegmentAnalysisResult, String> {
    create_ai_remix_segment_analysis(segments).await
}

#[tauri::command]
async fn extract_ai_remix_segment_content(
    segments: Vec<AiRemixSegmentInput>,
) -> Result<AiRemixSegmentContentAnalysisResult, String> {
    create_ai_remix_segment_content_analysis(segments).await
}

#[tauri::command]
async fn plan_ai_remix(
    script: String,
    segments: Vec<AiRemixSegmentInput>,
) -> Result<AiRemixPlanResult, String> {
    create_ai_remix_plan(script, segments).await
}

#[tauri::command]
fn build_ai_remix_variants(
    shots: Vec<AiRemixVariantShotInput>,
    requested_count: usize,
) -> Result<AiRemixVariantPlanResult, String> {
    create_ai_remix_variants(shots, requested_count)
}

#[tauri::command]
fn get_tts_config_status() -> Result<TtsConfigStatus, String> {
    read_tts_config_status()
}

#[tauri::command]
fn get_api_config_status() -> Result<ApiConfigStatus, String> {
    read_api_config_status()
}

#[tauri::command]
fn save_api_config(input: ApiConfigInput) -> Result<ApiConfigStatus, String> {
    store_api_config(input)
}

#[tauri::command]
fn delete_api_credential(kind: ApiCredentialKind) -> Result<ApiConfigStatus, String> {
    remove_api_credential(kind)
}

#[tauri::command]
fn load_project_snapshot() -> Result<ProjectSnapshotLoadResult, String> {
    read_project_snapshot()
}

#[tauri::command]
fn save_project_snapshot(snapshot: ProjectSnapshot) -> Result<(), String> {
    store_project_snapshot(snapshot)
}

#[tauri::command]
fn delete_project_snapshot() -> Result<(), String> {
    remove_project_snapshot()
}

#[tauri::command]
fn load_material_library() -> Result<MaterialLibraryLoadResult, String> {
    read_material_library()
}

#[tauri::command]
fn save_material_library(snapshot: MaterialLibrarySnapshot) -> Result<(), String> {
    store_material_library(snapshot)
}

#[tauri::command]
async fn synthesize_tts(
    text: String,
    output_directory: String,
    speaker: Option<String>,
) -> Result<TtsSynthesisResult, String> {
    create_tts_audio(text, output_directory, speaker).await
}

#[tauri::command]
async fn synthesize_tts_shot(
    text: String,
    speaker: Option<String>,
    session_id: String,
    shot_index: usize,
) -> Result<TtsSynthesisResult, String> {
    create_tts_shot_audio(text, speaker, session_id, shot_index).await
}

#[tauri::command]
fn cleanup_tts_session(session_id: String) -> Result<(), String> {
    remove_tts_session(session_id)
}

#[tauri::command]
fn concat_selected_segments(
    segment_paths: Vec<String>,
    output_directory: String,
    settings: RemixSettings,
    task_context: Option<TaskProgressContext>,
) -> Result<MixVideoResult, String> {
    concat_video_segments(segment_paths, output_directory, settings, task_context)
}

#[tauri::command]
fn concat_narrated_segments(
    segments: Vec<NarratedSegmentInput>,
    output_directory: String,
    settings: RemixSettings,
    audio_settings: NarratedAudioSettings,
    subtitle_settings: NarratedSubtitleSettings,
    task_context: Option<TaskProgressContext>,
) -> Result<MixVideoResult, String> {
    create_narrated_video(
        segments,
        output_directory,
        settings,
        audio_settings,
        subtitle_settings,
        task_context,
    )
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            analyze_ai_remix_segments,
            build_ai_remix_variants,
            cancel_task,
            check_ffmpeg_environment,
            cleanup_temp_files,
            cleanup_tts_session,
            concat_narrated_segments,
            concat_selected_segments,
            create_task,
            delete_api_credential,
            delete_project_snapshot,
            extract_ai_remix_segment_content,
            export_current_video,
            finish_task,
            generate_thumbnail,
            get_task_progress,
            get_video_encoder_capabilities,
            get_tts_config_status,
            get_api_config_status,
            is_existing_directory,
            list_video_files_in_folder,
            load_material_library,
            load_project_snapshot,
            open_path_in_file_manager,
            plan_ai_remix,
            read_video_metadata,
            save_api_config,
            save_material_library,
            save_project_snapshot,
            split_current_video,
            split_video_by_scenes,
            synthesize_tts,
            synthesize_tts_shot,
            update_task_progress
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri app");
}
