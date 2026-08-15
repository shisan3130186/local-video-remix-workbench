import { ref } from "vue";
import type { Ref } from "vue";
import type { RemixExportSettings } from "../../services/videoMixService";
import type { ImportedVideo } from "../../types/videoProbe";
import type { TaskLogLevel, VideoProcessingState } from "../../types/workbench";
import { isTaskCancelledError } from "../task-center";
import type { TaskRunHandle } from "../task-center";
import { deleteSourceVideoFile, exportCoverImage, exportCurrentVideo } from "./services/remixExportService";

interface UseBasicExportOptions {
  importedVideos: Readonly<Ref<ImportedVideo[]>>;
  selectedVideo: Readonly<Ref<ImportedVideo | null>>;
  outputDirectory: Readonly<Ref<string | null>>;
  remixExportSettings: Readonly<Ref<RemixExportSettings>>;
  coverImagePath?: Readonly<Ref<string | null>>;
  validatePictureInPicture: () => string | null;
  validateBgm: () => string | null;
  validatePlaybackSpeed: () => string | null;
  validateWatermark: () => string | null;
  validateWatermarkRemoval: () => string | null;
  appendExportLog: (message: string, level: TaskLogLevel) => void;
  clearExportLogs: () => void;
  addExportResult: (type: "基础导出", path: string) => void;
  runTask: <T>(label: string, runner: (task: TaskRunHandle) => Promise<T>) => Promise<T>;
}

