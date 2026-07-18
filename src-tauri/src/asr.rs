use crate::api_config::{get_api_config_status, load_speech_api_key};
use crate::task_runtime::{ensure_not_cancelled, update_task, TaskProgressContext};
use crate::temp_storage::TaskTempDirectory;
use crate::video_engine::audio::normalize_audio_for_asr;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use reqwest::header::HeaderMap;
use reqwest::tls::Certificate;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};
use uuid::Uuid;

const ASR_ENDPOINT: &str = "https://openspeech.bytedance.com/api/v3/auc/bigmodel/recognize/flash";
const ASR_RESOURCE_ID: &str = "volc.bigasr.auc_turbo";
const ASR_MODEL_VERSION: &str = "bigmodel-flash-v1";
const ASR_SUCCESS_CODE: i64 = 20_000_000;
const ASR_SILENCE_CODE: i64 = 20_000_003;
const ASR_BUSY_CODE: i64 = 55_000_031;
const CACHE_VERSION: u8 = 1;
const CACHE_DIRECTORY: &str = "com.shisan.local-video-remix-workbench";
const CACHE_FILE_DIRECTORY: &str = "asr-cache-v1";
const MAX_CACHE_FILE_BYTES: u64 = 8 * 1024 * 1024;
const RETRYABLE_ERROR_PREFIX: &str = "__RETRYABLE_SERVICE_ERROR__:";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AsrConfigStatus {
    configured: bool,
    resource_id: &'static str,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AsrUtterance {
    start_time_ms: i64,
    end_time_ms: i64,
    text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AsrRecognitionResult {
    source_path: String,
    source_file_name: String,
    text: String,
    utterances: Vec<AsrUtterance>,
    duration_seconds: Option<f64>,
    normalized_audio_bytes: u64,
    cache_hit: bool,
    model_version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct SourceIdentity {
    canonical_path: String,
    file_size_bytes: u64,
    modified_time_ms: u128,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AsrCacheEntry {
    cache_version: u8,
    model_version: String,
    source: SourceIdentity,
    result: AsrRecognitionResult,
}

pub fn get_asr_config_status() -> Result<AsrConfigStatus, String> {
    let status = get_api_config_status()?;
    Ok(AsrConfigStatus {
        configured: status.tts_configured,
        resource_id: ASR_RESOURCE_ID,
    })
}

pub async fn recognize_speech(
    file_path: String,
    task_context: Option<TaskProgressContext>,
) -> Result<AsrRecognitionResult, String> {
    let source = build_source_identity(Path::new(&file_path))?;
    ensure_task_active(task_context.as_ref())?;

    if let Some(mut cached) = load_cached_result(&source)? {
        cached.cache_hit = true;
        update_context(task_context.as_ref(), 100.0, "已读取本地识别缓存");
        return Ok(cached);
    }

    let task_temp = TaskTempDirectory::create("asr")?;
    let normalized_path = task_temp.path().join("speech.mp3");
    let normalize_context = task_context
        .as_ref()
        .map(|context| context.child(0.05, 0.38, "正在提取并压缩声音"));
    let normalized = normalize_audio_for_asr(
        Path::new(&source.canonical_path),
        &normalized_path,
        normalize_context.as_ref(),
    )?;
    ensure_task_active(task_context.as_ref())?;
    update_context(task_context.as_ref(), 42.0, "正在准备云端识别数据");

    let audio =
        fs::read(&normalized.path).map_err(|error| format!("无法读取待上传的声音数据：{error}"))?;
    let api_key = load_speech_api_key()?;
    ensure_task_active(task_context.as_ref())?;
    update_context(task_context.as_ref(), 48.0, "正在进行云端语音识别");

    let parsed = request_asr(&api_key, &audio).await?;
    ensure_task_active(task_context.as_ref())?;
    update_context(task_context.as_ref(), 92.0, "正在整理句子时间轴");

    let result = AsrRecognitionResult {
        source_path: source.canonical_path.clone(),
        source_file_name: Path::new(&source.canonical_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(&source.canonical_path)
            .to_string(),
        text: parsed.text,
        utterances: parsed.utterances,
        duration_seconds: normalized.duration_seconds,
        normalized_audio_bytes: normalized.byte_count,
        cache_hit: false,
        model_version: ASR_MODEL_VERSION.to_string(),
    };
    save_cached_result(&source, &result)?;
    update_context(task_context.as_ref(), 100.0, "语音识别完成");
    Ok(result)
}

struct ParsedAsrResponse {
    text: String,
    utterances: Vec<AsrUtterance>,
}

async fn request_asr(api_key: &str, audio: &[u8]) -> Result<ParsedAsrResponse, String> {
    let request_id = Uuid::new_v4().to_string();
    let response = build_asr_client()?
        .post(ASR_ENDPOINT)
        .header("X-Api-Key", api_key)
        .header("X-Api-Resource-Id", ASR_RESOURCE_ID)
        .header("X-Api-Request-Id", &request_id)
        .header("X-Api-Sequence", "-1")
        .timeout(Duration::from_secs(180))
        .json(&build_asr_request_body(audio, &request_id))
        .send()
        .await
        .map_err(|error| retryable_service_error(format_asr_request_error(error)))?;

    let http_status = response.status();
    let headers = response.headers().clone();
    let response_body = response
        .text()
        .await
        .map_err(|error| retryable_service_error(format!("读取火山语音识别响应失败：{error}")))?;
    let api_code = read_api_status_code(&headers).or_else(|| read_body_status_code(&response_body));

    if !http_status.is_success() {
        let detail = read_error_detail(&headers, &response_body);
        let message = format!(
            "火山语音识别请求失败（HTTP {}）：{}",
            http_status.as_u16(),
            detail
        );
        return if is_retryable_status(http_status.as_u16(), api_code, &detail) {
            Err(retryable_service_error(message))
        } else {
            Err(message)
        };
    }

    match api_code {
        Some(ASR_SUCCESS_CODE) | None => parse_asr_response(&response_body),
        Some(ASR_SILENCE_CODE) => {
            Err("没有检测到可识别的人声，请确认文件有清晰说话声音。".to_string())
        }
        Some(code) => {
            let detail = read_error_detail(&headers, &response_body);
            let message = format!("火山语音识别服务拒绝了本次请求（{code}）：{detail}");
            if is_retryable_status(http_status.as_u16(), Some(code), &detail) {
                Err(retryable_service_error(message))
            } else {
                Err(message)
            }
        }
    }
}

fn build_asr_request_body(audio: &[u8], request_id: &str) -> Value {
    json!({
        "user": {
            "uid": request_id
        },
        "audio": {
            "data": STANDARD.encode(audio),
            "format": "mp3"
        },
        "request": {
            "model_name": "bigmodel",
            "show_utterances": true,
            "enable_punc": true,
            "enable_itn": true,
            "enable_ddc": false
        }
    })
}

fn parse_asr_response(body: &str) -> Result<ParsedAsrResponse, String> {
    let value: Value =
        serde_json::from_str(body).map_err(|_| "火山语音识别返回了无法解析的数据。".to_string())?;
    let result = value
        .get("result")
        .ok_or_else(|| "火山语音识别已返回，但缺少识别结果。".to_string())?;
    let utterances = result
        .get("utterances")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(parse_utterance)
                .collect::<Vec<AsrUtterance>>()
        })
        .unwrap_or_default();
    let text = result
        .get("text")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            utterances
                .iter()
                .map(|item| item.text.as_str())
                .collect::<Vec<_>>()
                .join("")
        });

    if text.is_empty() {
        return Err("识别完成，但没有得到有效文字。请确认人声清晰后重试。".to_string());
    }

    Ok(ParsedAsrResponse { text, utterances })
}

