import { ref } from "vue";
import type { Ref } from "vue";
import type { CanvasAspectRatio, CanvasBackgroundMode } from "../../services/videoMixService";
import type { ImportedVideo } from "../../types/videoProbe";
import type { TaskLogLevel } from "../../types/workbench";
import { isTaskCancelledError } from "../task-center";
import type { TaskRunHandle } from "../task-center";
import type { OutputSettings } from "../../types/outputSettings";
import { exportCurrentVideo } from "./services/remixExportService";
import type { WatermarkRemovalSettings, WatermarkSettings } from "../watermark";

interface UseBasicExportOptions {
  selectedVideo: Readonly<Ref<ImportedVideo | null>>;
  outputDirectory: Readonly<Ref<string | null>>;
  canvasAspectRatio: Readonly<Ref<CanvasAspectRatio>>;
  canvasBackgroundMode: Readonly<Ref<CanvasBackgroundMode>>;
  outputSettings: Readonly<Ref<OutputSettings>>;
  watermarkSettings: Readonly<Ref<WatermarkSettings>>;
  watermarkRemovalSettings: Readonly<Ref<WatermarkRemovalSettings>>;
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

    options.appendExportLog("开始导出。", "info");
    options.appendExportLog("导出中。", "info");
    isExporting.value = true;

    try {
      const result = await options.runTask("导出当前视频", (task) =>
        exportCurrentVideo(
          options.selectedVideo.value?.filePath as string,
          options.outputDirectory.value as string,
          options.selectedVideo.value?.durationSeconds ?? null,
          {
            canvasAspectRatio: options.canvasAspectRatio.value,
            canvasBackgroundMode: options.canvasBackgroundMode.value,
            outputSettings: options.outputSettings.value,
            watermarkSettings: options.watermarkSettings.value,
            watermarkRemovalSettings: options.watermarkRemovalSettings.value,
          },
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
      if (options.watermarkSettings.value.enabled) {
        options.appendExportLog(
          options.watermarkSettings.value.kind === "text"
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

  return { exportError, exportResultPath, exportSelectedVideo, isExporting };
}
