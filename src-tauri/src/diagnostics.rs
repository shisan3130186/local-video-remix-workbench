use crate::api_config::get_api_config_status;
use crate::temp_storage::temp_root;
use crate::video_engine::probe::check_environment;
use crate::video_engine::tool_paths::{ffmpeg_program, ffprobe_program, is_bundled_program};
use serde::Serialize;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::panic;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const APP_DIRECTORY: &str = "com.shisan.local-video-remix-workbench";
const LOG_DIRECTORY: &str = "logs";
const MAX_STARTUP_LOG_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticInfo {
    app_version: String,
    app_identifier: &'static str,
    generated_at_unix_seconds: u64,
    operating_system: String,
    architecture: String,
    executable_path: String,
    data_directory: String,
    logs_directory: String,
    legal_directory: String,
    temp_directory: String,
    ffmpeg_path: String,
    ffprobe_path: String,
    ffmpeg_available: bool,
    ffprobe_available: bool,
    bundled_ffmpeg: bool,
    ai_configured: bool,
    tts_configured: bool,
    asr_configured: bool,
    latest_crash_log: Option<String>,
}

pub fn install_panic_hook() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let timestamp = unix_timestamp();
        if let Ok(directory) = ensure_logs_directory() {
            let path = directory.join(format!("crash-{timestamp}.log"));
            let thread = std::thread::current();
            let thread_name = thread.name().unwrap_or("unnamed");
            let payload = panic_payload(panic_info);
            let location = panic_info
                .location()
                .map(|location| {
                    format!(
                        "{}:{}:{}",
                        location.file(),
                        location.line(),
                        location.column()
                    )
                })
                .unwrap_or_else(|| "unknown".to_string());
            let content = format!(
                "SmartCut crash report\nversion={}\nunixTime={}\nthread={}\nlocation={}\nmessage={}\n",
                env!("CARGO_PKG_VERSION"),
                timestamp,
                thread_name,
                location,
                payload
            );
            let _ = fs::write(path, content);
        }
        default_hook(panic_info);
    }));
}

pub fn write_startup_log() {
    let Ok(directory) = ensure_logs_directory() else {
        return;
    };
    let path = directory.join("startup.log");
    if fs::metadata(&path)
        .ok()
        .is_some_and(|metadata| metadata.len() > MAX_STARTUP_LOG_BYTES)
    {
        let _ = fs::remove_file(&path);
    }
    let environment = check_environment();
    let entry = format!(
        "unixTime={} version={} os={} arch={} ffmpegAvailable={} ffprobeAvailable={} bundled={}\n",
        unix_timestamp(),
        env!("CARGO_PKG_VERSION"),
        env::consts::OS,
        env::consts::ARCH,
        environment.ffmpeg.available,
        environment.ffprobe.available,
        environment.ffmpeg.bundled && environment.ffprobe.bundled,
    );
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(entry.as_bytes());
    }
}

pub fn get_diagnostic_info() -> Result<DiagnosticInfo, String> {
    let environment = check_environment();
    let api_status = get_api_config_status()?;
    let executable_path = env::current_exe().unwrap_or_default();
    let ffmpeg_path = ffmpeg_program();
    let ffprobe_path = ffprobe_program();
    let data_directory = app_data_directory()?;
    let logs_directory = ensure_logs_directory()?;
    let legal_directory = resolve_legal_directory();

    Ok(DiagnosticInfo {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        app_identifier: APP_DIRECTORY,
        generated_at_unix_seconds: unix_timestamp(),
        operating_system: env::consts::OS.to_string(),
        architecture: env::consts::ARCH.to_string(),
        executable_path: display_path(&executable_path),
        data_directory: display_path(&data_directory),
        logs_directory: display_path(&logs_directory),
        legal_directory: display_path(&legal_directory),
        temp_directory: display_path(&temp_root()),
        ffmpeg_path: display_path(&ffmpeg_path),
        ffprobe_path: display_path(&ffprobe_path),
        ffmpeg_available: environment.ffmpeg.available,
        ffprobe_available: environment.ffprobe.available,
        bundled_ffmpeg: is_bundled_program(&ffmpeg_path) && is_bundled_program(&ffprobe_path),
        ai_configured: api_status.ai_configured,
        tts_configured: api_status.tts_configured,
        asr_configured: api_status.tts_configured,
        latest_crash_log: latest_crash_log(&logs_directory),
    })
}

