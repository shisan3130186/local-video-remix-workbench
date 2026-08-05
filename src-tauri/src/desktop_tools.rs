use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameFileInput {
    pub source_path: String,
    pub target_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameFileResult {
    pub source_path: String,
    pub target_path: String,
}

pub fn list_files_in_folder(folder_path: String) -> Result<Vec<String>, String> {
    let folder = Path::new(folder_path.trim());
    if !folder.is_dir() {
        return Err("请选择有效的文件夹。".to_string());
    }
    let mut files = fs::read_dir(folder)
        .map_err(|error| format!("无法读取文件夹：{error}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .map(|path| path.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    files.sort_by_key(|path| path.to_lowercase());
    Ok(files)
}

pub fn batch_rename_files(items: Vec<RenameFileInput>) -> Result<Vec<RenameFileResult>, String> {
    if items.is_empty() {
        return Err("请先添加需要改名的文件。".to_string());
    }

    let mut targets = HashSet::new();
    let mut prepared = Vec::with_capacity(items.len());
    for item in items {
        let source = PathBuf::from(item.source_path.trim());
        let target = PathBuf::from(item.target_path.trim());
        if !source.is_file() {
            return Err(format!("源文件不存在：{}", source.to_string_lossy()));
        }
        if source == target {
            prepared.push((source, target));
            continue;
        }
        let parent = target
            .parent()
            .ok_or_else(|| "目标文件缺少所在目录。".to_string())?;
        if !parent.is_dir() {
            return Err(format!("目标目录不存在：{}", parent.to_string_lossy()));
        }
        let normalized_target = target.to_string_lossy().to_lowercase();
        if !targets.insert(normalized_target) {
            return Err("改名结果存在重名，请调整规则后重试。".to_string());
        }
        if target.exists() {
            return Err(format!(
                "目标文件已存在，未执行覆盖：{}",
                target.to_string_lossy()
            ));
        }
        prepared.push((source, target));
    }

    let batch_id = Uuid::new_v4();
    let mut staged: Vec<(PathBuf, PathBuf, PathBuf)> = Vec::new();
    for (index, (source, target)) in prepared.iter().enumerate() {
        if source == target {
            continue;
        }
        let extension = source
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| format!(".{value}"))
            .unwrap_or_default();
        let temp = source.with_file_name(format!(".smartcut-rename-{batch_id}-{index}{extension}"));
        if let Err(error) = fs::rename(source, &temp) {
            for (original, _, staged_path) in staged.iter().rev() {
                let _ = fs::rename(staged_path, original);
            }
            return Err(format!("无法准备批量改名：{error}"));
        }
        staged.push((source.clone(), target.clone(), temp));
    }

    let mut completed = Vec::new();
    for (source, target, temp) in &staged {
        if let Err(error) = fs::rename(temp, target) {
            for (done_source, done_target) in completed.iter().rev() {
                let _ = fs::rename(done_target, done_source);
            }
            for (pending_source, _, pending_temp) in &staged {
                if pending_temp.exists() {
                    let _ = fs::rename(pending_temp, pending_source);
                }
            }
            return Err(format!("批量改名执行失败：{error}"));
        }
        completed.push((source.clone(), target.clone()));
    }

    Ok(prepared
        .into_iter()
        .map(|(source, target)| RenameFileResult {
            source_path: source.to_string_lossy().to_string(),
            target_path: target.to_string_lossy().to_string(),
        })
        .collect())
}

pub fn write_utf8_text_file(path: String, content: String) -> Result<(), String> {
    let target = validate_output_file(&path)?;
    fs::write(target, content.as_bytes()).map_err(|error| format!("无法保存文本文件：{error}"))
}

pub fn write_base64_file(path: String, data: String) -> Result<(), String> {
    let target = validate_output_file(&path)?;
    let payload = data
        .split_once(',')
        .map(|(_, value)| value)
        .unwrap_or(data.as_str());
    let bytes = STANDARD
        .decode(payload.trim())
        .map_err(|error| format!("图片数据无法解析：{error}"))?;
    fs::write(target, bytes).map_err(|error| format!("无法保存图片文件：{error}"))
}

fn validate_output_file(path: &str) -> Result<&Path, String> {
    let target = Path::new(path.trim());
    let parent = target
        .parent()
        .ok_or_else(|| "输出文件缺少所在目录。".to_string())?;
    if !parent.is_dir() {
        return Err("输出目录不存在。".to_string());
    }
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_test_directory(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("smartcut-{label}-{}", Uuid::new_v4()))
    }

    #[test]
    fn rejects_empty_rename_list() {
        assert!(batch_rename_files(Vec::new()).is_err());
    }

    #[test]
    fn rejects_invalid_base64() {
        let target = std::env::temp_dir().join(format!("smartcut-{}.png", Uuid::new_v4()));
        let error =
            write_base64_file(target.to_string_lossy().to_string(), "%%%".to_string()).unwrap_err();
        assert!(error.contains("无法解析"));
    }

    #[test]
    fn renames_multiple_files_without_overwriting() {
        let root = temp_test_directory("rename");
        fs::create_dir_all(&root).unwrap();
        let first = root.join("first.txt");
        let second = root.join("second.txt");
        fs::write(&first, b"one").unwrap();
        fs::write(&second, b"two").unwrap();
        let first_target = root.join("sample_001.txt");
        let second_target = root.join("sample_002.txt");

        let result = batch_rename_files(vec![
            RenameFileInput {
                source_path: first.to_string_lossy().to_string(),
                target_path: first_target.to_string_lossy().to_string(),
            },
            RenameFileInput {
                source_path: second.to_string_lossy().to_string(),
                target_path: second_target.to_string_lossy().to_string(),
            },
        ])
        .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(fs::read(&first_target).unwrap(), b"one");
        assert_eq!(fs::read(&second_target).unwrap(), b"two");
        assert!(!first.exists());
        assert!(!second.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuses_to_overwrite_an_existing_target() {
        let root = temp_test_directory("rename-conflict");
        fs::create_dir_all(&root).unwrap();
        let source = root.join("source.txt");
        let target = root.join("target.txt");
        fs::write(&source, b"source").unwrap();
        fs::write(&target, b"keep").unwrap();

        let error = batch_rename_files(vec![RenameFileInput {
            source_path: source.to_string_lossy().to_string(),
            target_path: target.to_string_lossy().to_string(),
        }])
        .unwrap_err();

        assert!(error.contains("未执行覆盖"));
        assert_eq!(fs::read(&source).unwrap(), b"source");
        assert_eq!(fs::read(&target).unwrap(), b"keep");
        fs::remove_dir_all(root).unwrap();
    }
}