export function useBasicExport(options: UseBasicExportOptions) {
  const isExporting = ref(false);
  const exportError = ref<string | null>(null);
  const exportResultPath = ref<string | null>(null);
  const videoProcessingStates = ref<Record<string, VideoProcessingState>>({});
  const activeProcessingVideoId = ref<string | null>(null);

  async function exportSelectedVideo() {
    exportError.value = null;
    exportResultPath.value = null;
    options.clearExportLogs();
    const video = options.selectedVideo.value;

    if (!video) return fail("请先选择一个要导出的视频。", "导出失败");
    if (!options.outputDirectory.value) return fail("请先选择输出目录。", "导出失败");
    const validationError = validateSettings(options);
    if (validationError) return fail(validationError, "导出失败");
    if (!options.remixExportSettings.value.outputSettings.keepOriginal && !confirmSourceRemoval()) {
      options.appendExportLog("已取消导出：未确认删除原文件。", "info");
      return;
    }

    isExporting.value = true;
    try {
      const variantCount = normalizeVariantCount(options.remixExportSettings.value.outputSettings.variantCount);
      const verifiedOutputPaths: string[] = [];
      const result = await options.runTask("导出当前视频", async (task) => {
        let lastResult: Awaited<ReturnType<typeof exportCurrentVideo>> | null = null;
        for (let variantIndex = 0; variantIndex < variantCount; variantIndex += 1) {
          task.throwIfCancelled();
          lastResult = await exportCurrentVideo(
            video.filePath,
            options.outputDirectory.value as string,
            video.durationSeconds,
            buildJobSettings(options.remixExportSettings.value, {
              video,
              index: variantIndex,
              variantIndex,
              totalVariants: variantCount,
            }),
            task.progress((variantIndex / variantCount) * 100, ((variantIndex + 1) / variantCount) * 100, `正在导出版本 ${variantIndex + 1}/${variantCount}`),
          );
          exportResultPath.value = lastResult.outputPath;
          verifiedOutputPaths.push(lastResult.outputPath);
          options.addExportResult("基础导出", lastResult.outputPath);
          options.appendExportLog(`导出成功：${lastResult.outputPath}`, "success");
          await exportCoverForResult(options, lastResult.outputPath);
        }
        return lastResult;
      });
      if (result && !options.remixExportSettings.value.outputSettings.keepOriginal) {
        await deleteSourceVideoFile(video.filePath, verifiedOutputPaths);
        options.appendExportLog(`已按“不保留原文件”删除：${video.fileName}`, "info");
      }
    } catch (error) {
      handleExportError(error, exportError, options.appendExportLog, "导出");
    } finally {
      isExporting.value = false;
    }
  }

  async function exportImportedVideos() {
    exportError.value = null;
    exportResultPath.value = null;
    options.clearExportLogs();
    const videos = [...options.importedVideos.value];

    if (videos.length === 0) return fail("请先导入至少一个视频。", "批量处理失败");
    if (!options.outputDirectory.value) return fail("请先选择输出目录。", "批量处理失败");
    const validationError = validateSettings(options);
    if (validationError) return fail(validationError, "批量处理失败");
    if (!options.remixExportSettings.value.outputSettings.keepOriginal && !confirmSourceRemoval()) {
      options.appendExportLog("已取消批量处理：未确认删除原文件。", "info");
      return;
    }

    const jobs = buildExportJobs(videos, options.remixExportSettings.value);
    const workerCount = resolveWorkerCount(options.remixExportSettings.value.outputSettings.threadMode, jobs.length);
    const completedByVideo = new Map<string, number>();
    const completedOutputPathsByVideo = new Map<string, string[]>();
    const failedVideoIds = new Set<string>();
    videoProcessingStates.value = Object.fromEntries(videos.map((video) => [video.id, { status: "pending", progress: 0, message: "等待处理" }]));
    activeProcessingVideoId.value = null;
    isExporting.value = true;
    options.appendExportLog(`开始处理 ${jobs.length} 个导出任务，使用 ${workerCount} 个处理线程。`, "info");

    try {
      await options.runTask("批量处理视频效果", async (task) => {
        let nextJobIndex = 0;
        let completedJobCount = 0;
        const worker = async () => {
          while (true) {
            task.throwIfCancelled();
            const job = jobs[nextJobIndex++];
            if (!job) return;
            activeProcessingVideoId.value = job.video.id;
            const completedVariants = completedByVideo.get(job.video.id) ?? 0;
            videoProcessingStates.value = { ...videoProcessingStates.value, [job.video.id]: { status: "processing", progress: Math.round((completedVariants / job.totalVariants) * 100), message: `正在处理第 ${job.variantIndex + 1}/${job.totalVariants} 个版本` } };
            try {
              const result = await exportCurrentVideo(job.video.filePath, options.outputDirectory.value as string, job.video.durationSeconds, buildJobSettings(options.remixExportSettings.value, job), workerCount === 1 ? task.progress((job.index / jobs.length) * 100, ((job.index + 1) / jobs.length) * 100, `正在处理 ${job.index + 1}/${jobs.length}`) : undefined);
              completedByVideo.set(job.video.id, completedVariants + 1);
              completedOutputPathsByVideo.set(job.video.id, [
                ...(completedOutputPathsByVideo.get(job.video.id) ?? []),
                result.outputPath,
              ]);
              completedJobCount += 1;
              const done = completedVariants + 1 === job.totalVariants;
              videoProcessingStates.value = { ...videoProcessingStates.value, [job.video.id]: { status: done ? "completed" : "processing", progress: Math.round(((completedVariants + 1) / job.totalVariants) * 100), message: done ? "处理完成" : `已完成 ${completedVariants + 1}/${job.totalVariants} 个版本` } };
              options.addExportResult("基础导出", result.outputPath);
              options.appendExportLog(`处理完成：${result.outputPath}`, "success");
              await exportCoverForResult(options, result.outputPath);
              if (workerCount > 1) await task.update((completedJobCount / jobs.length) * 100, `已完成 ${completedJobCount}/${jobs.length} 个导出任务`);
            } catch (error) {
              if (isTaskCancelledError(error)) throw error;
              failedVideoIds.add(job.video.id);
              const message = error instanceof Error ? error.message : String(error ?? "视频处理失败。");
              videoProcessingStates.value = { ...videoProcessingStates.value, [job.video.id]: { status: "failed", progress: 0, message } };
              options.appendExportLog(`处理失败：${job.video.fileName}，${message}`, "error");
            }
          }
        };
        await Promise.all(Array.from({ length: workerCount }, () => worker()));
        if (!options.remixExportSettings.value.outputSettings.keepOriginal) {
          for (const video of videos) {
            const expected = jobs.filter((job) => job.video.id === video.id).length;
            if (!failedVideoIds.has(video.id) && completedByVideo.get(video.id) === expected) {
              await deleteSourceVideoFile(
                video.filePath,
                completedOutputPathsByVideo.get(video.id) ?? [],
              );
              options.appendExportLog(`已按“不保留原文件”删除：${video.fileName}`, "info");
            }
          }
        }
      });
    } catch (error) {
      handleExportError(error, exportError, options.appendExportLog, "批量处理");
      if (isTaskCancelledError(error) && activeProcessingVideoId.value) videoProcessingStates.value = { ...videoProcessingStates.value, [activeProcessingVideoId.value]: { status: "cancelled", progress: videoProcessingStates.value[activeProcessingVideoId.value]?.progress ?? 0, message: "已取消" } };
    } finally {
      activeProcessingVideoId.value = null;
      isExporting.value = false;
    }
  }

  function fail(message: string, prefix: string) {
    exportError.value = message;
    options.appendExportLog(`${prefix}：${message}`, "error");
  }

  return { activeProcessingVideoId, exportError, exportImportedVideos, exportResultPath, exportSelectedVideo, isExporting, videoProcessingStates };
}

