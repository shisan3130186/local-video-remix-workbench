use base64::{engine::general_purpose::STANDARD, Engine as _};
use reqwest::tls::Certificate;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::env;
use std::error::Error as StdError;
use std::fs;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRemixVisualSegmentInput {
    segment_id: String,
    duration_seconds: f64,
    thumbnail_path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRemixSegmentInput {
    segment_id: String,
    duration_seconds: f64,
    description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiRemixSegmentAnalysis {
    segment_id: String,
    description: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRemixSegmentAnalysisResult {
    segments: Vec<AiRemixSegmentAnalysis>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRemixPlanResult {
    shots: Vec<AiRemixShot>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiRemixShot {
    text: String,
    segment_id: String,
    #[serde(default)]
    alternative_segment_ids: Vec<String>,
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

#[derive(Clone, Copy)]
enum AiRequestStage {
    VisualAnalysis,
    RemixPlanning,
}

impl AiRequestStage {
    fn label(self) -> &'static str {
        match self {
            Self::VisualAnalysis => "片段画面理解",
            Self::RemixPlanning => "纯文字分镜规划",
        }
    }

    fn timeout(self) -> Duration {
        match self {
            Self::VisualAnalysis => Duration::from_secs(180),
            Self::RemixPlanning => Duration::from_secs(45),
        }
    }
}

pub async fn analyze_ai_remix_segments(
    segments: Vec<AiRemixVisualSegmentInput>,
) -> Result<AiRemixSegmentAnalysisResult, String> {
    validate_visual_inputs(&segments)?;
    let expected_ids = segments
        .iter()
        .map(|segment| segment.segment_id.clone())
        .collect::<Vec<_>>();
    let content = build_visual_analysis_content(&segments)?;
    let response_content = request_ai_completion(
        json!([
            {
                "role": "system",
                "content": "你是短视频素材理解助手。请分别描述每张片段预览图中真正可见的主体、动作、场景和用途。描述要客观、简短，不能猜测图片之外的信息。只允许返回 JSON 对象，不要返回 Markdown、代码围栏、解释或其他字段。格式必须是：{\"segments\":[{\"segmentId\":\"segment-001\",\"description\":\"画面描述\"}]}"
            },
            {
                "role": "user",
                "content": content
            }
        ]),
        700,
        AiRequestStage::VisualAnalysis,
    )
    .await?;

    Ok(AiRemixSegmentAnalysisResult {
        segments: parse_and_validate_analyses(&response_content, &expected_ids)?,
    })
}

pub async fn plan_ai_remix(
    script: String,
    segments: Vec<AiRemixSegmentInput>,
) -> Result<AiRemixPlanResult, String> {
    let normalized_script = script.trim();
    validate_planning_inputs(normalized_script, &segments)?;
    let expected_ids = segments
        .iter()
        .map(|segment| segment.segment_id.clone())
        .collect::<Vec<_>>();
    let planning_prompt = build_planning_prompt(normalized_script, &segments);
    let content = request_ai_completion(
        json!([
            {
                "role": "system",
                "content": "你是短视频分镜规划助手。请把用户文案拆成按叙事顺序排列的镜头句子，并根据已有的片段画面描述，为每句话选择画面最匹配的主片段。只选择与文案相关的片段，不要求使用全部素材；不同分镜的主片段不得重复。每个分镜最多返回 3 个相关备选片段。只允许返回 JSON 对象，不要返回 Markdown、代码围栏、解释或其他字段。格式必须是：{\"shots\":[{\"text\":\"镜头对应的文案\",\"segmentId\":\"segment-001\",\"alternativeSegmentIds\":[\"segment-002\"]}]}"
            },
            {
                "role": "user",
                "content": planning_prompt
            }
        ]),
        800,
        AiRequestStage::RemixPlanning,
    )
    .await?;

    Ok(AiRemixPlanResult {
        shots: parse_and_validate_shots(&content, &expected_ids)?,
    })
}

async fn request_ai_completion(
    messages: Value,
    max_tokens: u32,
    stage: AiRequestStage,
) -> Result<String, String> {
    let api_key = required_environment_variable("AI_API_KEY")?;
    let base_url = required_environment_variable("AI_BASE_URL")?;
    let model = required_environment_variable("AI_MODEL")?;
    let endpoint = build_chat_completions_url(&base_url)?;
    let response = build_ai_client()?
        .post(endpoint)
        .bearer_auth(api_key)
        .timeout(stage.timeout())
        .json(&build_ai_request_body(model, messages, max_tokens))
        .send()
        .await
        .map_err(|error| format_ai_request_error(error, stage))?;

    if !response.status().is_success() {
        return Err(format!(
            "{}请求失败（HTTP {}）。请检查 AI_BASE_URL、AI_MODEL、API Key 和账户额度。",
            stage.label(),
            response.status()
        ));
    }

    let response_body = response
        .json::<ChatCompletionResponse>()
        .await
        .map_err(|error| format!("{}响应格式无法识别：{error}", stage.label()))?;

    response_body
        .choices
        .first()
        .map(|choice| choice.message.content.trim().to_string())
        .filter(|content| !content.is_empty())
        .ok_or_else(|| "AI 没有返回有效结果。".to_string())
}

fn build_ai_request_body(model: String, messages: Value, max_tokens: u32) -> Value {
    json!({
        "model": model,
        "temperature": 0,
        "max_tokens": max_tokens,
        "thinking": {
            "type": "disabled"
        },
        "messages": messages
    })
}

fn build_ai_client() -> Result<Client, String> {
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
        .map_err(|error| format!("无法创建 AI 请求客户端：{error}"))
}

fn format_ai_request_error(error: reqwest::Error, stage: AiRequestStage) -> String {
    let reason = if error.is_timeout() {
        if error.is_connect() {
            "连接火山引擎超时"
        } else {
            "等待火山引擎返回结果超时"
        }
    } else if error.is_connect() {
        "无法建立到火山引擎的网络连接"
    } else if error.is_request() {
        "AI 请求发送失败"
    } else {
        "AI 网络请求失败"
    };
    let detail = error
        .source()
        .map(ToString::to_string)
        .unwrap_or_else(|| error.to_string());

    format!(
        "{}阶段：{reason}。请检查网络后重试；如果仍然失败，请查看任务日志。（{detail}）",
        stage.label()
    )
}

fn required_environment_variable(name: &str) -> Result<String, String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("未配置环境变量 {name}。"))
}

fn validate_visual_inputs(segments: &[AiRemixVisualSegmentInput]) -> Result<(), String> {
    if segments.is_empty() {
        return Err("请至少提供 1 个需要理解的片段。".to_string());
    }

    if segments.len() > 2 {
        return Err("单次最多理解 2 个片段，请分批提交。".to_string());
    }

    let mut segment_ids = HashSet::new();

    for segment in segments {
        validate_segment_identity(
            &segment.segment_id,
            segment.duration_seconds,
            &mut segment_ids,
        )?;

        if !Path::new(&segment.thumbnail_path).is_file() {
            return Err(format!("片段 {} 缺少可用的预览图。", segment.segment_id));
        }
    }

    Ok(())
}

fn validate_planning_inputs(script: &str, segments: &[AiRemixSegmentInput]) -> Result<(), String> {
    if script.is_empty() {
        return Err("请先输入用于规划混剪的文案。".to_string());
    }

    if segments.len() < 2 {
        return Err("AI 智能混剪至少需要 2 个片段。".to_string());
    }

    let mut segment_ids = HashSet::new();

    for segment in segments {
        validate_segment_identity(
            &segment.segment_id,
            segment.duration_seconds,
            &mut segment_ids,
        )?;

        if segment.description.trim().is_empty() {
            return Err(format!("片段 {} 缺少画面描述。", segment.segment_id));
        }
    }

    Ok(())
}

fn validate_segment_identity<'a>(
    segment_id: &'a str,
    duration_seconds: f64,
    segment_ids: &mut HashSet<&'a str>,
) -> Result<(), String> {
    if segment_id.trim().is_empty() {
        return Err("片段编号不能为空。".to_string());
    }

    if !segment_ids.insert(segment_id) {
        return Err(format!("片段编号重复：{segment_id}。"));
    }

    if !duration_seconds.is_finite() || duration_seconds <= 0.0 {
        return Err(format!("片段 {segment_id} 的时长无效。"));
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

fn build_visual_analysis_content(
    segments: &[AiRemixVisualSegmentInput],
) -> Result<Vec<Value>, String> {
    let mut content = vec![json!({
        "type": "text",
        "text": "请逐一理解下面的片段预览图，并按片段编号返回简短、客观的画面描述。"
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

fn build_planning_prompt(script: &str, segments: &[AiRemixSegmentInput]) -> String {
    let segment_lines = segments
        .iter()
        .map(|segment| {
            format!(
                "- {}；时长：{:.3} 秒；画面描述：{}",
                segment.segment_id,
                segment.duration_seconds,
                segment.description.trim()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "用户文案：\n{script}\n\n可用片段：\n{segment_lines}\n\n请将文案拆成适合短视频节奏的分镜句子，为每句话选择最匹配的主片段，并给出最多 3 个备选片段。可以舍弃无关素材，主片段之间不能重复。"
    )
}

fn parse_and_validate_analyses(
    content: &str,
    expected_ids: &[String],
) -> Result<Vec<AiRemixSegmentAnalysis>, String> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct AnalysisResponse {
        segments: Vec<AiRemixSegmentAnalysis>,
    }

    let parsed = serde_json::from_str::<AnalysisResponse>(content)
        .map_err(|_| "AI 必须严格返回只包含 segments 的 JSON 对象。".to_string())?;

    if parsed.segments.len() != expected_ids.len() {
        return Err(format!(
            "AI 返回了 {} 个片段描述，但本批次需要 {} 个。",
            parsed.segments.len(),
            expected_ids.len()
        ));
    }

    let expected = expected_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let mut returned_ids = HashSet::new();

    for analysis in &parsed.segments {
        if !expected.contains(analysis.segment_id.as_str()) {
            return Err(format!("AI 返回了未知片段编号：{}。", analysis.segment_id));
        }

        if !returned_ids.insert(analysis.segment_id.as_str()) {
            return Err(format!("AI 重复返回片段编号：{}。", analysis.segment_id));
        }

        if analysis.description.trim().is_empty() {
            return Err(format!("片段 {} 的画面描述为空。", analysis.segment_id));
        }
    }

    Ok(parsed.segments)
}

fn parse_and_validate_shots(
    content: &str,
    expected_ids: &[String],
) -> Result<Vec<AiRemixShot>, String> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct ShotsResponse {
        shots: Vec<AiRemixShot>,
    }

    let parsed = serde_json::from_str::<ShotsResponse>(content)
        .map_err(|_| "AI 必须严格返回只包含 shots 的 JSON 对象。".to_string())?;

    if parsed.shots.len() < 2 {
        return Err("AI 至少需要返回 2 个分镜。".to_string());
    }

    if parsed.shots.len() > expected_ids.len() {
        return Err(format!(
            "AI 返回了 {} 个分镜，但本次只有 {} 个可用片段。",
            parsed.shots.len(),
            expected_ids.len()
        ));
    }

    let expected = expected_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let mut primary_ids = HashSet::new();

    for (index, shot) in parsed.shots.iter().enumerate() {
        if shot.text.trim().is_empty() {
            return Err(format!("AI 返回的第 {} 个分镜文案为空。", index + 1));
        }

        if !expected.contains(shot.segment_id.as_str()) {
            return Err(format!("AI 返回了未知主片段编号：{}。", shot.segment_id));
        }

        if !primary_ids.insert(shot.segment_id.as_str()) {
            return Err(format!("AI 重复使用了主片段编号：{}。", shot.segment_id));
        }

        if shot.alternative_segment_ids.len() > 3 {
            return Err(format!("第 {} 个分镜返回的备选片段超过 3 个。", index + 1));
        }

        let mut alternative_ids = HashSet::new();

        for alternative_id in &shot.alternative_segment_ids {
            if !expected.contains(alternative_id.as_str()) {
                return Err(format!("AI 返回了未知备选片段编号：{alternative_id}。"));
            }

            if alternative_id == &shot.segment_id {
                return Err(format!(
                    "第 {} 个分镜的主片段不能同时作为备选片段。",
                    index + 1
                ));
            }

            if !alternative_ids.insert(alternative_id.as_str()) {
                return Err(format!("第 {} 个分镜包含重复备选片段。", index + 1));
            }
        }
    }

    Ok(parsed.shots)
}

#[cfg(test)]
mod tests {
    use super::{
        build_ai_client, build_ai_request_body, build_chat_completions_url,
        parse_and_validate_analyses, parse_and_validate_shots, AiRequestStage,
    };
    use serde_json::json;
    use std::time::Duration;

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
    fn builds_client_with_embedded_root_certificates() {
        build_ai_client().unwrap();
    }

    #[test]
    fn uses_stage_specific_timeouts() {
        assert_eq!(
            AiRequestStage::VisualAnalysis.timeout(),
            Duration::from_secs(180)
        );
        assert_eq!(
            AiRequestStage::RemixPlanning.timeout(),
            Duration::from_secs(45)
        );
    }

    #[test]
    fn disables_model_thinking_in_request_body() {
        let body = build_ai_request_body(
            "doubao-seed-2-1-pro".to_string(),
            json!([{"role":"user","content":"测试"}]),
            800,
        );

        assert_eq!(body["thinking"]["type"], "disabled");
        assert_eq!(body["max_tokens"], 800);
        assert_eq!(body["model"], "doubao-seed-2-1-pro");
    }

    #[test]
    fn accepts_complete_segment_analysis() {
        let result = parse_and_validate_analyses(
            r#"{"segments":[{"segmentId":"segment-001","description":"手持清洁工具靠近水槽"},{"segmentId":"segment-002","description":"清洗后的桌面保持整洁"}]}"#,
            &expected_ids(),
        )
        .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].segment_id, "segment-001");
    }

    #[test]
    fn rejects_missing_segment_analysis() {
        let error = parse_and_validate_analyses(
            r#"{"segments":[{"segmentId":"segment-001","description":"水槽画面"}]}"#,
            &expected_ids(),
        )
        .unwrap_err();
        assert!(error.contains("需要 2 个"));
    }

    #[test]
    fn rejects_unknown_segment_analysis() {
        let error = parse_and_validate_analyses(
            r#"{"segments":[{"segmentId":"segment-001","description":"水槽画面"},{"segmentId":"segment-999","description":"未知画面"}]}"#,
            &expected_ids(),
        )
        .unwrap_err();
        assert!(error.contains("未知片段"));
    }

    #[test]
    fn accepts_valid_shot_plan() {
        let result = parse_and_validate_shots(
            r#"{"shots":[{"text":"先展示结果","segmentId":"segment-002","alternativeSegmentIds":[]},{"text":"再展示过程","segmentId":"segment-001","alternativeSegmentIds":["segment-002"]}]}"#,
            &expected_ids(),
        )
        .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].segment_id, "segment-002");
    }

    #[test]
    fn rejects_duplicate_primary_ids() {
        let error = parse_and_validate_shots(
            r#"{"shots":[{"text":"镜头一","segmentId":"segment-001","alternativeSegmentIds":[]},{"text":"镜头二","segmentId":"segment-001","alternativeSegmentIds":[]}]}"#,
            &expected_ids(),
        )
        .unwrap_err();
        assert!(error.contains("重复"));
    }

    #[test]
    fn rejects_single_shot() {
        let error = parse_and_validate_shots(
            r#"{"shots":[{"text":"镜头一","segmentId":"segment-001","alternativeSegmentIds":[]}]}"#,
            &expected_ids(),
        )
        .unwrap_err();
        assert!(error.contains("至少"));
    }

    #[test]
    fn rejects_unknown_primary_id() {
        let error = parse_and_validate_shots(
            r#"{"shots":[{"text":"镜头一","segmentId":"segment-001","alternativeSegmentIds":[]},{"text":"镜头二","segmentId":"segment-999","alternativeSegmentIds":[]}]}"#,
            &expected_ids(),
        )
        .unwrap_err();
        assert!(error.contains("未知"));
    }

    #[test]
    fn rejects_unknown_alternative_id() {
        let error = parse_and_validate_shots(
            r#"{"shots":[{"text":"镜头一","segmentId":"segment-001","alternativeSegmentIds":["segment-999"]},{"text":"镜头二","segmentId":"segment-002","alternativeSegmentIds":[]}]}"#,
            &expected_ids(),
        )
        .unwrap_err();
        assert!(error.contains("未知备选"));
    }

    #[test]
    fn rejects_too_many_alternatives() {
        let expected = vec![
            "segment-001".to_string(),
            "segment-002".to_string(),
            "segment-003".to_string(),
            "segment-004".to_string(),
            "segment-005".to_string(),
        ];
        let error = parse_and_validate_shots(
            r#"{"shots":[{"text":"镜头一","segmentId":"segment-001","alternativeSegmentIds":["segment-002","segment-003","segment-004","segment-005"]},{"text":"镜头二","segmentId":"segment-002","alternativeSegmentIds":[]}]}"#,
            &expected,
        )
        .unwrap_err();
        assert!(error.contains("超过 3 个"));
    }

    #[test]
    fn rejects_non_json_content() {
        let error = parse_and_validate_shots("```json", &expected_ids()).unwrap_err();
        assert!(error.contains("严格返回"));
    }
}
