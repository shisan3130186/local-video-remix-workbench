import { computed, onUnmounted, ref } from "vue";
import {
  cancelTask,
  cleanupTempFiles,
  createTask,
  finishTask,
  getTaskProgress,
  updateTaskProgress,
} from "./services/taskCenterService";
import type {
  TaskProgressContext,
  TaskRunHandle,
  TaskSnapshot,
  TempCleanupResult,
} from "./types";

const TASK_CANCELLED_MESSAGE = "任务已取消。";

export class TaskCancelledError extends Error {
  constructor() {
    super(TASK_CANCELLED_MESSAGE);
    this.name = "TaskCancelledError";
  }
}

export function isTaskCancelledError(error: unknown) {
  return (
    error instanceof TaskCancelledError ||
    (error instanceof Error && error.message.includes("任务已取消")) ||
    String(error ?? "").includes("任务已取消")
  );
}

export function useTaskCenter() {
  const activeTask = ref<TaskSnapshot | null>(null);
  const cleanupResult = ref<TempCleanupResult | null>(null);
  const cleanupError = ref<string | null>(null);
  const isCleaning = ref(false);
  let pollingTimer: number | null = null;
  let pollingGeneration = 0;

  const isTaskRunning = computed(
    () => activeTask.value?.status === "running" || activeTask.value?.status === "cancelling",
  );
  const isCancelling = computed(() => activeTask.value?.status === "cancelling");

  async function runTask<T>(
    label: string,
    runner: (handle: TaskRunHandle) => Promise<T>,
  ): Promise<T> {
    if (isTaskRunning.value) {
      throw new Error("当前已有任务正在执行，请等待结束或先取消。 ");
    }

    const taskId = createTaskId();
    activeTask.value = await createTask(taskId, label);
    startPolling(taskId);
    const handle: TaskRunHandle = {
      taskId,
      progress(startPercent, endPercent, stage): TaskProgressContext {
        return { taskId, startPercent, endPercent, stage };
      },
      async update(progressPercent, stage) {
        activeTask.value = await updateTaskProgress(taskId, progressPercent, stage);
      },
      throwIfCancelled() {
        if (activeTask.value?.taskId === taskId && activeTask.value.cancelRequested) {
          throw new TaskCancelledError();
        }
      },
    };

    try {
      const result = await runner(handle);
      handle.throwIfCancelled();
      activeTask.value = await finishTask(taskId, "completed");
      return result;
    } catch (error) {
      if (isTaskCancelledError(error) || activeTask.value?.cancelRequested) {
        activeTask.value = await finishTask(taskId, "cancelled", TASK_CANCELLED_MESSAGE);
        throw new TaskCancelledError();
      }
      const message = formatError(error, "任务执行失败。 ");
      activeTask.value = await finishTask(taskId, "failed", message);
      throw error;
    } finally {
      stopPolling();
    }
  }

  async function cancelActiveTask() {
    if (!activeTask.value || !isTaskRunning.value || isCancelling.value) {
      return;
    }
    activeTask.value = {
      ...activeTask.value,
      status: "cancelling",
      cancelRequested: true,
      stage: "正在安全取消任务...",
    };
    activeTask.value = await cancelTask(activeTask.value.taskId);
  }

  async function cleanupStaleTempFiles() {
    if (isTaskRunning.value) {
      return null;
    }
    isCleaning.value = true;
    cleanupError.value = null;
    try {
      cleanupResult.value = await cleanupTempFiles();
      return cleanupResult.value;
    } catch (error) {
      cleanupError.value = formatError(error, "临时文件清理失败。 ");
      return null;
    } finally {
      isCleaning.value = false;
    }
  }

  function startPolling(taskId: string) {
    stopPolling();
    const generation = pollingGeneration;
    pollingTimer = window.setInterval(async () => {
      try {
        const snapshot = await getTaskProgress(taskId);
        if (generation === pollingGeneration && activeTask.value?.taskId === taskId) {
          activeTask.value = snapshot;
        }
      } catch {
        // 单次轮询失败不影响实际任务，下一轮继续同步。
      }
    }, 200);
  }

  function stopPolling() {
    pollingGeneration += 1;
    if (pollingTimer !== null) {
      window.clearInterval(pollingTimer);
      pollingTimer = null;
    }
  }

  onUnmounted(stopPolling);

  return {
    activeTask,
    cancelActiveTask,
    cleanupError,
    cleanupResult,
    cleanupStaleTempFiles,
    isCancelling,
    isCleaning,
    isTaskRunning,
    runTask,
  };
}

function createTaskId() {
  return `task-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 12)}`;
}

function formatError(error: unknown, fallback: string) {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error ?? "").trim() || fallback;
}