fn parse_utterance(value: &Value) -> Option<AsrUtterance> {
    let text = value.get("text")?.as_str()?.trim().to_string();
    if text.is_empty() {
        return None;
    }
    let start_time_ms = read_integer(value.get("start_time")?)?;
    let end_time_ms = read_integer(value.get("end_time")?)?;
    Some(AsrUtterance {
        start_time_ms,
        end_time_ms: end_time_ms.max(start_time_ms),
        text,
    })
}

fn read_integer(value: &Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| value.as_u64().and_then(|number| i64::try_from(number).ok()))
        .or_else(|| value.as_str().and_then(|text| text.parse::<i64>().ok()))
}

fn read_api_status_code(headers: &HeaderMap) -> Option<i64> {
    headers
        .get("X-Api-Status-Code")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<i64>().ok())
}

fn read_body_status_code(body: &str) -> Option<i64> {
    let value = serde_json::from_str::<Value>(body).ok()?;
    value.get("code").and_then(read_integer)
}

fn read_error_detail(headers: &HeaderMap, body: &str) -> String {
    headers
        .get("X-Api-Message")
        .and_then(|value| value.to_str().ok())
        .map(compact_detail)
        .or_else(|| {
            serde_json::from_str::<Value>(body).ok().and_then(|value| {
                value
                    .get("message")
                    .and_then(Value::as_str)
                    .or_else(|| value.get("error").and_then(Value::as_str))
                    .map(compact_detail)
            })
        })
        .unwrap_or_else(|| "请检查TTS/ASR语音密钥和火山语音额度后重试。".to_string())
}

