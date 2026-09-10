import { ref } from "vue";
import type { Ref } from "vue";
import { concatSelectedSegments } from "../../services/videoMixService";
import type {
  RemixSegmentInput,
  RemixExportSettings,
  SegmentCategory,
  SegmentCategoryOption,
} from "../../services/videoMixService";
import type { ExportResultType, TaskLogLevel } from "../../types/workbench";
import type { ImportedVideo } from "../../types/videoProbe";
import type { MaterialFolder, MaterialFolderVideoMode } from "../materials/types";
import { isTaskCancelledError } from "../task-center";
import type { TaskRunHandle } from "../task-center";
import { buildMixOptionSummary, formatRemixCanvasLog, formatSmoothRemixLog } from "./remixResultMessages";
import { pickCategorizedSegments, pickRandomSegments } from "./services/remixExportService";
import { createAiRemixOutputDirectory } from "../../services/fileManagerService";
import { getTtsConfigStatus, synthesizeTts } from "../tts/services/ttsService";

interface UseRemixGenerationOptions {
  importedVideos: Readonly<Ref<ImportedVideo[]>>;
  materialFolders: Readonly<Ref<MaterialFolder[]>>;
  batchScript: Readonly<Ref<string>>;
  asrSourceFilePath: Readonly<Ref<string | null>>;
  outputDirectory: Readonly<Ref<string | null>>;
  segmentPaths: Readonly<Ref<string[]>>;
  segmentCategories: Readonly<Ref<Record<string, SegmentCategory | "">>>;
  categoryOptions: SegmentCategoryOption[];
  remixExportSettings: Readonly<Ref<RemixExportSettings>>;
  validatePictureInPicture: () => string | null;
  validateBgm: () => string | null;
  validateWatermark: () => string | null;
  validateWatermarkRemoval: () => string | null;
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
  segmentInputs?: RemixSegmentInput[];
  settings?: RemixExportSettings;
  scriptLabel?: string;
  outputDirectory?: string;
  synthesizeNarration?: boolean;
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
      categorizedPick = options.materialFolders.value.length >= 2
        ? pickFolderSegmentsForCategory(options.materialFolders.value)
        : pickCategorizedSegments(
            options.segmentPaths.value,
            options.segmentCategories.value,
            options.categoryOptions,
          );
    } catch (error) {
      setMixError("分类混剪失败", error, "分类混剪失败。");
      return;
    }

    randomSelectedSegments.value = categorizedPick.pickedSegments.map(({ path }) => path);
    const segmentInputs = options.materialFolders.value.length >= 2
      ? buildFolderSegmentInputs(options.materialFolders.value, randomSelectedSegments.value, 0)
      : undefined;
    options.appendMixLog("开始分类混剪。", "info");
    options.appendMixLog("分类顺序：开头钩子 -> 产品展示 -> 使用过程 -> 细节特写 -> 效果展示 -> 结尾引导。", "info");
    categorizedPick.pickedSegments.forEach((segment) => {
      options.appendMixLog(`${segment.label}：抽中 ${formatFileName(segment.path)}。`, "info");
    });
    if (categorizedPick.skippedCategories.length > 0) {
      options.appendMixLog(`已自动跳过空分类：${categorizedPick.skippedCategories.join("、")}。`, "info");
    }
    options.appendMixLog(`分类混剪中，${buildRunningSettingsSummary()}`, "info");
    await generateSingleMix("分类混剪", "分类混剪失败", "分类混剪成功", segmentInputs);
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
    segmentInputs?: RemixSegmentInput[],
  ) {
    isMixing.value = true;
    try {
      const result = await options.runTask(resultType, (task) =>
        concatSelectedSegments(
          randomSelectedSegments.value,
          options.outputDirectory.value as string,
          options.remixExportSettings.value,
          task.progress(0, 100, `正在生成${resultType}`),
          segmentInputs,
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

    const mode = resolveBatchVideoMode(options.materialFolders.value);
    const baseError =
      options.validatePictureInPicture() ??
      (mode === "audio" ? null : options.validateBgm()) ??
      options.validateWatermark();
    const removalError = options.validateWatermarkRemoval();
    if (removalError) return setBatchError(removalError);
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

    const scripts = mode === "custom" ? [] : splitBatchScripts(options.batchScript.value);
    if (mode !== "custom" && scripts.length === 0) {
      return setBatchError(mode === "audio" ? "请先完成音频识别并使用识别结果。" : "请先输入至少一条文案。");
    }
    if (mode === "audio" && !options.asrSourceFilePath.value) {
      return setBatchError("音频模式请先选择要混入成片的音频或带声音视频。");
    }
    let ttsSpeaker: string | null = null;
    if (mode === "script") {
      try {
        const ttsConfig = await getTtsConfigStatus();
        if (!ttsConfig.configured) {
          return setBatchError("文案模式需要真实 AI 配音，请先在 API 密钥中完成 TTS 配置。");
        }
        ttsSpeaker = ttsConfig.speaker.trim();
        if (!ttsSpeaker) return setBatchError("文案模式未找到可用音色，请先完成 TTS 配置。");
      } catch (error) {
        return setBatchError(error instanceof Error ? error.message : "读取 TTS 配置失败。");
      }
    }
    const folderSettingsError = validateFolderBatchSettings(options.materialFolders.value, mode);
    if (folderSettingsError) return setBatchError(folderSettingsError);
    const plannedCount = resolveFolderOutputCount(
      options.materialFolders.value,
      batchGenerateCount.value,
      mode,
      scripts.length,
    );
    let outputDirectories: Map<string, string>;
    try {
      outputDirectories = await resolveFolderOutputDirectories(
        options.materialFolders.value,
        options.outputDirectory.value,
        scripts,
      );
    } catch (error) {
      return setBatchError(error instanceof Error ? error.message : "创建分类输出目录失败。");
    }
    const entries = Array.from({ length: plannedCount }, (_value, index) => {
      const segmentPaths = options.materialFolders.value.length > 0
        ? pickFolderSegmentsForBatch(options.materialFolders.value, index)
        : pickRandomSegments(options.segmentPaths.value, randomPickCount.value);
      return {
        version: index + 1,
        segmentPaths,
        segmentInputs: options.materialFolders.value.length > 0
          ? buildFolderSegmentInputs(options.materialFolders.value, segmentPaths, index)
          : undefined,
        settings: mode === "custom"
          ? undefined
          : buildModeExportSettings(
              options.remixExportSettings.value,
              mode,
              scripts[index % scripts.length],
              options.asrSourceFilePath.value,
            ),
        scriptLabel: mode === "custom" ? undefined : scripts[index % scripts.length],
        synthesizeNarration: mode === "script",
        outputDirectory: outputDirectories.get(scripts[index % scripts.length] ?? "")
          ?? outputDirectories.get("")
          ?? options.outputDirectory.value as string,
      };
    });
    options.appendBatchMixLog(
      `开始批量生成：计划生成 ${plannedCount} 条，${mode === "custom" ? "自定义选材" : mode === "audio" ? "音频识别字幕与原音混入" : "文案字幕成片"}，${buildRunningSettingsSummary()}`,
      "info",
    );
    await runBatchMixEntries(entries, false, ttsSpeaker);
  }

  async function retryFailedBatchMixes() {
    if (batchMixFailures.value.length === 0) return;
    const entries = batchMixFailures.value.map(({ version, segmentPaths, segmentInputs, settings, scriptLabel, outputDirectory, synthesizeNarration }) => ({
      version,
      segmentPaths,
      segmentInputs,
      settings,
      scriptLabel,
      outputDirectory,
      synthesizeNarration,
    }));
    let ttsSpeaker: string | null = null;
    if (entries.some((entry) => entry.synthesizeNarration)) {
      const ttsConfig = await getTtsConfigStatus();
      if (!ttsConfig.configured || !ttsConfig.speaker.trim()) {
        return setBatchError("重试文案模式任务前，请先完成 TTS 配置。");
      }
      ttsSpeaker = ttsConfig.speaker.trim();
    }
    options.appendBatchMixLog(`开始重试 ${entries.length} 条失败任务，成功结果不会重复生成。`, "info");
    await runBatchMixEntries(entries, true, ttsSpeaker);
  }

  async function runBatchMixEntries(entries: BatchMixEntry[], preserveResults: boolean, configuredTtsSpeaker: string | null) {
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
          const workerCount = resolveBatchWorkerCount(
            options.remixExportSettings.value.outputSettings.threadMode,
            entries.length,
          );
          let nextEntryIndex = 0;
          let completedEntryCount = 0;
          options.appendBatchMixLog(`本次批量混剪使用 ${workerCount} 个处理线程。`, "info");
          const worker = async () => {
            while (true) {
              task.throwIfCancelled();
              const index = nextEntryIndex++;
              const entry = entries[index];
              if (!entry) return;
              options.appendBatchMixLog(
                `正在生成第 ${entry.version} 条，使用 ${entry.segmentPaths.length} 个随机片段。`,
                "info",
              );
              try {
                let settings = entry.settings ?? options.remixExportSettings.value;
                if (entry.synthesizeNarration) {
                  if (!configuredTtsSpeaker) throw new Error("TTS 配置已失效，请重新提交文案模式任务。");
                  if (!entry.scriptLabel) throw new Error("文案模式缺少本条视频的文案。");
                  options.appendBatchMixLog(`第 ${entry.version} 条正在生成 AI 配音。`, "info");
                  const narration = await synthesizeTts(
                    entry.scriptLabel,
                    entry.outputDirectory ?? options.outputDirectory.value as string,
                    configuredTtsSpeaker,
                  );
                  settings = {
                    ...settings,
                    bgmSettings: {
                      ...settings.bgmSettings,
                      enabled: true,
                      audioFilePath: narration.outputPath,
                      originalVolume: 0,
                      originalVolumeMin: 0,
                      originalVolumeMax: 0,
                      originalFadeEnabled: false,
                      dynamicAdjustEnabled: false,
                      bgmVolume: 1,
                      bgmVolumeMin: 1,
                      bgmVolumeMax: 1,
                      bgmFadeEnabled: false,
                      loopPlaybackEnabled: false,
                      fadeInSeconds: 0,
                      fadeOutSeconds: 0,
                    },
                  };
                }
                const result = await concatSelectedSegments(
                  entry.segmentPaths,
                  entry.outputDirectory ?? options.outputDirectory.value as string,
                  settings,
                  task.progress(
                    (index / entries.length) * 100,
                    ((index + 1) / entries.length) * 100,
                    `正在生成批量视频 ${index + 1}/${entries.length}`,
                  ),
                  entry.segmentInputs,
                );
                batchMixResults.value.push(result.outputPath);
                completedEntryCount += 1;
                options.addExportResult("批量生成", result.outputPath);
                options.appendBatchMixLog(formatRemixCanvasLog(result), "info");
                options.appendBatchMixLog(formatSmoothRemixLog(result), "info");
                options.appendBatchMixLog(
                  `第 ${entry.version} 条生成成功：${result.outputPath}${entry.scriptLabel ? `；${entry.synthesizeNarration ? "已生成 AI 配音并" : "已"}写入文案“${shortenScript(entry.scriptLabel)}”` : ""}${buildMixOptionSummary(settings)}`,
                  "success",
                );
                if (workerCount > 1) {
                  await task.update(
                    (completedEntryCount / entries.length) * 100,
                    `已完成批量视频 ${completedEntryCount}/${entries.length}`,
                  );
                }
              } catch (error) {
                if (isTaskCancelledError(error)) throw error;
                const message = error instanceof Error ? error.message : String(error ?? "批量生成失败。");
                batchMixFailures.value.push({ ...entry, message });
                options.appendBatchMixLog(`第 ${entry.version} 条生成失败，继续下一条：${message}`, "error");
              }
            }
          };
          await Promise.all(Array.from({ length: workerCount }, () => worker()));
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
      options.validateWatermark() ??
      options.validateWatermarkRemoval() ??
      (!options.outputDirectory.value ? "请先选择输出目录。" : null) ??
      (includePlaybackSpeed ? options.validatePlaybackSpeed() : null);
    if (validationError) {
      mixError.value = validationError;
      options.appendMixLog(`${label}：${validationError}`, "error");
    }
    return validationError;
  }

  function pickFolderSegmentsForBatch(folders: MaterialFolder[], versionIndex: number) {
    const selected: string[] = [];
    const usedPaths = new Set<string>();

    for (const folder of folders) {
      const candidates = folder.videoPaths.filter((path) =>
        options.importedVideos.value.some((video) => video.filePath === path),
      );
      if (candidates.length === 0) continue;

      const requestedCount = Math.max(1, Math.floor(folder.settings.materialCount || 1));
      const ordered = folder.settings.extractionOrder === "ordered";
      const allowRepeat = folder.settings.allowMaterialRepeat;
      const available = allowRepeat
        ? candidates
        : candidates.filter((path) => !usedPaths.has(path));
      if (available.length === 0) continue;

      const fixedFirstMaterial = resolveFixedFirstMaterial(folder, candidates);
      if (fixedFirstMaterial) {
        selected.push(fixedFirstMaterial);
        if (!allowRepeat) usedPaths.add(fixedFirstMaterial);
      }

      const fixedMaterialOffset = fixedFirstMaterial && folder.settings.fixedMaterialMode === "default" ? 1 : 0;
      for (let offset = fixedMaterialOffset; offset < requestedCount; offset += 1) {
        const pool = allowRepeat ? candidates : available.filter((path) => !usedPaths.has(path));
        if (pool.length === 0) break;
        const path = ordered
          ? pool[(versionIndex + offset) % pool.length]
          : pool[Math.floor(Math.random() * pool.length)];
        selected.push(path);
        if (!allowRepeat) usedPaths.add(path);
      }
    }

    if (selected.length < 2) {
      throw new Error("文件夹规则至少需要抽取 2 个可用视频素材，才能生成混剪。");
    }
    return selected;
  }

  function buildFolderSegmentInputs(
    folders: MaterialFolder[],
    paths: string[],
    versionIndex: number,
  ) {
    const folderByPath = new Map<string, MaterialFolder>();
    for (const folder of folders) {
      for (const path of folder.videoPaths) folderByPath.set(path, folder);
    }

    return paths.map((path) => {
      const folder = folderByPath.get(path);
      const video = options.importedVideos.value.find((item) => item.filePath === path);
      if (!folder || !video || !hasClipRange(folder)) return { path };
      const clip = resolveClipRange(folder, video.durationSeconds, versionIndex);
      return clip ? { path, ...clip } : { path };
    });
  }

  function pickFolderSegmentsForCategory(folders: MaterialFolder[]) {
    const pickedSegments = folders.flatMap((folder) => {
      const candidates = folder.videoPaths.filter((path) =>
        options.importedVideos.value.some((video) => video.filePath === path),
      );
      if (candidates.length === 0) return [];
      const count = Math.max(1, Math.floor(folder.settings.materialCount || 1));
      const ordered = folder.settings.extractionOrder === "ordered";
      return Array.from({ length: Math.min(count, candidates.length) }, (_value, index) => {
        const path = ordered
          ? candidates[index]
          : candidates[Math.floor(Math.random() * candidates.length)];
        return { category: "environment" as SegmentCategory, label: folder.folderName, path };
      });
    });

    if (pickedSegments.length < 2) {
      throw new Error("至少需要 2 个分类文件夹各有可用素材，才能生成分类混剪。");
    }
    return { pickedSegments, skippedCategories: [] };
  }

  function buildRunningSettingsSummary() {
    const settings = options.remixExportSettings.value;
    return `变速倍数 ${settings.playbackSpeed.toFixed(2)}x，平滑混剪${settings.smoothRemixEnabled ? "已开启" : "未开启"}，BGM${settings.bgmSettings.enabled ? "已开启" : "未开启"}，添加水印${settings.watermarkSettings.enabled ? "已开启" : "未开启"}，原水印处理${settings.watermarkRemovalSettings.enabled ? "已开启" : "未开启"}。`;
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

function hasClipRange(folder: MaterialFolder) {
  return folder.settings.clipMinSeconds > 0 || folder.settings.clipMaxSeconds > 0;
}

function resolveClipRange(
  folder: MaterialFolder,
  sourceDurationSeconds: number | null,
  versionIndex: number,
): Pick<RemixSegmentInput, "startSeconds" | "durationSeconds"> | null {
  const sourceDuration = sourceDurationSeconds ?? 0;
  if (!Number.isFinite(sourceDuration) || sourceDuration <= 0.5) return null;

  const min = Math.max(0, folder.settings.clipMinSeconds);
  const max = Math.max(0, folder.settings.clipMaxSeconds);
  if (max > 0 && min > max) {
    throw new Error(`${folder.folderName} 的素材截取范围中，最短时长不能大于最长时长。`);
  }

  const minimumDuration = Math.min(sourceDuration, Math.max(0.5, min || 0.5));
  const maximumDuration = Math.min(sourceDuration, max > 0 ? max : minimumDuration);
  if (maximumDuration < minimumDuration) {
    throw new Error(`${folder.folderName} 的素材截取范围超过了素材自身时长。`);
  }

  const duration = maximumDuration === minimumDuration
    ? maximumDuration
    : minimumDuration + Math.random() * (maximumDuration - minimumDuration);
  const maxStart = Math.max(0, sourceDuration - duration);
  const startSeconds = maxStart === 0
    ? 0
    : ((versionIndex + Math.random()) * 997 % 1) * maxStart;
  return {
    startSeconds: Number(startSeconds.toFixed(3)),
    durationSeconds: Number(duration.toFixed(3)),
  };
}

function resolveFolderOutputCount(
  folders: MaterialFolder[],
  fallback: number,
  mode: MaterialFolderVideoMode,
  scriptCount: number,
) {
  if (folders.length === 0) return fallback;
  const settings = folders[0].settings;
  if (mode === "custom") return Math.max(1, settings.exportCount);
  return Math.max(1, scriptCount) * Math.max(1, settings.variantCount);
}

function validateFolderBatchSettings(
  folders: MaterialFolder[],
  mode: MaterialFolderVideoMode,
) {
  if (folders.length <= 1) return null;
  const first = folders[0].settings;
  for (const folder of folders.slice(1)) {
    if (folder.settings.videoMode !== first.videoMode) {
      return "多个素材文件夹的成片模式不一致，请先使用“应用全部”统一为自定义、文案或音频模式。";
    }
    if (folder.settings.outputMode !== first.outputMode) {
      return "多个素材文件夹的视频输出方式不一致，请先统一为按文案分类或不分类。";
    }
    if (mode === "custom" && folder.settings.exportCount !== first.exportCount) {
      return "多个素材文件夹的混剪导出数量不一致，请先统一数量后再创建任务。";
    }
    if (mode !== "custom" && folder.settings.variantCount !== first.variantCount) {
      return "多个素材文件夹的每文案裂变数量不一致，请先统一数量后再创建任务。";
    }
  }
  return null;
}

function resolveBatchWorkerCount(mode: RemixExportSettings["outputSettings"]["threadMode"], jobCount: number) {
  if (mode === "single") return 1;
  const available = typeof navigator === "undefined" ? 2 : navigator.hardwareConcurrency || 2;
  const capped = Math.max(1, Math.min(4, Math.floor(available)));
  return Math.min(jobCount, mode === "multi" ? Math.max(2, capped) : capped);
}

function resolveBatchVideoMode(folders: MaterialFolder[]): MaterialFolderVideoMode {
  const modes = new Set(folders.map((folder) => folder.settings.videoMode));
  if (modes.has("audio")) return "audio";
  if (modes.has("script")) return "script";
  return "custom";
}

function splitBatchScripts(value: string) {
  return value
    .split(/\r?\n\s*\r?\n/)
    .map((item) => item.trim())
    .filter(Boolean);
}

function buildModeExportSettings(
  settings: RemixExportSettings,
  mode: Exclude<MaterialFolderVideoMode, "custom">,
  script: string,
  audioSourceFilePath: string | null,
): RemixExportSettings {
  return {
    ...settings,
    subtitleSettings: {
      ...settings.subtitleSettings,
      enabled: true,
      text: script,
    },
    bgmSettings: mode === "audio"
      ? {
          ...settings.bgmSettings,
          enabled: true,
          audioFilePath: audioSourceFilePath,
          originalVolume: 0,
          originalVolumeMin: 0,
          originalVolumeMax: 0,
          originalFadeEnabled: false,
          dynamicAdjustEnabled: false,
          bgmVolume: 1,
          bgmVolumeMin: 1,
          bgmVolumeMax: 1,
          bgmFadeEnabled: false,
          loopPlaybackEnabled: false,
          fadeInSeconds: 0,
          fadeOutSeconds: 0,
        }
      : settings.bgmSettings,
  };
}

function shortenScript(script: string) {
  return script.length > 24 ? `${script.slice(0, 24)}…` : script;
}

async function resolveFolderOutputDirectories(
  folders: MaterialFolder[],
  rootDirectory: string,
  scripts: string[],
) {
  const directories = new Map<string, string>();
  if (folders[0]?.settings.outputMode !== "byScript") {
    directories.set("", rootDirectory);
    return directories;
  }
  if (scripts.length === 0) {
    directories.set("", await createAiRemixOutputDirectory(rootDirectory, "批量混剪成片"));
    return directories;
  }
  for (const [index, script] of [...new Set(scripts)].entries()) {
    directories.set(
      script,
      await createAiRemixOutputDirectory(
        rootDirectory,
        `文案_${String(index + 1).padStart(2, "0")}_${shortenScript(script)}`,
      ),
    );
  }
  return directories;
}

function resolveFixedFirstMaterial(folder: MaterialFolder, candidates: string[]) {
  if (!folder.settings.fixedFirstMaterialEnabled || !folder.settings.fixedFirstMaterialPath) return null;
  if (folder.settings.fixedFirstMaterialKind === "file") return folder.settings.fixedFirstMaterialPath;
  const folderPath = folder.settings.fixedFirstMaterialPath.replace(/[\\/]+$/, "").toLowerCase();
  return candidates.find((path) => path.toLowerCase().startsWith(folderPath)) ?? null;
}

function formatFileName(filePath: string) {
  return filePath.split(/[\\/]/).pop() ?? filePath;
}
