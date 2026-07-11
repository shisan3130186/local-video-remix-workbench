use base64::{engine::general_purpose::STANDARD, Engine as _};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRemixSegmentInput {
    segment_id: String,
    duration_seconds: f64,
    thumbnail_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRemixPlanResult {
    ordered_segment_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChoice {
    message: ChatCompletionMessage,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionMessage {
    content: String,
}

pub async fn plan_ai_remix(
    script: String,
    segments: Vec<AiRemixSegmentInput>,
) -> Result<AiRemixPlanResult, String> {
    let normalized_script = script.trim();
    validate_inputs(normalized_script, &segments)?;

    let api_key = required_environment_variable("AI_API_KEY")?;
    let base_url = required_environment_variable("AI_BASE_URL")?;
    let model = required_environment_variable("AI_MODEL")?;
    let endpoint = build_chat_completions_url(&base_url)?;
    let content = build_multimodal_content(normalized_script, &segments)?;

    let response = Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|error| format!("无法创建 AI 请求客户端：{error}"))?
        .post(endpoint)
        .bearer_auth(api_key)
        .json(&json!({
            "model": model,
            "temperature": 0,
            "messages": [
                {
                    "role": "system",
                    "content": "你是短视频剪辑规划助手。根据用户文案、片段编号、时长和预览图，给出最符合文案叙事的完整片段顺序。必须使用全部片段且每个片段只能出现一次。只允许返回 JSON 对象，不要返回 Markdown、代码围栏、解释或其他字段。格式必须是：{\"orderedSegmentIds\":[\"segment-001\",\"segment-002\"]}"
                },
                {
                    "role": "user",
                    "content": content
                }
            ]
        }))
        .send()
        .await
        .map_err(|error| format!("无法连接 AI 服务：{error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "AI 服务请求失败（HTTP {}）。请检查 AI_BASE_URL、AI_MODEL 和 API Key。",
            response.status()
        ));
    }

    let response_body = response
        .json::<ChatCompletionResponse>()
        .await
        .map_err(|error| format!("AI 服务响应格式无法识别：{error}"))?;
    let content = response_body
        .choices
        .first()
        .map(|choice| choice.message.content.trim())
        .filter(|content| !content.is_empty())
        .ok_or_else(|| "AI 没有返回排序结果。".to_string())?;
    let expected_ids = segments
        .iter()
        .map(|segment| segment.segment_id.clone())
        .collect::<Vec<_>>();

    Ok(AiRemixPlanResult {
        ordered_segment_ids: parse_and_validate_ordered_ids(content, &expected_ids)?,
    })
}

fn required_environment_variable(name: &str) -> Result<String, String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("未配置环境变量 {name}。"))
}

fn validate_inputs(script: &str, segments: &[AiRemixSegmentInput]) -> Result<(), String> {
    if script.is_empty() {
        return Err("请先输入用于规划混剪的文案。".to_string());
    }

    if segments.len() < 2 {
        return Err("AI 智能混剪至少需要 2 个片段。".to_string());
    }

    let mut segment_ids = HashSet::new();

    for segment in segments {
        if segment.segment_id.trim().is_empty() {
            return Err("片段编号不能为空。".to_string());
        }

        if !segment_ids.insert(segment.segment_id.as_str()) {
            return Err(format!("片段编号重复：{}。", segment.segment_id));
        }

        if !segment.duration_seconds.is_finite() || segment.duration_seconds <= 0.0 {
            return Err(format!("片段 {} 的时长无效。", segment.segment_id));
        }

        if !Path::new(&segment.thumbnail_path).is_file() {
            return Err(format!("片段 {} 缺少可用的预览图。", segment.segment_id));
        }
    }

    Ok(())
}

fn build_chat_completions_url(base_url: &str) -> Result<String, String> {
    let normalized = base_url.trim().trim_end_matches('/');

    if normalized.is_empty() {
        return Err("AI_BASE_URL 不能为空。".to_string());
    }

    if normalized.ends_with("/chat/completions") {
        Ok(normalized.to_string())
    } else {
        Ok(format!("{normalized}/chat/completions"))
    }
}

