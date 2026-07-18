use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const LIBRARY_VERSION: u8 = 1;
const CONFIG_DIRECTORY: &str = "com.shisan.local-video-remix-workbench";
const LIBRARY_FILE_NAME: &str = "script-library-v1.json";
const MAX_LIBRARY_FILE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_ENTRIES: usize = 500;
const MAX_TITLE_CHARACTERS: usize = 200;
const MAX_TEXT_CHARACTERS: usize = 100_000;
const MAX_PATH_CHARACTERS: usize = 32_768;
const MAX_UTTERANCES: usize = 20_000;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScriptLibraryUtterance {
    start_time_ms: i64,
    end_time_ms: i64,
    text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScriptLibraryEntry {
    id: String,
    title: String,
    text: String,
    source_path: Option<String>,
    source_file_name: Option<String>,
    duration_seconds: Option<f64>,
    #[serde(default)]
    utterances: Vec<ScriptLibraryUtterance>,
    created_at_ms: u128,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveScriptLibraryEntryInput {
    title: String,
    text: String,
    source_path: Option<String>,
    source_file_name: Option<String>,
    duration_seconds: Option<f64>,
    #[serde(default)]
    utterances: Vec<ScriptLibraryUtterance>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveScriptLibraryEntryResult {
    entry: ScriptLibraryEntry,
    created: bool,
    message: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredScriptLibrary {
    version: u8,
    #[serde(default)]
    entries: Vec<ScriptLibraryEntry>,
}

pub fn load_script_library() -> Result<Vec<ScriptLibraryEntry>, String> {
    load_library_from_path(&library_file_path()?)
}

pub fn save_script_library_entry(
    input: SaveScriptLibraryEntryInput,
) -> Result<SaveScriptLibraryEntryResult, String> {
    let path = library_file_path()?;
    save_entry_to_path(&path, input)
}

pub fn delete_script_library_entry(id: String) -> Result<Vec<ScriptLibraryEntry>, String> {
    let path = library_file_path()?;
    delete_entry_from_path(&path, &id)
}

fn delete_entry_from_path(path: &Path, id: &str) -> Result<Vec<ScriptLibraryEntry>, String> {
    validate_id(id)?;
    let mut entries = load_library_from_path(path)?;
    let old_length = entries.len();
    entries.retain(|entry| entry.id != id);
    if entries.len() == old_length {
        return Err("没有找到要删除的文案。".to_string());
    }
    save_library_to_path(path, &entries)?;
    Ok(entries)
}

fn save_entry_to_path(
    path: &Path,
    input: SaveScriptLibraryEntryInput,
) -> Result<SaveScriptLibraryEntryResult, String> {
    let normalized = normalize_input(input)?;
    let mut entries = load_library_from_path(path)?;
    if let Some(existing) = entries
        .iter()
        .find(|entry| entry.text == normalized.text && entry.source_path == normalized.source_path)
    {
        return Ok(SaveScriptLibraryEntryResult {
            entry: existing.clone(),
            created: false,
            message: "这条识别文案已经保存在文案库中。".to_string(),
        });
    }

    let entry = ScriptLibraryEntry {
        id: Uuid::new_v4().to_string(),
        title: normalized.title,
        text: normalized.text,
        source_path: normalized.source_path,
        source_file_name: normalized.source_file_name,
        duration_seconds: normalized.duration_seconds,
        utterances: normalized.utterances,
        created_at_ms: current_timestamp_millis()?,
    };
    entries.insert(0, entry.clone());
    entries.truncate(MAX_ENTRIES);
    save_library_to_path(path, &entries)?;

    Ok(SaveScriptLibraryEntryResult {
        entry,
        created: true,
        message: "识别文字已保存到本地文案库。".to_string(),
    })
}

fn normalize_input(
    input: SaveScriptLibraryEntryInput,
) -> Result<SaveScriptLibraryEntryInput, String> {
    let title = input.title.trim();
    let text = input.text.trim();
    if title.is_empty() {
        return Err("文案标题不能为空。".to_string());
    }
    if title.chars().count() > MAX_TITLE_CHARACTERS {
        return Err("文案标题过长，请缩短后再保存。".to_string());
    }
    if text.is_empty() {
        return Err("文案内容不能为空。".to_string());
    }
    if text.chars().count() > MAX_TEXT_CHARACTERS {
        return Err("识别文字过长，暂时无法保存到文案库。".to_string());
    }
    if input.utterances.len() > MAX_UTTERANCES {
        return Err("句子时间轴数量过多，暂时无法保存。".to_string());
    }
    validate_optional_path(input.source_path.as_deref())?;
    if input
        .duration_seconds
        .is_some_and(|value| !value.is_finite() || value < 0.0)
    {
        return Err("音视频时长无效。".to_string());
    }
    for utterance in &input.utterances {
        if utterance.start_time_ms < 0
            || utterance.end_time_ms < utterance.start_time_ms
            || utterance.text.trim().is_empty()
        {
            return Err("句子时间轴数据无效。".to_string());
        }
    }

    Ok(SaveScriptLibraryEntryInput {
        title: title.to_string(),
        text: text.to_string(),
        source_path: normalize_optional_text(input.source_path),
        source_file_name: normalize_optional_text(input.source_file_name),
        duration_seconds: input.duration_seconds,
        utterances: input.utterances,
    })
}

fn load_library_from_path(path: &Path) -> Result<Vec<ScriptLibraryEntry>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let metadata = fs::metadata(path).map_err(|error| format!("无法读取文案库：{error}"))?;
    if metadata.len() > MAX_LIBRARY_FILE_BYTES {
        return Err("文案库文件异常过大，请备份后清理。".to_string());
    }
    let bytes = fs::read(path).map_err(|error| format!("无法读取文案库：{error}"))?;
    let stored = serde_json::from_slice::<StoredScriptLibrary>(&bytes)
        .map_err(|error| format!("文案库文件无法识别：{error}"))?;
    if stored.version != LIBRARY_VERSION {
        return Err("文案库版本不兼容。".to_string());
    }
    validate_entries(&stored.entries)?;
    Ok(stored.entries)
}

fn save_library_to_path(path: &Path, entries: &[ScriptLibraryEntry]) -> Result<(), String> {
    validate_entries(entries)?;
    let stored = StoredScriptLibrary {
        version: LIBRARY_VERSION,
        entries: entries.to_vec(),
    };
    let bytes = serde_json::to_vec(&stored).map_err(|error| format!("无法整理文案库：{error}"))?;
    if bytes.len() as u64 > MAX_LIBRARY_FILE_BYTES {
        return Err("文案库已达到容量上限，请删除不需要的旧文案。".to_string());
    }
    let parent = path
        .parent()
        .ok_or_else(|| "无法确定文案库保存目录。".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("无法创建文案库目录：{error}"))?;
    let temp_path = path.with_extension(format!("{}.tmp", Uuid::new_v4()));
    fs::write(&temp_path, bytes).map_err(|error| format!("无法写入文案库：{error}"))?;
    if path.exists() {
        fs::remove_file(path).map_err(|error| format!("无法更新旧文案库：{error}"))?;
    }
    fs::rename(temp_path, path).map_err(|error| format!("无法完成文案库保存：{error}"))
}

fn validate_entries(entries: &[ScriptLibraryEntry]) -> Result<(), String> {
    if entries.len() > MAX_ENTRIES {
        return Err("文案库记录数量异常。".to_string());
    }
    for entry in entries {
        validate_id(&entry.id)?;
        normalize_input(SaveScriptLibraryEntryInput {
            title: entry.title.clone(),
            text: entry.text.clone(),
            source_path: entry.source_path.clone(),
            source_file_name: entry.source_file_name.clone(),
            duration_seconds: entry.duration_seconds,
            utterances: entry.utterances.clone(),
        })?;
    }
    Ok(())
}

fn validate_id(id: &str) -> Result<(), String> {
    Uuid::parse_str(id)
        .map(|_| ())
        .map_err(|_| "文案编号无效。".to_string())
}

fn validate_optional_path(path: Option<&str>) -> Result<(), String> {
    if path.is_some_and(|value| value.chars().count() > MAX_PATH_CHARACTERS) {
        return Err("文案来源路径过长。".to_string());
    }
    Ok(())
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let normalized = value.trim();
        (!normalized.is_empty()).then(|| normalized.to_string())
    })
}

fn current_timestamp_millis() -> Result<u128, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .map_err(|error| format!("无法生成文案保存时间：{error}"))
}

