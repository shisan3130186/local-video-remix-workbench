import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { computed, ref } from "vue";
import type { Ref } from "vue";
import type { SegmentCategory } from "../../services/videoMixService";
import type { ImportedVideo } from "../../types/videoProbe";
import type { TaskLogLevel } from "../../types/workbench";
import { isTaskCancelledError } from "../task-center";
import type { TaskRunHandle } from "../task-center";
import type { ProjectMaterialsSnapshot } from "../project-recovery/types";
import { listVideoFilesInFolder } from "./services/materialService";
import { loadImportedVideos, splitImportedVideos } from "./services/materialWorkflow";
import { useMaterialCovers } from "./useMaterialCovers";
import {
  cloneMaterialFolderSettings,
  DEFAULT_MATERIAL_FOLDER_SETTINGS,
} from "./types";
import type {
  MaterialFolder,
  MaterialFolderSettings,
  SceneSensitivity,
  SplitMode,
  SplitOutputGrouping,
} from "./types";

interface UseMaterialsOptions {
  outputDirectory: Readonly<Ref<string | null>>;
  appendSplitLog: (message: string, level: TaskLogLevel) => void;
  clearSplitLogs: () => void;
  runTask: <T>(label: string, runner: (task: TaskRunHandle) => Promise<T>) => Promise<T>;
}

