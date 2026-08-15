mod ai_remix;
mod api_config;
mod asr;
mod desktop_tools;
mod diagnostics;
mod material_library;
mod membership;
mod project_snapshot;
mod script_library;
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
    AiRemixVariantShotInput, AiRemixVisualSegmentInput, ScriptRewriteInput, ScriptRewriteResult,
};
use api_config::{
    delete_api_credential as remove_api_credential,
    get_api_config_status as read_api_config_status, save_api_config as store_api_config,
    ApiConfigInput, ApiConfigStatus, ApiCredentialKind,
};
use asr::{
    get_asr_config_status as read_asr_config_status, recognize_speech as create_asr_recognition,
    AsrConfigStatus, AsrRecognitionResult,
};
use desktop_tools::{
    batch_rename_files as rename_files, create_ai_remix_output_directory,
    export_jianying_draft_package, list_files_in_folder as read_files_in_folder,
    write_base64_file as store_base64_file, write_utf8_text_file as store_utf8_text_file,
    RenameFileInput, RenameFileResult,
};
use diagnostics::{
    create_diagnostic_report as write_diagnostic_report,
    get_diagnostic_info as read_diagnostic_info, DiagnosticInfo,
};
use material_library::{
    load_material_library as read_material_library,
    save_material_library as store_material_library, MaterialLibraryLoadResult,
    MaterialLibrarySnapshot,
};
use membership::{
    change_account_password as change_remote_account_password,
    get_account_status as read_account_status, login_account as login_remote_account,
    logout_account as logout_remote_account, redeem_membership as redeem_remote_membership,
    refresh_account as refresh_remote_account, register_account as register_remote_account,
    AccountStatus,
};
use project_snapshot::{
    delete_project_snapshot as remove_project_snapshot,
    load_project_snapshot as read_project_snapshot,
    save_project_snapshot as store_project_snapshot, ProjectSnapshot, ProjectSnapshotLoadResult,
};
use script_library::{
    delete_script_library_entry as remove_script_library_entry,
    load_script_library as read_script_library,
    save_script_library_entry as store_script_library_entry, SaveScriptLibraryEntryInput,
    SaveScriptLibraryEntryResult, ScriptLibraryEntry,
};
use std::fs;
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
    synthesize_preview_audio, synthesize_tts as create_tts_audio,
    synthesize_tts_shot as create_tts_shot_audio, TtsConfigStatus, TtsSynthesisResult,
};
use video_engine::image_video::{
    convert_images_to_videos as create_image_videos, ImageVideoBatchResult,
};
use video_engine::import::list_supported_videos_in_folder;
use video_engine::mix::{
    concat_video_segments, export_single_video, MixVideoResult, RemixSegmentInput, RemixSettings,
};
use video_engine::narrated_mix::{
    concat_narrated_segments as create_narrated_video, NarratedAudioSettings, NarratedSegmentInput,
};
use video_engine::output::{detect_video_encoder_capabilities, EncoderCapabilities};
use video_engine::probe::{
    check_environment, probe_video_metadata, FfmpegEnvironmentResult, VideoMetadata,
};
use video_engine::render::RenderVideoResult;
use video_engine::split::{split_video_by_duration, split_video_by_scene, SplitVideoResult};
use video_engine::subtitle::NarratedSubtitleSettings;
use video_engine::thumbnail::{generate_video_thumbnail, ThumbnailFitMode, VideoThumbnailResult};

#[tauri::command]
fn check_ffmpeg_environment() -> FfmpegEnvironmentResult {
    check_environment()
}

#[tauri::command]
fn get_diagnostic_info() -> Result<DiagnosticInfo, String> {
    read_diagnostic_info()
}

