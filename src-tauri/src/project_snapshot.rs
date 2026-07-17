use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const SNAPSHOT_VERSION: u8 = 1;
const CONFIG_DIRECTORY: &str = "com.shisan.local-video-remix-workbench";
const SNAPSHOT_FILE_NAME: &str = "latest-project-snapshot.json";
const MAX_SNAPSHOT_FILE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_TRACKED_PATHS: usize = 10_000;
const MAX_PATH_CHARACTERS: usize = 32_768;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSnapshot {
    version: u8,
    saved_at: String,
    project: Value,
    #[serde(default)]
    file_paths: Vec<String>,
    #[serde(default)]
    directory_paths: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSnapshotLoadResult {
    snapshot: Option<ProjectSnapshot>,
    missing_file_paths: Vec<String>,
    missing_directory_paths: Vec<String>,
}

pub fn load_project_snapshot() -> Result<ProjectSnapshotLoadResult, String> {
    let path = snapshot_file_path()?;
    load_project_snapshot_from_path(&path)
}

pub fn save_project_snapshot(snapshot: ProjectSnapshot) -> Result<(), String> {
    let path = snapshot_file_path()?;
    save_project_snapshot_to_path(&path, &snapshot)
}

pub fn delete_project_snapshot() -> Result<(), String> {
    let path = snapshot_file_path()?;
    delete_project_snapshot_from_path(&path)
}

fn load_project_snapshot_from_path(path: &Path) -> Result<ProjectSnapshotLoadResult, String> {
    if !path.exists() {
        return Ok(ProjectSnapshotLoadResult {
            snapshot: None,
            missing_file_paths: Vec::new(),
            missing_directory_paths: Vec::new(),
        });
    }

    let metadata = fs::metadata(path).map_err(|error| format!("无法读取项目快照：{error}"))?;
    if metadata.len() > MAX_SNAPSHOT_FILE_BYTES {
        return Err("项目快照异常过大，请清除旧快照后重新保存。".to_string());
    }

    let bytes = fs::read(path).map_err(|error| format!("无法读取项目快照：{error}"))?;
    let snapshot = serde_json::from_slice::<ProjectSnapshot>(&bytes)
        .map_err(|error| format!("项目快照无法识别：{error}"))?;
    validate_snapshot(&snapshot)?;

    let missing_file_paths = unique_paths(&snapshot.file_paths)
        .into_iter()
        .filter(|path| !Path::new(path).is_file())
        .collect();
    let missing_directory_paths = unique_paths(&snapshot.directory_paths)
        .into_iter()
        .filter(|path| !Path::new(path).is_dir())
        .collect();

    Ok(ProjectSnapshotLoadResult {
        snapshot: Some(snapshot),
        missing_file_paths,
        missing_directory_paths,
    })
}

fn save_project_snapshot_to_path(path: &Path, snapshot: &ProjectSnapshot) -> Result<(), String> {
    validate_snapshot(snapshot)?;
    let bytes =
        serde_json::to_vec(snapshot).map_err(|error| format!("无法整理项目快照：{error}"))?;
    if bytes.len() as u64 > MAX_SNAPSHOT_FILE_BYTES {
        return Err("项目快照超过2MB限制，请减少当前素材数量后重试。".to_string());
    }

    let parent = path
        .parent()
        .ok_or_else(|| "无法确定项目快照目录。".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("无法创建项目快照目录：{error}"))?;

    let temporary_path = path.with_extension("json.tmp");
    fs::write(&temporary_path, bytes).map_err(|error| format!("无法写入项目快照：{error}"))?;

    if path.exists() {
        fs::remove_file(path).map_err(|error| format!("无法替换旧项目快照：{error}"))?;
    }

    if let Err(error) = fs::rename(&temporary_path, path) {
        let _ = fs::remove_file(&temporary_path);
        return Err(format!("无法完成项目快照保存：{error}"));
    }

    Ok(())
}

fn delete_project_snapshot_from_path(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(path).map_err(|error| format!("无法清除项目快照：{error}"))
}

fn validate_snapshot(snapshot: &ProjectSnapshot) -> Result<(), String> {
    if snapshot.version != SNAPSHOT_VERSION {
        return Err("项目快照版本不兼容，请清除旧快照后重新开始。".to_string());
    }

    if snapshot.saved_at.trim().is_empty() || snapshot.saved_at.chars().count() > 80 {
        return Err("项目快照保存时间无效。".to_string());
    }

    if !snapshot.project.is_object() {
        return Err("项目快照主体格式无效。".to_string());
    }

    if contains_forbidden_secret_field(&snapshot.project) {
        return Err("项目快照包含禁止保存的密钥字段。".to_string());
    }

    validate_paths(&snapshot.file_paths)?;
    validate_paths(&snapshot.directory_paths)
}

fn validate_paths(paths: &[String]) -> Result<(), String> {
    if paths.len() > MAX_TRACKED_PATHS {
        return Err("项目快照记录的路径数量过多。".to_string());
    }

    if paths
        .iter()
        .any(|path| path.trim().is_empty() || path.chars().count() > MAX_PATH_CHARACTERS)
    {
        return Err("项目快照包含无效路径。".to_string());
    }

    Ok(())
}

fn contains_forbidden_secret_field(value: &Value) -> bool {
    match value {
        Value::Object(map) => map.iter().any(|(key, nested)| {
            is_forbidden_secret_key(key) || contains_forbidden_secret_field(nested)
        }),
        Value::Array(items) => items.iter().any(contains_forbidden_secret_field),
        _ => false,
    }
}

fn is_forbidden_secret_key(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().replace(['_', '-'], "").as_str(),
        "apikey" | "aiapikey" | "ttsapikey" | "password" | "credential" | "secret"
    )
}

