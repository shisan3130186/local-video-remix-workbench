export type TaskStatus = "running" | "cancelling" | "completed" | "failed" | "cancelled";

export interface TaskSnapshot {
  taskId: string;
  label: string;
  status: TaskStatus;
  progressPercent: number;
  stage: string;
  message: string | null;
  cancelRequested: boolean;
}

export interface TaskProgressContext {
  taskId: string;
  startPercent: number;
  endPercent: number;
  stage: string;
}

export interface TempCleanupResult {
  removedEntries: number;
  releasedBytes: number;
  message: string;
}

export interface TaskRunHandle {
  taskId: string;
  progress(startPercent: number, endPercent: number, stage: string): TaskProgressContext;
  update(progressPercent: number, stage: string): Promise<void>;
  throwIfCancelled(): void;
}
