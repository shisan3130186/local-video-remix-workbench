use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const LIBRARY_VERSION: u8 = 1;
const DATABASE_SCHEMA_VERSION: i64 = 1;
const CONFIG_DIRECTORY: &str = "com.shisan.local-video-remix-workbench";
const DATABASE_FILE_NAME: &str = "material-library.sqlite3";
const MAX_MATERIALS: usize = 10_000;
const MAX_SEGMENTS: usize = 100_000;
const MAX_PATH_CHARACTERS: usize = 32_768;
const MAX_DESCRIPTION_CHARACTERS: usize = 20_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MaterialLibraryVideo {
    id: String,
    file_name: String,
    file_path: String,
    duration_seconds: Option<f64>,
    width: Option<u32>,
    height: Option<u32>,
    frame_rate: Option<f64>,
    has_audio: bool,
    file_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MaterialLibraryMaterial {
    video: MaterialLibraryVideo,
    cover_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MaterialLibrarySegment {
    path: String,
    category: String,
    thumbnail_path: Option<String>,
    segment_id: Option<String>,
    duration_seconds: Option<f64>,
    description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MaterialLibrarySnapshot {
    version: u8,
    saved_at: String,
    materials: Vec<MaterialLibraryMaterial>,
    selected_video_path: Option<String>,
    segment_duration_seconds: f64,
    split_output_directory: Option<String>,
    selected_segment_path: Option<String>,
    cover_frame_seconds: f64,
    segments: Vec<MaterialLibrarySegment>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialLibraryLoadResult {
    snapshot: Option<MaterialLibrarySnapshot>,
    missing_file_paths: Vec<String>,
    database_path: String,
}

pub fn load_material_library() -> Result<MaterialLibraryLoadResult, String> {
    let path = database_file_path()?;
    load_material_library_from_path(&path)
}

pub fn save_material_library(snapshot: MaterialLibrarySnapshot) -> Result<(), String> {
    let path = database_file_path()?;
    save_material_library_to_path(&path, &snapshot)
}

fn load_material_library_from_path(path: &Path) -> Result<MaterialLibraryLoadResult, String> {
    let connection = open_database(path)?;
    let snapshot = read_snapshot(&connection)?;
    let missing_file_paths = snapshot
        .as_ref()
        .map(collect_missing_file_paths)
        .unwrap_or_default();

    Ok(MaterialLibraryLoadResult {
        snapshot,
        missing_file_paths,
        database_path: path.to_string_lossy().to_string(),
    })
}

fn save_material_library_to_path(
    path: &Path,
    snapshot: &MaterialLibrarySnapshot,
) -> Result<(), String> {
    validate_snapshot(snapshot)?;
    let mut connection = open_database(path)?;
    let transaction = connection
        .transaction()
        .map_err(|error| format!("无法开始保存素材库：{error}"))?;

    replace_snapshot(&transaction, snapshot)?;
    transaction
        .commit()
        .map_err(|error| format!("无法完成素材库保存：{error}"))
}

fn open_database(path: &Path) -> Result<Connection, String> {
    let parent = path
        .parent()
        .ok_or_else(|| "无法确定素材库数据库目录。".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("无法创建素材库目录：{error}"))?;

    let connection = Connection::open(path).map_err(|error| format!("无法打开素材库：{error}"))?;
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .map_err(|error| format!("无法启用素材库完整性检查：{error}"))?;
    ensure_schema(&connection)?;
    Ok(connection)
}

fn ensure_schema(connection: &Connection) -> Result<(), String> {
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|error| format!("无法读取素材库版本：{error}"))?;

    match version {
        0 => {
            connection
                .execute_batch(
                    "
                    CREATE TABLE material_library_state (
                        id INTEGER PRIMARY KEY CHECK (id = 1),
                        version INTEGER NOT NULL,
                        saved_at TEXT NOT NULL,
                        selected_video_path TEXT,
                        segment_duration_seconds REAL NOT NULL,
                        split_output_directory TEXT,
                        selected_segment_path TEXT,
                        cover_frame_seconds REAL NOT NULL
                    );

                    CREATE TABLE material_library_materials (
                        file_path TEXT PRIMARY KEY,
                        position INTEGER NOT NULL,
                        material_id TEXT NOT NULL,
                        file_name TEXT NOT NULL,
                        duration_seconds REAL,
                        width INTEGER,
                        height INTEGER,
                        frame_rate REAL,
                        has_audio INTEGER NOT NULL,
                        file_size_bytes INTEGER NOT NULL,
                        cover_path TEXT
                    );

                    CREATE TABLE material_library_segments (
                        path TEXT PRIMARY KEY,
                        position INTEGER NOT NULL,
                        category TEXT NOT NULL,
                        thumbnail_path TEXT,
                        segment_id TEXT,
                        duration_seconds REAL,
                        description TEXT
                    );

                    PRAGMA user_version = 1;
                    ",
                )
                .map_err(|error| format!("无法初始化素材库：{error}"))?;
        }
        DATABASE_SCHEMA_VERSION => {}
        other => {
            return Err(format!("素材库版本 {other} 暂不兼容，请升级软件后重试。"));
        }
    }

    Ok(())
}

fn replace_snapshot(
    transaction: &Transaction<'_>,
    snapshot: &MaterialLibrarySnapshot,
) -> Result<(), String> {
    transaction
        .execute("DELETE FROM material_library_segments", [])
        .map_err(|error| format!("无法更新切片索引：{error}"))?;
    transaction
        .execute("DELETE FROM material_library_materials", [])
        .map_err(|error| format!("无法更新素材索引：{error}"))?;
    transaction
        .execute("DELETE FROM material_library_state", [])
        .map_err(|error| format!("无法更新素材库状态：{error}"))?;

    transaction
        .execute(
            "INSERT INTO material_library_state (
                id, version, saved_at, selected_video_path, segment_duration_seconds,
                split_output_directory, selected_segment_path, cover_frame_seconds
            ) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                snapshot.version,
                snapshot.saved_at,
                snapshot.selected_video_path,
                snapshot.segment_duration_seconds,
                snapshot.split_output_directory,
                snapshot.selected_segment_path,
                snapshot.cover_frame_seconds,
            ],
        )
        .map_err(|error| format!("无法保存素材库状态：{error}"))?;

    for (position, material) in snapshot.materials.iter().enumerate() {
        let video = &material.video;
        transaction
            .execute(
                "INSERT INTO material_library_materials (
                    file_path, position, material_id, file_name, duration_seconds, width,
                    height, frame_rate, has_audio, file_size_bytes, cover_path
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    video.file_path,
                    position as i64,
                    video.id,
                    video.file_name,
                    video.duration_seconds,
                    video.width,
                    video.height,
                    video.frame_rate,
                    video.has_audio,
                    video.file_size_bytes,
                    material.cover_path,
                ],
            )
            .map_err(|error| format!("无法保存素材 {}：{error}", video.file_name))?;
    }

    for (position, segment) in snapshot.segments.iter().enumerate() {
        transaction
            .execute(
                "INSERT INTO material_library_segments (
                    path, position, category, thumbnail_path, segment_id,
                    duration_seconds, description
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    segment.path,
                    position as i64,
                    segment.category,
                    segment.thumbnail_path,
                    segment.segment_id,
                    segment.duration_seconds,
                    segment.description,
                ],
            )
            .map_err(|error| format!("无法保存切片索引：{error}"))?;
    }

    Ok(())
}

