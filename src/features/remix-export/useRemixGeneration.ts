import { ref } from "vue";
import type { Ref } from "vue";
import { concatSelectedSegments } from "../../services/videoMixService";
import type {
  RemixExportSettings,
  SegmentCategory,
  SegmentCategoryOption,
} from "../../services/videoMixService";
import type { ExportResultType, TaskLogLevel } from "../../types/workbench";
import { isTaskCancelledError } from "../task-center";
import type { TaskRunHandle } from "../task-center";
import { buildMixOptionSummary, formatRemixCanvasLog, formatSmoothRemixLog } from "./remixResultMessages";
import { pickCategorizedSegments, pickRandomSegments } from "./services/remixExportService";

interface UseRemixGenerationOptions {
  outputDirectory: Readonly<Ref<string | null>>;
  segmentPaths: Readonly<Ref<string[]>>;
  segmentCategories: Readonly<Ref<Record<string, SegmentCategory | "">>>;
  categoryOptions: SegmentCategoryOption[];
  remixExportSettings: Readonly<Ref<RemixExportSettings>>;
  validatePictureInPicture: () => string | null;
  validateBgm: () => string | null;
  validatePlaybackSpeed: () => string | null;
  appendMixLog: (message: string, level: TaskLogLevel) => void;
  appendBatchMixLog: (message: string, level: TaskLogLevel) => void;
  clearMixLogs: () => void;
  clearBatchMixLogs: () => void;
  addExportResult: (type: ExportResultType, path: string) => void;
  runTask: <T>(label: string, runner: (task: TaskRunHandle) => Promise<T>) => Promise<T>;
}

interface BatchMixEntry {
  version: number;
  segmentPaths: string[];
}

interface BatchMixFailure extends BatchMixEntry {
  message: string;
}