fn build_multimodal_content(
    script: &str,
    segments: &[AiRemixSegmentInput],
) -> Result<Vec<Value>, String> {
    let mut content = vec![json!({
        "type": "text",
        "text": format!(
            "用户文案：\n{script}\n\n请结合下面全部片段的画面和时长进行排序。返回 orderedSegmentIds，必须包含全部片段编号且不重复。"
        )
    })];

    for segment in segments {
        let image_bytes = fs::read(&segment.thumbnail_path)
            .map_err(|error| format!("无法读取片段 {} 的预览图：{error}", segment.segment_id))?;
        let encoded_image = STANDARD.encode(image_bytes);

        content.push(json!({
            "type": "text",
            "text": format!(
                "片段编号：{}；时长：{:.3} 秒",
                segment.segment_id, segment.duration_seconds
            )
        }));
        content.push(json!({
            "type": "image_url",
            "image_url": {
                "url": format!("data:image/jpeg;base64,{encoded_image}")
            }
        }));
    }

    Ok(content)
}

fn parse_and_validate_ordered_ids(
    content: &str,
    expected_ids: &[String],
) -> Result<Vec<String>, String> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct OrderedIdsResponse {
        ordered_segment_ids: Vec<String>,
    }

    let parsed = serde_json::from_str::<OrderedIdsResponse>(content)
        .map_err(|_| "AI 必须严格返回只包含 orderedSegmentIds 的 JSON 对象。".to_string())?;

    if parsed.ordered_segment_ids.len() != expected_ids.len() {
        return Err(format!(
            "AI 返回了 {} 个片段，但本次需要 {} 个片段。",
            parsed.ordered_segment_ids.len(),
            expected_ids.len()
        ));
    }

    let expected = expected_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let mut returned = HashSet::new();

    for segment_id in &parsed.ordered_segment_ids {
        if !expected.contains(segment_id.as_str()) {
            return Err(format!("AI 返回了未知片段编号：{segment_id}。"));
        }

        if !returned.insert(segment_id.as_str()) {
            return Err(format!("AI 重复返回了片段编号：{segment_id}。"));
        }
    }

    if returned.len() != expected.len() {
        return Err("AI 返回结果缺少片段编号。".to_string());
    }

    Ok(parsed.ordered_segment_ids)
}

#[cfg(test)]
mod tests {
    use super::{build_chat_completions_url, parse_and_validate_ordered_ids};

    fn expected_ids() -> Vec<String> {
        vec!["segment-001".to_string(), "segment-002".to_string()]
    }

    #[test]
    fn appends_chat_completions_to_base_url() {
        assert_eq!(
            build_chat_completions_url("https://ark.cn-beijing.volces.com/api/v3/").unwrap(),
            "https://ark.cn-beijing.volces.com/api/v3/chat/completions"
        );
    }

    #[test]
    fn keeps_complete_chat_completions_url() {
        assert_eq!(
            build_chat_completions_url("https://ark.cn-beijing.volces.com/api/v3/chat/completions")
                .unwrap(),
            "https://ark.cn-beijing.volces.com/api/v3/chat/completions"
        );
    }

    #[test]
    fn accepts_complete_ordering() {
        let result = parse_and_validate_ordered_ids(
            r#"{"orderedSegmentIds":["segment-002","segment-001"]}"#,
            &expected_ids(),
        )
        .unwrap();
        assert_eq!(result, vec!["segment-002", "segment-001"]);
    }

    #[test]
    fn rejects_duplicate_ids() {
        let error = parse_and_validate_ordered_ids(
            r#"{"orderedSegmentIds":["segment-001","segment-001"]}"#,
            &expected_ids(),
        )
        .unwrap_err();
        assert!(error.contains("重复"));
    }

    #[test]
    fn rejects_missing_ids() {
        let error = parse_and_validate_ordered_ids(
            r#"{"orderedSegmentIds":["segment-001"]}"#,
            &expected_ids(),
        )
        .unwrap_err();
        assert!(error.contains("需要 2 个片段"));
    }

    #[test]
    fn rejects_unknown_ids() {
        let error = parse_and_validate_ordered_ids(
            r#"{"orderedSegmentIds":["segment-001","segment-999"]}"#,
            &expected_ids(),
        )
        .unwrap_err();
        assert!(error.contains("未知"));
    }

    #[test]
    fn rejects_non_json_content() {
        let error = parse_and_validate_ordered_ids("```json", &expected_ids()).unwrap_err();
        assert!(error.contains("严格返回"));
    }
}
