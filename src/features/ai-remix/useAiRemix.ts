import { ref } from "vue";
import type { Ref } from "vue";
import { concatSelectedSegments } from "../../services/videoMixService";
import type { MixVideoResult, RemixExportSettings } from "../../services/videoMixService";
import type { SegmentCategory } from "../../services/videoMixService";
import type { TaskLogLevel } from "../../types/workbench";
import { runRetryableRequest } from "../../services/retryableRequest";
import { planAiRemix } from "./services/aiRemixService";
import {
  analyzeAiRemixContent,
  analyzeAiRemixSegmentDescriptions,
  createAiRemixVariants,
  createAiRemixPlannedShots,
  prepareAiRemixSegments,
} from "./services/aiRemixWorkflow";
import type {
  AiRemixContentAnalysis,
  AiRemixGenerationFailure,
  AiRemixMatchMode,
  AiRemixPlannedShot,
  AiRemixSegment,
  AiRemixVariant,
} from "./types";
import { sanitizeAiRemixContentAnalysis } from "./analysisCache";
import type { ProjectAiSnapshot } from "../project-recovery/types";
import { isTaskCancelledError } from "../task-center";
import type { TaskRunHandle } from "../task-center";

interface UseAiRemixOptions {
  outputDirectory: Readonly<Ref<string | null>>;
  generationOutputDirectory: Readonly<Ref<string | null>>;
  matchMode: Readonly<Ref<AiRemixMatchMode>>;
  remixExportSettings: Readonly<Ref<RemixExportSettings>>;
  appendAiRemixLog: (message: string, level: TaskLogLevel) => void;
  appendSplitLog: (message: string, level: TaskLogLevel) => void;
  clearAiRemixLogs: () => void;
  onSegmentThumbnailsPrepared: (entries: Record<string, string>) => void;
  onSegmentCategorySuggested: (segmentPath: string, category: SegmentCategory) => void;
  validateExportSettings: () => string | null;
  setMixing: (isMixing: boolean) => void;
  onGenerated: (result: MixVideoResult) => void;
  formatCanvasLog: (result: MixVideoResult) => string;
  formatSmoothLog: (result: MixVideoResult) => string;
  runTask: <T>(label: string, runner: (task: TaskRunHandle) => Promise<T>) => Promise<T>;
}