export function useRemixGeneration(options: UseRemixGenerationOptions) {
  const randomPickCount = ref(1);
  const randomPickError = ref<string | null>(null);
  const randomSelectedSegments = ref<string[]>([]);
  const isMixing = ref(false);
  const mixError = ref<string | null>(null);
  const mixResultPath = ref<string | null>(null);
  const batchGenerateCount = ref(3);
  const isBatchMixing = ref(false);
  const batchMixError = ref<string | null>(null);
  const batchMixResults = ref<string[]>([]);
  const batchMixFailures = ref<BatchMixFailure[]>([]);

  function pickSegmentsRandomly() {
    randomPickError.value = null;
    randomSelectedSegments.value = [];
    resetMixState();

    try {
      randomSelectedSegments.value = pickRandomSegments(
        options.segmentPaths.value,
        randomPickCount.value,
      );
    } catch (error) {
      randomPickError.value =
        error instanceof Error ? error.message : String(error ?? "随机抽取失败。");
    }
  }

  async function concatCategorizedSegments() {
    resetCurrentMix();
    const validationError = validateCommonMix("分类混剪失败");
    if (validationError) return;

    let categorizedPick;
    try {
      categorizedPick = pickCategorizedSegments(
        options.segmentPaths.value,
        options.segmentCategories.value,
        options.categoryOptions,
      );
    } catch (error) {
      setMixError("分类混剪失败", error, "分类混剪失败。");
      return;
    }

    randomSelectedSegments.value = categorizedPick.pickedSegments.map(({ path }) => path);
    options.appendMixLog("开始分类混剪。", "info");
    options.appendMixLog("分类顺序：开头钩子 -> 产品展示 -> 使用过程 -> 细节特写 -> 效果展示 -> 结尾引导。", "info");
    categorizedPick.pickedSegments.forEach((segment) => {
      options.appendMixLog(`${segment.label}：抽中 ${formatFileName(segment.path)}。`, "info");
    });
    if (categorizedPick.skippedCategories.length > 0) {
      options.appendMixLog(`已自动跳过空分类：${categorizedPick.skippedCategories.join("、")}。`, "info");
    }
    options.appendMixLog(`分类混剪中，${buildRunningSettingsSummary()}`, "info");
    await generateSingleMix("分类混剪", "分类混剪失败", "分类混剪成功");
  }

  async function concatRandomSegments() {
    resetCurrentMix();
    const validationError = validateCommonMix("拼接失败", false);
    if (validationError) return;

    if (randomSelectedSegments.value.length < 2) {
      mixError.value = "至少需要随机抽取 2 个片段才能拼接。";
      options.appendMixLog(`拼接失败：${mixError.value}`, "error");
      return;
    }

    const speedError = options.validatePlaybackSpeed();
    if (speedError) {
      mixError.value = speedError;
      options.appendMixLog(`拼接失败：${speedError}`, "error");
      return;
    }

    options.appendMixLog("开始拼接。", "info");
    options.appendMixLog(`拼接中，${buildRunningSettingsSummary()}`, "info");
    await generateSingleMix("拼接导出", "拼接失败", "拼接成功");
  }

  async function generateSingleMix(
    resultType: Extract<ExportResultType, "分类混剪" | "拼接导出">,
    failureLabel: string,
    successLabel: string,
  ) {
    isMixing.value = true;
    try {
      const result = await options.runTask(resultType, (task) =>
        concatSelectedSegments(
          randomSelectedSegments.value,
          options.outputDirectory.value as string,
          options.remixExportSettings.value,
          task.progress(0, 100, `正在生成${resultType}`),
        ),
      );
      recordMixResult(resultType, result.outputPath);
      options.appendMixLog(formatRemixCanvasLog(result), "info");
      options.appendMixLog(formatSmoothRemixLog(result), "info");
      options.appendMixLog(
        `${successLabel}：已使用 ${result.inputCount} 个片段生成 ${result.outputPath}${buildMixOptionSummary(options.remixExportSettings.value)}`,
        "success",
      );
    } catch (error) {
      if (isTaskCancelledError(error)) {
        mixError.value = `${resultType}已取消，可以重新开始。`;
        options.appendMixLog(mixError.value, "info");
        return;
      }
      setMixError(failureLabel, error, resultType === "分类混剪" ? "分类混剪失败。" : "片段拼接失败。");
    } finally {
      isMixing.value = false;
    }
  }

  async function generateBatchMixes() {
    batchMixError.value = null;
    batchMixResults.value = [];
    batchMixFailures.value = [];
    options.clearBatchMixLogs();

    const baseError = options.validatePictureInPicture() ?? options.validateBgm();
    if (baseError) return setBatchError(baseError);
    if (!options.outputDirectory.value) return setBatchError("请先选择输出目录。");
    if (!Number.isInteger(batchGenerateCount.value) || batchGenerateCount.value <= 0) {
      return setBatchError("批量生成数量必须大于 0。");
    }
    if (!Number.isInteger(randomPickCount.value) || randomPickCount.value < 2) {
      return setBatchError("每条混剪至少需要抽取 2 个片段。");
    }
    const speedError = options.validatePlaybackSpeed();
    if (speedError) return setBatchError(speedError);

    const entries = Array.from({ length: batchGenerateCount.value }, (_value, index) => ({
      version: index + 1,
      segmentPaths: pickRandomSegments(options.segmentPaths.value, randomPickCount.value),
    }));
    options.appendBatchMixLog(
      `开始批量生成：计划生成 ${batchGenerateCount.value} 条，${buildRunningSettingsSummary()}`,
      "info",
    );
    await runBatchMixEntries(entries, false);
  }

  async function retryFailedBatchMixes() {
    if (batchMixFailures.value.length === 0) return;
    const entries = batchMixFailures.value.map(({ version, segmentPaths }) => ({
      version,
      segmentPaths,
    }));
    options.appendBatchMixLog(`开始重试 ${entries.length} 条失败任务，成功结果不会重复生成。`, "info");
    await runBatchMixEntries(entries, true);
  }

  async function runBatchMixEntries(entries: BatchMixEntry[], preserveResults: boolean) {
    if (!preserveResults) {
      batchMixResults.value = [];
    }
    batchMixFailures.value = [];
    batchMixError.value = null;
    isBatchMixing.value = true;

    try {
      await options.runTask(
        preserveResults ? "重试失败的批量视频" : "随机批量生成",
        async (task) => {
          for (const [index, entry] of entries.entries()) {
            task.throwIfCancelled();
            options.appendBatchMixLog(
              `正在生成第 ${entry.version} 条，使用 ${entry.segmentPaths.length} 个随机片段。`,
              "info",
            );
            try {
              const result = await concatSelectedSegments(
                entry.segmentPaths,
                options.outputDirectory.value as string,
                options.remixExportSettings.value,
                task.progress(
                  (index / entries.length) * 100,
                  ((index + 1) / entries.length) * 100,
                  `正在生成批量视频 ${index + 1}/${entries.length}`,
                ),
              );
              batchMixResults.value.push(result.outputPath);
              options.addExportResult("批量生成", result.outputPath);
              options.appendBatchMixLog(formatRemixCanvasLog(result), "info");
              options.appendBatchMixLog(formatSmoothRemixLog(result), "info");
              options.appendBatchMixLog(
                `第 ${entry.version} 条生成成功：${result.outputPath}${buildMixOptionSummary(options.remixExportSettings.value)}`,
                "success",
              );
            } catch (error) {
              if (isTaskCancelledError(error)) throw error;
              const message = error instanceof Error ? error.message : String(error ?? "批量生成失败。");
              batchMixFailures.value.push({ ...entry, message });
              options.appendBatchMixLog(`第 ${entry.version} 条生成失败，继续下一条：${message}`, "error");
            }
          }
        },
      );

      const failureCount = batchMixFailures.value.length;
      options.appendBatchMixLog(
        `批量生成结束：成功 ${entries.length - failureCount} 条，失败 ${failureCount} 条。`,
        failureCount > 0 ? "error" : "success",
      );
      batchMixError.value = failureCount > 0 ? `${failureCount} 条生成失败，可以单独重试失败项。` : null;
    } catch (error) {
      if (isTaskCancelledError(error)) {
        batchMixError.value = "批量任务已取消，已完成的结果会保留。";
        options.appendBatchMixLog(batchMixError.value, "info");
      } else {
        setBatchError(error instanceof Error ? error.message : String(error ?? "批量生成失败。"));
      }
    } finally {
      isBatchMixing.value = false;
    }
  }

  function validateCommonMix(label: string, includePlaybackSpeed = true) {
    const validationError =
      options.validatePictureInPicture() ??
      options.validateBgm() ??
      (!options.outputDirectory.value ? "请先选择输出目录。" : null) ??
      (includePlaybackSpeed ? options.validatePlaybackSpeed() : null);
    if (validationError) {
      mixError.value = validationError;
      options.appendMixLog(`${label}：${validationError}`, "error");
    }
    return validationError;
  }

  function buildRunningSettingsSummary() {
    const settings = options.remixExportSettings.value;
    return `变速倍数 ${settings.playbackSpeed.toFixed(2)}x，平滑混剪${settings.smoothRemixEnabled ? "已开启" : "未开启"}，BGM${settings.bgmSettings.enabled ? "已开启" : "未开启"}。`;
  }

  function resetCurrentMix() {
    mixError.value = null;
    mixResultPath.value = null;
    options.clearMixLogs();
  }

  function resetRandomPickState() {
    randomPickError.value = null;
    randomSelectedSegments.value = [];
    resetMixState();
  }

  function resetMixState() {
    isMixing.value = false;
    mixError.value = null;
    mixResultPath.value = null;
    options.clearMixLogs();
    resetBatchMixState();
  }

  function resetBatchMixState() {
    isBatchMixing.value = false;
    batchMixError.value = null;
    batchMixResults.value = [];
    batchMixFailures.value = [];
    options.clearBatchMixLogs();
  }

  function recordMixResult(type: ExportResultType, path: string) {
    mixResultPath.value = path;
    options.addExportResult(type, path);
  }

  function setMixing(value: boolean) {
    isMixing.value = value;
  }

  function setMixError(label: string, error: unknown, fallback: string) {
    mixError.value = error instanceof Error ? error.message : String(error ?? fallback);
    options.appendMixLog(`${label}：${mixError.value}`, "error");
  }

  function setBatchError(message: string) {
    batchMixError.value = message;
    options.appendBatchMixLog(`批量生成失败：${message}`, "error");
  }

  return {
    batchGenerateCount,
    batchMixError,
    batchMixFailures,
    batchMixResults,
    concatCategorizedSegments,
    concatRandomSegments,
    generateBatchMixes,
    isBatchMixing,
    isMixing,
    mixError,
    mixResultPath,
    pickSegmentsRandomly,
    randomPickCount,
    randomPickError,
    randomSelectedSegments,
    recordMixResult,
    resetRandomPickState,
    retryFailedBatchMixes,
    setMixing,
  };
}

function formatFileName(filePath: string) {
  return filePath.split(/[\\/]/).pop() ?? filePath;
}