interface ExportJob { video: ImportedVideo; index: number; variantIndex: number; totalVariants: number }

function buildExportJobs(videos: ImportedVideo[], settings: RemixExportSettings): ExportJob[] {
  const totalVariants = normalizeVariantCount(settings.outputSettings.variantCount);
  return videos.flatMap((video) => Array.from({ length: totalVariants }, (_, variantIndex) => ({ video, variantIndex, totalVariants }))).map((job, index) => ({ ...job, index }));
}

function buildJobSettings(settings: RemixExportSettings, job: ExportJob): RemixExportSettings {
  const paddedIndex = String(job.index + 1).padStart(3, "0");
  const sourceName = job.video.fileName.replace(/\.[^.]+$/, "").trim() || "视频";
  const variantSuffix = job.totalVariants > 1 ? `_${String(job.variantIndex + 1).padStart(2, "0")}` : "";
  const outputName = settings.outputSettings.namingMode === "serial" ? `处理_${paddedIndex}` : `${sourceName}_处理${variantSuffix}`;
  return { ...settings, outputName };
}

function normalizeVariantCount(value: number) { return Number.isInteger(value) ? Math.max(1, Math.min(100, value)) : 1; }

function resolveWorkerCount(mode: RemixExportSettings["outputSettings"]["threadMode"], jobCount: number) {
  if (mode === "single") return 1;
  const available = typeof navigator === "undefined" ? 2 : navigator.hardwareConcurrency || 2;
  const capped = Math.max(1, Math.min(4, Math.floor(available)));
  return Math.min(jobCount, mode === "multi" ? Math.max(2, capped) : capped);
}

function confirmSourceRemoval() { return typeof window === "undefined" || window.confirm("导出成功后将永久删除对应原视频。导出失败的文件不会删除，是否继续？"); }

function validateSettings(options: UseBasicExportOptions) {
  return options.validateWatermark() ?? options.validateWatermarkRemoval() ?? options.validatePictureInPicture() ?? options.validateBgm() ?? options.validatePlaybackSpeed();
}

function handleExportError(error: unknown, target: Ref<string | null>, appendLog: UseBasicExportOptions["appendExportLog"], label: string) {
  if (isTaskCancelledError(error)) { target.value = `${label}任务已取消，可以重新开始。`; appendLog(target.value, "info"); return; }
  target.value = error instanceof Error ? error.message : String(error ?? `${label}失败。`);
  appendLog(`${label}失败：${target.value}`, "error");
}

async function exportCoverForResult(options: UseBasicExportOptions, videoOutputPath: string) {
  const coverImagePath = options.coverImagePath?.value;
  if (!coverImagePath) return;
  const coverOutputPath = await exportCoverImage(coverImagePath, videoOutputPath);
  options.appendExportLog(`已输出封面图片：${coverOutputPath}`, "success");
}
