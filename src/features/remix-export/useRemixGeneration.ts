import { ref } from "vue";
import type { Ref } from "vue";
import { concatSelectedSegments } from "../../services/videoMixService";
import type {
  RemixExportSettings,
  SegmentCategory,
  SegmentCategoryOption,
} from "../../services/videoMixService";
import type { ExportResultType, TaskLogLevel } from "../../types/workbench";
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
      const result = await concatSelectedSegments(
        randomSelectedSegments.value,
        options.outputDirectory.value as string,
        options.remixExportSettings.value,
      );
      recordMixResult(resultType, result.outputPath);
      options.appendMixLog(formatRemixCanvasLog(result), "info");
      options.appendMixLog(formatSmoothRemixLog(result), "info");
      options.appendMixLog(
        `${successLabel}：已使用 ${result.inputCount} 个片段生成 ${result.outputPath}${buildMixOptionSummary(options.remixExportSettings.value)}`,
        "success",
      );
    } catch (error) {
      setMixError(failureLabel, error, resultType === "分类混剪" ? "分类混剪失败。" : "片段拼接失败。");
    } finally {
      isMixing.value = false;
    }
  }

  async function generateBatchMixes() {
    batchMixError.value = null;
    batchMixResults.value = [];
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

    options.appendBatchMixLog(
      `开始批量生成：计划生成 ${batchGenerateCount.value} 条，${buildRunningSettingsSummary()}`,
      "info",
    );
    isBatchMixing.value = true;

    try {
      for (let index = 0; index < batchGenerateCount.value; index += 1) {
        const pickedSegments = pickRandomSegments(
          options.segmentPaths.value,
          randomPickCount.value,
        );
        options.appendBatchMixLog(
          `正在生成第 ${index + 1} 条，使用 ${pickedSegments.length} 个随机片段。`,
          "info",
        );
        const result = await concatSelectedSegments(
          pickedSegments,
          options.outputDirectory.value,
          options.remixExportSettings.value,
        );
        batchMixResults.value.push(result.outputPath);
        options.addExportResult("批量生成", result.outputPath);
        options.appendBatchMixLog(formatRemixCanvasLog(result), "info");
        options.appendBatchMixLog(formatSmoothRemixLog(result), "info");
        options.appendBatchMixLog(
          `第 ${index + 1} 条生成成功：${result.outputPath}${buildMixOptionSummary(options.remixExportSettings.value)}`,
          "success",
        );
      }
      options.appendBatchMixLog(`批量生成完成：共生成 ${batchMixResults.value.length} 条。`, "success");
    } catch (error) {
      setBatchError(error instanceof Error ? error.message : String(error ?? "批量生成失败。"));
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
    setMixing,
  };
}

function formatFileName(filePath: string) {
  return filePath.split(/[\\/]/).pop() ?? filePath;
}
