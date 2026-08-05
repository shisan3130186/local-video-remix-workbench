use crate::api_config::{get_api_config_status, load_tts_service_config};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use reqwest::tls::Certificate;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const TTS_ENDPOINT: &str = "https://openspeech.bytedance.com/api/v3/tts/unidirectional";
const TTS_SUCCESS_CODE: i64 = 20_000_000;
const RETRYABLE_ERROR_PREFIX: &str = "__RETRYABLE_SERVICE_ERROR__:";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsConfigStatus {
    configured: bool,
    resource_id: String,
    speaker: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsSynthesisResult {
    output_path: String,
    text_length: usize,
    audio_byte_count: usize,
    speaker: String,
    resource_id: String,
    words: Vec<TtsWordTiming>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TtsWordTiming {
    confidence: f64,
    start_time: f64,
    end_time: f64,
    word: String,
}

struct TtsConfig {
    api_key: String,
    resource_id: String,
    speaker: String,
}

#[derive(Debug, Deserialize)]
struct TtsResponseChunk {
    code: i64,
    message: Option<String>,
    data: Option<String>,
    sentence: Option<TtsSentence>,
}

#[derive(Debug, Deserialize)]
struct TtsSentence {
    #[serde(default)]
    words: Vec<TtsWordTiming>,
}

#[derive(Debug, PartialEq)]
struct ParsedTtsResponse {
    audio: Vec<u8>,
    words: Vec<TtsWordTiming>,
}

pub fn get_tts_config_status() -> Result<TtsConfigStatus, String> {
    let status = get_api_config_status()?;

    Ok(TtsConfigStatus {
        configured: status.tts_configured,
        resource_id: status.tts_resource_id,
        speaker: status.tts_speaker,
    })
}

pub async fn synthesize_tts(
    text: String,
    output_directory: String,
    speaker: Option<String>,
) -> Result<TtsSynthesisResult, String> {
    let normalized_text = text.trim();

    if normalized_text.is_empty() {
        return Err("请输入需要生成配音的文案。".to_string());
    }

    let output_dir = Path::new(&output_directory);
    if !output_dir.is_dir() {
        return Err("请先选择有效的输出目录。".to_string());
    }

    let config = load_tts_config(speaker)?;
    let parsed = request_tts_audio(normalized_text, &config).await?;
    let output_path = output_dir.join(format!("tts_{}.mp3", current_timestamp_millis()?));

    save_tts_result(normalized_text, output_path, config, parsed)
}

pub async fn synthesize_tts_shot(
    text: String,
    speaker: Option<String>,
    session_id: String,
    shot_index: usize,
) -> Result<TtsSynthesisResult, String> {
    let normalized_text = text.trim();

    if normalized_text.is_empty() {
        return Err("分镜配音文案不能为空。".to_string());
    }

    let session_dir = tts_session_directory(&session_id)?;
    fs::create_dir_all(&session_dir).map_err(|error| format!("无法创建临时配音目录：{error}"))?;

    let config = load_tts_config(speaker)?;
    let parsed = request_tts_audio(normalized_text, &config).await?;
    let output_path = session_dir.join(format!("shot_{shot_index:03}.mp3"));

    save_tts_result(normalized_text, output_path, config, parsed)
}

pub fn cleanup_tts_session(session_id: String) -> Result<(), String> {
    let session_dir = tts_session_directory(&session_id)?;

    if !session_dir.exists() {
        return Ok(());
    }

    fs::remove_dir_all(session_dir).map_err(|error| format!("无法清理临时配音文件：{error}"))
}

/// 合成试听样本（base64 mp3），用于音色库"未提供官方试听"时的实时合成
#[tauri::command]
pub async fn synthesize_preview_audio(
    text: String,
    speaker: Option<String>,
) -> Result<String, String> {
    let normalized_text = text.trim();
    if normalized_text.is_empty() {
        return Err("试听文案不能为空。".to_string());
    }
    let config = load_tts_config(speaker)?;
    let parsed = request_tts_audio(normalized_text, &config).await?;
    if parsed.audio.is_empty() {
        return Err("火山语音服务已返回结果，但没有收到音频数据。".to_string());
    }
    Ok(STANDARD.encode(&parsed.audio))
}

async fn request_tts_audio(
    normalized_text: &str,
    config: &TtsConfig,
) -> Result<ParsedTtsResponse, String> {
    let request_id = Uuid::new_v4().to_string();
    let response = build_tts_client()?
        .post(TTS_ENDPOINT)
        .header("X-Api-Key", &config.api_key)
        .header("X-Api-Resource-Id", &config.resource_id)
        .header("X-Api-Request-Id", request_id)
        .timeout(Duration::from_secs(120))
        .json(&build_tts_request_body(normalized_text, &config.speaker))
        .send()
        .await
        .map_err(|error| retryable_service_error(format_tts_request_error(error)))?;

    let status = response.status();
    let response_body = response
        .text()
        .await
        .map_err(|error| format!("读取火山语音响应失败：{error}"))?;

    if !status.is_success() {
        let detail = parse_http_error_detail(&response_body);
        let message = format!(
            "火山语音服务请求失败（HTTP {}）：{}",
            status.as_u16(),
            detail
        );

        return if is_retryable_service_response(status.as_u16(), &detail) {
            Err(retryable_service_error(message))
        } else {
            Err(message)
        };
    }

    let parsed = parse_tts_response(&response_body)?;
    if parsed.audio.is_empty() {
        return Err("火山语音服务已返回结果，但没有收到音频数据。".to_string());
    }

    Ok(parsed)
}

fn retryable_service_error(message: String) -> String {
    format!("{RETRYABLE_ERROR_PREFIX}{message}")
}

fn is_retryable_service_response(status: u16, detail: &str) -> bool {
    matches!(status, 408 | 425 | 429 | 500 | 502 | 503 | 504) && !indicates_exhausted_quota(detail)
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

fn save_tts_result(
    normalized_text: &str,
    output_path: PathBuf,
    config: TtsConfig,
    parsed: ParsedTtsResponse,
) -> Result<TtsSynthesisResult, String> {
    fs::write(&output_path, &parsed.audio).map_err(|error| format!("无法保存配音文件：{error}"))?;

    Ok(TtsSynthesisResult {
        output_path: output_path.to_string_lossy().to_string(),
        text_length: normalized_text.chars().count(),
        audio_byte_count: parsed.audio.len(),
        speaker: config.speaker,
        resource_id: config.resource_id,
        words: parsed.words,
    })
}

fn tts_session_directory(session_id: &str) -> Result<PathBuf, String> {
    if session_id.is_empty()
        || session_id.len() > 80
        || !session_id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
    {
        return Err("临时配音会话编号无效。".to_string());
    }

    Ok(env::temp_dir()
        .join("local-video-remix-workbench")
        .join("tts-remix")
        .join(session_id))
}

fn load_tts_config(speaker_override: Option<String>) -> Result<TtsConfig, String> {
    let config = load_tts_service_config(speaker_override)?;

    Ok(TtsConfig {
        api_key: config.api_key,
        resource_id: config.resource_id,
        speaker: config.speaker,
    })
}

fn build_tts_request_body(text: &str, speaker: &str) -> Value {
    json!({
        "req_params": {
            "text": text,
            "speaker": speaker,
            "audio_params": {
                "format": "mp3",
                "sample_rate": 24000
            }
        }
    })
}

fn build_tts_client() -> Result<Client, String> {
    let root_certificates = webpki_root_certs::TLS_SERVER_ROOT_CERTS
        .iter()
        .map(|certificate| Certificate::from_der(certificate.as_ref()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("无法加载内置 HTTPS 根证书：{error}"))?;

    Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .http1_only()
        .tls_certs_only(root_certificates)
        .build()
        .map_err(|error| format!("无法创建火山语音请求客户端：{error}"))
}

fn format_tts_request_error(error: reqwest::Error) -> String {
    if error.is_timeout() {
        if error.is_connect() {
            return "连接火山语音服务超时，请检查网络后重试。".to_string();
        }

        return "等待火山语音生成配音超时，请稍后重试。".to_string();
    }

    if error.is_connect() {
        return "无法连接火山语音服务，请检查网络、代理或证书环境。".to_string();
    }

    "火山语音请求失败，请稍后重试。".to_string()
}

fn compact_error_detail(value: &str) -> String {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return "服务没有返回错误说明。".to_string();
    }

    normalized.chars().take(240).collect()
}

fn parse_http_error_detail(body: &str) -> String {
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|value| {
            value
                .get("message")
                .and_then(Value::as_str)
                .or_else(|| value.get("error").and_then(Value::as_str))
                .map(compact_error_detail)
        })
        .unwrap_or_else(|| "请检查TTS Key、模型资源和音色ID后重试。".to_string())
}

fn parse_tts_response(body: &str) -> Result<ParsedTtsResponse, String> {
    let normalized = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && *line != "[DONE]")
        .map(|line| line.strip_prefix("data:").unwrap_or(line).trim())
        .collect::<Vec<_>>()
        .join("\n");

    if normalized.is_empty() {
        return Err("火山语音服务返回了空响应。".to_string());
    }

    let mut audio = Vec::new();
    let mut words = Vec::new();
    let stream = serde_json::Deserializer::from_str(&normalized).into_iter::<TtsResponseChunk>();

    for item in stream {
        let chunk = item.map_err(|_| "火山语音服务返回了无法解析的数据。".to_string())?;

        if !is_tts_success_code(chunk.code) {
            return Err(format!(
                "火山语音服务拒绝了本次请求：{}",
                chunk
                    .message
                    .as_deref()
                    .map(compact_error_detail)
                    .unwrap_or_else(|| "未知错误。".to_string())
            ));
        }

        if let Some(data) = chunk.data.filter(|value| !value.is_empty()) {
            let decoded = STANDARD
                .decode(data)
                .map_err(|_| "火山语音服务返回的音频数据无效。".to_string())?;
            audio.extend(decoded);
        }

        if let Some(sentence) = chunk.sentence {
            words.extend(sentence.words);
        }
    }

    Ok(ParsedTtsResponse { audio, words })
}

fn is_tts_success_code(code: i64) -> bool {
    code == 0 || code == TTS_SUCCESS_CODE
}

fn current_timestamp_millis() -> Result<u128, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .map_err(|error| format!("无法生成配音文件名：{error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_official_tts_request_body() {
        let body = build_tts_request_body("测试文案", "speaker-id");

        assert_eq!(body["req_params"]["text"], "测试文案");
        assert_eq!(body["req_params"]["speaker"], "speaker-id");
        assert_eq!(body["req_params"]["audio_params"]["format"], "mp3");
        assert_eq!(body["req_params"]["audio_params"]["sample_rate"], 24000);
    }

    #[test]
    fn parses_single_tts_response() {
        let audio = STANDARD.encode([1_u8, 2, 3]);
        let body = format!(
            r#"{{"code":0,"message":"OK","data":"{audio}","sentence":{{"words":[{{"confidence":0.9,"startTime":0.1,"endTime":0.3,"word":"你"}}]}}}}"#
        );

        let parsed = parse_tts_response(&body).unwrap();

        assert_eq!(parsed.audio, vec![1, 2, 3]);
        assert_eq!(parsed.words.len(), 1);
        assert_eq!(parsed.words[0].word, "你");
    }

    #[test]
    fn accepts_official_tts_success_code() {
        let audio = STANDARD.encode([4_u8, 5, 6]);
        let body = format!(r#"{{"code":20000000,"message":"OK","data":"{audio}"}}"#);

        let parsed = parse_tts_response(&body).unwrap();

        assert_eq!(parsed.audio, vec![4, 5, 6]);
    }

    #[test]
    fn joins_multiple_tts_audio_chunks() {
        let first = STANDARD.encode([1_u8, 2]);
        let second = STANDARD.encode([3_u8, 4]);
        let body =
            format!("{{\"code\":0,\"data\":\"{first}\"}}\n{{\"code\":0,\"data\":\"{second}\"}}");

        let parsed = parse_tts_response(&body).unwrap();

        assert_eq!(parsed.audio, vec![1, 2, 3, 4]);
    }

    #[test]
    fn rejects_tts_service_error() {
        let error = parse_tts_response(r#"{"code":1001,"message":"invalid speaker"}"#).unwrap_err();

        assert!(error.contains("invalid speaker"));
    }

    #[test]
    fn rejects_invalid_tts_audio_data() {
        let error = parse_tts_response(r#"{"code":0,"data":"not-base64"}"#).unwrap_err();

        assert_eq!(error, "火山语音服务返回的音频数据无效。");
    }

    #[test]
    fn extracts_only_safe_http_error_message() {
        let detail = parse_http_error_detail(
            r#"{"message":"invalid api key","request":{"text":"private script"}}"#,
        );

        assert_eq!(detail, "invalid api key");
        assert!(!detail.contains("private script"));
    }

    #[test]
    fn marks_temporary_tts_failures_as_retryable() {
        assert!(is_retryable_service_response(429, "rate limit exceeded"));
        assert!(is_retryable_service_response(503, "service unavailable"));
        assert!(!is_retryable_service_response(403, "invalid api key"));
    }

    #[test]
    fn does_not_retry_exhausted_tts_quota() {
        assert!(!is_retryable_service_response(429, "quota exhausted"));
        assert!(!is_retryable_service_response(429, "余额不足"));
    }

    #[test]
    fn builds_tts_client_with_embedded_roots() {
        build_tts_client().unwrap();
    }

    #[test]
    fn builds_safe_tts_session_directory() {
        let path = tts_session_directory("session-123").unwrap();

        assert!(path.ends_with(Path::new("tts-remix").join("session-123")));
    }

    #[test]
    fn rejects_unsafe_tts_session_id() {
        assert_eq!(
            tts_session_directory("../private").unwrap_err(),
            "临时配音会话编号无效。"
        );
    }
}
