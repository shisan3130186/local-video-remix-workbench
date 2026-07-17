import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { computed, ref } from "vue";
import type { Ref } from "vue";
import type { SegmentCategory } from "../../services/videoMixService";
import type { ImportedVideo } from "../../types/videoProbe";
import type { TaskLogLevel } from "../../types/workbench";
import type { ProjectMaterialsSnapshot } from "../project-recovery/types";
import { listVideoFilesInFolder } from "./services/materialService";
import { loadImportedVideos, splitImportedVideos } from "./services/materialWorkflow";
import { useMaterialCovers } from "./useMaterialCovers";

interface UseMaterialsOptions {
  outputDirectory: Readonly<Ref<string | null>>;
  appendSplitLog: (message: string, level: TaskLogLevel) => void;
  clearSplitLogs: () => void;
}

export function useMaterials(options: UseMaterialsOptions) {
  const importedVideos = ref<ImportedVideo[]>([]);
  const selectedVideo = ref<ImportedVideo | null>(null);
  const isImporting = ref(false);
  const importError = ref<string | null>(null);
  const segmentDurationSeconds = ref(5);
  const isSplitting = ref(false);
  const splitError = ref<string | null>(null);
  const splitOutputDirectory = ref<string | null>(null);
  const splitSegmentCount = ref<number | null>(null);
  const splitSegmentPaths = ref<string[]>([]);
  const segmentCategories = ref<Record<string, SegmentCategory | "">>({});
  const segmentThumbnailPaths = ref<Record<string, string>>({});
  const selectedSegmentPath = ref<string | null>(null);

  const covers = useMaterialCovers({
    selectedVideo,
    outputDirectory: options.outputDirectory,
  });

  const segmentThumbnailUrls = computed(() =>
    Object.fromEntries(
      Object.entries(segmentThumbnailPaths.value).map(([key, path]) => [
        key,
        convertFileSrc(path),
      ]),
    ),
  );

  async function importVideos() {
    importError.value = null;
    isImporting.value = true;

    try {
      const selected = await open({
        multiple: true,
        filters: [{ name: "视频文件", extensions: ["mp4", "mov", "avi", "mkv"] }],
      });

      if (!selected) {
        return false;
      }

      await replaceImportedVideos(Array.isArray(selected) ? selected : [selected]);
      return true;
    } catch (error) {
      importError.value =
        error instanceof Error ? error.message : String(error ?? "视频导入失败。");
      return false;
    } finally {
      isImporting.value = false;
    }
  }

  async function importVideoFolder() {
    importError.value = null;
    isImporting.value = true;

    try {
      const selected = await open({ directory: true, multiple: false });

      if (!selected || Array.isArray(selected)) {
        return false;
      }

      const filePaths = await listVideoFilesInFolder(selected);

      if (filePaths.length === 0) {
        importedVideos.value = [];
        selectedVideo.value = null;
        importError.value = "该文件夹中没有检测到 mp4 / mov / avi / mkv 视频文件。";
        return false;
      }

      await replaceImportedVideos(filePaths);
      return true;
    } catch (error) {
      importError.value =
        error instanceof Error ? error.message : String(error ?? "文件夹导入失败。");
      return false;
    } finally {
      isImporting.value = false;
    }
  }

  async function replaceImportedVideos(filePaths: string[]) {
    const videos = await loadImportedVideos(filePaths);
    covers.resetMaterialCovers();
    importedVideos.value = videos;
    selectedVideo.value = videos[0] ?? null;
    resetSplitState();

    if (selectedVideo.value) {
      covers.selectVideoCover(selectedVideo.value);
    }
  }

  function selectVideo(video: ImportedVideo) {
    selectedVideo.value = video;
    resetSplitState();
    covers.selectVideoCover(video);
  }

  function selectSegment(segmentPath: string) {
    selectedSegmentPath.value = segmentPath;
  }

  async function splitAllMaterials(prepareSegmentAssets: (paths: string[]) => Promise<void>) {
    resetSplitRun();

    if (importedVideos.value.length === 0) {
      splitError.value = "请先导入至少一个要切片的视频。";
      options.appendSplitLog(`切片失败：${splitError.value}`, "error");
      return;
    }

    if (!options.outputDirectory.value) {
      splitError.value = "请先选择输出目录。";
      options.appendSplitLog(`切片失败：${splitError.value}`, "error");
      return;
    }

    if (!Number.isFinite(segmentDurationSeconds.value) || segmentDurationSeconds.value <= 0) {
      splitError.value = "切片秒数必须大于 0。";
      options.appendSplitLog(`切片失败：${splitError.value}`, "error");
      return;
    }

    options.appendSplitLog(
      `开始切片全部素材，共 ${importedVideos.value.length} 个视频。`,
      "info",
    );
    isSplitting.value = true;

    try {
      const { segmentPaths, failedVideoNames } = await splitImportedVideos({
        videos: importedVideos.value,
        outputDirectory: options.outputDirectory.value,
        segmentDurationSeconds: segmentDurationSeconds.value,
        appendLog: options.appendSplitLog,
      });

      if (segmentPaths.length < 2) {
        throw new Error("成功生成的片段不足 2 个，无法继续 AI 混剪。请检查任务日志。 ");
      }

      splitOutputDirectory.value = options.outputDirectory.value;
      splitSegmentCount.value = segmentPaths.length;
      splitSegmentPaths.value = segmentPaths;
      segmentCategories.value = buildEmptySegmentCategoryMap(segmentPaths);
      await prepareSegmentAssets(segmentPaths);
      options.appendSplitLog(
        `全部素材切片完成：${importedVideos.value.length - failedVideoNames.length} 个成功，共 ${segmentPaths.length} 个片段。`,
        failedVideoNames.length > 0 ? "error" : "success",
      );

      if (failedVideoNames.length > 0) {
        splitError.value = `${failedVideoNames.length} 个视频切片失败，已保留其他视频生成的片段。`;
      }
    } catch (error) {
      splitError.value =
        error instanceof Error ? error.message : String(error ?? "视频切片失败。");
      options.appendSplitLog(`切片失败：${splitError.value}`, "error");
    } finally {
      isSplitting.value = false;
    }
  }

  function resetSplitRun() {
    splitError.value = null;
    splitOutputDirectory.value = null;
    splitSegmentCount.value = null;
    splitSegmentPaths.value = [];
    options.clearSplitLogs();
  }

  function resetSplitState() {
    resetSplitRun();
    segmentCategories.value = {};
    segmentThumbnailPaths.value = {};
    selectedSegmentPath.value = null;
  }

  function mergeSegmentThumbnailPaths(entries: Record<string, string>) {
    segmentThumbnailPaths.value = { ...segmentThumbnailPaths.value, ...entries };
  }

  function updateSegmentCategory(segmentPath: string, category: SegmentCategory | "") {
    segmentCategories.value = { ...segmentCategories.value, [segmentPath]: category };
  }

  function restoreMaterials(snapshot: ProjectMaterialsSnapshot) {
    importError.value = null;
    splitError.value = null;
    importedVideos.value = snapshot.importedVideos;
    selectedVideo.value =
      snapshot.importedVideos.find(
        (video) => video.filePath === snapshot.selectedVideoPath,
      ) ?? snapshot.importedVideos[0] ?? null;
    segmentDurationSeconds.value = snapshot.segmentDurationSeconds;
    splitOutputDirectory.value = snapshot.splitOutputDirectory;
    splitSegmentPaths.value = snapshot.splitSegmentPaths;
    splitSegmentCount.value = snapshot.splitSegmentPaths.length || null;
    segmentCategories.value = snapshot.segmentCategories;
    segmentThumbnailPaths.value = snapshot.segmentThumbnailPaths;
    selectedSegmentPath.value = snapshot.selectedSegmentPath;
    covers.videoCoverPaths.value = snapshot.videoCoverPaths;
    covers.selectedCoverPath.value = snapshot.selectedCoverPath;
    covers.coverFrameSeconds.value = snapshot.coverFrameSeconds;
    covers.coverError.value = null;

    if (selectedVideo.value) {
      covers.selectVideoCover(selectedVideo.value);
    }
  }

  return {
    ...covers,
    importError,
    importedVideos,
    importVideoFolder,
    importVideos,
    isImporting,
    isSplitting,
    mergeSegmentThumbnailPaths,
    restoreMaterials,
    segmentCategories,
    segmentDurationSeconds,
    segmentThumbnailPaths,
    segmentThumbnailUrls,
    selectSegment,
    selectedSegmentPath,
    selectedVideo,
    selectVideo,
    splitAllMaterials,
    splitError,
    splitOutputDirectory,
    splitSegmentCount,
    splitSegmentPaths,
    updateSegmentCategory,
  };
}

function buildEmptySegmentCategoryMap(segmentPaths: string[]) {
  return Object.fromEntries(segmentPaths.map((segmentPath) => [segmentPath, ""])) as Record<
    string,
    SegmentCategory | ""
  >;
}
