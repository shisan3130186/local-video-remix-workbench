import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, ref } from "vue";
import type { Ref } from "vue";
import type { MixVideoResult, RemixExportSettings } from "../../services/videoMixService";
import {
  cleanupTtsSession,
  concatNarratedSegments,
  getTtsConfigStatus,
  synthesizeTts,
  synthesizeTtsShot,
} from "./services/ttsService";
import type {
  NarratedAudioSettings,
  NarratedSegmentInput,
  NarratedShotSource,
  NarratedSubtitlePosition,
  NarratedSubtitleSettings,
  NarratedSubtitleSize,
  NarratedVideoFailure,
  TtsConfigStatus,
  TtsSynthesisResult,
} from "./types";

interface UseTtsOptions {
  text: Ref<string>;
  outputDirectory: Ref<string | null>;
  appendLog: (message: string, level: "info" | "success" | "error") => void;
  clearLogs: () => void;
  validateExportSettings: () => string | null;
  setMixing: (isMixing: boolean) => void;
  onGenerated: (result: MixVideoResult) => void;
  formatCanvasLog: (result: MixVideoResult) => string;
  formatSmoothLog: (result: MixVideoResult) => string;
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
  const isGeneratingNarratedVideo = ref(false);
  const ttsConfigError = ref<string | null>(null);
  const ttsError = ref<string | null>(null);
  const narratedVideoError = ref<string | null>(null);
  const narrationProgressText = ref<string | null>(null);
  const narratedVideoSummaryText = ref<string | null>(null);
  const narratedVideoResults = ref<string[]>([]);
  const narratedVideoFailures = ref<NarratedVideoFailure[]>([]);
  const ttsResult = ref<TtsSynthesisResult | null>(null);
  const ttsVideoEnabled = ref(false);
  const ttsKeepOriginalAudio = ref(false);
  const ttsOriginalAudioVolume = ref(0.15);
  const ttsSubtitleEnabled = ref(true);
  const ttsSubtitlePosition = ref<NarratedSubtitlePosition>("bottom");
  const ttsSubtitleSize = ref<NarratedSubtitleSize>("medium");

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

  async function generateNarratedVideo(
    shots: NarratedShotSource[],
    settings: RemixExportSettings,
  ) {
    await generateNarratedVideos([shots], settings);
  }