pub fn create_diagnostic_report() -> Result<String, String> {
    let info = get_diagnostic_info()?;
    let directory = ensure_logs_directory()?;
    let path = directory.join(format!(
        "diagnostic-report-{}.txt",
        info.generated_at_unix_seconds
    ));
    fs::write(&path, build_report(&info)).map_err(|error| format!("无法生成诊断报告：{error}"))?;
    Ok(display_path(&path))
}

fn build_report(info: &DiagnosticInfo) -> String {
    format!(
        "SmartCut diagnostic report\n\
version={}\n\
identifier={}\n\
unixTime={}\n\
os={}\n\
architecture={}\n\
executable={}\n\
dataDirectory={}\n\
logsDirectory={}\n\
legalDirectory={}\n\
tempDirectory={}\n\
ffmpegPath={}\n\
ffprobePath={}\n\
ffmpegAvailable={}\n\
ffprobeAvailable={}\n\
bundledFfmpeg={}\n\
aiConfigured={}\n\
ttsConfigured={}\n\
asrConfigured={}\n\
latestCrashLog={}\n\
\nThis report does not contain API keys, video contents or script contents.\n",
        info.app_version,
        info.app_identifier,
        info.generated_at_unix_seconds,
        info.operating_system,
        info.architecture,
        info.executable_path,
        info.data_directory,
        info.logs_directory,
        info.legal_directory,
        info.temp_directory,
        info.ffmpeg_path,
        info.ffprobe_path,
        info.ffmpeg_available,
        info.ffprobe_available,
        info.bundled_ffmpeg,
        info.ai_configured,
        info.tts_configured,
        info.asr_configured,
        info.latest_crash_log.as_deref().unwrap_or("none"),
    )
}

fn ensure_logs_directory() -> Result<PathBuf, String> {
    let directory = app_data_directory()?.join(LOG_DIRECTORY);
    fs::create_dir_all(&directory).map_err(|error| format!("无法创建诊断目录：{error}"))?;
    Ok(directory)
}

fn app_data_directory() -> Result<PathBuf, String> {
    env::var_os("APPDATA")
        .map(PathBuf::from)
        .map(|path| path.join(APP_DIRECTORY))
        .ok_or_else(|| "无法找到Windows应用配置目录。".to_string())
}

fn resolve_legal_directory() -> PathBuf {
    let mut candidates = Vec::new();
    if let Ok(executable) = env::current_exe() {
        if let Some(directory) = executable.parent() {
            candidates.push(directory.join("resources").join("legal"));
            candidates.push(directory.join("legal"));
        }
    }
    if let Ok(directory) = env::current_dir() {
        candidates.push(directory.join("src-tauri").join("resources").join("legal"));
        candidates.push(directory.join("resources").join("legal"));
    }
    candidates
        .into_iter()
        .find(|candidate| candidate.is_dir())
        .unwrap_or_default()
}

fn latest_crash_log(directory: &Path) -> Option<String> {
    fs::read_dir(directory)
        .ok()?
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("crash-") || !name.ends_with(".log") {
                return None;
            }
            let modified = entry.metadata().ok()?.modified().ok()?;
            Some((modified, display_path(&entry.path())))
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
}

fn panic_payload(info: &panic::PanicHookInfo<'_>) -> String {
    if let Some(message) = info.payload().downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = info.payload().downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_string()
    }
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::{build_report, DiagnosticInfo};

    #[test]
    fn diagnostic_report_only_contains_configuration_status() {
        let report = build_report(&DiagnosticInfo {
            app_version: "1.0.0".to_string(),
            app_identifier: "test.app",
            generated_at_unix_seconds: 1,
            operating_system: "windows".to_string(),
            architecture: "x86_64".to_string(),
            executable_path: "app.exe".to_string(),
            data_directory: "data".to_string(),
            logs_directory: "logs".to_string(),
            legal_directory: "legal".to_string(),
            temp_directory: "temp".to_string(),
            ffmpeg_path: "ffmpeg.exe".to_string(),
            ffprobe_path: "ffprobe.exe".to_string(),
            ffmpeg_available: true,
            ffprobe_available: true,
            bundled_ffmpeg: true,
            ai_configured: true,
            tts_configured: true,
            asr_configured: true,
            latest_crash_log: None,
        });
        assert!(report.contains("aiConfigured=true"));
        assert!(report.contains("does not contain API keys"));
        assert!(!report.contains("fake-secret"));
    }
}