interface ContentAnalysisOptions {
  mode: AiRemixMatchMode;
  dimensions: {
    shotType: boolean;
    personAction: boolean;
    sellingPoints: boolean;
    usableCopy: boolean;
  };
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
  const failedAiVariants = ref<AiRemixVariant[]>([]);
  const isAnalyzingAiContent = ref(false);
  const aiContentAnalysisError = ref<string | null>(null);
  const aiContentAnalysisProgressText = ref<string | null>(null);

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
    failedAiVariants.value = [];
    isAnalyzingAiContent.value = false;
    aiContentAnalysisError.value = null;
    aiContentAnalysisProgressText.value = null;
    options.clearAiRemixLogs();
  }

  async function prepareSegmentAssets(segmentPaths: string[], task?: TaskRunHandle) {
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
          task,
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

  async function ensureAiSegmentDescriptions(updateProgress: (message: string) => void) {
    aiPreparedSegments.value = await analyzeAiRemixSegmentDescriptions(
      aiPreparedSegments.value,
      {
        appendLog: options.appendAiRemixLog,
        updateProgress,
      },
    );
  }

  async function analyzePreparedSegmentContent(configuration: ContentAnalysisOptions = {
    mode: "cloud",
    dimensions: { shotType: true, personAction: true, sellingPoints: true, usableCopy: true },
  }) {
    aiContentAnalysisError.value = null;
    aiContentAnalysisProgressText.value = null;
    if (aiPreparedSegments.value.length === 0) {
      aiContentAnalysisError.value = "请先完成视频切片。";
      return;
    }

    isAnalyzingAiContent.value = true;
    try {
      if (!Object.values(configuration.dimensions).some(Boolean)) {
        throw new Error("请至少选择一个内容提炼维度。");
      }
      if (configuration.mode === "local") {
        aiContentAnalysisProgressText.value = "正在使用本地规则提炼文件名与片段时长";
        aiPreparedSegments.value = aiPreparedSegments.value.map((segment) => ({
          ...segment,
          contentAnalysis: buildLocalContentAnalysis(segment, configuration.dimensions),
        }));
        options.appendAiRemixLog("本地内容提炼完成：已按所选维度从素材文件名和时长生成可编辑结果。", "success");
        return;
      }
      await ensureAiSegmentDescriptions((message) => {
        aiContentAnalysisProgressText.value = message;
      });
      const missingDescriptionCount = aiPreparedSegments.value.filter(
        (segment) => !segment.description?.trim(),
      ).length;
      if (missingDescriptionCount > 0) {
        throw new Error(
          `仍有 ${missingDescriptionCount} 个片段缺少画面描述，请再次运行；已成功结果会继续保留。`,
        );
      }

      const result = await analyzeAiRemixContent(aiPreparedSegments.value, {
        appendLog: options.appendAiRemixLog,
        updateProgress(message) {
          aiContentAnalysisProgressText.value = message;
        },
      });
      aiPreparedSegments.value = result.segments.map((segment) => ({
        ...segment,
        contentAnalysis: selectContentAnalysisDimensions(segment.contentAnalysis, configuration.dimensions),
      }));
      result.categoriesBySegmentId.forEach((category, segmentId) => {
        const segment = aiPreparedSegments.value.find((item) => item.segmentId === segmentId);
        if (segment) options.onSegmentCategorySuggested(segment.path, category);
      });
      if (result.failedCount > 0) {
        aiContentAnalysisError.value = `${result.failedCount} 个片段提炼失败，再次运行会只补失败片段。`;
      }
    } catch (error) {
      aiContentAnalysisError.value =
        error instanceof Error ? error.message : String(error ?? "内容提炼失败。");
      options.appendAiRemixLog(`内容提炼失败：${aiContentAnalysisError.value}`, "error");
    } finally {
      isAnalyzingAiContent.value = false;
      aiContentAnalysisProgressText.value = null;
    }
  }

  function updateAiSegmentContentAnalysis(
    segmentPath: string,
    contentAnalysis: AiRemixContentAnalysis,
  ) {
    const sanitized = sanitizeAiRemixContentAnalysis(contentAnalysis);
    if (!sanitized) return;
    aiPreparedSegments.value = aiPreparedSegments.value.map((segment) =>
      segment.path === segmentPath ? { ...segment, contentAnalysis: sanitized } : segment,
    );
    options.appendAiRemixLog(`已保存 ${formatFileName(segmentPath)} 的人工提炼修改。`, "success");
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
      `开始准备 AI 逐句分镜，共 ${aiPreparedSegments.value.length} 个候选片段；当前使用${options.matchMode.value === "local" ? "本地快速匹配" : "云端精准匹配"}。`,
      "info",
    );

    try {
      await ensureAiSegmentDescriptions((message) => {
        aiPlanningProgressText.value = message;
      });
      const segmentsWithDescriptions = aiPreparedSegments.value.filter(
        (segment) => segment.description?.trim(),
      );

      if (segmentsWithDescriptions.length !== aiPreparedSegments.value.length) {
        throw new Error("仍有片段在自动重试后缺少画面描述，请再次生成分镜；已成功识别的片段会继续保留。");
      }

      aiPlanningProgressText.value = "正在为软件自动断好的短句匹配画面...";
      options.appendAiRemixLog("片段画面理解完成，开始为固定短句匹配画面。", "info");
      const result = await runRetryableRequest(
        () =>
          planAiRemix(
            aiScript.value,
            segmentsWithDescriptions.map((segment) => ({
              segmentId: segment.segmentId,
              durationSeconds: segment.durationSeconds,
              description: segment.description as string,
            })),
            options.matchMode.value,
          ),
        {
          onRetry({ nextAttempt, maxAttempts, delayMs, message }) {
            aiPlanningProgressText.value = `AI 分镜临时失败，正在准备第 ${nextAttempt}/${maxAttempts} 次尝试...`;
            options.appendAiRemixLog(
              `固定短句画面匹配遇到临时故障，${Math.ceil(delayMs / 1000)} 秒后进行第 ${nextAttempt}/${maxAttempts} 次尝试：${message}`,
              "info",
            );
          },
        },
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
    failedAiVariants.value = [];

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

  async function generateAiRemixVideos(
    preparedVariants?: AiRemixVariant[],
    preserveSuccessfulResults = false,
  ) {
    const variants = preparedVariants ?? (await prepareAiRemixVariants());

    if (!variants || variants.length === 0 || !options.outputDirectory.value) {
      return;
    }

    isGeneratingAiRemix.value = true;
    options.setMixing(true);
    if (!preserveSuccessfulResults) {
      aiGeneratedResults.value = [];
    }
    aiGenerationFailures.value = [];
    failedAiVariants.value = [];
    options.appendAiRemixLog(
      `开始顺序生成 ${variants.length} 条 AI 差异视频，每条使用 ${aiPlannedShots.value.length} 个分镜。`,
      "info",
    );

    try {
      await options.runTask(
        preserveSuccessfulResults ? "重试失败的AI视频" : "生成AI差异视频",
        async (task) => {
          for (const [index, variant] of variants.entries()) {
            task.throwIfCancelled();
            aiGenerationProgressText.value = `正在生成第 ${index + 1}/${variants.length} 条差异视频...`;

            try {
              const result = await concatSelectedSegments(
                variant.shots.map((shot) => shot.segment.path),
                options.generationOutputDirectory.value ?? options.outputDirectory.value as string,
                options.remixExportSettings.value,
                task.progress(
                  (index / variants.length) * 100,
                  ((index + 1) / variants.length) * 100,
                  `正在生成AI视频 ${index + 1}/${variants.length}`,
                ),
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
              if (isTaskCancelledError(error)) throw error;
              const message =
                error instanceof Error
                  ? error.message
                  : String(error ?? "AI 混剪视频生成失败。");
              aiGenerationFailures.value.push({ version: variant.version, message });
              failedAiVariants.value.push(variant);
              options.appendAiRemixLog(
                `第 ${variant.version}/${variants.length} 条差异视频生成失败，继续生成下一条：${message}`,
                "error",
              );
            }
          }
        },
      );

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
    } catch (error) {
      if (isTaskCancelledError(error)) {
        aiGenerateError.value = "AI视频生成任务已取消，已完成的结果会保留。";
        options.appendAiRemixLog(aiGenerateError.value, "info");
      } else {
        aiGenerateError.value =
          error instanceof Error ? error.message : String(error ?? "AI视频生成失败。");
        options.appendAiRemixLog(aiGenerateError.value, "error");
      }
    } finally {
      isGeneratingAiRemix.value = false;
      aiGenerationProgressText.value = null;
      options.setMixing(false);
    }
  }

  async function generateAiRemixVideo() {
    await generateAiRemixVideos();
  }

  async function retryFailedAiRemixVideos() {
    if (failedAiVariants.value.length === 0) return;
    const variants = [...failedAiVariants.value];
    options.appendAiRemixLog(
      `开始重试 ${variants.length} 条失败AI视频，成功结果不会重复生成。`,
      "info",
    );
    await generateAiRemixVideos(variants, true);
  }

  function restoreAiRemixState(
    snapshot: ProjectAiSnapshot,
    preparedSegments: AiRemixSegment[],
    plannedShots: AiRemixPlannedShot[],
  ) {
    aiScript.value = snapshot.script;
    aiGenerateCount.value = Math.min(10, Math.max(1, Math.floor(snapshot.generateCount || 3)));
    aiPreparedSegments.value = preparedSegments;
    aiPlannedShots.value = plannedShots;
    isPreparingAiSegments.value = false;
    isPlanningAiRemix.value = false;
    isGeneratingAiRemix.value = false;
    aiPreparationError.value = null;
    aiPlanError.value = null;
    aiGenerateError.value = null;
    aiPlanningProgressText.value = null;
    aiGenerationProgressText.value = null;
    aiGenerationSummaryText.value = null;
    aiGeneratedResults.value = [];
    aiGenerationFailures.value = [];
    failedAiVariants.value = [];
    isAnalyzingAiContent.value = false;
    aiContentAnalysisError.value = null;
    aiContentAnalysisProgressText.value = null;
  }

  function restoreAiPreparedSegments(preparedSegments: AiRemixSegment[]) {
    aiPreparedSegments.value = preparedSegments;
    aiPlannedShots.value = [];
    aiPreparationError.value = null;
    aiPlanError.value = null;
    aiPlanningProgressText.value = null;
  }

  return {
    aiContentAnalysisError,
    aiContentAnalysisProgressText,
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
    analyzePreparedSegmentContent,
    generateAiRemixVideo,
    generateAiRemixVideos,
    isGeneratingAiRemix,
    isAnalyzingAiContent,
    isPlanningAiRemix,
    isPreparingAiSegments,
    moveAiRemixShot,
    prepareSegmentAssets,
    prepareAiRemixVariants,
    removeAiRemixShot,
    replaceAiRemixShotSegment,
    requestAiRemixPlan,
    retryFailedAiRemixVideos,
    resetAiRemixState,
    restoreAiPreparedSegments,
    restoreAiRemixState,
    updateAiSegmentContentAnalysis,
  };
}

function buildLocalContentAnalysis(
  segment: AiRemixSegment,
  dimensions: ContentAnalysisOptions["dimensions"],
): AiRemixContentAnalysis {
  const fileName = segment.path.split(/[\\/]/).pop()?.replace(/\.[^.]+$/, "") ?? segment.segmentId;
  return {
    theme: dimensions.usableCopy ? fileName : "",
    sellingPoints: dimensions.sellingPoints ? [fileName] : [],
    action: dimensions.personAction ? `时长 ${segment.durationSeconds.toFixed(1)} 秒的素材片段` : "",
    tags: dimensions.shotType ? [segment.durationSeconds <= 3 ? "短镜头" : "常规镜头"] : [],
  };
}

function selectContentAnalysisDimensions(
  value: AiRemixContentAnalysis | null,
  dimensions: ContentAnalysisOptions["dimensions"],
): AiRemixContentAnalysis | null {
  if (!value) return null;
  return {
    theme: dimensions.usableCopy ? value.theme : "",
    sellingPoints: dimensions.sellingPoints ? value.sellingPoints : [],
    action: dimensions.personAction ? value.action : "",
    tags: dimensions.shotType ? value.tags : [],
  };
}

function formatFileName(filePath: string) {
  return filePath.split(/[\\/]/).pop() ?? filePath;
}
