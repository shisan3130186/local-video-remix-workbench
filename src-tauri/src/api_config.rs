use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_AI_BASE_URL: &str = "https://ark.cn-beijing.volces.com/api/v3";
pub const DEFAULT_TTS_RESOURCE_ID: &str = "seed-tts-2.0";
pub const DEFAULT_TTS_SPEAKER: &str = "zh_female_vv_uranus_bigtts";

const CONFIG_VERSION: u8 = 1;
const CONFIG_DIRECTORY: &str = "com.shisan.local-video-remix-workbench";
const CONFIG_FILE_NAME: &str = "secure-api-config.bin";
const MAX_CONFIG_FILE_BYTES: u64 = 64 * 1024;
const MAX_API_KEY_CHARACTERS: usize = 4096;
const MAX_URL_CHARACTERS: usize = 2048;
const MAX_IDENTIFIER_CHARACTERS: usize = 512;

#[derive(Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
struct StoredApiConfig {
    version: u8,
    ai_api_key: Option<String>,
    ai_base_url: Option<String>,
    ai_model: Option<String>,
    tts_api_key: Option<String>,
    tts_resource_id: Option<String>,
    tts_speaker: Option<String>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiConfigInput {
    ai_api_key: String,
    ai_base_url: String,
    ai_model: String,
    tts_api_key: String,
    tts_resource_id: String,
    tts_speaker: String,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ApiCredentialKind {
    Ai,
    Tts,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ApiConfigStatus {
    pub platform: &'static str,
    pub ai_configured: bool,
    pub ai_key_stored: bool,
    pub ai_environment_fallback: bool,
    pub ai_base_url: String,
    pub ai_model: String,
    pub tts_configured: bool,
    pub tts_key_stored: bool,
    pub tts_environment_fallback: bool,
    pub tts_resource_id: String,
    pub tts_speaker: String,
}

#[derive(Clone)]
pub struct AiServiceConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

#[derive(Clone)]
pub struct TtsServiceConfig {
    pub api_key: String,
    pub resource_id: String,
    pub speaker: String,
}

pub fn get_api_config_status() -> Result<ApiConfigStatus, String> {
    build_status(&load_stored_config()?)
}

pub fn save_api_config(input: ApiConfigInput) -> Result<ApiConfigStatus, String> {
    let mut stored = load_stored_config()?;
    merge_input(&mut stored, input)?;
    save_stored_config(&stored)?;
    build_status(&stored)
}

pub fn delete_api_credential(kind: ApiCredentialKind) -> Result<ApiConfigStatus, String> {
    let mut stored = load_stored_config()?;

    match kind {
        ApiCredentialKind::Ai => stored.ai_api_key = None,
        ApiCredentialKind::Tts => stored.tts_api_key = None,
    }

    save_stored_config(&stored)?;
    build_status(&stored)
}

pub fn load_ai_service_config() -> Result<AiServiceConfig, String> {
    let stored = load_stored_config()?;
    let api_key = stored
        .ai_api_key
        .or_else(|| environment_value("AI_API_KEY"))
        .ok_or_else(|| {
            "尚未配置AI API Key。请打开右侧“API 密钥”设置并保存，或使用环境变量 AI_API_KEY。"
                .to_string()
        })?;
    let base_url = stored
        .ai_base_url
        .or_else(|| environment_value("AI_BASE_URL"))
        .unwrap_or_else(|| DEFAULT_AI_BASE_URL.to_string());
    let model = stored
        .ai_model
        .or_else(|| environment_value("AI_MODEL"))
        .ok_or_else(|| {
            "尚未配置AI模型接入点ID。请打开右侧“API 密钥”设置填写模型接入点。".to_string()
        })?;

    Ok(AiServiceConfig {
        api_key,
        base_url,
        model,
    })
}

pub fn load_tts_service_config(
    speaker_override: Option<String>,
) -> Result<TtsServiceConfig, String> {
    let stored = load_stored_config()?;
    let api_key = stored
        .tts_api_key
        .or_else(|| environment_value("TTS_API_KEY"))
        .ok_or_else(|| {
            "尚未配置TTS/ASR语音密钥。请打开右侧“API 密钥”设置并保存，或使用环境变量 TTS_API_KEY。"
                .to_string()
        })?;
    let resource_id = stored
        .tts_resource_id
        .or_else(|| environment_value("TTS_RESOURCE_ID"))
        .unwrap_or_else(|| DEFAULT_TTS_RESOURCE_ID.to_string());
    let speaker = speaker_override
        .and_then(|value| normalize_optional_text(&value))
        .or(stored.tts_speaker)
        .or_else(|| environment_value("TTS_SPEAKER"))
        .unwrap_or_else(|| DEFAULT_TTS_SPEAKER.to_string());

    Ok(TtsServiceConfig {
        api_key,
        resource_id,
        speaker,
    })
}

pub fn load_speech_api_key() -> Result<String, String> {
    let stored = load_stored_config()?;
    stored
        .tts_api_key
        .or_else(|| environment_value("TTS_API_KEY"))
        .ok_or_else(|| "尚未配置TTS/ASR语音密钥。请打开右侧“API 密钥”设置并保存。".to_string())
}

fn build_status(stored: &StoredApiConfig) -> Result<ApiConfigStatus, String> {
    let ai_environment_key = environment_value("AI_API_KEY");
    let ai_api_key = stored.ai_api_key.clone().or(ai_environment_key.clone());
    let ai_base_url = stored
        .ai_base_url
        .clone()
        .or_else(|| environment_value("AI_BASE_URL"))
        .unwrap_or_else(|| DEFAULT_AI_BASE_URL.to_string());
    let ai_model = stored
        .ai_model
        .clone()
        .or_else(|| environment_value("AI_MODEL"))
        .unwrap_or_default();
    let tts_environment_key = environment_value("TTS_API_KEY");
    let tts_api_key = stored.tts_api_key.clone().or(tts_environment_key.clone());
    let tts_resource_id = stored
        .tts_resource_id
        .clone()
        .or_else(|| environment_value("TTS_RESOURCE_ID"))
        .unwrap_or_else(|| DEFAULT_TTS_RESOURCE_ID.to_string());
    let tts_speaker = stored
        .tts_speaker
        .clone()
        .or_else(|| environment_value("TTS_SPEAKER"))
        .unwrap_or_else(|| DEFAULT_TTS_SPEAKER.to_string());

    Ok(ApiConfigStatus {
        platform: "volcengine",
        ai_configured: ai_api_key.is_some() && !ai_model.is_empty(),
        ai_key_stored: stored.ai_api_key.is_some(),
        ai_environment_fallback: stored.ai_api_key.is_none() && ai_environment_key.is_some(),
        ai_base_url,
        ai_model,
        tts_configured: tts_api_key.is_some(),
        tts_key_stored: stored.tts_api_key.is_some(),
        tts_environment_fallback: stored.tts_api_key.is_none() && tts_environment_key.is_some(),
        tts_resource_id,
        tts_speaker,
    })
}

fn merge_input(stored: &mut StoredApiConfig, input: ApiConfigInput) -> Result<(), String> {
    if let Some(value) = normalize_optional_text(&input.ai_api_key) {
        validate_length("AI API Key", &value, MAX_API_KEY_CHARACTERS)?;
        stored.ai_api_key = Some(value);
    }

    if let Some(value) = normalize_optional_text(&input.tts_api_key) {
        validate_length("TTS API Key", &value, MAX_API_KEY_CHARACTERS)?;
        stored.tts_api_key = Some(value);
    }

    let ai_base_url = normalize_optional_text(&input.ai_base_url)
        .unwrap_or_else(|| DEFAULT_AI_BASE_URL.to_string());
    validate_length("AI连接地址", &ai_base_url, MAX_URL_CHARACTERS)?;
    validate_base_url(&ai_base_url)?;
    stored.ai_base_url = Some(ai_base_url);

    stored.ai_model = normalize_optional_text(&input.ai_model);
    if let Some(model) = &stored.ai_model {
        validate_length("AI模型接入点ID", model, MAX_IDENTIFIER_CHARACTERS)?;
    }

    if stored.ai_api_key.is_some() && stored.ai_model.is_none() {
        return Err("保存AI API Key时必须同时填写模型接入点ID。".to_string());
    }

    let resource_id = normalize_optional_text(&input.tts_resource_id)
        .unwrap_or_else(|| DEFAULT_TTS_RESOURCE_ID.to_string());
    validate_length("TTS资源ID", &resource_id, MAX_IDENTIFIER_CHARACTERS)?;
    stored.tts_resource_id = Some(resource_id);

    let speaker = normalize_optional_text(&input.tts_speaker)
        .unwrap_or_else(|| DEFAULT_TTS_SPEAKER.to_string());
    validate_length("TTS音色ID", &speaker, MAX_IDENTIFIER_CHARACTERS)?;
    stored.tts_speaker = Some(speaker);
    stored.version = CONFIG_VERSION;

    Ok(())
}

fn validate_base_url(value: &str) -> Result<(), String> {
    if !value.starts_with("https://") {
        return Err("AI连接地址必须以 https:// 开头。".to_string());
    }

    Ok(())
}

fn validate_length(label: &str, value: &str, maximum: usize) -> Result<(), String> {
    if value.chars().count() > maximum {
        return Err(format!("{label}内容过长，请检查后重新填写。"));
    }

    Ok(())
}

fn environment_value(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .and_then(|value| normalize_optional_text(&value))
}

fn normalize_optional_text(value: &str) -> Option<String> {
    let normalized = value.trim();
    (!normalized.is_empty()).then(|| normalized.to_string())
}

fn load_stored_config() -> Result<StoredApiConfig, String> {
    let path = config_file_path()?;
    load_stored_config_from_path(&path)
}

fn load_stored_config_from_path(path: &Path) -> Result<StoredApiConfig, String> {
    if !path.exists() {
        return Ok(StoredApiConfig::default());
    }

    let metadata = fs::metadata(path).map_err(|error| format!("无法读取密钥配置文件：{error}"))?;
    if metadata.len() > MAX_CONFIG_FILE_BYTES {
        return Err("密钥配置文件异常，请删除后重新配置。".to_string());
    }

    let encrypted = fs::read(path).map_err(|error| format!("无法读取密钥配置文件：{error}"))?;
    let mut plaintext = unprotect_data(&encrypted)?;
    let parsed = serde_json::from_slice::<StoredApiConfig>(&plaintext)
        .map_err(|error| format!("密钥配置文件无法识别：{error}"));
    plaintext.fill(0);
    let stored = parsed?;

    if stored.version != CONFIG_VERSION {
        return Err("密钥配置文件版本不兼容，请删除后重新配置。".to_string());
    }

    Ok(stored)
}

fn save_stored_config(stored: &StoredApiConfig) -> Result<(), String> {
    let path = config_file_path()?;
    save_stored_config_to_path(&path, stored)
}

fn save_stored_config_to_path(path: &Path, stored: &StoredApiConfig) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "无法确定密钥配置目录。".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("无法创建密钥配置目录：{error}"))?;

    let mut plaintext =
        serde_json::to_vec(stored).map_err(|error| format!("无法整理密钥配置：{error}"))?;
    let encrypted_result = protect_data(&plaintext);
    plaintext.fill(0);
    let encrypted = encrypted_result?;
    fs::write(path, encrypted).map_err(|error| format!("无法保存密钥配置：{error}"))
}

fn config_file_path() -> Result<PathBuf, String> {
    let app_data = env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "无法找到Windows应用配置目录。".to_string())?;

