use crate::video_engine::tool_paths::ffmpeg_program;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

pub const TASK_CANCELLED_MESSAGE: &str = "任务已取消。";

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TaskStatus {
    Running,
    Cancelling,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSnapshot {
    task_id: String,
    label: String,
    status: TaskStatus,
    progress_percent: f64,
    stage: String,
    message: Option<String>,
    cancel_requested: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskProgressContext {
    pub task_id: String,
    pub start_percent: f64,
    pub end_percent: f64,
    pub stage: String,
}

impl TaskProgressContext {
    pub fn child(&self, start_ratio: f64, end_ratio: f64, stage: impl Into<String>) -> Self {
        let width = self.end_percent - self.start_percent;
        Self {
            task_id: self.task_id.clone(),
            start_percent: self.start_percent + width * start_ratio.clamp(0.0, 1.0),
            end_percent: self.start_percent + width * end_ratio.clamp(0.0, 1.0),
            stage: stage.into(),
        }
    }
}

struct TaskEntry {
    label: String,
    status: TaskStatus,
    progress_percent: f64,
    stage: String,
    message: Option<String>,
    cancel_requested: Arc<AtomicBool>,
}

static TASKS: OnceLock<Mutex<HashMap<String, TaskEntry>>> = OnceLock::new();

fn tasks() -> &'static Mutex<HashMap<String, TaskEntry>> {
    TASKS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn create_task(task_id: String, label: String) -> Result<TaskSnapshot, String> {
    validate_task_id(&task_id)?;
    let entry = TaskEntry {
        label,
        status: TaskStatus::Running,
        progress_percent: 0.0,
        stage: "正在准备任务...".to_string(),
        message: None,
        cancel_requested: Arc::new(AtomicBool::new(false)),
    };
    let mut registry = tasks()
        .lock()
        .map_err(|_| "任务中心暂时不可用。".to_string())?;
    registry.insert(task_id.clone(), entry);
    snapshot_from_registry(&registry, &task_id)
}

pub fn get_task(task_id: &str) -> Result<TaskSnapshot, String> {
    let registry = tasks()
        .lock()
        .map_err(|_| "任务中心暂时不可用。".to_string())?;
    snapshot_from_registry(&registry, task_id)
}

pub fn update_task(
    task_id: &str,
    progress_percent: f64,
    stage: impl Into<String>,
) -> Result<TaskSnapshot, String> {
    let mut registry = tasks()
        .lock()
        .map_err(|_| "任务中心暂时不可用。".to_string())?;
    let entry = registry
        .get_mut(task_id)
        .ok_or_else(|| "没有找到当前任务。".to_string())?;
    if matches!(entry.status, TaskStatus::Running | TaskStatus::Cancelling) {
        entry.progress_percent = progress_percent.clamp(0.0, 100.0);
        entry.stage = stage.into();
    }
    snapshot_from_registry(&registry, task_id)
}

pub fn cancel_task(task_id: &str) -> Result<TaskSnapshot, String> {
    let mut registry = tasks()
        .lock()
        .map_err(|_| "任务中心暂时不可用。".to_string())?;
    let entry = registry
        .get_mut(task_id)
        .ok_or_else(|| "没有找到当前任务。".to_string())?;
    if entry.status == TaskStatus::Running {
        entry.status = TaskStatus::Cancelling;
        entry.stage = "正在安全取消任务...".to_string();
        entry.cancel_requested.store(true, Ordering::SeqCst);
    }
    snapshot_from_registry(&registry, task_id)
}

pub fn finish_task(
    task_id: &str,
    status: TaskStatus,
    message: Option<String>,
) -> Result<TaskSnapshot, String> {
    if matches!(status, TaskStatus::Running | TaskStatus::Cancelling) {
        return Err("结束任务时状态无效。".to_string());
    }
    let mut registry = tasks()
        .lock()
        .map_err(|_| "任务中心暂时不可用。".to_string())?;
    let entry = registry
        .get_mut(task_id)
        .ok_or_else(|| "没有找到当前任务。".to_string())?;
    entry.status = status;
    entry.message = message;
    match status {
        TaskStatus::Completed => {
            entry.progress_percent = 100.0;
            entry.stage = "任务已完成".to_string();
        }
        TaskStatus::Cancelled => entry.stage = "任务已取消".to_string(),
        TaskStatus::Failed => entry.stage = "任务失败".to_string(),
        TaskStatus::Running | TaskStatus::Cancelling => {}
    }
    snapshot_from_registry(&registry, task_id)
}

pub fn is_cancel_requested(task_id: &str) -> bool {
    tasks()
        .lock()
        .ok()
        .and_then(|registry| {
            registry
                .get(task_id)
                .map(|entry| entry.cancel_requested.clone())
        })
        .map(|flag| flag.load(Ordering::SeqCst))
        .unwrap_or(false)
}

pub fn ensure_not_cancelled(task_id: &str) -> Result<(), String> {
    if is_cancel_requested(task_id) {
        Err(TASK_CANCELLED_MESSAGE.to_string())
    } else {
        Ok(())
    }
}

pub fn run_ffmpeg(
    args: Vec<String>,
    context: Option<&TaskProgressContext>,
    duration_seconds: Option<f64>,
    fallback_error: &str,
) -> Result<(), String> {
    if let Some(context) = context {
        ensure_not_cancelled(&context.task_id)?;
        let _ = update_task(
            &context.task_id,
            context.start_percent,
            context.stage.clone(),
        );
    }

    let mut command = Command::new(ffmpeg_program());
    command
        .args(["-progress", "pipe:1", "-nostats"])
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|error| format!("无法调用 ffmpeg：{error}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "无法读取FFmpeg进度。".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "无法读取FFmpeg错误信息。".to_string())?;
    let (progress_tx, progress_rx) = mpsc::channel::<String>();
    let progress_thread = thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            let _ = progress_tx.send(line);
        }
    });
    let stderr_thread = thread::spawn(move || {
        let mut reader = BufReader::new(stderr);
        let mut buffer = String::new();
        let _ = reader.read_to_string(&mut buffer);
        buffer
    });

    let status = loop {
        while let Ok(line) = progress_rx.try_recv() {
            if let (Some(context), Some(duration)) = (context, duration_seconds) {
                if let Some(elapsed) = parse_ffmpeg_elapsed_seconds(&line) {
                    let ratio = if duration > 0.0 {
                        (elapsed / duration).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    let progress = context.start_percent
                        + (context.end_percent - context.start_percent) * ratio;
                    let _ = update_task(&context.task_id, progress, context.stage.clone());
                }
            }
        }

        if let Some(context) = context {
            if is_cancel_requested(&context.task_id) {
                let _ = child.kill();
                let _ = child.wait();
                let _ = progress_thread.join();
                let _ = stderr_thread.join();
                return Err(TASK_CANCELLED_MESSAGE.to_string());
            }
        }

        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => thread::sleep(Duration::from_millis(50)),
            Err(error) => return Err(format!("等待FFmpeg任务结束失败：{error}")),
        }
    };

    let _ = progress_thread.join();
    let stderr = stderr_thread.join().unwrap_or_default();
    if !status.success() {
        let detail = stderr.trim();
        return Err(if detail.is_empty() {
            fallback_error.to_string()
        } else {
            detail.to_string()
        });
    }

    if let Some(context) = context {
        let _ = update_task(&context.task_id, context.end_percent, context.stage.clone());
    }
    Ok(())
}

