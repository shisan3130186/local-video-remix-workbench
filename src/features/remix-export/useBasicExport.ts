import { ref } from "vue";
import type { Ref } from "vue";
import type { RemixExportSettings } from "../../services/videoMixService";
import type { ImportedVideo } from "../../types/videoProbe";
import type { TaskLogLevel, VideoProcessingState } from "../../types/workbench";
import { isTaskCancelledError } from "../task-center";
import type { TaskRunHandle } from "../task-center";
import { exportCurrentVideo } from "./services/remixExportService";

interface UseBasicExportOptions {
  importedVideos: Readonly<Ref<ImportedVideo[]>>;
  selectedVideo: Readonly<Ref<ImportedVideo | null>>;
  outputDirectory: Readonly<Ref<string | null>>;
  remixExportSettings: Readonly<Ref<RemixExportSettings>>;
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

    if (!options.selectedVideo.value) {
      exportError.value = "请先选择一个要导出的视频。";
      options.appendExportLog(`导出失败：${exportError.value}`, "error");
      return;
    }

    if (!options.outputDirectory.value) {
      exportError.value = "请先选择输出目录。";
      options.appendExportLog(`导出失败：${exportError.value}`, "error");
      return;
    }

    const watermarkError = options.validateWatermark();
    if (watermarkError) {
      exportError.value = watermarkError;
      options.appendExportLog(`导出失败：${watermarkError}`, "error");
      return;
    }
    const watermarkRemovalError = options.validateWatermarkRemoval();
    if (watermarkRemovalError) {
      exportError.value = watermarkRemovalError;
      options.appendExportLog(`导出失败：${watermarkRemovalError}`, "error");
      return;
    }
    const effectValidationError =
      options.validatePictureInPicture() ??
      options.validateBgm() ??
      options.validatePlaybackSpeed();
    if (effectValidationError) {
      exportError.value = effectValidationError;
      options.appendExportLog(`导出失败：${effectValidationError}`, "error");
      return;
    }

    options.appendExportLog("开始导出。", "info");
    options.appendExportLog("导出中。", "info");
    isExporting.value = true;