fn compact_detail(value: &str) -> String {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        "服务没有返回错误说明。".to_string()
    } else {
        normalized.chars().take(240).collect()
    }
}

fn retryable_service_error(message: String) -> String {
    format!("{RETRYABLE_ERROR_PREFIX}{message}")
}

fn is_retryable_status(http_status: u16, api_code: Option<i64>, detail: &str) -> bool {
    (api_code == Some(ASR_BUSY_CODE)
        || matches!(http_status, 408 | 425 | 429 | 500 | 502 | 503 | 504))
        && !indicates_exhausted_quota(detail)
}

fn indicates_exhausted_quota(detail: &str) -> bool {
    let normalized = detail.to_ascii_lowercase();
    [
        "insufficient quota",
        "quota exhausted",
        "quota exceeded",
        "insufficient balance",
        "余额不足",
        "额度不足",
        "额度耗尽",
    ]
    .iter()
    .any(|keyword| normalized.contains(keyword))
}

fn format_asr_request_error(error: reqwest::Error) -> String {
    if error.is_timeout() {
        return "等待火山语音识别超时，请稍后重试。".to_string();
    }
    if error.is_connect() {
        return "无法连接火山语音识别服务，请检查网络、代理或证书环境。".to_string();
    }
    "火山语音识别请求失败，请稍后重试。".to_string()
}

fn build_asr_client() -> Result<Client, String> {
    let root_certificates = webpki_root_certs::TLS_SERVER_ROOT_CERTS
        .iter()
        .map(|certificate| Certificate::from_der(certificate.as_ref()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("无法加载内置HTTPS根证书：{error}"))?;

    Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .http1_only()
        .tls_certs_only(root_certificates)
        .build()
        .map_err(|error| format!("无法创建火山语音识别客户端：{error}"))
}

fn ensure_task_active(context: Option<&TaskProgressContext>) -> Result<(), String> {
    if let Some(context) = context {
        ensure_not_cancelled(&context.task_id)?;
    }
    Ok(())
}

fn update_context(context: Option<&TaskProgressContext>, ratio_percent: f64, stage: &str) {
    if let Some(context) = context {
        let width = context.end_percent - context.start_percent;
        let progress = context.start_percent + width * (ratio_percent / 100.0).clamp(0.0, 1.0);
        let _ = update_task(&context.task_id, progress, stage);
    }
}

fn build_source_identity(path: &Path) -> Result<SourceIdentity, String> {
    let canonical =
        fs::canonicalize(path).map_err(|error| format!("无法读取待识别文件：{error}"))?;
    let metadata =
        fs::metadata(&canonical).map_err(|error| format!("无法读取文件信息：{error}"))?;
    if !metadata.is_file() {
        return Err("选择的路径不是音频或视频文件。".to_string());
    }
    let modified_time_ms = metadata
        .modified()
        .unwrap_or(UNIX_EPOCH)
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    Ok(SourceIdentity {
        canonical_path: canonical.to_string_lossy().to_string(),
        file_size_bytes: metadata.len(),
        modified_time_ms,
    })
}

fn load_cached_result(source: &SourceIdentity) -> Result<Option<AsrRecognitionResult>, String> {
    let path = cache_file_path(source)?;
    if !path.exists() {
        return Ok(None);
    }
    let metadata = fs::metadata(&path).map_err(|error| format!("无法读取语音识别缓存：{error}"))?;
    if metadata.len() > MAX_CACHE_FILE_BYTES {
        let _ = fs::remove_file(path);
        return Ok(None);
    }
    let content = fs::read(&path).map_err(|error| format!("无法读取语音识别缓存：{error}"))?;
    let entry = match serde_json::from_slice::<AsrCacheEntry>(&content) {
        Ok(entry) => entry,
        Err(_) => {
            let _ = fs::remove_file(path);
            return Ok(None);
        }
    };
    if entry.cache_version != CACHE_VERSION
        || entry.model_version != ASR_MODEL_VERSION
        || entry.source != *source
    {
        return Ok(None);
    }
    Ok(Some(entry.result))
}

fn save_cached_result(
    source: &SourceIdentity,
    result: &AsrRecognitionResult,
) -> Result<(), String> {
    let path = cache_file_path(source)?;
    let parent = path
        .parent()
        .ok_or_else(|| "无法确定语音识别缓存目录。".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("无法创建语音识别缓存目录：{error}"))?;
    let entry = AsrCacheEntry {
        cache_version: CACHE_VERSION,
        model_version: ASR_MODEL_VERSION.to_string(),
        source: source.clone(),
        result: result.clone(),
    };
    let content =
        serde_json::to_vec(&entry).map_err(|error| format!("无法整理识别缓存：{error}"))?;
    let temp_path = path.with_extension(format!("{}.tmp", Uuid::new_v4()));
    fs::write(&temp_path, content).map_err(|error| format!("无法写入识别缓存：{error}"))?;
    if path.exists() {
        fs::remove_file(&path).map_err(|error| format!("无法更新旧识别缓存：{error}"))?;
    }
    fs::rename(&temp_path, &path).map_err(|error| format!("无法完成识别缓存保存：{error}"))
}

fn cache_file_path(source: &SourceIdentity) -> Result<PathBuf, String> {
    Ok(cache_root()?.join(format!("{:016x}.json", cache_key(source))))
}

fn cache_root() -> Result<PathBuf, String> {
    let app_data = env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "无法找到Windows应用配置目录。".to_string())?;
    Ok(app_data.join(CACHE_DIRECTORY).join(CACHE_FILE_DIRECTORY))
}

