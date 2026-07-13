import { ref } from "vue";
import type { Ref } from "vue";
import { concatSelectedSegments } from "../../services/videoMixService";
import type { MixVideoResult, RemixExportSettings } from "../../services/videoMixService";
import type { TaskLogLevel } from "../../types/workbench";
import { planAiRemix } from "./services/aiRemixService";
import {
  analyzeAiRemixSegmentDescriptions,
  createAiRemixPlannedShots,
  prepareAiRemixSegments,
} from "./services/aiRemixWorkflow";
import type { AiRemixPlannedShot, AiRemixSegment } from "./types";

interface UseAiRemixOptions {
  outputDirectory: Readonly<Ref<string | null>>;
  segmentPaths: Readonly<Ref<string[]>>;
  remixExportSettings: Readonly<Ref<RemixExportSettings>>;
  appendAiRemixLog: (message: string, level: TaskLogLevel) => void;
  appendSplitLog: (message: string, level: TaskLogLevel) => void;
  clearAiRemixLogs: () => void;
  onSegmentThumbnailsPrepared: (entries: Record<string, string>) => void;
  validateExportSettings: () => string | null;
  setMixing: (isMixing: boolean) => void;
  onGenerated: (result: MixVideoResult) => void;
  formatCanvasLog: (result: MixVideoResult) => string;
  formatSmoothLog: (result: MixVideoResult) => string;
}