    Ok(app_data.join(CONFIG_DIRECTORY).join(CONFIG_FILE_NAME))
}

#[cfg(target_os = "windows")]
pub(crate) fn protect_data(data: &[u8]) -> Result<Vec<u8>, String> {
    use std::ptr::null;
    use std::slice;
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{
        CryptProtectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let data_length =
        u32::try_from(data.len()).map_err(|_| "密钥配置内容过大，无法安全保存。".to_string())?;
    let input = CRYPT_INTEGER_BLOB {
        cbData: data_length,
        pbData: data.as_ptr().cast_mut(),
    };
    let mut output = CRYPT_INTEGER_BLOB::default();
    let succeeded = unsafe {
        CryptProtectData(
            &input,
            null(),
            null(),
            null(),
            null(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };

    if succeeded == 0 {
        return Err(format!(
            "Windows无法加密密钥配置：{}",
            std::io::Error::last_os_error()
        ));
    }

    let encrypted =
        unsafe { slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec() };
    unsafe {
        LocalFree(output.pbData.cast());
    }

    Ok(encrypted)
}

#[cfg(target_os = "windows")]
pub(crate) fn unprotect_data(data: &[u8]) -> Result<Vec<u8>, String> {
    use std::ptr::{null, null_mut};
    use std::slice;
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{
        CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let data_length =
        u32::try_from(data.len()).map_err(|_| "密钥配置文件过大，无法安全读取。".to_string())?;
    let input = CRYPT_INTEGER_BLOB {
        cbData: data_length,
        pbData: data.as_ptr().cast_mut(),
    };
    let mut output = CRYPT_INTEGER_BLOB::default();
    let succeeded = unsafe {
        CryptUnprotectData(
            &input,
            null_mut(),
            null(),
            null(),
            null(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };

    if succeeded == 0 {
        return Err(
            "无法解密密钥配置。请确认仍在原Windows账户中使用，或删除后重新配置。".to_string(),
        );
    }

    let plaintext =
        unsafe { slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec() };
    unsafe {
        LocalFree(output.pbData.cast());
    }

    Ok(plaintext)
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn protect_data(_data: &[u8]) -> Result<Vec<u8>, String> {
    Err("当前安全密钥存储只支持Windows。".to_string())
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn unprotect_data(_data: &[u8]) -> Result<Vec<u8>, String> {
    Err("当前安全密钥存储只支持Windows。".to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        build_status, load_stored_config_from_path, merge_input, protect_data,
        save_stored_config_to_path, unprotect_data, ApiConfigInput, StoredApiConfig,
        DEFAULT_AI_BASE_URL, DEFAULT_TTS_RESOURCE_ID, DEFAULT_TTS_SPEAKER,
    };
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn empty_input() -> ApiConfigInput {
        ApiConfigInput {
            ai_api_key: String::new(),
            ai_base_url: String::new(),
            ai_model: String::new(),
            tts_api_key: String::new(),
            tts_resource_id: String::new(),
            tts_speaker: String::new(),
        }
    }

    #[test]
    fn applies_safe_defaults_without_storing_keys() {
        let mut stored = StoredApiConfig::default();
        merge_input(&mut stored, empty_input()).unwrap();

        assert_eq!(stored.ai_base_url.as_deref(), Some(DEFAULT_AI_BASE_URL));
        assert_eq!(
            stored.tts_resource_id.as_deref(),
            Some(DEFAULT_TTS_RESOURCE_ID)
        );
        assert_eq!(stored.tts_speaker.as_deref(), Some(DEFAULT_TTS_SPEAKER));
        assert!(stored.ai_api_key.is_none());
        assert!(stored.tts_api_key.is_none());
    }

    #[test]
    fn blank_key_fields_preserve_existing_secrets() {
        let mut stored = StoredApiConfig {
            ai_api_key: Some("existing-ai-key".to_string()),
            ai_model: Some("existing-model".to_string()),
            tts_api_key: Some("existing-tts-key".to_string()),
            ..StoredApiConfig::default()
        };
        let mut input = empty_input();
        input.ai_model = "existing-model".to_string();
        merge_input(&mut stored, input).unwrap();

        assert_eq!(stored.ai_api_key.as_deref(), Some("existing-ai-key"));
        assert_eq!(stored.tts_api_key.as_deref(), Some("existing-tts-key"));
    }

    #[test]
    fn rejects_ai_key_without_model_endpoint() {
        let mut stored = StoredApiConfig::default();
        let mut input = empty_input();
        input.ai_api_key = "new-ai-key".to_string();

        let error = merge_input(&mut stored, input).unwrap_err();
        assert!(error.contains("模型接入点ID"));
    }

    #[test]
    fn status_never_contains_secret_fields() {
        let stored = StoredApiConfig {
            ai_api_key: Some("secret-ai-key".to_string()),
            ai_base_url: Some(DEFAULT_AI_BASE_URL.to_string()),
            ai_model: Some("model-id".to_string()),
            tts_api_key: Some("secret-tts-key".to_string()),
            tts_resource_id: Some(DEFAULT_TTS_RESOURCE_ID.to_string()),
            tts_speaker: Some(DEFAULT_TTS_SPEAKER.to_string()),
            version: 1,
        };
        let status = build_status(&stored).unwrap();
        let serialized = serde_json::to_string(&status).unwrap();

        assert!(!serialized.contains("secret-ai-key"));
        assert!(!serialized.contains("secret-tts-key"));
        assert!(status.ai_key_stored);
        assert!(status.tts_key_stored);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_data_protection_round_trip() {
        let plaintext = b"fake-test-secret-not-a-real-key";
        let encrypted = protect_data(plaintext).unwrap();

        assert_ne!(encrypted, plaintext);
        assert_eq!(unprotect_data(&encrypted).unwrap(), plaintext);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn encrypted_config_file_round_trip_does_not_store_plaintext_key() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("api-config-test-{unique}"));
        let path = directory.join("secure-api-config.bin");
        let stored = StoredApiConfig {
            version: 1,
            ai_api_key: Some("fake-ai-key-for-test".to_string()),
            ai_base_url: Some(DEFAULT_AI_BASE_URL.to_string()),
            ai_model: Some("fake-model".to_string()),
            tts_api_key: Some("fake-tts-key-for-test".to_string()),
            tts_resource_id: Some(DEFAULT_TTS_RESOURCE_ID.to_string()),
            tts_speaker: Some(DEFAULT_TTS_SPEAKER.to_string()),
        };

        save_stored_config_to_path(&path, &stored).unwrap();
        let raw_file = fs::read(&path).unwrap();
        assert!(!String::from_utf8_lossy(&raw_file).contains("fake-ai-key-for-test"));
        assert!(load_stored_config_from_path(&path).unwrap() == stored);

        fs::remove_dir_all(directory).unwrap();
    }
}
