use serde::Serialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

const TEMP_ROOT_NAME: &str = "local-video-remix-workbench";
const STALE_TEMP_AGE: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TempCleanupResult {
    removed_entries: usize,
    released_bytes: u64,
    message: String,
}

pub struct TaskTempDirectory {
    path: PathBuf,
}

impl TaskTempDirectory {
    pub fn create(category: &str) -> Result<Self, String> {
        Ok(Self {
            path: create_task_temp_directory(category)?,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TaskTempDirectory {
    fn drop(&mut self) {
        remove_task_temp_directory(&self.path);
    }
}

pub fn create_task_temp_directory(category: &str) -> Result<PathBuf, String> {
    validate_category(category)?;
    let directory = temp_root().join(category).join(Uuid::new_v4().to_string());
    fs::create_dir_all(&directory).map_err(|error| format!("无法创建任务临时目录：{error}"))?;
    Ok(directory)
}

pub fn cleanup_workspace_temp_files() -> Result<TempCleanupResult, String> {
    let root = temp_root();
    let (mut removed_entries, mut released_bytes) = cleanup_stale_workspace_root(&root)?;

    let system_temp = env::temp_dir();
    if let Ok(entries) = fs::read_dir(&system_temp) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if (name.starts_with("local-video-remix-tts-")
                || name.starts_with("local-video-remix-narrated-"))
                && is_stale(&entry.path())
            {
                let (count, bytes) = remove_path_if_exists(&entry.path())?;
                removed_entries += count;
                released_bytes += bytes;
            }
        }
    }

    Ok(TempCleanupResult {
        removed_entries,
        released_bytes,
        message: if removed_entries == 0 {
            "没有发现超过24小时的遗留临时文件。".to_string()
        } else {
            format!("已清理 {removed_entries} 个临时项目。")
        },
    })
}

fn cleanup_stale_workspace_root(root: &Path) -> Result<(usize, u64), String> {
    if !root.is_dir() {
        return Ok((0, 0));
    }
    let mut removed_entries = 0;
    let mut released_bytes = 0;
    for category in fs::read_dir(root)
        .map_err(|error| format!("无法读取临时目录：{error}"))?
        .flatten()
    {
        let category_path = category.path();
        if category_path.is_dir() {
            for session in fs::read_dir(&category_path)
                .map_err(|error| format!("无法读取任务临时目录：{error}"))?
                .flatten()
            {
                if is_stale(&session.path()) {
                    let (count, bytes) = remove_path_if_exists(&session.path())?;
                    removed_entries += count;
                    released_bytes += bytes;
                }
            }
            remove_if_empty(&category_path);
        } else if is_stale(&category_path) {
            let (count, bytes) = remove_path_if_exists(&category_path)?;
            removed_entries += count;
            released_bytes += bytes;
        }
    }
    remove_if_empty(root);
    Ok((removed_entries, released_bytes))
}

fn is_stale(path: &Path) -> bool {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .is_some_and(|age| age >= STALE_TEMP_AGE)
}

fn remove_if_empty(path: &Path) {
    if fs::read_dir(path)
        .ok()
        .is_some_and(|mut entries| entries.next().is_none())
    {
        let _ = fs::remove_dir(path);
    }
}

pub fn remove_task_temp_directory(path: &Path) {
    if path.starts_with(temp_root()) {
        let _ = fs::remove_dir_all(path);
    }
}

pub fn temp_root() -> PathBuf {
    env::temp_dir().join(TEMP_ROOT_NAME)
}

fn validate_category(category: &str) -> Result<(), String> {
    if category.is_empty()
        || !category
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
    {
        return Err("临时目录分类无效。".to_string());
    }
    Ok(())
}

fn remove_path_if_exists(path: &Path) -> Result<(usize, u64), String> {
    if !path.exists() {
        return Ok((0, 0));
    }
    let size = calculate_path_size(path);
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|error| format!("无法清理临时目录：{error}"))?;
    } else {
        fs::remove_file(path).map_err(|error| format!("无法清理临时文件：{error}"))?;
    }
    Ok((1, size))
}

fn calculate_path_size(path: &Path) -> u64 {
    if let Ok(metadata) = fs::metadata(path) {
        if metadata.is_file() {
            return metadata.len();
        }
    }
    fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| calculate_path_size(&entry.path()))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::{
        cleanup_workspace_temp_files, create_task_temp_directory, remove_task_temp_directory,
        temp_root,
    };

    #[test]
    fn task_temp_directory_stays_inside_workspace_root() {
        let directory = create_task_temp_directory("test").expect("create temp directory");
        assert!(directory.starts_with(temp_root()));
        remove_task_temp_directory(&directory);
        assert!(!directory.exists());
    }

    #[test]
    fn cleanup_does_not_remove_fresh_active_directory() {
        let directory = create_task_temp_directory("test-active").expect("create temp directory");
        cleanup_workspace_temp_files().expect("cleanup temp files");
        assert!(directory.exists());
        remove_task_temp_directory(&directory);
    }
}