export function useAiRemix(options: UseAiRemixOptions) {
  const aiScript = ref("");
  const aiPreparedSegments = ref<AiRemixSegment[]>([]);
  const aiPlannedShots = ref<AiRemixPlannedShot[]>([]);
  const isPreparingAiSegments = ref(false);
  const aiPreparationError = ref<string | null>(null);
  const isPlanningAiRemix = ref(false);
  const aiPlanningProgressText = ref<string | null>(null);
  const isGeneratingAiRemix = ref(false);
  const aiPlanError = ref<string | null>(null);
  const aiGenerateError = ref<string | null>(null);

  function resetAiRemixState(clearScript: boolean) {
    if (clearScript) {
      aiScript.value = "";
    }

    aiPreparedSegments.value = [];
    aiPlannedShots.value = [];
    isPreparingAiSegments.value = false;
    aiPreparationError.value = null;
    isPlanningAiRemix.value = false;
    aiPlanningProgressText.value = null;
    isGeneratingAiRemix.value = false;
    aiPlanError.value = null;
    aiGenerateError.value = null;
    options.clearAiRemixLogs();
  }

  async function prepareSegmentAssets(segmentPaths: string[]) {
    isPreparingAiSegments.value = true;
    aiPreparationError.value = null;
    aiPreparedSegments.value = [];
    aiPlannedShots.value = [];

    try {
      const outputDirectory = options.outputDirectory.value;

      if (!outputDirectory) {
        throw new Error("请先选择输出目录。");
      }

      const { preparedSegments, thumbnailEntries, preparationErrors } =
        await prepareAiRemixSegments({
          segmentPaths,
          outputDirectory,
          onSegmentError(message) {
            options.appendSplitLog(message, "error");
          },
        });

      options.onSegmentThumbnailsPrepared(thumbnailEntries);

      if (Object.keys(thumbnailEntries).length > 0) {
        options.appendSplitLog(
          `片段预览图生成完成：${Object.keys(thumbnailEntries).length} 张。`,
          "success",
        );
      }

      if (preparationErrors.length === 0 && preparedSegments.length === segmentPaths.length) {
        aiPreparedSegments.value = preparedSegments;
        options.appendSplitLog(
          `AI 片段信息准备完成：${preparedSegments.length} 个片段。`,
          "success",
        );
      } else {
        aiPreparationError.value = `有 ${preparationErrors.length} 个片段缺少预览图或时长，请重新切片后再试。`;
      }
    } catch (error) {
      aiPreparationError.value =
        error instanceof Error ? error.message : String(error ?? "片段信息准备失败。");
      throw error;
    } finally {
      isPreparingAiSegments.value = false;
    }
  }

  async function ensureAiSegmentDescriptions() {
    aiPreparedSegments.value = await analyzeAiRemixSegmentDescriptions(
      aiPreparedSegments.value,
      {
        appendLog: options.appendAiRemixLog,
        updateProgress(message) {
          aiPlanningProgressText.value = message;
        },
      },
    );
  }

  async function requestAiRemixPlan() {
    aiPlanError.value = null;

    if (aiPreparedSegments.value.length !== options.segmentPaths.value.length) {
      aiPlanError.value = "片段预览图或时长尚未准备完整，请重新切片后再试。";
      return;
    }

    if (aiScript.value.trim().length === 0) {
      aiPlanError.value = "请先输入用于规划混剪的文案。";
      return;
    }

    isPlanningAiRemix.value = true;
    aiPlanningProgressText.value = null;
    options.appendAiRemixLog(
      `开始准备 AI 逐句分镜，共 ${aiPreparedSegments.value.length} 个候选片段。`,
      "info",
    );

    try {
      await ensureAiSegmentDescriptions();
      const segmentsWithDescriptions = aiPreparedSegments.value.filter(
        (segment) => segment.description?.trim(),
      );

      if (segmentsWithDescriptions.length !== aiPreparedSegments.value.length) {
        throw new Error("仍有片段缺少画面描述，请重试。已成功识别的片段会继续保留。 ");
      }

      aiPlanningProgressText.value = "正在根据文案和画面描述生成分镜...";
      options.appendAiRemixLog("片段画面理解完成，开始生成纯文字分镜规划。", "info");
      const result = await planAiRemix(
        aiScript.value,
        segmentsWithDescriptions.map((segment) => ({
          segmentId: segment.segmentId,
          durationSeconds: segment.durationSeconds,
          description: segment.description as string,
        })),
      );
      const plannedShots = createAiRemixPlannedShots(result, aiPreparedSegments.value);

      aiPlannedShots.value = plannedShots;
      options.appendAiRemixLog(
        `AI 分镜完成：${plannedShots.map((shot) => shot.segment.segmentId).join(" -> ")}。`,
        "success",
      );
    } catch (error) {
      aiPlanError.value =
        error instanceof Error ? error.message : String(error ?? "AI 分镜规划失败。");
      options.appendAiRemixLog(`AI 分镜规划失败：${aiPlanError.value}`, "error");
    } finally {
      isPlanningAiRemix.value = false;
      aiPlanningProgressText.value = null;
    }
  }

  function moveAiRemixShot(index: number, direction: -1 | 1) {
    const targetIndex = index + direction;

    if (index < 0 || targetIndex < 0 || targetIndex >= aiPlannedShots.value.length) {
      return;
    }

    const reordered = [...aiPlannedShots.value];
    [reordered[index], reordered[targetIndex]] = [reordered[targetIndex], reordered[index]];
    aiPlannedShots.value = reordered;
  }

  function removeAiRemixShot(index: number) {
    if (index < 0 || index >= aiPlannedShots.value.length) {
      return;
    }

    aiPlannedShots.value = aiPlannedShots.value.filter(
      (_shot, shotIndex) => shotIndex !== index,
    );
  }

  function replaceAiRemixShotSegment(index: number, segmentId: string) {
    const shot = aiPlannedShots.value[index];
    const replacement = aiPreparedSegments.value.find(
      (segment) => segment.segmentId === segmentId,
    );

    if (!shot || !replacement || replacement.segmentId === shot.segment.segmentId) {
      return;
    }

    const alternativeMap = new Map(
      [shot.segment, ...shot.alternativeSegments]
        .filter((segment) => segment.segmentId !== replacement.segmentId)
        .map((segment) => [segment.segmentId, segment]),
    );
    const updatedShots = [...aiPlannedShots.value];
    updatedShots[index] = {
      ...shot,
      segment: replacement,
      alternativeSegments: Array.from(alternativeMap.values()).slice(0, 3),
    };
    aiPlannedShots.value = updatedShots;
  }

  async function generateAiRemixVideo() {
    aiGenerateError.value = null;

    if (aiPlannedShots.value.length < 2) {
      aiGenerateError.value = "至少保留 2 个分镜才能生成 AI 混剪视频。";
      return;
    }

    if (!options.outputDirectory.value) {
      aiGenerateError.value = "请先选择输出目录。";
      return;
    }

    const validationError = options.validateExportSettings();

    if (validationError) {
      aiGenerateError.value = validationError;
      return;
    }

    isGeneratingAiRemix.value = true;
    options.setMixing(true);
    options.appendAiRemixLog(
      `开始生成 AI 混剪视频，使用 ${aiPlannedShots.value.length} 个分镜。`,
      "info",
    );

    try {
      const result = await concatSelectedSegments(
        aiPlannedShots.value.map((shot) => shot.segment.path),
        options.outputDirectory.value,
        options.remixExportSettings.value,
      );
      options.onGenerated(result);
      options.appendAiRemixLog(options.formatCanvasLog(result), "info");
      options.appendAiRemixLog(options.formatSmoothLog(result), "info");
      options.appendAiRemixLog(`AI 智能混剪生成成功：${result.outputPath}`, "success");
    } catch (error) {
      aiGenerateError.value =
        error instanceof Error ? error.message : String(error ?? "AI 混剪视频生成失败。");
      options.appendAiRemixLog(`AI 混剪生成失败：${aiGenerateError.value}`, "error");
    } finally {
      isGeneratingAiRemix.value = false;
      options.setMixing(false);
    }
  }

  return {
    aiGenerateError,
    aiPlanError,
    aiPlannedShots,
    aiPlanningProgressText,
    aiPreparationError,
    aiPreparedSegments,
    aiScript,
    generateAiRemixVideo,
    isGeneratingAiRemix,
    isPlanningAiRemix,
    isPreparingAiSegments,
    moveAiRemixShot,
    prepareSegmentAssets,
    removeAiRemixShot,
    replaceAiRemixShotSegment,
    requestAiRemixPlan,
    resetAiRemixState,
  };
}