export function useMaterials(options: UseMaterialsOptions) {
  const importedVideos = ref<ImportedVideo[]>([]);
  const materialFolders = ref<MaterialFolder[]>([]);
  const selectedVideo = ref<ImportedVideo | null>(null);
  const isImporting = ref(false);
  const importError = ref<string | null>(null);
  const segmentDurationSeconds = ref(5);
  const splitMode = ref<SplitMode>("scene");
  const splitOutputGrouping = ref<SplitOutputGrouping>("file");
  const sceneSensitivity = ref<SceneSensitivity>("balanced");
  const minimumSegmentSeconds = ref(2);
  const maximumSegmentSeconds = ref(10);
  const trimStartSeconds = ref(0);
  const trimEndSeconds = ref(0);
  const sceneConfidenceScore = ref(0.7);
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

      await addMaterialFolderFromFiles(
        Array.isArray(selected) ? selected : [selected],
        "单独导入",
      );
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

      await addMaterialFolderFromFiles(filePaths, folderNameFromPath(selected), selected);
      return true;
    } catch (error) {
      importError.value =
        error instanceof Error ? error.message : String(error ?? "文件夹导入失败。");
      return false;
    } finally {
      isImporting.value = false;
    }
  }

  async function addMaterialFolderFromFiles(
    filePaths: string[],
    folderName: string,
    folderPath = getParentPath(filePaths[0] ?? ""),
  ) {
    const videos = await loadImportedVideos(filePaths);
    if (videos.length === 0) {
      throw new Error("没有读取到可用的视频素材。");
    }

    const existingPaths = new Set(importedVideos.value.map((video) => video.filePath));
    const newVideos = videos.filter((video) => !existingPaths.has(video.filePath));
    const folder: MaterialFolder = {
      id: createMaterialFolderId(folderPath),
      folderPath,
      folderName: folderName || "素材文件夹",
      videoPaths: videos.map((video) => video.filePath),
      settings: cloneMaterialFolderSettings(DEFAULT_MATERIAL_FOLDER_SETTINGS),
    };

    materialFolders.value = [
      ...materialFolders.value.filter((item) => item.folderPath !== folderPath),
      folder,
    ];
    importedVideos.value = [...importedVideos.value, ...newVideos];
    selectedVideo.value = selectedVideo.value ?? newVideos[0] ?? videos[0] ?? null;
    covers.resetMaterialCovers();
    resetSplitState();

    if (selectedVideo.value) {
      covers.selectVideoCover(selectedVideo.value);
    }
  }

  function removeMaterialFolder(folderId: string) {
    const folder = materialFolders.value.find((item) => item.id === folderId);
    if (!folder) return;

    const removedPaths = new Set(folder.videoPaths);
    materialFolders.value = materialFolders.value.filter((item) => item.id !== folderId);
    importedVideos.value = importedVideos.value.filter((video) => !removedPaths.has(video.filePath));
    selectedVideo.value = importedVideos.value[0] ?? null;
    covers.resetMaterialCovers();
    if (selectedVideo.value) covers.selectVideoCover(selectedVideo.value);
    resetSplitState();
  }

  function updateMaterialFolderSettings(
    folderId: string,
    patch: Partial<MaterialFolderSettings>,
  ) {
    materialFolders.value = materialFolders.value.map((folder) => {
      if (folder.id !== folderId) return folder;
      const nextSettings = {
        ...folder.settings,
        ...patch,
      };
      nextSettings.materialCount = Math.min(
        999,
        Math.max(1, Math.floor(Number(nextSettings.materialCount) || 1)),
      );
      nextSettings.variantCount = Math.min(
        10,
        Math.max(1, Math.floor(Number(nextSettings.variantCount) || 1)),
      );
      return { ...folder, settings: nextSettings };
    });
  }

  function applyMaterialFolderSettingsToAll(settings: MaterialFolderSettings) {
    const nextSettings = cloneMaterialFolderSettings(settings);
    materialFolders.value = materialFolders.value.map((folder) => ({
      ...folder,
      settings: cloneMaterialFolderSettings(nextSettings),
    }));
  }

  function selectVideo(video: ImportedVideo) {
    selectedVideo.value = video;
    resetSplitState();
    covers.selectVideoCover(video);
  }

  function selectSegment(segmentPath: string) {
    selectedSegmentPath.value = segmentPath;
  }

  function clearImportedVideos() {
    covers.resetMaterialCovers();
    materialFolders.value = [];
    importedVideos.value = [];
    selectedVideo.value = null;
    importError.value = null;
    resetSplitState();
  }

  async function splitAllMaterials(
    prepareSegmentAssets: (paths: string[], task: TaskRunHandle) => Promise<void>,
  ) {
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

    if (
      splitMode.value === "duration" &&
      (!Number.isFinite(segmentDurationSeconds.value) || segmentDurationSeconds.value <= 0)
    ) {
      splitError.value = "切片秒数必须大于 0。";
      options.appendSplitLog(`切片失败：${splitError.value}`, "error");
      return;
    }

    if (splitMode.value === "scene") {
      const smartSettingsError = validateSmartSplitSettings(
        minimumSegmentSeconds.value,
        maximumSegmentSeconds.value,
      );
      if (smartSettingsError) {
        splitError.value = smartSettingsError;
        options.appendSplitLog(`智能切片失败：${splitError.value}`, "error");
        return;
      }
    }
    if (!Number.isFinite(trimStartSeconds.value) || trimStartSeconds.value < 0 || !Number.isFinite(trimEndSeconds.value) || trimEndSeconds.value < 0) {
      splitError.value = "片头和片尾裁切时长不能小于 0。";
      options.appendSplitLog(`切片失败：${splitError.value}`, "error");
      return;
    }

    options.appendSplitLog(
      `${splitMode.value === "scene" ? "开始智能切片" : "开始固定时长切片"}，共 ${importedVideos.value.length} 个视频。`,
      "info",
    );
    isSplitting.value = true;

    try {
      await options.runTask("切片并准备全部素材", async (task) => {
        const { segmentPaths, failedVideoNames, detectedSceneCount } = await splitImportedVideos({
          videos: importedVideos.value,
          outputDirectory: options.outputDirectory.value as string,
          splitMode: splitMode.value,
          segmentDurationSeconds: segmentDurationSeconds.value,
          smartSplitSettings: {
            sensitivity: confidenceToSceneSensitivity(sceneConfidenceScore.value),
            minimumSegmentSeconds: minimumSegmentSeconds.value,
            maximumSegmentSeconds: maximumSegmentSeconds.value,
          },
          trimStartSeconds: trimStartSeconds.value,
          trimEndSeconds: trimEndSeconds.value,
          outputGrouping: splitOutputGrouping.value,
          appendLog: options.appendSplitLog,
          task,
        });

        task.throwIfCancelled();
        if (segmentPaths.length < 2) {
          throw new Error("成功生成的片段不足 2 个，无法继续 AI 混剪。请检查任务日志。 ");
        }

        splitOutputDirectory.value = options.outputDirectory.value;
        splitSegmentCount.value = segmentPaths.length;
        splitSegmentPaths.value = segmentPaths;
        selectedSegmentPath.value = segmentPaths[0] ?? null;
        segmentCategories.value = buildEmptySegmentCategoryMap(segmentPaths);
        await prepareSegmentAssets(segmentPaths, task);
        task.throwIfCancelled();
        options.appendSplitLog(
          splitMode.value === "scene"
            ? `全部素材智能切片完成：${importedVideos.value.length - failedVideoNames.length} 个成功，共 ${segmentPaths.length} 个片段，识别到 ${detectedSceneCount} 个画面变化候选点。`
            : `全部素材切片完成：${importedVideos.value.length - failedVideoNames.length} 个成功，共 ${segmentPaths.length} 个片段。`,
          failedVideoNames.length > 0 ? "error" : "success",
        );

        if (failedVideoNames.length > 0) {
          splitError.value = `${failedVideoNames.length} 个视频切片失败，已保留其他视频生成的片段。`;
        }
      });
    } catch (error) {
      if (isTaskCancelledError(error)) {
        splitError.value = "切片任务已取消，可以重新开始。";
        options.appendSplitLog(splitError.value, "info");
        return;
      }
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
    materialFolders.value = normalizeMaterialFolders(
      snapshot.materialFolders,
      snapshot.importedVideos,
    );
    selectedVideo.value =
      snapshot.importedVideos.find(
        (video) => video.filePath === snapshot.selectedVideoPath,
      ) ?? snapshot.importedVideos[0] ?? null;
    segmentDurationSeconds.value = snapshot.segmentDurationSeconds;
    splitMode.value = snapshot.splitMode ?? "scene";
    splitOutputGrouping.value = snapshot.splitOutputGrouping ?? "file";
    trimStartSeconds.value = snapshot.trimStartSeconds ?? 0;
    trimEndSeconds.value = snapshot.trimEndSeconds ?? 0;
    sceneConfidenceScore.value = snapshot.sceneConfidenceScore ?? 0.7;
    minimumSegmentSeconds.value = snapshot.minimumSegmentSeconds ?? 2;
    maximumSegmentSeconds.value = snapshot.maximumSegmentSeconds ?? 10;
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

  function addRelinkedMaterial(video: ImportedVideo, coverPath: string | null) {
    importedVideos.value = [
      ...importedVideos.value.filter((item) => item.filePath !== video.filePath),
      video,
    ];
    materialFolders.value = materialFolders.value.map((folder) =>
      folder.videoPaths.includes(video.filePath)
        ? folder
        : folder.videoPaths.includes(selectedVideo.value?.filePath ?? "")
          ? { ...folder, videoPaths: [...folder.videoPaths, video.filePath] }
          : folder,
    );
    if (coverPath) {
      covers.videoCoverPaths.value = {
        ...covers.videoCoverPaths.value,
        [video.id]: coverPath,
      };
    }
    if (!selectedVideo.value) {
      selectedVideo.value = video;
      covers.selectVideoCover(video);
    }
  }

  return {
    ...covers,
    addRelinkedMaterial,
    applyMaterialFolderSettingsToAll,
    clearImportedVideos,
    importError,
    importedVideos,
    importVideoFolder,
    importVideos,
    isImporting,
    isSplitting,
    maximumSegmentSeconds,
    materialFolders,
    mergeSegmentThumbnailPaths,
    minimumSegmentSeconds,
    sceneConfidenceScore,
    restoreMaterials,
    removeMaterialFolder,
    segmentCategories,
    segmentDurationSeconds,
    sceneSensitivity,
    trimEndSeconds,
    trimStartSeconds,
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
    splitMode,
    splitOutputGrouping,
    updateMaterialFolderSettings,
    updateSegmentCategory,
  };
}

function confidenceToSceneSensitivity(value: number): SceneSensitivity {
  if (!Number.isFinite(value) || value >= 0.75) return "stable";
  if (value <= 0.4) return "sensitive";
  return "balanced";
}

function createMaterialFolderId(folderPath: string) {
  return `material-folder-${folderPath.toLowerCase()}-${Date.now().toString(36)}`;
}

function folderNameFromPath(folderPath: string) {
  return folderPath.split(/[\\/]/).filter(Boolean).pop() ?? "素材文件夹";
}

function getParentPath(filePath: string) {
  const parts = filePath.split(/[\\/]/);
  parts.pop();
  return parts.join("\\");
}

function normalizeMaterialFolders(
  folders: MaterialFolder[] | undefined,
  importedVideos: ImportedVideo[],
) {
  const validPaths = new Set(importedVideos.map((video) => video.filePath));
  const normalized = (folders ?? [])
    .map((folder) => ({
      ...folder,
      videoPaths: folder.videoPaths.filter((path) => validPaths.has(path)),
      settings: {
        ...DEFAULT_MATERIAL_FOLDER_SETTINGS,
        ...(folder.settings ?? {}),
        fixedFirstMaterialKind:
          folder.settings?.fixedFirstMaterialKind ??
          DEFAULT_MATERIAL_FOLDER_SETTINGS.fixedFirstMaterialKind,
      },
    }))
    .filter((folder) => folder.videoPaths.length > 0);

  if (normalized.length > 0) return normalized;
  if (importedVideos.length === 0) return [];

  const fallbackPath = getParentPath(importedVideos[0].filePath);
  return [{
    id: createMaterialFolderId(fallbackPath),
    folderPath: fallbackPath,
    folderName: folderNameFromPath(fallbackPath),
    videoPaths: importedVideos.map((video) => video.filePath),
    settings: cloneMaterialFolderSettings(DEFAULT_MATERIAL_FOLDER_SETTINGS),
  }];
}

function validateSmartSplitSettings(minimum: number, maximum: number) {
  if (!Number.isFinite(minimum) || minimum < 0.5 || minimum > 10) {
    return "最短片段时长必须在 0.5 到 10 秒之间。";
  }
  if (!Number.isFinite(maximum) || maximum < 2 || maximum > 60) {
    return "最长片段时长必须在 2 到 60 秒之间。";
  }
  if (maximum < minimum + 0.5) {
    return "最长片段时长需要至少比最短片段多 0.5 秒。";
  }
  return null;
}

function buildEmptySegmentCategoryMap(segmentPaths: string[]) {
  return Object.fromEntries(segmentPaths.map((segmentPath) => [segmentPath, ""])) as Record<
    string,
    SegmentCategory | ""
  >;
}