    try {
      const result = await options.runTask("导出当前视频", (task) =>
        exportCurrentVideo(
          options.selectedVideo.value?.filePath as string,
          options.outputDirectory.value as string,
          options.selectedVideo.value?.durationSeconds ?? null,
          options.remixExportSettings.value,
          task.progress(0, 100, "正在导出当前视频"),
        ),
      );
      exportResultPath.value = result.outputPath;
      options.addExportResult("基础导出", result.outputPath);
      options.appendExportLog(`导出成功：${result.outputPath}`, "success");
      options.appendExportLog(
        `输出设置：${result.outputResolution}，${result.outputFrameRate}，${result.outputQuality}，${result.outputEncoder}，约 ${result.outputVideoBitrateKbps} kbps。`,
        "info",
      );
      if (options.remixExportSettings.value.watermarkSettings.enabled) {
        options.appendExportLog(
          options.remixExportSettings.value.watermarkSettings.kind === "text"
            ? "已添加文字水印。"
            : "已添加图片水印。",
          "info",
        );
      }
    } catch (error) {
      if (isTaskCancelledError(error)) {
        exportError.value = "导出任务已取消，可以重新开始。";
        options.appendExportLog(exportError.value, "info");
        return;
      }
      exportError.value =
        error instanceof Error ? error.message : String(error ?? "视频导出失败。");
      options.appendExportLog(`导出失败：${exportError.value}`, "error");
    } finally {
      isExporting.value = false;
    }
  }

  async function exportImportedVideos() {
    exportError.value = null;
    exportResultPath.value = null;
    options.clearExportLogs();

    if (options.importedVideos.value.length === 0) {
      exportError.value = "请先导入至少一个视频。";
      options.appendExportLog(`批量处理失败：${exportError.value}`, "error");
      return;
    }

    if (!options.outputDirectory.value) {
      exportError.value = "请先选择输出目录。";
      options.appendExportLog(`批量处理失败：${exportError.value}`, "error");
      return;
    }

    const watermarkError = options.validateWatermark();
    if (watermarkError) {
      exportError.value = watermarkError;
      options.appendExportLog(`批量处理失败：${watermarkError}`, "error");
      return;
    }
    const watermarkRemovalError = options.validateWatermarkRemoval();
    if (watermarkRemovalError) {
      exportError.value = watermarkRemovalError;
      options.appendExportLog(`批量处理失败：${watermarkRemovalError}`, "error");
      return;
    }
    const effectValidationError =
      options.validatePictureInPicture() ??
      options.validateBgm() ??
      options.validatePlaybackSpeed();
    if (effectValidationError) {
      exportError.value = effectValidationError;
      options.appendExportLog(`批量处理失败：${effectValidationError}`, "error");
      return;
    }

    const videos = [...options.importedVideos.value];
    videoProcessingStates.value = Object.fromEntries(
      videos.map((video) => [
        video.id,
        { status: "pending", progress: 0, message: "等待处理" },
      ]),
    );
    activeProcessingVideoId.value = null;
    isExporting.value = true;
    options.appendExportLog(`开始处理 ${videos.length} 个视频。`, "info");

    try {
      await options.runTask("批量处理视频效果", async (task) => {
        for (const [index, video] of videos.entries()) {
          task.throwIfCancelled();
          const startPercent = (index / videos.length) * 100;
          const endPercent = ((index + 1) / videos.length) * 100;
          activeProcessingVideoId.value = video.id;
          videoProcessingStates.value = {
            ...videoProcessingStates.value,
            [video.id]: { status: "processing", progress: 0, message: "正在处理" },
          };
          options.appendExportLog(`正在处理第 ${index + 1}/${videos.length} 个视频：${video.fileName}`, "info");

          try {
            const result = await exportCurrentVideo(
              video.filePath,
              options.outputDirectory.value as string,
              video.durationSeconds,
              options.remixExportSettings.value,
              task.progress(startPercent, endPercent, `正在处理 ${index + 1}/${videos.length}`),
            );
            const resolvedResult = await result;
            videoProcessingStates.value = {
              ...videoProcessingStates.value,
              [video.id]: { status: "completed", progress: 100, message: "处理完成" },
            };
            options.addExportResult("基础导出", resolvedResult.outputPath);
            options.appendExportLog(`处理完成：${resolvedResult.outputPath}`, "success");
          } catch (error) {
            if (isTaskCancelledError(error)) throw error;
            const message = error instanceof Error ? error.message : String(error ?? "视频处理失败。");
            videoProcessingStates.value = {
              ...videoProcessingStates.value,
              [video.id]: { status: "failed", progress: 0, message },
            };
            options.appendExportLog(`处理失败：${video.fileName}，${message}`, "error");
          }
        }
      });
    } catch (error) {
      if (isTaskCancelledError(error)) {
        exportError.value = "批量处理任务已取消，可以重新开始。";
        options.appendExportLog(exportError.value, "info");
        if (activeProcessingVideoId.value) {
          videoProcessingStates.value = {
            ...videoProcessingStates.value,
            [activeProcessingVideoId.value]: {
              status: "cancelled",
              progress: videoProcessingStates.value[activeProcessingVideoId.value]?.progress ?? 0,
              message: "已取消",
            },
          };
        }
      } else {
        exportError.value = error instanceof Error ? error.message : String(error ?? "批量处理失败。");
        options.appendExportLog(`批量处理失败：${exportError.value}`, "error");
      }
    } finally {
      activeProcessingVideoId.value = null;
      isExporting.value = false;
    }
  }

  return {
    activeProcessingVideoId,
    exportError,
    exportImportedVideos,
    exportResultPath,
    exportSelectedVideo,
    isExporting,
    videoProcessingStates,
  };
}
