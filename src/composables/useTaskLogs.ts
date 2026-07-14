import { computed, ref } from "vue";
import type {
  ExportResultItem,
  ExportResultType,
  GroupedTaskLogEntry,
  TaskLogEntry,
  TaskLogLevel,
} from "../types/workbench";

function currentTime() {
  return new Date().toLocaleTimeString("zh-CN", { hour12: false });
}

function appendTaskLog(logs: TaskLogEntry[], message: string, level: TaskLogLevel) {
  logs.push({
    id: Date.now() + logs.length,
    time: currentTime(),
    message,
    level,
  });
}

export function useTaskLogs() {
  const exportLogs = ref<TaskLogEntry[]>([]);
  const splitLogs = ref<TaskLogEntry[]>([]);
  const mixLogs = ref<TaskLogEntry[]>([]);
  const batchMixLogs = ref<TaskLogEntry[]>([]);
  const aiRemixLogs = ref<TaskLogEntry[]>([]);
  const exportResultItems = ref<ExportResultItem[]>([]);

  const taskLogs = computed<GroupedTaskLogEntry[]>(() => [
    ...exportLogs.value.map((log) => ({ ...log, group: "基础导出" })),
    ...splitLogs.value.map((log) => ({ ...log, group: "视频切片" })),
    ...mixLogs.value.map((log) => ({ ...log, group: "片段拼接" })),
    ...batchMixLogs.value.map((log) => ({ ...log, group: "批量生成" })),
    ...aiRemixLogs.value.map((log) => ({ ...log, group: "AI 智能混剪" })),
  ]);

  function appendExportLog(message: string, level: TaskLogLevel) {
    appendTaskLog(exportLogs.value, message, level);
  }

  function clearExportLogs() {
    exportLogs.value = [];
  }

  function appendSplitLog(message: string, level: TaskLogLevel) {
    appendTaskLog(splitLogs.value, message, level);
  }

  function clearSplitLogs() {
    splitLogs.value = [];
  }

  function appendMixLog(message: string, level: TaskLogLevel) {
    appendTaskLog(mixLogs.value, message, level);
  }

  function clearMixLogs() {
    mixLogs.value = [];
  }

  function appendBatchMixLog(message: string, level: TaskLogLevel) {
    appendTaskLog(batchMixLogs.value, message, level);
  }

  function clearBatchMixLogs() {
    batchMixLogs.value = [];
  }

  function appendAiRemixLog(message: string, level: TaskLogLevel) {
    appendTaskLog(aiRemixLogs.value, message, level);
  }

  function clearAiRemixLogs() {
    aiRemixLogs.value = [];
  }

  function addExportResult(type: ExportResultType, path: string) {
    exportResultItems.value.unshift({
      id: Date.now() + exportResultItems.value.length,
      type,
      path,
      time: currentTime(),
    });
  }

  return {
    aiRemixLogs,
    appendAiRemixLog,
    appendBatchMixLog,
    appendExportLog,
    appendMixLog,
    appendSplitLog,
    addExportResult,
    batchMixLogs,
    clearAiRemixLogs,
    clearBatchMixLogs,
    clearExportLogs,
    clearMixLogs,
    clearSplitLogs,
    exportLogs,
    exportResultItems,
    mixLogs,
    splitLogs,
    taskLogs,
  };
}
