import { open } from "@tauri-apps/plugin-dialog";
import { computed, ref } from "vue";
import { runRetryableRequest } from "../../services/retryableRequest";
import { isTaskCancelledError } from "../task-center";
import type { TaskRunHandle } from "../task-center";
import { getAsrConfigStatus, recognizeSpeech } from "./services/asrService";
import type { AsrConfigStatus, AsrRecognitionResult } from "./types";

interface UseAsrOptions {
  appendLog: (message: string, level: "info" | "success" | "error") => void;
  clearLogs: () => void;
  runTask: <T>(label: string, runner: (task: TaskRunHandle) => Promise<T>) => Promise<T>;
}

const DEFAULT_CONFIG: AsrConfigStatus = {
  configured: false,
  resourceId: "volc.bigasr.auc_turbo",
};

export function useAsr(options: UseAsrOptions) {
  const config = ref<AsrConfigStatus>({ ...DEFAULT_CONFIG });
  const sourceFilePath = ref<string | null>(null);
  const result = ref<AsrRecognitionResult | null>(null);
  const error = ref<string | null>(null);
  const isLoadingConfig = ref(false);
  const isRecognizing = ref(false);

  const sourceFileName = computed(() =>
    sourceFilePath.value ? formatFileName(sourceFilePath.value) : null,
  );

  async function loadConfig() {
    isLoadingConfig.value = true;
    error.value = null;
    try {
      config.value = await getAsrConfigStatus();
      return config.value;
    } catch (loadError) {
      error.value = formatError(loadError, "读取语音识别配置失败。");
      return null;
    } finally {
      isLoadingConfig.value = false;
    }
  }

  async function selectSourceFile() {
    error.value = null;
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "带声音的视频或音频",
            extensions: [
              "mp4",
              "mov",
              "avi",
              "mkv",
              "webm",
              "mp3",
              "wav",
              "m4a",
              "aac",
              "flac",
              "ogg",
              "opus",
              "wma",
            ],
          },
        ],
      });
      if (!selected || Array.isArray(selected)) {
        return;
      }
      if (selected !== sourceFilePath.value) {
        result.value = null;
      }
      sourceFilePath.value = selected;
    } catch (selectError) {
      error.value = formatError(selectError, "选择音频或视频失败。");
    }
  }

  async function recognize() {
    error.value = null;
    if (!sourceFilePath.value) {
      error.value = "请先选择带人声的音频或视频文件。";
      return;
    }
    if (!config.value.configured) {
      error.value = "尚未配置TTS/ASR语音密钥，请先打开右侧“API 密钥”完成配置。";
      return;
    }

    const selectedPath = sourceFilePath.value;
    options.clearLogs();
    options.appendLog(`开始识别：${formatFileName(selectedPath)}。`, "info");
    isRecognizing.value = true;

    try {
      const nextResult = await options.runTask("云端语音识别", async (task) => {
        const recognized = await runRetryableRequest(
          () => {
            task.throwIfCancelled();
            return recognizeSpeech(
              selectedPath,
              task.progress(0, 100, "正在准备语音识别"),
            );
          },
          {
            onRetry({ nextAttempt, maxAttempts, delayMs, message }) {
              options.appendLog(
                `语音识别遇到临时故障，${Math.ceil(delayMs / 1000)} 秒后进行第 ${nextAttempt}/${maxAttempts} 次尝试：${message}`,
                "info",
              );
            },
          },
        );
        task.throwIfCancelled();
        return recognized;
      });
      result.value = nextResult;
      options.appendLog(
        nextResult.cacheHit
          ? "已读取本地识别缓存，本次没有重复调用云端接口。"
          : `识别完成，共得到 ${nextResult.utterances.length} 条句子时间轴。`,
        "success",
      );
    } catch (recognizeError) {
      if (isTaskCancelledError(recognizeError)) {
        error.value = "语音识别已取消。";
        options.appendLog(error.value, "info");
      } else {
        error.value = formatError(recognizeError, "语音识别失败。");
        options.appendLog(error.value, "error");
      }
    } finally {
      isRecognizing.value = false;
    }
  }

  function clearSelection() {
    sourceFilePath.value = null;
    result.value = null;
    error.value = null;
  }

  return {
    asrConfig: config,
    asrError: error,
    asrResult: result,
    asrSourceFileName: sourceFileName,
    asrSourceFilePath: sourceFilePath,
    clearAsrSelection: clearSelection,
    isLoadingAsrConfig: isLoadingConfig,
    isRecognizingAsr: isRecognizing,
    loadAsrConfig: loadConfig,
    recognizeAsr: recognize,
    selectAsrSourceFile: selectSourceFile,
  };
}

function formatFileName(filePath: string) {
  return filePath.split(/[\\/]/).pop() ?? filePath;
}

function formatError(value: unknown, fallback: string) {
  if (value instanceof Error) {
    return value.message;
  }
  return String(value ?? "").trim() || fallback;
}
