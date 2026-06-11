use std::fs;
use std::path::Path;

const SUPPORTED_VIDEO_EXTENSIONS: [&str; 4] = ["mp4", "mov", "avi", "mkv"];

pub fn list_supported_videos_in_folder(folder_path: String) -> Result<Vec<String>, String> {
    let metadata =
        fs::metadata(&folder_path).map_err(|error| format!("无法读取文件夹：{error}"))?;

    if !metadata.is_dir() {
        return Err("选择的路径不是文件夹。".to_string());
    }

    let mut video_paths = Vec::new();
    let entries =
        fs::read_dir(&folder_path).map_err(|error| format!("无法读取文件夹内容：{error}"))?;

    for entry in entries {
        let entry = entry.map_err(|error| format!("读取文件夹项目失败：{error}"))?;
        let path = entry.path();

        if path.is_file() && is_supported_video_file(&path) {
            video_paths.push(path.to_string_lossy().to_string());
        }
    }

    video_paths.sort();
    Ok(video_paths)
}

fn is_supported_video_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            SUPPORTED_VIDEO_EXTENSIONS
                .iter()
                .any(|supported| extension.eq_ignore_ascii_case(supported))
        })
        .unwrap_or(false)
}
