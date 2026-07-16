use crate::video_engine::tool_paths::ffmpeg_program;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::process::Command;

const SUBTITLE_TARGET_CHARACTERS: usize = 12;
const SUBTITLE_MAX_CHARACTERS: usize = 16;
const MIN_SUBTITLE_DURATION_SECONDS: f64 = 0.7;

#[derive(Debug, PartialEq)]
struct SubtitleCue {
    text: String,
    start_seconds: f64,
    end_seconds: f64,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NarratedSubtitleSettings {
    pub enabled: bool,
    pub position: NarratedSubtitlePosition,
    pub size: NarratedSubtitleSize,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum NarratedSubtitlePosition {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum NarratedSubtitleSize {
    Small,
    Medium,
    Large,
}

pub fn ensure_ass_filter_available() -> Result<(), String> {
    let output = Command::new(ffmpeg_program())
        .args(["-hide_banner", "-filters"])
        .output()
        .map_err(|error| format!("无法检查FFmpeg字幕能力：{error}"))?;

    if !output.status.success() {
        return Err("无法检查FFmpeg字幕能力，请确认FFmpeg可以正常运行。".to_string());
    }

    let filters = String::from_utf8_lossy(&output.stdout);
    let has_ass_filter = filters
        .lines()
        .any(|line| line.split_whitespace().nth(1) == Some("ass"));

    if !has_ass_filter {
        return Err(
            "当前FFmpeg缺少字幕烧录能力（ass滤镜），请更换完整版本FFmpeg或关闭自动字幕。"
                .to_string(),
        );
    }

    Ok(())
}

pub fn prepare_ass_subtitle(
    output_path: &Path,
    text: &str,
    duration_seconds: f64,
    settings: NarratedSubtitleSettings,
) -> Result<Option<String>, String> {
    if !settings.enabled {
        return Ok(None);
    }

    let content = build_ass_document(text, duration_seconds, settings)?;
    fs::write(output_path, content).map_err(|error| format!("无法创建临时字幕文件：{error}"))?;
    Ok(Some(build_ass_filter(output_path)?))
}

fn build_ass_document(
    text: &str,
    duration_seconds: f64,
    settings: NarratedSubtitleSettings,
) -> Result<String, String> {
    if !duration_seconds.is_finite() || duration_seconds <= 0.0 {
        return Err("字幕时长无效。".to_string());
    }

    let normalized_text = normalize_subtitle_text(text);
    if normalized_text.is_empty() {
        return Err("字幕文字不能为空。".to_string());
    }

    let cue_texts = split_subtitle_cues(&normalized_text);
    if cue_texts.is_empty() {
        return Err("字幕文字不能为空。".to_string());
    }
    let cues = build_timed_cues(cue_texts, duration_seconds);
    let longest_cue = cues
        .iter()
        .map(|cue| cue.text.chars().count())
        .max()
        .unwrap_or_default();
    let font_size = effective_font_size(settings.size, longest_cue);
    let alignment = subtitle_alignment(settings.position);
    let margin_v = subtitle_margin(settings.position);
    let dialogue_lines = cues
        .iter()
        .map(|cue| {
            format!(
                "Dialogue: 0,{},{},Default,,0,0,0,,{}",
                format_ass_time(cue.start_seconds),
                format_ass_time(cue.end_seconds),
                wrap_and_escape_ass_text(&cue.text)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    Ok(format!(
        "[Script Info]\n\
         ScriptType: v4.00+\n\
         PlayResX: 1920\n\
         PlayResY: 1080\n\
         ScaledBorderAndShadow: yes\n\
         WrapStyle: 0\n\n\
         [V4+ Styles]\n\
         Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n\
         Style: Default,Microsoft YaHei,{font_size},&H00FFFFFF,&H00FFFFFF,&H64000000,&H64000000,-1,0,0,0,100,100,0,0,3,4,0,{alignment},80,80,{margin_v},1\n\n\
         [Events]\n\
         Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n\
         {dialogue_lines}\n"
    ))
}

fn normalize_subtitle_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn split_subtitle_cues(text: &str) -> Vec<String> {
    let mut clauses = Vec::new();
    let mut current = String::new();

    for character in text.chars() {
        if is_sentence_separator(character) {
            push_trimmed_clause(&mut clauses, &mut current);
        } else {
            current.push(character);
        }
    }
    push_trimmed_clause(&mut clauses, &mut current);

    clauses
        .into_iter()
        .flat_map(split_long_clause)
        .filter(|clause| !clause.is_empty())
        .collect()
}

fn push_trimmed_clause(clauses: &mut Vec<String>, current: &mut String) {
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        clauses.push(trimmed.to_string());
    }
    current.clear();
}

fn split_long_clause(clause: String) -> Vec<String> {
    let mut remaining = clause.trim().to_string();
    let mut parts = Vec::new();

    while remaining.chars().count() > SUBTITLE_MAX_CHARACTERS {
        let characters = remaining.chars().collect::<Vec<_>>();
        let split_index = find_natural_length_split(&characters);
        let first = characters[..split_index]
            .iter()
            .collect::<String>()
            .trim()
            .to_string();
        remaining = characters[split_index..]
            .iter()
            .collect::<String>()
            .trim()
            .to_string();
        if !first.is_empty() {
            parts.push(first);
        }
    }

    if !remaining.is_empty() {
        parts.push(remaining);
    }
    parts
}

fn find_natural_length_split(characters: &[char]) -> usize {
    let upper_bound = SUBTITLE_MAX_CHARACTERS.min(characters.len() - 1);
    let lower_bound = 6.min(upper_bound);

    for index in (lower_bound..=upper_bound).rev() {
        if characters[index - 1].is_whitespace() {
            return index;
        }
    }

    SUBTITLE_TARGET_CHARACTERS.min(upper_bound)
}

fn build_timed_cues(texts: Vec<String>, duration_seconds: f64) -> Vec<SubtitleCue> {
    let weights = texts
        .iter()
        .map(|text| subtitle_character_weight(text) as f64)
        .collect::<Vec<_>>();
    let total_weight = weights.iter().sum::<f64>().max(1.0);
    let minimum_total = MIN_SUBTITLE_DURATION_SECONDS * texts.len() as f64;
    let durations = if duration_seconds >= minimum_total {
        let flexible_duration = duration_seconds - minimum_total;
        weights
            .iter()
            .map(|weight| MIN_SUBTITLE_DURATION_SECONDS + flexible_duration * weight / total_weight)
            .collect::<Vec<_>>()
    } else {
        weights
            .iter()
            .map(|weight| duration_seconds * weight / total_weight)
            .collect::<Vec<_>>()
    };

    let cue_count = texts.len();
    let mut elapsed = 0.0;
    texts
        .into_iter()
        .zip(durations)
        .enumerate()
        .map(|(index, (text, cue_duration))| {
            let start_seconds = elapsed;
            elapsed = if index + 1 == cue_count {
                duration_seconds
            } else {
                elapsed + cue_duration
            };
            SubtitleCue {
                text,
                start_seconds,
                end_seconds: elapsed,
            }
        })
        .collect()
}

fn subtitle_character_weight(text: &str) -> usize {
    text.chars()
        .filter(|character| !character.is_whitespace())
        .count()
        .max(1)
}

fn is_sentence_separator(character: char) -> bool {
    matches!(
        character,
        '，' | '。' | '！' | '？' | '；' | '、' | '：' | ',' | '.' | '!' | '?' | ';' | ':'
    )
}

fn wrap_and_escape_ass_text(text: &str) -> String {
    let characters = text.chars().collect::<Vec<_>>();
    let split_index = find_two_line_split(&characters);
    let lines = match split_index {
        Some(index) => vec![&characters[..index], &characters[index..]],
        None => vec![characters.as_slice()],
    };

    lines
        .into_iter()
        .map(|line| escape_ass_text(&line.iter().collect::<String>()))
        .collect::<Vec<_>>()
        .join("\\N")
}

fn find_two_line_split(characters: &[char]) -> Option<usize> {
    if characters.len() <= 18 {
        return None;
    }

    let midpoint = characters.len() / 2;
    let search_radius = 6.min(midpoint);

    for offset in 0..=search_radius {
        for candidate in [midpoint.saturating_sub(offset), midpoint + offset] {
            if candidate > 0
                && candidate < characters.len()
                && is_subtitle_break_character(characters[candidate - 1])
            {
                return Some(candidate);
            }
        }
    }

    Some(midpoint)
}

fn is_subtitle_break_character(character: char) -> bool {
    matches!(
        character,
        '，' | '。' | '！' | '？' | '；' | '、' | ',' | '.' | '!' | '?' | ';' | ' '
    )
}

fn escape_ass_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('{', "\\{")
        .replace('}', "\\}")
}

fn effective_font_size(size: NarratedSubtitleSize, character_count: usize) -> u32 {
    let base_size = match size {
        NarratedSubtitleSize::Small => 38.0,
        NarratedSubtitleSize::Medium => 46.0,
        NarratedSubtitleSize::Large => 56.0,
    };
    let scale = if character_count <= 24 {
        1.0
    } else {
        (24.0 / character_count as f64).max(0.6)
    };

    (base_size * scale).round() as u32
}

fn subtitle_alignment(position: NarratedSubtitlePosition) -> u8 {
    match position {
        NarratedSubtitlePosition::Top => 8,
        NarratedSubtitlePosition::Middle => 5,
        NarratedSubtitlePosition::Bottom => 2,
    }
}

fn subtitle_margin(position: NarratedSubtitlePosition) -> u32 {
    match position {
        NarratedSubtitlePosition::Middle => 0,
        NarratedSubtitlePosition::Top | NarratedSubtitlePosition::Bottom => 72,
    }
}

fn format_ass_time(duration_seconds: f64) -> String {
    let total_centiseconds = (duration_seconds * 100.0).ceil() as u64;
    let hours = total_centiseconds / 360_000;
    let minutes = (total_centiseconds / 6_000) % 60;
    let seconds = (total_centiseconds / 100) % 60;
    let centiseconds = total_centiseconds % 100;
    format!("{hours}:{minutes:02}:{seconds:02}.{centiseconds:02}")
}

fn build_ass_filter(path: &Path) -> Result<String, String> {
    let path_text = path
        .to_str()
        .ok_or_else(|| "临时字幕路径包含无法识别的字符。".to_string())?;
    let escaped_path = path_text
        .replace('\\', "/")
        .replace(':', "\\:")
        .replace('\'', "\\'");
    Ok(format!("ass=filename='{escaped_path}'"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_settings() -> NarratedSubtitleSettings {
        NarratedSubtitleSettings {
            enabled: true,
            position: NarratedSubtitlePosition::Bottom,
            size: NarratedSubtitleSize::Medium,
        }
    }

    #[test]
    fn builds_single_cue_ass_document() {
        let content = build_ass_document("这是第一句自动字幕。", 3.21, default_settings()).unwrap();

        assert!(content.contains("Fontname, Fontsize"));
        assert!(content.contains("Microsoft YaHei,46"));
        assert!(content.contains("Dialogue: 0,0:00:00.00,0:00:03.21"));
        assert!(content.contains("这是第一句自动字幕"));
    }

    #[test]
    fn escapes_ass_override_characters() {
        let content = build_ass_document(r"测试{字幕}\\路径", 1.0, default_settings()).unwrap();

        assert!(content.contains(r"测试\{字幕\}\\\\路径"));
    }

    #[test]
    fn splits_natural_clauses_into_sequential_cues() {
        let content = build_ass_document(
            "哈喽老婆们，显瘦百搭小裙子，上身温柔显气质，日常出门超合适",
            5.2,
            default_settings(),
        )
        .unwrap();

        assert_eq!(
            split_subtitle_cues("哈喽老婆们，显瘦百搭小裙子，上身温柔显气质，日常出门超合适"),
            vec![
                "哈喽老婆们",
                "显瘦百搭小裙子",
                "上身温柔显气质",
                "日常出门超合适"
            ]
        );
        assert_eq!(
            content
                .lines()
                .filter(|line| line.starts_with("Dialogue:"))
                .count(),
            4
        );
        assert!(!content.contains(r"\N"));
    }

    #[test]
    fn splits_unpunctuated_long_text_into_readable_lengths() {
        let cues =
            split_subtitle_cues("这是一段没有任何标点但是仍然需要自动拆分显示的超长字幕文案");

        assert!(cues.len() >= 2);
        assert!(cues
            .iter()
            .all(|cue| cue.chars().count() <= SUBTITLE_MAX_CHARACTERS));
        assert_eq!(
            cues.concat(),
            "这是一段没有任何标点但是仍然需要自动拆分显示的超长字幕文案"
        );
    }

    #[test]
    fn allocates_full_duration_and_keeps_readable_minimum_when_possible() {
        let cues = build_timed_cues(
            vec!["短句".to_string(), "这是一条更长的字幕".to_string()],
            3.0,
        );

        assert_eq!(cues.first().unwrap().start_seconds, 0.0);
        assert_eq!(cues.last().unwrap().end_seconds, 3.0);
        assert!(cues
            .iter()
            .all(|cue| cue.end_seconds - cue.start_seconds >= 0.7));
        assert_eq!(cues[0].end_seconds, cues[1].start_seconds);
    }

    #[test]
    fn maps_positions_to_ass_alignment() {
        assert_eq!(subtitle_alignment(NarratedSubtitlePosition::Top), 8);
        assert_eq!(subtitle_alignment(NarratedSubtitlePosition::Middle), 5);
        assert_eq!(subtitle_alignment(NarratedSubtitlePosition::Bottom), 2);
    }

    #[test]
    fn shrinks_very_long_large_subtitle() {
        assert!(effective_font_size(NarratedSubtitleSize::Large, 72) < 56);
        assert!(effective_font_size(NarratedSubtitleSize::Large, 72) >= 33);
    }

    #[test]
    fn escapes_windows_filter_path() {
        let filter = build_ass_filter(Path::new(r"C:\Temp Folder\subtitle.ass")).unwrap();

        assert_eq!(filter, r"ass=filename='C\:/Temp Folder/subtitle.ass'");
    }

    #[test]
    fn rejects_empty_subtitle() {
        assert_eq!(
            build_ass_document("   ", 1.0, default_settings()).unwrap_err(),
            "字幕文字不能为空。"
        );
        assert_eq!(
            build_ass_document("，，。！", 1.0, default_settings()).unwrap_err(),
            "字幕文字不能为空。"
        );
    }

    #[test]
    fn disabled_subtitle_does_not_require_text_or_duration() {
        let result = prepare_ass_subtitle(
            Path::new("unused.ass"),
            "",
            0.0,
            NarratedSubtitleSettings {
                enabled: false,
                position: NarratedSubtitlePosition::Bottom,
                size: NarratedSubtitleSize::Medium,
            },
        )
        .unwrap();

        assert!(result.is_none());
        assert!(!Path::new("unused.ass").exists());
    }
}
