import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, ref } from "vue";
import type { Ref } from "vue";
import { getTtsConfigStatus, synthesizeTts } from "./services/ttsService";
import type { TtsConfigStatus, TtsSynthesisResult } from "./types";

interface UseTtsOptions {
  text: Ref<string>;
  outputDirectory: Ref<string | null>;
  appendLog: (message: string, level: "info" | "success" | "error") => void;
  clearLogs: () => void;
}

const FALLBACK_RESOURCE_ID = "seed-tts-2.0";
const FALLBACK_SPEAKER = "zh_female_vv_uranus_bigtts";

export function useTts(options: UseTtsOptions) {
  const config = ref<TtsConfigStatus>({
    configured: false,
    resourceId: FALLBACK_RESOURCE_ID,
    speaker: FALLBACK_SPEAKER,
  });
  const speaker = ref(FALLBACK_SPEAKER);
  const isLoadingConfig = ref(false);
  const isGeneratingTts = ref(false);
  const ttsConfigError = ref<string | null>(null);
  const ttsError = ref<string | null>(null);
  const ttsResult = ref<TtsSynthesisResult | null>(null);

  const ttsAudioUrl = computed(() =>
    ttsResult.value ? convertFileSrc(ttsResult.value.outputPath) : null,
  );

  async function loadTtsConfig() {
    isLoadingConfig.value = true;
    ttsConfigError.value = null;

    try {
      config.value = await getTtsConfigStatus();
      speaker.value = config.value.speaker;
    } catch (error) {
      ttsConfigError.value = formatError(error, "读取TTS配置失败。");
    } finally {
      isLoadingConfig.value = false;
    }
  }

  async function generateTts() {
    options.clearLogs();
    ttsError.value = null;

    const normalizedText = options.text.value.trim();
    if (!normalizedText) {
      ttsError.value = "请输入需要生成配音的文案。";
      return;
    }

    if (!options.outputDirectory.value) {
      ttsError.value = "请先选择输出目录。";
      return;
    }

    if (!config.value.configured) {
      ttsError.value =
        "未配置 TTS_API_KEY。请在启动软件的同一个 PowerShell 窗口中设置后重新启动。";
      return;
    }

    const normalizedSpeaker = speaker.value.trim();
    if (!normalizedSpeaker) {
      ttsError.value = "音色ID不能为空。";
      return;
    }

    isGeneratingTts.value = true;
    options.appendLog(`正在生成TTS配音，共 ${normalizedText.length} 个字符。`, "info");

    try {
      const result = await synthesizeTts(
        normalizedText,
        options.outputDirectory.value,
        normalizedSpeaker,
      );
      ttsResult.value = result;
      speaker.value = result.speaker;
      options.appendLog(
        `TTS配音生成完成：${formatFileName(result.outputPath)}。`,
        "success",
      );
    } catch (error) {
      ttsError.value = formatError(error, "生成TTS配音失败。");
      options.appendLog(ttsError.value, "error");
    } finally {
      isGeneratingTts.value = false;
    }
  }

  function resetTtsSettings() {
    speaker.value = config.value.speaker || FALLBACK_SPEAKER;
    ttsError.value = null;
    ttsResult.value = null;
  }

  return {
    generateTts,
    isGeneratingTts,
    isLoadingTtsConfig: isLoadingConfig,
    loadTtsConfig,
    resetTtsSettings,
    ttsAudioUrl,
    ttsConfig: config,
    ttsConfigError,
    ttsError,
    ttsResult,
    ttsSpeaker: speaker,
  };
}

function formatError(error: unknown, fallback: string) {
  if (error instanceof Error) {
    return error.message;
  }

  const value = String(error ?? "").trim();
  return value || fallback;
}

function formatFileName(path: string) {
  return path.split(/[\\/]/).pop() ?? path;
}