#[tauri::command]
fn create_diagnostic_report() -> Result<String, String> {
    write_diagnostic_report()
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
fn batch_rename_files(items: Vec<RenameFileInput>) -> Result<Vec<RenameFileResult>, String> {
    rename_files(items)
}

#[tauri::command]
fn list_files_in_folder(folder_path: String) -> Result<Vec<String>, String> {
    read_files_in_folder(folder_path)
}

#[tauri::command]
fn write_utf8_text_file(path: String, content: String) -> Result<(), String> {
    store_utf8_text_file(path, content)
}

#[tauri::command]
fn write_base64_file(path: String, data: String) -> Result<(), String> {
    store_base64_file(path, data)
}

#[tauri::command]
fn create_ai_remix_output_directory_command(
    output_directory: String,
    label: String,
) -> Result<String, String> {
    create_ai_remix_output_directory(output_directory, label)
}

#[tauri::command]
fn export_jianying_draft_package_command(
    output_directory: String,
    project_name: String,
    video_paths: Vec<String>,
    script: String,
) -> Result<String, String> {
    export_jianying_draft_package(output_directory, project_name, video_paths, script)
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
    duration_seconds: Option<f64>,
    task_context: Option<TaskProgressContext>,
    settings: RemixSettings,
) -> Result<RenderVideoResult, String> {
    export_single_video(
        input_file_path,
        output_directory,
        duration_seconds,
        settings,
        task_context,
    )
}

#[tauri::command]
fn delete_source_video_file(
    input_file_path: String,
    output_file_paths: Vec<String>,
) -> Result<(), String> {
    let path = Path::new(&input_file_path);
    if !path.is_file() {
        return Err("The source video file no longer exists.".to_string());
    }
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !matches!(extension.as_str(), "mp4" | "mov" | "avi" | "mkv" | "webm") {
        return Err("Only imported video files can be removed after export.".to_string());
    }
    if output_file_paths.is_empty() {
        return Err(
            "No verified export result was provided; source deletion was refused.".to_string(),
        );
    }
    let source_path = path
        .canonicalize()
        .map_err(|error| format!("Unable to verify source video path: {error}"))?;
    for output_file_path in output_file_paths {
        let output_path = Path::new(&output_file_path);
        if !output_path.is_file() {
            return Err(format!(
                "A required export result no longer exists: {output_file_path}"
            ));
        }
        let verified_output_path = output_path
            .canonicalize()
            .map_err(|error| format!("Unable to verify export result path: {error}"))?;
        if verified_output_path == source_path {
            return Err(
                "The export result points to the source video; deletion was refused.".to_string(),
            );
        }
    }
    fs::remove_file(path).map_err(|error| format!("Unable to remove source video: {error}"))
}

#[tauri::command]
fn export_cover_image(
    cover_image_path: String,
    video_output_path: String,
) -> Result<String, String> {
    let source = Path::new(&cover_image_path);
    let video = Path::new(&video_output_path);
    if !source.is_file() || !video.is_file() {
        return Err("Cover source or rendered video does not exist.".to_string());
    }
    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("jpg");
    let stem = video
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Unable to build cover output name.".to_string())?;
    let target = video.with_file_name(format!("{stem}_cover.{extension}"));
    fs::copy(source, &target).map_err(|error| format!("Unable to export cover image: {error}"))?;
    Ok(target.to_string_lossy().to_string())
}

#[tauri::command]
fn split_current_video(
    input_file_path: String,
    output_directory: String,
    input_duration_seconds: Option<f64>,
    settings: video_engine::split::DurationSplitSettings,
    task_context: Option<TaskProgressContext>,
) -> Result<SplitVideoResult, String> {
    split_video_by_duration(
        input_file_path,
        output_directory,
        input_duration_seconds,
        settings,
        task_context,
    )
}

#[tauri::command]
fn split_video_by_scenes(
    input_file_path: String,
    output_directory: String,
    input_duration_seconds: Option<f64>,
    settings: video_engine::split::SceneSplitSettings,
    task_context: Option<TaskProgressContext>,
) -> Result<SplitVideoResult, String> {
    split_video_by_scene(
        input_file_path,
        output_directory,
        input_duration_seconds,
        settings,
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
    match_mode: String,
) -> Result<AiRemixPlanResult, String> {
    create_ai_remix_plan(script, segments, match_mode).await
}

#[tauri::command]
async fn rewrite_scripts(input: ScriptRewriteInput) -> Result<ScriptRewriteResult, String> {
    ai_remix::rewrite_scripts(input).await
}

#[tauri::command]
fn convert_images_to_videos(
    image_paths: Vec<String>,
    output_directory: String,
    duration_seconds: f64,
    aspect_ratio: video_engine::canvas::CanvasAspectRatio,
    output_settings: video_engine::output::OutputSettings,
    task_context: Option<TaskProgressContext>,
) -> Result<ImageVideoBatchResult, String> {
    create_image_videos(
        image_paths,
        output_directory,
        duration_seconds,
        aspect_ratio,
        output_settings,
        task_context,
    )
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
fn get_asr_config_status() -> Result<AsrConfigStatus, String> {
    read_asr_config_status()
}

#[tauri::command]
async fn recognize_speech(
    file_path: String,
    task_context: Option<TaskProgressContext>,
) -> Result<AsrRecognitionResult, String> {
    create_asr_recognition(file_path, task_context).await
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
fn get_account_status() -> Result<AccountStatus, String> {
    read_account_status()
}

#[tauri::command]
async fn register_account(
    email: String,
    password: String,
    display_name: String,
) -> Result<AccountStatus, String> {
    register_remote_account(email, password, display_name).await
}

#[tauri::command]
async fn login_account(email: String, password: String) -> Result<AccountStatus, String> {
    login_remote_account(email, password).await
}

#[tauri::command]
async fn refresh_account() -> Result<AccountStatus, String> {
    refresh_remote_account().await
}

#[tauri::command]
async fn redeem_membership(
    redemption_code: String,
    allow_device_rebind: bool,
) -> Result<AccountStatus, String> {
    redeem_remote_membership(redemption_code, allow_device_rebind).await
}

#[tauri::command]
async fn change_account_password(
    current_password: String,
    new_password: String,
) -> Result<AccountStatus, String> {
    change_remote_account_password(current_password, new_password).await
}

#[tauri::command]
async fn logout_account() -> Result<AccountStatus, String> {
    logout_remote_account().await
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
fn load_script_library() -> Result<Vec<ScriptLibraryEntry>, String> {
    read_script_library()
}

#[tauri::command]
fn save_script_library_entry(
    input: SaveScriptLibraryEntryInput,
) -> Result<SaveScriptLibraryEntryResult, String> {
    store_script_library_entry(input)
}

#[tauri::command]
fn delete_script_library_entry(id: String) -> Result<Vec<ScriptLibraryEntry>, String> {
    remove_script_library_entry(id)
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
    output_directory: Option<String>,
) -> Result<TtsSynthesisResult, String> {
    create_tts_shot_audio(text, speaker, session_id, shot_index, output_directory).await
}

#[tauri::command]
fn cleanup_tts_session(session_id: String) -> Result<(), String> {
    remove_tts_session(session_id)
}

#[tauri::command]
fn concat_selected_segments(
    segment_paths: Vec<String>,
    segment_inputs: Option<Vec<RemixSegmentInput>>,
    output_directory: String,
    settings: RemixSettings,
    task_context: Option<TaskProgressContext>,
) -> Result<MixVideoResult, String> {
    concat_video_segments(
        segment_paths,
        output_directory,
        settings,
        task_context,
        segment_inputs,
    )
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
    diagnostics::install_panic_hook();
    diagnostics::write_startup_log();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            analyze_ai_remix_segments,
            batch_rename_files,
            build_ai_remix_variants,
            cancel_task,
            check_ffmpeg_environment,
            cleanup_temp_files,
            cleanup_tts_session,
            concat_narrated_segments,
            concat_selected_segments,
            convert_images_to_videos,
            create_task,
            create_ai_remix_output_directory_command,
            create_diagnostic_report,
            delete_api_credential,
            change_account_password,
            delete_project_snapshot,
            delete_source_video_file,
            export_cover_image,
            delete_script_library_entry,
            extract_ai_remix_segment_content,
            export_current_video,
            export_jianying_draft_package_command,
            finish_task,
            generate_thumbnail,
            get_asr_config_status,
            get_diagnostic_info,
            get_task_progress,
            get_video_encoder_capabilities,
            get_tts_config_status,
            get_api_config_status,
            get_account_status,
            is_existing_directory,
            list_files_in_folder,
            list_video_files_in_folder,
            load_material_library,
            load_project_snapshot,
            load_script_library,
            open_path_in_file_manager,
            plan_ai_remix,
            read_video_metadata,
            recognize_speech,
            rewrite_scripts,
            login_account,
            logout_account,
            redeem_membership,
            refresh_account,
            register_account,
            save_api_config,
            save_material_library,
            save_project_snapshot,
            save_script_library_entry,
            split_current_video,
            split_video_by_scenes,
            synthesize_tts,
            synthesize_tts_shot,
            synthesize_preview_audio,
            update_task_progress,
            write_base64_file,
            write_utf8_text_file
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri app");
}