fn parse_ffmpeg_elapsed_seconds(line: &str) -> Option<f64> {
    if let Some(value) = line.strip_prefix("out_time_us=") {
        return value
            .trim()
            .parse::<f64>()
            .ok()
            .map(|value| value / 1_000_000.0);
    }
    if let Some(value) = line.strip_prefix("out_time_ms=") {
        return value
            .trim()
            .parse::<f64>()
            .ok()
            .map(|value| value / 1_000_000.0);
    }
    None
}

fn snapshot_from_registry(
    registry: &HashMap<String, TaskEntry>,
    task_id: &str,
) -> Result<TaskSnapshot, String> {
    let entry = registry
        .get(task_id)
        .ok_or_else(|| "没有找到当前任务。".to_string())?;
    Ok(TaskSnapshot {
        task_id: task_id.to_string(),
        label: entry.label.clone(),
        status: entry.status,
        progress_percent: entry.progress_percent,
        stage: entry.stage.clone(),
        message: entry.message.clone(),
        cancel_requested: entry.cancel_requested.load(Ordering::SeqCst),
    })
}

fn validate_task_id(task_id: &str) -> Result<(), String> {
    if task_id.len() < 8
        || task_id.len() > 80
        || !task_id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err("任务编号无效。".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        cancel_task, create_task, finish_task, parse_ffmpeg_elapsed_seconds, update_task,
        TaskProgressContext, TaskStatus,
    };
    use uuid::Uuid;

    #[test]
    fn parses_ffmpeg_microsecond_progress() {
        assert_eq!(
            parse_ffmpeg_elapsed_seconds("out_time_us=2500000"),
            Some(2.5)
        );
        assert_eq!(parse_ffmpeg_elapsed_seconds("progress=continue"), None);
    }

    #[test]
    fn creates_nested_progress_ranges() {
        let parent = TaskProgressContext {
            task_id: "task-12345678".to_string(),
            start_percent: 20.0,
            end_percent: 80.0,
            stage: "父任务".to_string(),
        };
        let child = parent.child(0.25, 0.75, "子任务");
        assert_eq!(child.start_percent, 35.0);
        assert_eq!(child.end_percent, 65.0);
    }

    #[test]
    fn tracks_task_progress_cancellation_and_final_state() {
        let task_id = format!("task-{}", Uuid::new_v4());
        let created = create_task(task_id.clone(), "测试任务".to_string()).unwrap();
        assert_eq!(created.status, TaskStatus::Running);

        let updated = update_task(&task_id, 42.5, "正在处理").unwrap();
        assert_eq!(updated.progress_percent, 42.5);
        assert_eq!(updated.stage, "正在处理");

        let cancelling = cancel_task(&task_id).unwrap();
        assert_eq!(cancelling.status, TaskStatus::Cancelling);
        assert!(cancelling.cancel_requested);

        let cancelled = finish_task(
            &task_id,
            TaskStatus::Cancelled,
            Some("任务已取消。".to_string()),
        )
        .unwrap();
        assert_eq!(cancelled.status, TaskStatus::Cancelled);
        assert_eq!(cancelled.message.as_deref(), Some("任务已取消。"));
    }
}
