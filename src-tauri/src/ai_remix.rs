use crate::api_config::load_ai_service_config;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use reqwest::tls::Certificate;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::error::Error as StdError;
use std::fs;
use std::path::Path;
use std::time::Duration;

const TARGET_SHOT_TEXT_MIN_CHARACTERS: usize = 6;
const TARGET_SHOT_TEXT_MAX_CHARACTERS: usize = 14;
const MAX_SHOT_TEXT_CHARACTERS: usize = 16;
const ESTIMATED_SPEECH_CHARACTERS_PER_SECOND: f64 = 4.2;
const ESTIMATED_SPEECH_TAIL_SECONDS: f64 = 0.35;
const SAFE_VIDEO_MIN_PLAYBACK_RATE: f64 = 0.92;
const SAFE_VIDEO_MAX_FREEZE_SECONDS: f64 = 0.8;

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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRemixVariantShotInput {
    segment_id: String,
    #[serde(default)]
    alternative_segment_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRemixVariantPlan {
    segment_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRemixVariantPlanResult {
    variants: Vec<AiRemixVariantPlan>,
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
            Self::RemixPlanning => "固定短句画面匹配",
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
    let fixed_shot_texts = split_script_into_shots(normalized_script)?;
    validate_planning_inputs(normalized_script, &fixed_shot_texts, &segments)?;
    let planning_prompt = build_planning_prompt(&fixed_shot_texts, &segments);
    let content = request_ai_completion(
        json!([
            {
                "role": "system",
                "content": "你是短视频画面匹配助手。软件已经完成文案断句，你不能修改、合并或新增分镜，只需要为每个 shotIndex 选择最匹配且时长合适的主片段。主片段不能重复。每个分镜最多返回 3 个内容相关、尽量更长的备选片段；备选片段可以与其他分镜的主片段重复，软件会自动清理冲突。必须覆盖全部 shotIndex。只允许返回 JSON 对象，不要返回 Markdown、代码围栏、解释或其他字段。格式必须是：{\"matches\":[{\"shotIndex\":1,\"segmentId\":\"segment-001\",\"alternativeSegmentIds\":[\"segment-002\"]}]}"
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
        shots: parse_and_validate_shots(&content, &fixed_shot_texts, &segments)?,
    })
}

pub fn build_ai_remix_variants(
    shots: Vec<AiRemixVariantShotInput>,
    requested_count: usize,
) -> Result<AiRemixVariantPlanResult, String> {
    validate_variant_inputs(&shots, requested_count)?;

    let candidate_lists = shots
        .iter()
        .map(|shot| {
            std::iter::once(shot.segment_id.clone())
                .chain(shot.alternative_segment_ids.iter().cloned())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut variants = Vec::new();
    let mut seen_variants = HashSet::new();

    push_variant_if_unique(
        candidate_lists
            .iter()
            .map(|candidates| candidates[0].clone())
            .collect(),
        &mut variants,
        &mut seen_variants,
    );

    if variants.len() < requested_count {
        let max_rotation_attempts = requested_count.saturating_mul(shots.len()).max(12);
        for rotation in 0..max_rotation_attempts {
            let mut used_segment_ids = HashSet::new();
            let mut selected_segment_ids = Vec::with_capacity(candidate_lists.len());

            for (shot_index, candidates) in candidate_lists.iter().enumerate() {
                let preferred_index = if candidates.len() == 1 {
                    0
                } else {
                    1 + (rotation + shot_index) % (candidates.len() - 1)
                };
                let selected =
                    select_unused_candidate(candidates, preferred_index, &used_segment_ids);
                used_segment_ids.insert(selected.clone());
                selected_segment_ids.push(selected);
            }

            push_variant_if_unique(selected_segment_ids, &mut variants, &mut seen_variants);
            if variants.len() >= requested_count {
                break;
            }
        }
    }

    if variants.len() < requested_count {
        append_mixed_radix_variants(
            &candidate_lists,
            requested_count,
            &mut variants,
            &mut seen_variants,
        );
    }

    Ok(AiRemixVariantPlanResult { variants })
}

fn validate_variant_inputs(
    shots: &[AiRemixVariantShotInput],
    requested_count: usize,
) -> Result<(), String> {
    if shots.len() < 2 {
        return Err("至少保留 2 个分镜才能生成差异视频。".to_string());
    }

    if !(1..=10).contains(&requested_count) {
        return Err("差异视频数量必须在 1 到 10 之间。".to_string());
    }

    let mut primary_segment_ids = HashSet::new();
    for (index, shot) in shots.iter().enumerate() {
        if shot.segment_id.trim().is_empty() {
            return Err(format!("第 {} 个分镜的主片段编号为空。", index + 1));
        }
        if !primary_segment_ids.insert(shot.segment_id.as_str()) {
            return Err(format!("主分镜重复使用了片段 {}。", shot.segment_id));
        }

        let mut candidate_ids = HashSet::new();
        candidate_ids.insert(shot.segment_id.as_str());
        for alternative_id in &shot.alternative_segment_ids {
            if alternative_id.trim().is_empty() {
                return Err(format!("第 {} 个分镜包含空的备选片段编号。", index + 1));
            }
            if !candidate_ids.insert(alternative_id.as_str()) {
                return Err(format!("第 {} 个分镜包含重复候选片段。", index + 1));
            }
        }
    }

    Ok(())
}

fn select_unused_candidate(
    candidates: &[String],
    preferred_index: usize,
    used_segment_ids: &HashSet<String>,
) -> String {
    (0..candidates.len())
        .map(|offset| (preferred_index + offset) % candidates.len())
        .find_map(|index| {
            let candidate = &candidates[index];
            (!used_segment_ids.contains(candidate)).then(|| candidate.clone())
        })
        .unwrap_or_else(|| candidates[preferred_index % candidates.len()].clone())
}

fn push_variant_if_unique(
    segment_ids: Vec<String>,
    variants: &mut Vec<AiRemixVariantPlan>,
    seen_variants: &mut HashSet<String>,
) {
    if segment_ids.iter().collect::<HashSet<_>>().len() != segment_ids.len() {
        return;
    }

    let key = segment_ids.join("\u{1f}");
    if seen_variants.insert(key) {
        variants.push(AiRemixVariantPlan { segment_ids });
    }
}

fn append_mixed_radix_variants(
    candidate_lists: &[Vec<String>],
    requested_count: usize,
    variants: &mut Vec<AiRemixVariantPlan>,
    seen_variants: &mut HashSet<String>,
) {
    let combination_limit = candidate_lists.iter().fold(1usize, |total, candidates| {
        total.saturating_mul(candidates.len())
    });
    let search_limit = combination_limit.min(100_000);

    for ordinal in 1..search_limit {
        let mut remainder = ordinal;
        let mut selected_segment_ids = Vec::with_capacity(candidate_lists.len());

        for candidates in candidate_lists {
            let candidate_index = remainder % candidates.len();
            remainder /= candidates.len();
            selected_segment_ids.push(candidates[candidate_index].clone());
        }

        push_variant_if_unique(selected_segment_ids, variants, seen_variants);
        if variants.len() >= requested_count {
            break;
        }
    }
}

async fn request_ai_completion(
    messages: Value,
    max_tokens: u32,
    stage: AiRequestStage,
) -> Result<String, String> {
    let config = load_ai_service_config()?;
    let endpoint = build_chat_completions_url(&config.base_url)?;
    let response = build_ai_client()?
        .post(endpoint)
        .bearer_auth(config.api_key)
        .timeout(stage.timeout())
        .json(&build_ai_request_body(config.model, messages, max_tokens))
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

fn validate_planning_inputs(
    script: &str,
    fixed_shot_texts: &[String],
    segments: &[AiRemixSegmentInput],
) -> Result<(), String> {
    if script.is_empty() {
        return Err("请先输入用于规划混剪的文案。".to_string());
    }

    if segments.len() < 2 {
        return Err("AI 智能混剪至少需要 2 个片段。".to_string());
    }

    if fixed_shot_texts.len() > segments.len() {
        return Err(format!(
            "软件已把文案拆成 {} 个短分镜，但当前只有 {} 个可用片段。请缩短文案，或切出更多片段后重试。",
            fixed_shot_texts.len(),
            segments.len()
        ));
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

fn build_planning_prompt(fixed_shot_texts: &[String], segments: &[AiRemixSegmentInput]) -> String {
    let shot_lines = fixed_shot_texts
        .iter()
        .enumerate()
        .map(|(index, text)| {
            format!(
                "- shotIndex：{}；固定文案：{}；预计配音：{:.2} 秒",
                index + 1,
                text,
                estimate_narration_duration(text)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
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
        "固定分镜（只匹配画面，不要修改文案）：\n{shot_lines}\n\n可用片段：\n{segment_lines}\n\n请为每个 shotIndex 返回一个主片段和最多3个相关备选片段。优先选择画面内容匹配、时长足够的片段；主片段不能重复，必须覆盖全部固定分镜。"
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
    fixed_shot_texts: &[String],
    segments: &[AiRemixSegmentInput],
) -> Result<Vec<AiRemixShot>, String> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct ShotMatch {
        shot_index: usize,
        segment_id: String,
        #[serde(default)]
        alternative_segment_ids: Vec<String>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct MatchesResponse {
        matches: Vec<ShotMatch>,
    }

    let mut parsed = serde_json::from_str::<MatchesResponse>(content)
        .map_err(|_| "AI 必须严格返回只包含 matches 的 JSON 对象。".to_string())?;

    if parsed.matches.len() != fixed_shot_texts.len() {
        return Err(format!(
            "AI 返回了 {} 个画面匹配，但软件已经固定了 {} 个分镜。请重新生成。",
            parsed.matches.len(),
            fixed_shot_texts.len()
        ));
    }

    let expected = segments
        .iter()
        .map(|segment| segment.segment_id.as_str())
        .collect::<HashSet<_>>();
    let duration_by_id = segments
        .iter()
        .map(|segment| (segment.segment_id.as_str(), segment.duration_seconds))
        .collect::<HashMap<_, _>>();
    let mut returned_shot_indices = HashSet::new();
    let mut primary_ids = HashSet::<String>::new();

    for shot_match in &parsed.matches {
        if shot_match.shot_index == 0 || shot_match.shot_index > fixed_shot_texts.len() {
            return Err(format!(
                "AI 返回了未知分镜序号：{}。",
                shot_match.shot_index
            ));
        }

        if !returned_shot_indices.insert(shot_match.shot_index) {
            return Err(format!(
                "AI 重复返回了第 {} 个分镜。",
                shot_match.shot_index
            ));
        }

        if !expected.contains(shot_match.segment_id.as_str()) {
            return Err(format!(
                "AI 返回了未知主片段编号：{}。",
                shot_match.segment_id
            ));
        }

        if !primary_ids.insert(shot_match.segment_id.clone()) {
            return Err(format!(
                "AI 重复使用了主片段编号：{}。",
                shot_match.segment_id
            ));
        }

        if shot_match.alternative_segment_ids.len() > 3 {
            return Err(format!(
                "第 {} 个分镜返回的备选片段超过 3 个。",
                shot_match.shot_index
            ));
        }

        let mut alternative_ids = HashSet::new();

        for alternative_id in &shot_match.alternative_segment_ids {
            if !expected.contains(alternative_id.as_str()) {
                return Err(format!("AI 返回了未知备选片段编号：{alternative_id}。"));
            }

            if alternative_id == &shot_match.segment_id {
                return Err(format!(
                    "第 {} 个分镜的主片段不能同时作为备选片段。",
                    shot_match.shot_index
                ));
            }

            if !alternative_ids.insert(alternative_id.as_str()) {
                return Err(format!(
                    "第 {} 个分镜包含重复备选片段。",
                    shot_match.shot_index
                ));
            }
        }
    }

    parsed
        .matches
        .sort_by_key(|shot_match| shot_match.shot_index);
    let mut shots = parsed
        .matches
        .into_iter()
        .map(|shot_match| AiRemixShot {
            text: fixed_shot_texts[shot_match.shot_index - 1].clone(),
            segment_id: shot_match.segment_id,
            alternative_segment_ids: shot_match.alternative_segment_ids,
        })
        .collect::<Vec<_>>();

    for shot in &mut shots {
        shot.alternative_segment_ids
            .retain(|alternative_id| !primary_ids.contains(alternative_id));
    }

    for (index, shot) in shots.iter().enumerate() {
        let longest_candidate_duration = std::iter::once(&shot.segment_id)
            .chain(shot.alternative_segment_ids.iter())
            .filter_map(|segment_id| duration_by_id.get(segment_id.as_str()).copied())
            .fold(0.0_f64, f64::max);
        let estimated_narration_duration = estimate_narration_duration(&shot.text);
        let safely_supported_duration = longest_candidate_duration / SAFE_VIDEO_MIN_PLAYBACK_RATE
            + SAFE_VIDEO_MAX_FREEZE_SECONDS;
        if estimated_narration_duration > safely_supported_duration + 0.001 {
            return Err(format!(
                "第 {} 个分镜预计配音约 {:.1} 秒，但主画面和备选画面最长只能安全适配到 {:.1} 秒。请重新生成更短分镜。",
                index + 1,
                estimated_narration_duration,
                safely_supported_duration
            ));
        }
    }

    Ok(shots)
}

fn split_script_into_shots(script: &str) -> Result<Vec<String>, String> {
    let normalized = script.trim();
    if count_effective_characters(normalized) < 2 {
        return Err("文案内容太短，至少需要能够拆成 2 个分镜。".to_string());
    }

    let characters = normalized.chars().collect::<Vec<_>>();
    let mut shots = Vec::new();
    let mut current = String::new();
    let mut current_effective_count = 0;
    let mut index = 0;

    while index < characters.len() {
        let character = characters[index];
        current.push(character);
        if character.is_alphanumeric() {
            current_effective_count += 1;
        }

        let reached_natural_boundary = current_effective_count >= TARGET_SHOT_TEXT_MIN_CHARACTERS
            && is_script_boundary(character);
        let reached_target_limit = current_effective_count >= TARGET_SHOT_TEXT_MAX_CHARACTERS;

        if reached_target_limit {
            while index + 1 < characters.len() && is_script_boundary(characters[index + 1]) {
                index += 1;
                current.push(characters[index]);
            }
        }

        if reached_natural_boundary || reached_target_limit {
            push_script_shot(&mut shots, &mut current);
            current_effective_count = 0;
        }

        index += 1;
    }

    push_script_shot(&mut shots, &mut current);
    merge_short_trailing_shot(&mut shots);

    if shots.len() == 1 {
        shots = split_single_script_shot(&shots[0])?;
    }

    if shots
        .iter()
        .any(|shot| count_effective_characters(shot) > MAX_SHOT_TEXT_CHARACTERS)
    {
        return Err("文案自动断句失败，请补充逗号或句号后重试。".to_string());
    }

    Ok(shots)
}

fn push_script_shot(shots: &mut Vec<String>, current: &mut String) {
    let normalized = current.trim();
    if !normalized.is_empty() {
        shots.push(normalized.to_string());
    }
    current.clear();
}

fn merge_short_trailing_shot(shots: &mut Vec<String>) {
    if shots.len() < 2 {
        return;
    }

    let trailing_count = shots
        .last()
        .map(|shot| count_effective_characters(shot))
        .unwrap_or(0);
    if trailing_count >= TARGET_SHOT_TEXT_MIN_CHARACTERS {
        return;
    }

    let previous_index = shots.len() - 2;
    let combined_count = count_effective_characters(&shots[previous_index]) + trailing_count;
    if combined_count <= MAX_SHOT_TEXT_CHARACTERS {
        let trailing = shots.pop().unwrap_or_default();
        shots[previous_index].push_str(&trailing);
    }
}

fn split_single_script_shot(shot: &str) -> Result<Vec<String>, String> {
    let total_effective_count = count_effective_characters(shot);
    let target_first_count = total_effective_count / 2;
    let mut current_effective_count = 0;

    for (byte_index, character) in shot.char_indices() {
        if character.is_alphanumeric() {
            current_effective_count += 1;
        }

        let split_index = byte_index + character.len_utf8();
        if current_effective_count >= target_first_count && split_index < shot.len() {
            let first = shot[..split_index].trim().to_string();
            let second = shot[split_index..].trim().to_string();
            if !first.is_empty() && !second.is_empty() {
                return Ok(vec![first, second]);
            }
        }
    }

    Err("文案内容太短，至少需要能够拆成 2 个分镜。".to_string())
}

fn is_script_boundary(character: char) -> bool {
    matches!(
        character,
        '，' | ',' | '。' | '.' | '！' | '!' | '？' | '?' | '；' | ';' | '：' | ':' | '、'
    )
}

fn count_effective_characters(text: &str) -> usize {
    text.chars()
        .filter(|character| character.is_alphanumeric())
        .count()
}

fn estimate_narration_duration(text: &str) -> f64 {
    count_effective_characters(text) as f64 / ESTIMATED_SPEECH_CHARACTERS_PER_SECOND
        + ESTIMATED_SPEECH_TAIL_SECONDS
}

#[cfg(test)]
mod tests {
    use super::{
        build_ai_client, build_ai_remix_variants, build_ai_request_body,
        build_chat_completions_url, parse_and_validate_analyses, parse_and_validate_shots,
        split_script_into_shots, AiRemixSegmentInput, AiRemixVariantShotInput, AiRequestStage,
        MAX_SHOT_TEXT_CHARACTERS,
    };
    use serde_json::json;
    use std::{collections::HashSet, time::Duration};

    fn expected_ids() -> Vec<String> {
        vec!["segment-001".to_string(), "segment-002".to_string()]
    }

    fn planning_segments(count: usize) -> Vec<AiRemixSegmentInput> {
        (1..=count)
            .map(|index| AiRemixSegmentInput {
                segment_id: format!("segment-{index:03}"),
                duration_seconds: 5.0,
                description: format!("测试画面 {index}"),
            })
            .collect()
    }

    fn fixed_shots() -> Vec<String> {
        vec!["先展示结果，".to_string(), "再展示过程。".to_string()]
    }

    fn variant_shots() -> Vec<AiRemixVariantShotInput> {
        vec![
            AiRemixVariantShotInput {
                segment_id: "segment-001".to_string(),
                alternative_segment_ids: vec!["segment-003".to_string(), "segment-005".to_string()],
            },
            AiRemixVariantShotInput {
                segment_id: "segment-002".to_string(),
                alternative_segment_ids: vec!["segment-004".to_string(), "segment-006".to_string()],
            },
        ]
    }

    #[test]
    fn keeps_primary_plan_as_first_variant() {
        let result = build_ai_remix_variants(variant_shots(), 3).unwrap();

        assert_eq!(result.variants.len(), 3);
        assert_eq!(
            result.variants[0].segment_ids,
            vec!["segment-001".to_string(), "segment-002".to_string()]
        );
    }

    #[test]
    fn builds_distinct_variants_without_repeating_segments_inside_a_video() {
        let result = build_ai_remix_variants(variant_shots(), 6).unwrap();
        let variant_keys = result
            .variants
            .iter()
            .map(|variant| variant.segment_ids.join("->"))
            .collect::<HashSet<_>>();

        assert_eq!(result.variants.len(), 6);
        assert_eq!(variant_keys.len(), result.variants.len());
        assert!(result.variants.iter().all(|variant| {
            variant.segment_ids.iter().collect::<HashSet<_>>().len() == variant.segment_ids.len()
        }));
    }

    #[test]
    fn returns_only_available_unique_variants_when_alternatives_are_missing() {
        let shots = vec![
            AiRemixVariantShotInput {
                segment_id: "segment-001".to_string(),
                alternative_segment_ids: vec![],
            },
            AiRemixVariantShotInput {
                segment_id: "segment-002".to_string(),
                alternative_segment_ids: vec![],
            },
        ];
        let result = build_ai_remix_variants(shots, 3).unwrap();

        assert_eq!(result.variants.len(), 1);
    }

    #[test]
    fn rejects_duplicate_primary_segments_for_variants() {
        let shots = vec![
            AiRemixVariantShotInput {
                segment_id: "segment-001".to_string(),
                alternative_segment_ids: vec![],
            },
            AiRemixVariantShotInput {
                segment_id: "segment-001".to_string(),
                alternative_segment_ids: vec![],
            },
        ];
        let error = build_ai_remix_variants(shots, 3).unwrap_err();

        assert!(error.contains("重复使用"));
    }

    #[test]
    fn rejects_variant_counts_outside_supported_range() {
        let zero_error = build_ai_remix_variants(variant_shots(), 0).unwrap_err();
        let eleven_error = build_ai_remix_variants(variant_shots(), 11).unwrap_err();

        assert!(zero_error.contains("1 到 10"));
        assert!(eleven_error.contains("1 到 10"));
    }

    #[test]
    fn skips_variants_that_repeat_a_conflicting_candidate() {
        let shots = vec![
            AiRemixVariantShotInput {
                segment_id: "segment-001".to_string(),
                alternative_segment_ids: vec!["segment-002".to_string()],
            },
            AiRemixVariantShotInput {
                segment_id: "segment-003".to_string(),
                alternative_segment_ids: vec!["segment-002".to_string()],
            },
        ];
        let result = build_ai_remix_variants(shots, 10).unwrap();

        assert_eq!(result.variants.len(), 3);
        assert!(result.variants.iter().all(|variant| {
            variant.segment_ids.iter().collect::<HashSet<_>>().len() == variant.segment_ids.len()
        }));
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
            r#"{"matches":[{"shotIndex":2,"segmentId":"segment-001","alternativeSegmentIds":[]},{"shotIndex":1,"segmentId":"segment-002","alternativeSegmentIds":[]}]}"#,
            &fixed_shots(),
            &planning_segments(2),
        )
        .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].segment_id, "segment-002");
        assert_eq!(result[0].text, "先展示结果，");
    }

    #[test]
    fn rejects_duplicate_primary_ids() {
        let error = parse_and_validate_shots(
            r#"{"matches":[{"shotIndex":1,"segmentId":"segment-001","alternativeSegmentIds":[]},{"shotIndex":2,"segmentId":"segment-001","alternativeSegmentIds":[]}]}"#,
            &fixed_shots(),
            &planning_segments(2),
        )
        .unwrap_err();
        assert!(error.contains("重复"));
    }

    #[test]
    fn rejects_missing_shot_match() {
        let error = parse_and_validate_shots(
            r#"{"matches":[{"shotIndex":1,"segmentId":"segment-001","alternativeSegmentIds":[]}]}"#,
            &fixed_shots(),
            &planning_segments(2),
        )
        .unwrap_err();
        assert!(error.contains("固定了 2 个分镜"));
    }

    #[test]
    fn rejects_unknown_shot_index() {
        let error = parse_and_validate_shots(
            r#"{"matches":[{"shotIndex":1,"segmentId":"segment-001","alternativeSegmentIds":[]},{"shotIndex":3,"segmentId":"segment-002","alternativeSegmentIds":[]}]}"#,
            &fixed_shots(),
            &planning_segments(2),
        )
        .unwrap_err();
        assert!(error.contains("未知分镜序号"));
    }

    #[test]
    fn rejects_duplicate_shot_index() {
        let error = parse_and_validate_shots(
            r#"{"matches":[{"shotIndex":1,"segmentId":"segment-001","alternativeSegmentIds":[]},{"shotIndex":1,"segmentId":"segment-002","alternativeSegmentIds":[]}]}"#,
            &fixed_shots(),
            &planning_segments(2),
        )
        .unwrap_err();
        assert!(error.contains("重复返回了第 1 个分镜"));
    }

    #[test]
    fn rejects_unknown_primary_id() {
        let error = parse_and_validate_shots(
            r#"{"matches":[{"shotIndex":1,"segmentId":"segment-001","alternativeSegmentIds":[]},{"shotIndex":2,"segmentId":"segment-999","alternativeSegmentIds":[]}]}"#,
            &fixed_shots(),
            &planning_segments(2),
        )
        .unwrap_err();
        assert!(error.contains("未知"));
    }

    #[test]
    fn rejects_unknown_alternative_id() {
        let error = parse_and_validate_shots(
            r#"{"matches":[{"shotIndex":1,"segmentId":"segment-001","alternativeSegmentIds":["segment-999"]},{"shotIndex":2,"segmentId":"segment-002","alternativeSegmentIds":[]}]}"#,
            &fixed_shots(),
            &planning_segments(2),
        )
        .unwrap_err();
        assert!(error.contains("未知备选"));
    }

    #[test]
    fn rejects_too_many_alternatives() {
        let error = parse_and_validate_shots(
            r#"{"matches":[{"shotIndex":1,"segmentId":"segment-001","alternativeSegmentIds":["segment-003","segment-004","segment-005","segment-006"]},{"shotIndex":2,"segmentId":"segment-002","alternativeSegmentIds":[]}]}"#,
            &fixed_shots(),
            &planning_segments(6),
        )
        .unwrap_err();
        assert!(error.contains("超过 3 个"));
    }

    #[test]
    fn rejects_non_json_content() {
        let error =
            parse_and_validate_shots("```json", &fixed_shots(), &planning_segments(2)).unwrap_err();
        assert!(error.contains("严格返回"));
    }

    #[test]
    fn removes_alternative_that_is_used_as_another_primary() {
        let result = parse_and_validate_shots(
            r#"{"matches":[{"shotIndex":1,"segmentId":"segment-001","alternativeSegmentIds":["segment-002"]},{"shotIndex":2,"segmentId":"segment-002","alternativeSegmentIds":[]}]}"#,
            &fixed_shots(),
            &planning_segments(2),
        )
        .unwrap();

        assert!(result[0].alternative_segment_ids.is_empty());
    }

    #[test]
    fn locally_splits_long_script_without_losing_text() {
        let script = "夏天拍照总想显瘦的姐妹看这件冰丝上衣！垂感超好不会皱，宽松版型遮住手臂拜拜肉，搭配短裤，半身裙，西装裤都行。";
        let shots = split_script_into_shots(script).unwrap();

        assert!(shots.len() >= 2);
        assert_eq!(shots.concat(), script);
        assert!(shots
            .iter()
            .all(|shot| super::count_effective_characters(shot) <= MAX_SHOT_TEXT_CHARACTERS));
    }

    #[test]
    fn locally_splits_unpunctuated_long_script() {
        let shots =
            split_script_into_shots("这是一段没有任何标点但是仍然需要自动拆成多个短分镜的测试文案")
                .unwrap();

        assert!(shots.len() >= 2);
        assert!(shots
            .iter()
            .all(|shot| super::count_effective_characters(shot) <= MAX_SHOT_TEXT_CHARACTERS));
    }

    #[test]
    fn locally_splits_short_script_into_two_shots() {
        let shots = split_script_into_shots("轻薄显瘦很好穿").unwrap();

        assert_eq!(shots.len(), 2);
        assert_eq!(shots.concat(), "轻薄显瘦很好穿");
    }
}