fn library_file_path() -> Result<PathBuf, String> {
    let app_data = env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "无法找到Windows应用配置目录。".to_string())?;
    Ok(app_data.join(CONFIG_DIRECTORY).join(LIBRARY_FILE_NAME))
}

#[cfg(test)]
mod tests {
    use super::{
        delete_entry_from_path, load_library_from_path, save_entry_to_path,
        SaveScriptLibraryEntryInput, ScriptLibraryUtterance,
    };
    use std::fs;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn test_path() -> PathBuf {
        std::env::temp_dir().join(format!("script-library-test-{}.json", Uuid::new_v4()))
    }

    fn input() -> SaveScriptLibraryEntryInput {
        SaveScriptLibraryEntryInput {
            title: "测试文案".to_string(),
            text: "这是识别得到的文案。".to_string(),
            source_path: Some("C:/video/test.mp4".to_string()),
            source_file_name: Some("test.mp4".to_string()),
            duration_seconds: Some(12.5),
            utterances: vec![ScriptLibraryUtterance {
                start_time_ms: 0,
                end_time_ms: 1200,
                text: "这是识别得到的文案。".to_string(),
            }],
        }
    }

    #[test]
    fn saves_loads_and_deduplicates_script_entries() {
        let path = test_path();
        let first = save_entry_to_path(&path, input()).unwrap();
        let second = save_entry_to_path(&path, input()).unwrap();
        assert!(first.created);
        assert!(!second.created);
        assert_eq!(load_library_from_path(&path).unwrap().len(), 1);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn deletes_saved_script_entry() {
        let path = test_path();
        let saved = save_entry_to_path(&path, input()).unwrap();
        let entries = delete_entry_from_path(&path, &saved.entry.id).unwrap();
        assert!(entries.is_empty());
        let _ = fs::remove_file(path);
    }
}
