use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub fn ffmpeg_program() -> PathBuf {
    resolve_program("FFMPEG_PATH", "ffmpeg")
}

pub fn ffprobe_program() -> PathBuf {
    resolve_program("FFPROBE_PATH", "ffprobe")
}

fn resolve_program(environment_name: &str, binary_name: &str) -> PathBuf {
    resolve_program_from(
        env::var_os(environment_name),
        env::current_exe().ok(),
        env::current_dir().ok(),
        binary_name,
    )
}

fn resolve_program_from(
    explicit_path: Option<OsString>,
    current_executable: Option<PathBuf>,
    current_directory: Option<PathBuf>,
    binary_name: &str,
) -> PathBuf {
    if let Some(path) = explicit_path.filter(|path| !path.is_empty()) {
        let path = PathBuf::from(path);
        return if path.is_dir() {
            path.join(tool_file_name(binary_name))
        } else {
            path
        };
    }

    let file_name = tool_file_name(binary_name);
    let mut candidates = Vec::new();

    if let Some(executable_directory) = current_executable.as_deref().and_then(Path::parent) {
        candidates.push(
            executable_directory
                .join("resources")
                .join("ffmpeg")
                .join(&file_name),
        );
        candidates.push(executable_directory.join("ffmpeg").join(&file_name));
        candidates.push(executable_directory.join(&file_name));
    }

    if let Some(current_directory) = current_directory {
        candidates.push(
            current_directory
                .join("src-tauri")
                .join("resources")
                .join("ffmpeg")
                .join(&file_name),
        );
        candidates.push(
            current_directory
                .join("resources")
                .join("ffmpeg")
                .join(&file_name),
        );
    }

    candidates
        .into_iter()
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| PathBuf::from(binary_name))
}

fn tool_file_name(binary_name: &str) -> OsString {
    if cfg!(windows) {
        OsString::from(format!("{binary_name}.exe"))
    } else {
        OsString::from(binary_name)
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_program_from;
    use std::ffi::OsString;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn explicit_tool_path_has_highest_priority() {
        let expected = PathBuf::from(r"C:\Tools\ffmpeg\ffmpeg.exe");
        let actual = resolve_program_from(
            Some(OsString::from(expected.as_os_str())),
            None,
            None,
            "ffmpeg",
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn finds_bundled_tool_in_resources_directory() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("video-remix-tool-path-{unique}"));
        let executable = root.join("app.exe");
        let tool_name = if cfg!(windows) {
            "ffmpeg.exe"
        } else {
            "ffmpeg"
        };
        let bundled_tool = root.join("resources").join("ffmpeg").join(tool_name);
        fs::create_dir_all(bundled_tool.parent().unwrap()).unwrap();
        fs::write(&bundled_tool, []).unwrap();

        let actual = resolve_program_from(None, Some(executable), None, "ffmpeg");

        assert_eq!(actual, bundled_tool);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn falls_back_to_system_path_name() {
        let actual = resolve_program_from(None, None, None, "ffprobe");
        assert_eq!(actual, PathBuf::from("ffprobe"));
    }
}
