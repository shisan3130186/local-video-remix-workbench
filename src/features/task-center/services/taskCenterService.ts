import { invoke } from "@tauri-apps/api/core";
import type { TaskSnapshot, TaskStatus, TempCleanupResult } from "../types";

export function createTask(taskId: string, label: string): Promise<TaskSnapshot> {
  return invoke<TaskSnapshot>("create_task", { taskId, label });
}

export function getTaskProgress(taskId: string): Promise<TaskSnapshot> {
  return invoke<TaskSnapshot>("get_task_progress", { taskId });
}

export function updateTaskProgress(
  taskId: string,
  progressPercent: number,
  stage: string,
): Promise<TaskSnapshot> {
  return invoke<TaskSnapshot>("update_task_progress", {
    taskId,
    progressPercent,
    stage,
  });
}

export function cancelTask(taskId: string): Promise<TaskSnapshot> {
  return invoke<TaskSnapshot>("cancel_task", { taskId });
}

export function finishTask(
  taskId: string,
  status: Extract<TaskStatus, "completed" | "failed" | "cancelled">,
  message?: string,
): Promise<TaskSnapshot> {
  return invoke<TaskSnapshot>("finish_task", {
    taskId,
    status,
    message: message ?? null,
  });
}

export function cleanupTempFiles(): Promise<TempCleanupResult> {
  return invoke<TempCleanupResult>("cleanup_temp_files");
}
