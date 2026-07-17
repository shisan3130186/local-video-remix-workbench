import { ref } from "vue";
import type { Ref } from "vue";
import { concatSelectedSegments } from "../../services/videoMixService";
import type { MixVideoResult, RemixExportSettings } from "../../services/videoMixService";
import type { TaskLogLevel } from "../../types/workbench";
import { planAiRemix } from "./services/aiRemixService";
import {
  analyzeAiRemixSegmentDescriptions,
  createAiRemixVariants,
  createAiRemixPlannedShots,
  prepareAiRemixSegments,
} from "./services/aiRemixWorkflow";
import type {
  AiRemixGenerationFailure,
  AiRemixPlannedShot,
  AiRemixSegment,
  AiRemixVariant,
} from "./types";

interface UseAiRemixOptions {
  outputDirectory: Readonly<Ref<string | null>>;
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
  const aiGenerateCount = ref(3);
  const aiGenerationProgressText = ref<string | null>(null);
  const aiGenerationSummaryText = ref<string | null>(null);
  const aiGeneratedResults = ref<string[]>([]);
  const aiGenerationFailures = ref<AiRemixGenerationFailure[]>([]);

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
    aiGenerateCount.value = 3;
    aiGenerationProgressText.value = null;
    aiGenerationSummaryText.value = null;
    aiGeneratedResults.value = [];
    aiGenerationFailures.value = [];
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

      aiPreparedSegments.value = preparedSegments;

