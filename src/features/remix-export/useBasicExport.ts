import { ref } from "vue";
import type { Ref } from "vue";
import type { CanvasAspectRatio, CanvasBackgroundMode } from "../../services/videoMixService";
import type { ImportedVideo } from "../../types/videoProbe";
import type { TaskLogLevel } from "../../types/workbench";
import { exportCurrentVideo } from "./services/remixExportService";

interface UseBasicExportOptions {
  selectedVideo: Readonly<Ref<ImportedVideo | null>>;
  outputDirectory: Readonly<Ref<string | null>>;
  canvasAspectRatio: Readonly<Ref<CanvasAspectRatio>>;
  canvasBackgroundMode: Readonly<Ref<CanvasBackgroundMode>>;
  appendExportLog: (message: string, level: TaskLogLevel) => void;
  clearExportLogs: () => void;
  addExportResult: (type: "基础导出", path: string) => void;
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

    options.appendExportLog("开始导出。", "info");
    options.appendExportLog("导出中。", "info");
    isExporting.value = true;

    try {
      const result = await exportCurrentVideo(
        options.selectedVideo.value.filePath,
        options.outputDirectory.value,
        options.canvasAspectRatio.value,
        options.canvasBackgroundMode.value,
      );
      exportResultPath.value = result.outputPath;
      options.addExportResult("基础导出", result.outputPath);
      options.appendExportLog(`导出成功：${result.outputPath}`, "success");
    } catch (error) {
      exportError.value =
        error instanceof Error ? error.message : String(error ?? "视频导出失败。");
      options.appendExportLog(`导出失败：${exportError.value}`, "error");
    } finally {
      isExporting.value = false;
    }
  }

  return { exportError, exportResultPath, exportSelectedVideo, isExporting };
}