fn cache_key(source: &SourceIdentity) -> u64 {
    let mut hasher = StableFnvHasher::default();
    source.canonical_path.hash(&mut hasher);
    source.file_size_bytes.hash(&mut hasher);
    source.modified_time_ms.hash(&mut hasher);
    ASR_MODEL_VERSION.hash(&mut hasher);
    hasher.finish()
}

struct StableFnvHasher(u64);

impl Default for StableFnvHasher {
    fn default() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
}

impl Hasher for StableFnvHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_asr_request_body, cache_key, is_retryable_status, parse_asr_response, AsrUtterance,
        SourceIdentity, ASR_BUSY_CODE,
    };

    #[test]
    fn builds_official_flash_request_body() {
        let body = build_asr_request_body(b"audio", "request-id");
        assert_eq!(body["audio"]["format"], "mp3");
        assert_eq!(body["request"]["model_name"], "bigmodel");
        assert_eq!(body["request"]["show_utterances"], true);
    }

    #[test]
    fn parses_text_and_sentence_timeline() {
        let parsed = parse_asr_response(
            r#"{"result":{"text":"你好，世界。","utterances":[{"start_time":120,"end_time":980,"text":"你好，"},{"start_time":"1000","end_time":"1800","text":"世界。"}]}}"#,
        )
        .unwrap();
        assert_eq!(parsed.text, "你好，世界。");
        assert_eq!(
            parsed.utterances,
            vec![
                AsrUtterance {
                    start_time_ms: 120,
                    end_time_ms: 980,
                    text: "你好，".to_string(),
                },
                AsrUtterance {
                    start_time_ms: 1000,
                    end_time_ms: 1800,
                    text: "世界。".to_string(),
                },
            ]
        );
    }

    #[test]
    fn falls_back_to_joined_utterance_text() {
        let parsed = parse_asr_response(
            r#"{"result":{"utterances":[{"start_time":0,"end_time":300,"text":"测试"}]}}"#,
        )
        .unwrap();
        assert_eq!(parsed.text, "测试");
    }

    #[test]
    fn marks_busy_failures_as_retryable_but_not_exhausted_quota() {
        assert!(is_retryable_status(200, Some(ASR_BUSY_CODE), "server busy"));
        assert!(is_retryable_status(503, None, "service unavailable"));
        assert!(!is_retryable_status(429, None, "账户额度不足"));
    }

    #[test]
    fn cache_key_changes_with_source_identity() {
        let first = SourceIdentity {
            canonical_path: "C:/video/a.mp4".to_string(),
            file_size_bytes: 100,
            modified_time_ms: 200,
        };
        let mut second = first.clone();
        second.modified_time_ms += 1;
        assert_ne!(cache_key(&first), cache_key(&second));
        assert_eq!(cache_key(&first), cache_key(&first));
    }
}