fn read_snapshot(connection: &Connection) -> Result<Option<MaterialLibrarySnapshot>, String> {
    let state = connection
        .query_row(
            "SELECT version, saved_at, selected_video_path, segment_duration_seconds,
                    split_output_directory, selected_segment_path, cover_frame_seconds
             FROM material_library_state WHERE id = 1",
            [],
            |row| {
                Ok((
                    row.get::<_, u8>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, f64>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, f64>(6)?,
                ))
            },
        )
        .optional()
        .map_err(|error| format!("无法读取素材库状态：{error}"))?;

    let Some((
        version,
        saved_at,
        selected_video_path,
        segment_duration_seconds,
        split_output_directory,
        selected_segment_path,
        cover_frame_seconds,
    )) = state
    else {
        return Ok(None);
    };

    let mut material_statement = connection
        .prepare(
            "SELECT material_id, file_name, file_path, duration_seconds, width, height,
                    frame_rate, has_audio, file_size_bytes, cover_path
             FROM material_library_materials ORDER BY position",
        )
        .map_err(|error| format!("无法读取素材索引：{error}"))?;
    let materials = material_statement
        .query_map([], |row| {
            Ok(MaterialLibraryMaterial {
                video: MaterialLibraryVideo {
                    id: row.get(0)?,
                    file_name: row.get(1)?,
                    file_path: row.get(2)?,
                    duration_seconds: row.get(3)?,
                    width: row.get(4)?,
                    height: row.get(5)?,
                    frame_rate: row.get(6)?,
                    has_audio: row.get(7)?,
                    file_size_bytes: row.get(8)?,
                },
                cover_path: row.get(9)?,
            })
        })
        .map_err(|error| format!("无法读取素材索引：{error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("无法整理素材索引：{error}"))?;

    let mut segment_statement = connection
        .prepare(
            "SELECT path, category, thumbnail_path, segment_id, duration_seconds, description
             FROM material_library_segments ORDER BY position",
        )
        .map_err(|error| format!("无法读取切片索引：{error}"))?;
    let segments = segment_statement
        .query_map([], |row| {
            Ok(MaterialLibrarySegment {
                path: row.get(0)?,
                category: row.get(1)?,
                thumbnail_path: row.get(2)?,
                segment_id: row.get(3)?,
                duration_seconds: row.get(4)?,
                description: row.get(5)?,
            })
        })
        .map_err(|error| format!("无法读取切片索引：{error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("无法整理切片索引：{error}"))?;

    let snapshot = MaterialLibrarySnapshot {
        version,
        saved_at,
        materials,
        selected_video_path,
        segment_duration_seconds,
        split_output_directory,
        selected_segment_path,
        cover_frame_seconds,
        segments,
    };
    validate_snapshot(&snapshot)?;
    Ok(Some(snapshot))
}

fn collect_missing_file_paths(snapshot: &MaterialLibrarySnapshot) -> Vec<String> {
    let mut seen = HashSet::new();
    snapshot
        .materials
        .iter()
        .flat_map(|material| {
            [
                Some(material.video.file_path.as_str()),
                material.cover_path.as_deref(),
            ]
        })
        .chain(snapshot.segments.iter().flat_map(|segment| {
            [
                Some(segment.path.as_str()),
                segment.thumbnail_path.as_deref(),
            ]
        }))
        .flatten()
        .filter(|path| !Path::new(path).is_file())
        .filter_map(|path| {
            if seen.insert(path.to_string()) {
                Some(path.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn validate_snapshot(snapshot: &MaterialLibrarySnapshot) -> Result<(), String> {
    if snapshot.version != LIBRARY_VERSION {
        return Err("素材库数据版本不兼容，请升级软件后重试。".to_string());
    }
    if snapshot.saved_at.trim().is_empty() || snapshot.saved_at.chars().count() > 80 {
        return Err("素材库保存时间无效。".to_string());
    }
    if snapshot.materials.len() > MAX_MATERIALS || snapshot.segments.len() > MAX_SEGMENTS {
        return Err("素材库记录数量异常过多。".to_string());
    }
    if !snapshot.segment_duration_seconds.is_finite()
        || snapshot.segment_duration_seconds <= 0.0
        || !snapshot.cover_frame_seconds.is_finite()
        || snapshot.cover_frame_seconds < 0.0
    {
        return Err("素材库参数无效。".to_string());
    }

    for material in &snapshot.materials {
        validate_path(&material.video.file_path)?;
        if let Some(path) = material.cover_path.as_deref() {
            validate_path(path)?;
        }
    }
    for segment in &snapshot.segments {
        validate_path(&segment.path)?;
        if let Some(path) = segment.thumbnail_path.as_deref() {
            validate_path(path)?;
        }
        if segment
            .description
            .as_ref()
            .is_some_and(|value| value.chars().count() > MAX_DESCRIPTION_CHARACTERS)
        {
            return Err("素材库中的AI分析文字异常过长。".to_string());
        }
    }

    Ok(())
}

fn validate_path(path: &str) -> Result<(), String> {
    if path.trim().is_empty() || path.chars().count() > MAX_PATH_CHARACTERS {
        return Err("素材库包含无效路径。".to_string());
    }
    Ok(())
}

fn database_file_path() -> Result<PathBuf, String> {
    let app_data = env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "无法找到Windows应用配置目录。".to_string())?;
    Ok(app_data.join(CONFIG_DIRECTORY).join(DATABASE_FILE_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn temporary_database_path() -> PathBuf {
        env::temp_dir().join(format!("material-library-{}.sqlite3", Uuid::new_v4()))
    }

    fn sample_snapshot(video_path: &Path, segment_path: &Path) -> MaterialLibrarySnapshot {
        MaterialLibrarySnapshot {
            version: LIBRARY_VERSION,
            saved_at: "2026-07-17T16:00:00.000Z".to_string(),
            materials: vec![MaterialLibraryMaterial {
                video: MaterialLibraryVideo {
                    id: "video-1".to_string(),
                    file_name: "source.mp4".to_string(),
                    file_path: video_path.to_string_lossy().to_string(),
                    duration_seconds: Some(10.0),
                    width: Some(1920),
                    height: Some(1080),
                    frame_rate: Some(30.0),
                    has_audio: true,
                    file_size_bytes: 1024,
                },
                cover_path: None,
            }],
            selected_video_path: Some(video_path.to_string_lossy().to_string()),
            segment_duration_seconds: 5.0,
            split_output_directory: None,
            selected_segment_path: Some(segment_path.to_string_lossy().to_string()),
            cover_frame_seconds: 1.0,
            segments: vec![MaterialLibrarySegment {
                path: segment_path.to_string_lossy().to_string(),
                category: "hook".to_string(),
                thumbnail_path: None,
                segment_id: Some("segment-001".to_string()),
                duration_seconds: Some(5.0),
                description: Some("人物展示产品".to_string()),
            }],
        }
    }

    #[test]
    fn saves_and_loads_material_library() {
        let database_path = temporary_database_path();
        let video_path = database_path.with_extension("source.mp4");
        let segment_path = database_path.with_extension("segment.mp4");
        fs::write(&video_path, b"video").unwrap();
        fs::write(&segment_path, b"segment").unwrap();
        let snapshot = sample_snapshot(&video_path, &segment_path);

        save_material_library_to_path(&database_path, &snapshot).unwrap();
        let result = load_material_library_from_path(&database_path).unwrap();

        assert_eq!(result.snapshot, Some(snapshot));
        assert!(result.missing_file_paths.is_empty());
        let _ = fs::remove_file(database_path);
        let _ = fs::remove_file(video_path);
        let _ = fs::remove_file(segment_path);
    }

    #[test]
    fn reports_missing_files_without_dropping_records() {
        let database_path = temporary_database_path();
        let video_path = database_path.with_extension("missing.mp4");
        let segment_path = database_path.with_extension("missing-segment.mp4");
        let snapshot = sample_snapshot(&video_path, &segment_path);

        save_material_library_to_path(&database_path, &snapshot).unwrap();
        let result = load_material_library_from_path(&database_path).unwrap();

        assert_eq!(result.snapshot, Some(snapshot));
        assert_eq!(result.missing_file_paths.len(), 2);
        let _ = fs::remove_file(database_path);
    }

    #[test]
    fn second_save_replaces_old_rows() {
        let database_path = temporary_database_path();
        let first_video = database_path.with_extension("first.mp4");
        let first_segment = database_path.with_extension("first-segment.mp4");
        let second_video = database_path.with_extension("second.mp4");
        let second_segment = database_path.with_extension("second-segment.mp4");
        let first = sample_snapshot(&first_video, &first_segment);
        let mut second = sample_snapshot(&second_video, &second_segment);
        second.materials[0].video.id = "video-2".to_string();

        save_material_library_to_path(&database_path, &first).unwrap();
        save_material_library_to_path(&database_path, &second).unwrap();
        let result = load_material_library_from_path(&database_path).unwrap();

        assert_eq!(result.snapshot, Some(second));
        let _ = fs::remove_file(database_path);
    }
}