      if (preparedSegments.length >= 2) {
        aiPreparationError.value =
          preparationErrors.length > 0
            ? `${preparedSegments.length}/${segmentPaths.length} 个片段准备完成，${preparationErrors.length} 个失败片段已自动跳过。可以继续填写文案。`
            : null;
        options.appendSplitLog(
          preparationErrors.length > 0
            ? `AI 片段信息部分完成：${preparedSegments.length}/${segmentPaths.length} 个可用，已跳过 ${preparationErrors.length} 个失败片段。`
            : `AI 片段信息准备完成：${preparedSegments.length} 个片段。`,
          preparationErrors.length > 0 ? "error" : "success",
        );
      } else {
        aiPreparationError.value = `只有 ${preparedSegments.length}/${segmentPaths.length} 个片段准备成功，AI 混剪至少需要 2 个可用片段。请查看任务日志。`;
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

    if (aiPreparedSegments.value.length < 2) {
      aiPlanError.value = "可用片段不足 2 个，请重新切片或查看任务日志。";
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

      aiPlanningProgressText.value = "正在为软件自动断好的短句匹配画面...";
      options.appendAiRemixLog("片段画面理解完成，开始为固定短句匹配画面。", "info");
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

  async function prepareAiRemixVariants(): Promise<AiRemixVariant[] | null> {
    aiGenerateError.value = null;
    aiGenerationProgressText.value = null;
    aiGenerationSummaryText.value = null;
    aiGeneratedResults.value = [];
    aiGenerationFailures.value = [];

    if (aiPlannedShots.value.length < 2) {
      aiGenerateError.value = "至少保留 2 个分镜才能生成 AI 混剪视频。";
      return null;
    }

    if (!options.outputDirectory.value) {
      aiGenerateError.value = "请先选择输出目录。";
      return null;
    }

    if (
      !Number.isInteger(aiGenerateCount.value) ||
      aiGenerateCount.value < 1 ||
      aiGenerateCount.value > 10
    ) {
      aiGenerateError.value = "生成数量必须是 1 到 10 之间的整数。";
      return null;
    }

    const validationError = options.validateExportSettings();

    if (validationError) {
      aiGenerateError.value = validationError;
      return null;
    }

    try {
      const variants = await createAiRemixVariants(
        aiPlannedShots.value,
        aiGenerateCount.value,
      );

      if (variants.length < aiGenerateCount.value) {
        aiGenerationSummaryText.value = `当前备选画面只能组成 ${variants.length} 条不重复视频，将按实际数量生成。`;
        options.appendAiRemixLog(aiGenerationSummaryText.value, "info");
      }

      return variants;
    } catch (error) {
      aiGenerateError.value =
        error instanceof Error
          ? error.message
          : String(error ?? "生成差异视频方案失败。");
      options.appendAiRemixLog(`生成差异视频方案失败：${aiGenerateError.value}`, "error");
      return null;
    }
  }

  async function generateAiRemixVideos(preparedVariants?: AiRemixVariant[]) {
    const variants = preparedVariants ?? (await prepareAiRemixVariants());

    if (!variants || variants.length === 0 || !options.outputDirectory.value) {
      return;
    }

    isGeneratingAiRemix.value = true;
    options.setMixing(true);
    options.appendAiRemixLog(
      `开始顺序生成 ${variants.length} 条 AI 差异视频，每条使用 ${aiPlannedShots.value.length} 个分镜。`,
      "info",
    );

    try {
      for (const [index, variant] of variants.entries()) {
        aiGenerationProgressText.value = `正在生成第 ${index + 1}/${variants.length} 条差异视频...`;

        try {
          const result = await concatSelectedSegments(
            variant.shots.map((shot) => shot.segment.path),
            options.outputDirectory.value,
            options.remixExportSettings.value,
          );
          aiGeneratedResults.value.push(result.outputPath);
          options.onGenerated(result);
          options.appendAiRemixLog(options.formatCanvasLog(result), "info");
          options.appendAiRemixLog(options.formatSmoothLog(result), "info");
          options.appendAiRemixLog(
            `第 ${variant.version}/${variants.length} 条差异视频生成成功：${result.outputPath}`,
            "success",
          );
        } catch (error) {
          const message =
            error instanceof Error
              ? error.message
              : String(error ?? "AI 混剪视频生成失败。");
          aiGenerationFailures.value.push({ version: variant.version, message });
          options.appendAiRemixLog(
            `第 ${variant.version}/${variants.length} 条差异视频生成失败，继续生成下一条：${message}`,
            "error",
          );
        }
      }

      const successCount = aiGeneratedResults.value.length;
      const failureCount = aiGenerationFailures.value.length;
      const availabilityNote =
        variants.length < aiGenerateCount.value
          ? ` 请求 ${aiGenerateCount.value} 条，备选画面实际只能组成 ${variants.length} 条不重复视频。`
          : "";
      aiGenerationSummaryText.value = `本次完成：成功 ${successCount} 条，失败 ${failureCount} 条，共尝试 ${variants.length} 条。${availabilityNote}`;

      if (successCount === 0 && failureCount > 0) {
        aiGenerateError.value = aiGenerationFailures.value[0].message;
      }

      options.appendAiRemixLog(
        aiGenerationSummaryText.value,
        failureCount > 0 ? "error" : "success",
      );
    } finally {
      isGeneratingAiRemix.value = false;
      aiGenerationProgressText.value = null;
      options.setMixing(false);
    }
  }

  async function generateAiRemixVideo() {
    await generateAiRemixVideos();
  }

  return {
    aiGenerateError,
    aiGenerateCount,
    aiGeneratedResults,
    aiGenerationFailures,
    aiGenerationProgressText,
    aiGenerationSummaryText,
    aiPlanError,
    aiPlannedShots,
    aiPlanningProgressText,
    aiPreparationError,
    aiPreparedSegments,
    aiScript,
    generateAiRemixVideo,
    generateAiRemixVideos,
    isGeneratingAiRemix,
    isPlanningAiRemix,
    isPreparingAiSegments,
    moveAiRemixShot,
    prepareSegmentAssets,
    prepareAiRemixVariants,
    removeAiRemixShot,
    replaceAiRemixShotSegment,
    requestAiRemixPlan,
    resetAiRemixState,
  };
}