fn unique_paths(paths: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    paths
        .iter()
        .filter_map(|path| {
            let normalized = path.trim();
            if normalized.is_empty() || !seen.insert(normalized.to_string()) {
                None
            } else {
                Some(normalized.to_string())
            }
        })
        .collect()
}

fn snapshot_file_path() -> Result<PathBuf, String> {
    let app_data = env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "无法找到Windows应用配置目录。".to_string())?;

    Ok(app_data.join(CONFIG_DIRECTORY).join(SNAPSHOT_FILE_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_snapshot(file_path: &Path, directory_path: &Path) -> ProjectSnapshot {
        ProjectSnapshot {
            version: SNAPSHOT_VERSION,
            saved_at: "2026-07-17T14:30:00.000Z".to_string(),
            project: json!({
                "outputDirectory": directory_path.to_string_lossy(),
                "ai": { "script": "测试文案" }
            }),
            file_paths: vec![file_path.to_string_lossy().to_string()],
            directory_paths: vec![directory_path.to_string_lossy().to_string()],
        }
    }

    fn temporary_path(label: &str) -> PathBuf {
        env::temp_dir().join(format!("project-snapshot-{label}-{}", std::process::id()))
    }

    #[test]
    fn saves_loads_and_deletes_project_snapshot() {
        let root = temporary_path("round-trip");
        let snapshot_path = root.join("snapshot.json");
        let material_path = root.join("material.mp4");
        fs::create_dir_all(&root).unwrap();
        fs::write(&material_path, b"video").unwrap();
        let snapshot = sample_snapshot(&material_path, &root);

        save_project_snapshot_to_path(&snapshot_path, &snapshot).unwrap();
        let loaded = load_project_snapshot_from_path(&snapshot_path).unwrap();

        assert!(loaded.snapshot.is_some());
        assert!(loaded.missing_file_paths.is_empty());
        assert!(loaded.missing_directory_paths.is_empty());

        delete_project_snapshot_from_path(&snapshot_path).unwrap();
        assert!(!snapshot_path.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn reports_missing_project_paths() {
        let root = temporary_path("missing");
        let snapshot_path = root.join("snapshot.json");
        fs::create_dir_all(&root).unwrap();
        let missing_file = root.join("missing.mp4");
        let missing_directory = root.join("missing-output");
        let snapshot = sample_snapshot(&missing_file, &missing_directory);

        save_project_snapshot_to_path(&snapshot_path, &snapshot).unwrap();
        let loaded = load_project_snapshot_from_path(&snapshot_path).unwrap();

        assert_eq!(loaded.missing_file_paths.len(), 1);
        assert_eq!(loaded.missing_directory_paths.len(), 1);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_secret_fields_in_project_snapshot() {
        let root = temporary_path("secret");
        let snapshot = ProjectSnapshot {
            version: SNAPSHOT_VERSION,
            saved_at: "2026-07-17T14:30:00.000Z".to_string(),
            project: json!({ "aiApiKey": "must-not-save" }),
            file_paths: vec![root.join("material.mp4").to_string_lossy().to_string()],
            directory_paths: vec![root.to_string_lossy().to_string()],
        };

        assert_eq!(
            validate_snapshot(&snapshot).unwrap_err(),
            "项目快照包含禁止保存的密钥字段。"
        );
    }

    #[test]
    fn rejects_incompatible_snapshot_version() {
        let root = temporary_path("version");
        let mut snapshot = sample_snapshot(&root.join("material.mp4"), &root);
        snapshot.version = 2;

        assert_eq!(
            validate_snapshot(&snapshot).unwrap_err(),
            "项目快照版本不兼容，请清除旧快照后重新开始。"
        );
    }
}