  async function generateNarratedVideos(
    variants: NarratedShotSource[][],
    settings: RemixExportSettings,
  ) {
    options.clearLogs();
    narratedVideoError.value = null;
    narrationProgressText.value = null;
    narratedVideoSummaryText.value = null;
    narratedVideoResults.value = [];
    narratedVideoFailures.value = [];

    const shots = variants[0] ?? [];

    if (shots.length < 2) {
      narratedVideoError.value = "至少保留 2 个分镜才能生成带配音视频。";
      return;
    }

    if (
      variants.some(
        (variant) =>
          variant.length !== shots.length ||
          variant.some((shot, index) => shot.text !== shots[index]?.text),
      )
    ) {
      narratedVideoError.value = "差异视频的分镜数量或文案不一致，请重新生成分镜。";
      return;
    }

    if (!options.outputDirectory.value) {
      narratedVideoError.value = "请先选择输出目录。";
      return;
    }

    if (!config.value.configured) {
      narratedVideoError.value =
        "未配置 TTS_API_KEY。请在启动软件的同一个 PowerShell 窗口中设置后重新启动。";
      return;
    }

    if (Math.abs(settings.playbackSpeed - 1) > 0.001) {
      narratedVideoError.value =
        "AI配音视频暂时只支持 1.0x 速度，请在视频效果中把变速恢复为 1.0x。";
      return;
    }

    const validationError = options.validateExportSettings();
    if (validationError) {
      narratedVideoError.value = validationError;
      return;
    }

    const normalizedSpeaker = speaker.value.trim();
    if (!normalizedSpeaker) {
      narratedVideoError.value = "音色ID不能为空。";
      return;
    }

    const sessionId = createSessionId();
    const narrationPaths: string[] = [];
    isGeneratingNarratedVideo.value = true;
    options.setMixing(true);
    options.appendLog(
      `开始为 ${shots.length} 个分镜逐句生成一次 AI 配音，之后复用到 ${variants.length} 条差异视频。`,
      "info",
    );

    try {
      for (const [index, shot] of shots.entries()) {
        narrationProgressText.value = `正在生成第 ${index + 1}/${shots.length} 句配音...`;
        const result = await synthesizeTtsShot(
          shot.text,
          normalizedSpeaker,
          sessionId,
          index + 1,
        );
        narrationPaths.push(result.outputPath);
        options.appendLog(`第 ${index + 1}/${shots.length} 句配音生成完成。`, "success");
      }

      options.appendLog(
        `逐句配音完成，开始顺序合成 ${variants.length} 条视频；单条失败不会中断剩余任务。`,
        "info",
      );

      for (const [variantIndex, variant] of variants.entries()) {
        narrationProgressText.value = `正在生成第 ${variantIndex + 1}/${variants.length} 条带配音视频...`;
        const narratedSegments: NarratedSegmentInput[] = variant.map((shot, shotIndex) => ({
          videoPath: shot.segmentPath,
          videoDurationSeconds: shot.segmentDurationSeconds,
          narrationPath: narrationPaths[shotIndex],
          subtitleText: shot.text,
          alternativeVideos: shot.alternativeSegments,
        }));

        try {
          const result = await concatNarratedSegments(
            narratedSegments,
            options.outputDirectory.value,
            settings,
            buildNarratedAudioSettings(
              ttsKeepOriginalAudio.value,
              ttsOriginalAudioVolume.value,
            ),
            buildNarratedSubtitleSettings(
              ttsSubtitleEnabled.value,
              ttsSubtitlePosition.value,
              ttsSubtitleSize.value,
            ),
          );
          narratedVideoResults.value.push(result.outputPath);
          options.onGenerated(result);
          options.appendLog(options.formatCanvasLog(result), "info");
          options.appendLog(options.formatSmoothLog(result), "info");
          options.appendLog(
            `第 ${variantIndex + 1}/${variants.length} 条带 AI 配音视频生成成功：${formatFileName(result.outputPath)}。`,
            "success",
          );
        } catch (error) {
          const message = formatError(error, "生成带 AI 配音视频失败。");
          narratedVideoFailures.value.push({
            version: variantIndex + 1,
            message,
          });
          options.appendLog(
            `第 ${variantIndex + 1}/${variants.length} 条带 AI 配音视频生成失败，继续生成下一条：${message}`,
            "error",
          );
        }
      }

      const successCount = narratedVideoResults.value.length;
      const failureCount = narratedVideoFailures.value.length;
      narratedVideoSummaryText.value = `本次完成：成功 ${successCount} 条，失败 ${failureCount} 条，共尝试 ${variants.length} 条；配音只生成了 1 次并已复用。`;

      if (successCount === 0 && failureCount > 0) {
        narratedVideoError.value = narratedVideoFailures.value[0].message;
      }

      options.appendLog(
        narratedVideoSummaryText.value,
        failureCount > 0 ? "error" : "success",
      );
    } catch (error) {
      narratedVideoError.value = formatError(error, "生成带AI配音视频失败。");
      options.appendLog(narratedVideoError.value, "error");
    } finally {
      try {
        await cleanupTtsSession(sessionId);
      } catch {
        options.appendLog("临时配音文件未能自动清理，但不影响已生成的视频。", "error");
      }
      isGeneratingNarratedVideo.value = false;
      narrationProgressText.value = null;
      options.setMixing(false);
    }
  }

  function resetTtsSettings() {
    speaker.value = config.value.speaker || FALLBACK_SPEAKER;
    ttsError.value = null;
    narratedVideoError.value = null;
    narrationProgressText.value = null;
    narratedVideoSummaryText.value = null;
    narratedVideoResults.value = [];
    narratedVideoFailures.value = [];
    ttsResult.value = null;
    ttsVideoEnabled.value = false;
    ttsKeepOriginalAudio.value = false;
    ttsOriginalAudioVolume.value = 0.15;
    ttsSubtitleEnabled.value = true;
    ttsSubtitlePosition.value = "bottom";
    ttsSubtitleSize.value = "medium";
  }

  return {
    generateTts,
    generateNarratedVideo,
    generateNarratedVideos,
    isGeneratingNarratedVideo,
    isGeneratingTts,
    isLoadingTtsConfig: isLoadingConfig,
    loadTtsConfig,
    narratedVideoError,
    narratedVideoFailures,
    narratedVideoResults,
    narratedVideoSummaryText,
    narrationProgressText,
    resetTtsSettings,
    ttsAudioUrl,
    ttsConfig: config,
    ttsConfigError,
    ttsError,
    ttsResult,
    ttsKeepOriginalAudio,
    ttsOriginalAudioVolume,
    ttsSubtitleEnabled,
    ttsSubtitlePosition,
    ttsSubtitleSize,
    ttsSpeaker: speaker,
    ttsVideoEnabled,
  };
}

function buildNarratedSubtitleSettings(
  enabled: boolean,
  position: NarratedSubtitlePosition,
  size: NarratedSubtitleSize,
): NarratedSubtitleSettings {
  return {
    enabled,
    position,
    size,
  };
}

function buildNarratedAudioSettings(
  keepOriginalAudio: boolean,
  originalAudioVolume: number,
): NarratedAudioSettings {
  return {
    keepOriginalAudio,
    originalAudioVolume,
  };
}

function createSessionId() {
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 12)}`;
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
