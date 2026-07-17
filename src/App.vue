<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { computed, onMounted, ref, watch } from "vue";
import type { CSSProperties } from "vue";
import ExportResultDrawer from "./components/ExportResultDrawer.vue";
import HomePage from "./components/HomePage.vue";
import PreviewPanel from "./components/PreviewPanel.vue";
import RightToolPanel from "./components/RightToolPanel.vue";
import TaskControlBar from "./components/TaskControlBar.vue";
import TaskLogDrawer from "./components/TaskLogDrawer.vue";
import ToolSettingModal from "./components/ToolSettingModal.vue";
import {
  SEGMENT_CATEGORY_OPTIONS as segmentCategoryOptions,
  WORKBENCH_MODULE_CARDS as moduleCards,
} from "./constants/workbench";
import { useRemixSettings } from "./composables/useRemixSettings";
import { useTaskLogs } from "./composables/useTaskLogs";
import { useAiRemix } from "./features/ai-remix";
import { MaterialPanel, useMaterials } from "./features/materials";
import {
  BatchResultDrawer,
  formatRemixCanvasLog,
  formatSmoothRemixLog,
  useRemixExport,
} from "./features/remix-export";
import {
  ProjectRecoveryDialog,
  ProjectSaveStatus,
  sanitizeProjectSnapshot,
  useProjectRecovery,
} from "./features/project-recovery";
import type { ProjectStateSnapshot } from "./features/project-recovery";
import { useTts } from "./features/tts";
import { useTaskCenter } from "./features/task-center";
import type {
  CanvasAspectRatio,
  CanvasBackgroundMode,
} from "./services/videoMixService";
import { isExistingDirectory, openPathInFileManager } from "./services/fileManagerService";
import { checkFfmpegEnvironment } from "./services/videoProbeService";
import type { FfmpegEnvironmentResult, ImportedVideo } from "./types/videoProbe";
import type { DrawerKey, ToolKey } from "./types/workbench";

const environment = ref<FfmpegEnvironmentResult | null>(null);
const isChecking = ref(true);
const checkError = ref<string | null>(null);
const outputDirectory = ref<string | null>(null);
const outputDirectoryError = ref<string | null>(null);
const fileManagerError = ref<string | null>(null);
const previewVideoRef = ref<HTMLVideoElement | null>(null);
const previewBackgroundVideoRef = ref<HTMLVideoElement | null>(null);
const isWorkspaceVisible = ref(true);
const activeTool = ref<ToolKey | null>(null);
const activeDrawer = ref<DrawerKey | null>(null);
const isWelcomeVisible = ref(true);
const isAdvancedMode = ref(false);

const {
  aiRemixLogs,
  appendAiRemixLog,
  appendBatchMixLog,
  appendExportLog,
  appendMixLog,
  appendSplitLog,
  appendTtsLog,
  addExportResult,
  clearAiRemixLogs,
  clearBatchMixLogs,
  clearExportLogs,
  clearMixLogs,
  clearSplitLogs,
  clearTtsLogs,
  exportResultItems,
  taskLogs,
} = useTaskLogs();

const {
  activeTask,
  cancelActiveTask,
  cleanupError,
  cleanupStaleTempFiles,
  isCleaning,
  isTaskRunning,
  runTask,
} = useTaskCenter();
const tempCleanupFeedback = ref<string | null>(null);

const {
  applyHorizontalMirror,
  applyVerticalMirror,
  bgmAudioFilePath,
  bgmEnabled,
  bgmFadeInSeconds,
  bgmFadeOutSeconds,
  bgmVolume,
  brightness,
  canvasAspectRatio,
  canvasBackgroundMode,
  contrast,
  effectScale,
  originalVolume,
  pipEnabled,
  pipMargin,
  pipOpacity,
  pipOverlayFilePath,
  pipPosition,
  pipSizeRatio,
  playbackSpeed,
  remixExportSettings,
  rotationMode,
  restoreRemixSettings,
  saturation,
  smoothRemixEnabled,
} = useRemixSettings();

const {
  coverError,
  coverFrameSeconds,
  generateCoverFrame,
  importError,
  importedVideos,
  importVideoFolder: importMaterialFolder,
  importVideos: importMaterialVideos,
  isGeneratingCover,
  isImporting,
  isSplitting,
  mergeSegmentThumbnailPaths,
  restoreMaterials,
  segmentCategories,
  segmentDurationSeconds,
  segmentThumbnailPaths,
  segmentThumbnailUrls,
  selectSegment,
  selectedCoverUrl,
  selectedCoverPath,
  selectedSegmentPath,
  selectedVideo,
  selectVideo: selectMaterialVideo,
  splitAllMaterials,
  splitError,
  splitOutputDirectory,
  splitSegmentCount,
  splitSegmentPaths,
  updateSegmentCategory,
  videoCoverUrls,
  videoCoverPaths,
} = useMaterials({
  outputDirectory,
  appendSplitLog,
  clearSplitLogs,
  runTask,
});

const {
  batchGenerateCount,
  batchMixError,
  batchMixFailures,
  batchMixResults,
  concatCategorizedSegments,
  concatRandomSegments,
  exportSelectedVideo,
  generateBatchMixes,
  isBatchMixing,
  isExporting,
  isMixing,
  mixError,
  pickSegmentsRandomly,
  randomPickCount,
  randomPickError,
  randomSelectedSegments,
  recordMixResult,
  resetRandomPickState,
  retryFailedBatchMixes,
  setMixing,
} = useRemixExport({
  selectedVideo,
  outputDirectory,
  canvasAspectRatio,
  canvasBackgroundMode,
  segmentPaths: splitSegmentPaths,
  segmentCategories,
  categoryOptions: segmentCategoryOptions,
  remixExportSettings,
  validatePictureInPicture: validatePictureInPictureSettings,
  validateBgm: validateBgmSettings,
  validatePlaybackSpeed,
  appendExportLog,
  appendMixLog,
  appendBatchMixLog,
  clearExportLogs,
  clearMixLogs,
  clearBatchMixLogs,
  addExportResult,
  runTask,
});

const {
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
  generateAiRemixVideos,
  isGeneratingAiRemix,
  isPlanningAiRemix,
  isPreparingAiSegments,
  moveAiRemixShot,
  prepareAiRemixVariants,
  prepareSegmentAssets,
  removeAiRemixShot,
  replaceAiRemixShotSegment,
  requestAiRemixPlan,
  resetAiRemixState,
  retryFailedAiRemixVideos,
  restoreAiRemixState,
} = useAiRemix({
  outputDirectory,
  remixExportSettings,
  appendAiRemixLog,
  appendSplitLog,
  clearAiRemixLogs,
  onSegmentThumbnailsPrepared(entries) {
    mergeSegmentThumbnailPaths(entries);
  },
  validateExportSettings() {
    return (
      validatePictureInPictureSettings() ?? validateBgmSettings() ?? validatePlaybackSpeed()
    );
  },
  setMixing,
  onGenerated(result) {
    recordMixResult("AI 智能混剪", result.outputPath);
  },
  formatCanvasLog: formatRemixCanvasLog,
  formatSmoothLog: formatSmoothRemixLog,
  runTask,
});

const {
  generateNarratedVideos,
  generateTts,
  isGeneratingNarratedVideo,
  isGeneratingTts,
  isLoadingTtsConfig,
  loadTtsConfig,
  narratedVideoError,
  narratedVideoFailures,
  narratedVideoResults,
  narratedVideoSummaryText,
  narrationProgressText,
  resetTtsSettings,
  retryFailedNarratedVideos,
  restoreTtsSettings,
  ttsAudioUrl,
  ttsConfig,
  ttsConfigError,
  ttsError,
  ttsResult,
  ttsKeepOriginalAudio,
  ttsOriginalAudioVolume,
  ttsSubtitleEnabled,
  ttsSubtitlePosition,
  ttsSubtitleSize,
  ttsSpeaker,
  ttsVideoEnabled,
} = useTts({
  text: aiScript,
  outputDirectory,
  appendLog: appendTtsLog,
  clearLogs: clearTtsLogs,
  validateExportSettings() {
    return (
      validatePictureInPictureSettings() ?? validateBgmSettings() ?? validatePlaybackSpeed()
    );
  },
  setMixing,
  onGenerated(result) {
    recordMixResult("AI配音混剪", result.outputPath);
  },
  formatCanvasLog: formatRemixCanvasLog,
  formatSmoothLog: formatSmoothRemixLog,
  runTask,
});

const projectState = computed(
  (): ProjectStateSnapshot => ({
    outputDirectory: outputDirectory.value,
    materials: {
      importedVideos: importedVideos.value,
      selectedVideoPath: selectedVideo.value?.filePath ?? null,
      segmentDurationSeconds: segmentDurationSeconds.value,
      splitOutputDirectory: splitOutputDirectory.value,
      splitSegmentPaths: splitSegmentPaths.value,
      segmentCategories: segmentCategories.value,
      segmentThumbnailPaths: segmentThumbnailPaths.value,
      selectedSegmentPath: selectedSegmentPath.value,
      videoCoverPaths: videoCoverPaths.value,
      selectedCoverPath: selectedCoverPath.value,
      coverFrameSeconds: coverFrameSeconds.value,
    },
    ai: {
      script: aiScript.value,
      generateCount: aiGenerateCount.value,
      preparedSegments: aiPreparedSegments.value.map(
        ({ thumbnailUrl: _thumbnailUrl, ...segment }) => segment,
      ),
      plannedShots: aiPlannedShots.value.map((shot) => ({
        shotId: shot.shotId,
        text: shot.text,
        segmentId: shot.segment.segmentId,
        alternativeSegmentIds: shot.alternativeSegments.map(
          (segment) => segment.segmentId,
        ),
      })),
    },
    remixSettings: remixExportSettings.value,
    tts: {
      speaker: ttsSpeaker.value,
      videoEnabled: ttsVideoEnabled.value,
      keepOriginalAudio: ttsKeepOriginalAudio.value,
      originalAudioVolume: ttsOriginalAudioVolume.value,
      subtitleEnabled: ttsSubtitleEnabled.value,
      subtitlePosition: ttsSubtitlePosition.value,
      subtitleSize: ttsSubtitleSize.value,
    },
  }),
);

const {
  discardPendingSnapshot,
  isRestoring: isRestoringProject,
  loadError: projectRecoveryLoadError,
  pendingResult: pendingProjectSnapshot,
  restoreError: projectRecoveryError,
  restorePendingSnapshot,
  saveError: projectSaveError,
  status: projectSaveStatus,
  statusText: projectSaveStatusText,
} = useProjectRecovery({
  projectState,
  hasMeaningfulState() {
    return (
      importedVideos.value.length > 0 ||
      splitSegmentPaths.value.length > 0 ||
      aiScript.value.trim().length > 0 ||
      Boolean(outputDirectory.value)
    );
  },
  applySnapshot(result) {
    if (!result.snapshot) {
      throw new Error("没有可恢复的项目记录。");
    }

    const sanitized = sanitizeProjectSnapshot(
      result.snapshot.project,
      result.missingFilePaths,
      result.missingDirectoryPaths,
    );
    outputDirectory.value = sanitized.project.outputDirectory;
    outputDirectoryError.value = result.missingDirectoryPaths.includes(
      result.snapshot.project.outputDirectory ?? "",
    )
      ? "原来的输出目录已经不存在，请重新选择一个文件夹。"
      : null;
    restoreMaterials(sanitized.project.materials);
    restoreAiRemixState(
      sanitized.project.ai,
      sanitized.preparedSegments,
      sanitized.plannedShots,
    );
    restoreRemixSettings(sanitized.project.remixSettings);
    restoreTtsSettings(sanitized.project.tts);
    resetRandomPickState();
    isWorkspaceVisible.value = true;

    appendSplitLog("已恢复上次项目进度。", "success");
    sanitized.notices.forEach((notice) => appendSplitLog(notice, "error"));
    return sanitized.notices;
  },
});

const isGeneratingCurrentAiVideo = computed(
  () =>
    isGeneratingAiRemix.value ||
    isGeneratingNarratedVideo.value ||
    isGeneratingTts.value,
);

const currentAiGenerateError = computed(() =>
  ttsVideoEnabled.value
    ? narratedVideoError.value ?? aiGenerateError.value
    : aiGenerateError.value,
);

const currentAiGenerationProgressText = computed(() =>
  ttsVideoEnabled.value ? narrationProgressText.value : aiGenerationProgressText.value,
);

const currentAiGenerationSummaryText = computed(() => {
  if (!ttsVideoEnabled.value) {
    return aiGenerationSummaryText.value;
  }

  return [aiGenerationSummaryText.value, narratedVideoSummaryText.value]
    .filter((message): message is string => Boolean(message))
    .join(" ") || null;
});

const currentAiGenerationSuccessCount = computed(() =>
  ttsVideoEnabled.value
    ? narratedVideoResults.value.length
    : aiGeneratedResults.value.length,
);

const currentAiGenerationFailureCount = computed(() =>
  ttsVideoEnabled.value
    ? narratedVideoFailures.value.length
    : aiGenerationFailures.value.length,
);

const totalFailedTaskCount = computed(
  () => currentAiGenerationFailureCount.value + batchMixFailures.value.length,
);

const canRetryFailures = computed(
  () => !isTaskRunning.value && totalFailedTaskCount.value > 0,
);

async function retryFailedTasks() {
  if (ttsVideoEnabled.value && narratedVideoFailures.value.length > 0) {
    await retryFailedNarratedVideos(remixExportSettings.value);
    return;
  }
  if (!ttsVideoEnabled.value && aiGenerationFailures.value.length > 0) {
    await retryFailedAiRemixVideos();
    return;
  }
  if (batchMixFailures.value.length > 0) {
    await retryFailedBatchMixes();
  }
}

async function cleanupTaskTempFiles(silentWhenEmpty = false) {
  const result = await cleanupStaleTempFiles();
  if (!result) {
    if (!silentWhenEmpty) {
      tempCleanupFeedback.value = cleanupError.value ?? "临时文件清理失败，请查看任务日志。";
    }
    return;
  }
  const releasedText =
    result.releasedBytes > 0
      ? ` 释放约 ${(result.releasedBytes / 1024 / 1024).toFixed(1)} MB。`
      : "";
  if (result && (!silentWhenEmpty || result.removedEntries > 0)) {
    tempCleanupFeedback.value = `${result.message}${releasedText}`;
    appendMixLog(
      tempCleanupFeedback.value,
      "success",
    );
  }
}

async function generateCurrentAiRemixVideo() {
  const variants = await prepareAiRemixVariants();

  if (!variants) {
    return;
  }

  if (!ttsVideoEnabled.value) {
    await generateAiRemixVideos(variants);
    return;
  }

  await generateNarratedVideos(
    variants.map((variant) =>
      variant.shots.map((shot) => ({
        text: shot.text,
        segmentPath: shot.segment.path,
        segmentDurationSeconds: shot.segment.durationSeconds,
        alternativeSegments: shot.alternativeSegments.map((segment) => ({
          videoPath: segment.path,
          durationSeconds: segment.durationSeconds,
        })),
      })),
    ),
    remixExportSettings.value,
  );
}

const statusText = computed(() => {
  if (isChecking.value) {
    return "正在检测 FFmpeg 环境...";
  }

  if (checkError.value) {
    return checkError.value;
  }

  return environment.value?.message ?? "未检测到 FFmpeg，请配置路径。";
});

const previewUrl = computed(() => {
  if (selectedSegmentPath.value) {
    return convertFileSrc(selectedSegmentPath.value);
  }

  if (!selectedVideo.value) {
    return null;
  }

  return convertFileSrc(selectedVideo.value.filePath);
});

const previewTitle = computed(() => {
  if (selectedSegmentPath.value) {
    return `片段预览：${formatFileName(selectedSegmentPath.value)}`;
  }

  return selectedVideo.value?.fileName ?? "等待导入素材";
});

const previewCanvasStyle = computed(
  (): CSSProperties => ({
    aspectRatio: getCanvasAspectRatioValue(canvasAspectRatio.value),
    "--preview-ratio": String(getCanvasRatioNumber(canvasAspectRatio.value)),
  }),
);

const shouldShowBlurBackground = computed(
  () => canvasAspectRatio.value !== "original" && canvasBackgroundMode.value === "blur",
);

const isAnyProcessing = computed(
  () =>
    isTaskRunning.value ||
    isExporting.value ||
    isMixing.value ||
    isBatchMixing.value ||
    isSplitting.value ||
    isGeneratingTts.value ||
    isGeneratingNarratedVideo.value ||
    isPlanningAiRemix.value ||
    isGeneratingAiRemix.value,
);

const primaryTaskActionLabel = computed(() => {
  if (importedVideos.value.length === 0) {
    return "请先导入素材";
  }

  if (aiPreparedSegments.value.length < 2) {
    return importedVideos.value.length > 1
      ? `切片全部素材（${importedVideos.value.length} 个）`
      : "切片当前视频";
  }

  if (aiPlannedShots.value.length < 2) {
    return aiScript.value.trim().length > 0 ? "生成 AI 分镜" : "请先填写文案";
  }

  if (!outputDirectory.value) {
    return "选择输出目录";
  }

  return ttsVideoEnabled.value ? "生成带AI配音的视频" : "生成当前分镜视频";
});

const primaryTaskActionDisabled = computed(
  () =>
    isAnyProcessing.value ||
    importedVideos.value.length === 0 ||
    (aiPreparedSegments.value.length >= 2 &&
      aiPlannedShots.value.length < 2 &&
      aiScript.value.trim().length === 0),
);

async function runPrimaryTaskAction() {
  if (primaryTaskActionDisabled.value) {
    return;
  }

  if (aiPreparedSegments.value.length < 2) {
    await splitSelectedVideo();
    return;
  }

  if (aiPlannedShots.value.length < 2) {
    await requestAiRemixPlan();
    return;
  }

  if (!outputDirectory.value) {
    await selectOutputDirectory();
    return;
  }

  await generateCurrentAiRemixVideo();
}

async function runEnvironmentCheck() {
  isChecking.value = true;
  checkError.value = null;

  try {
    environment.value = await checkFfmpegEnvironment();
  } catch (error) {
    checkError.value =
      error instanceof Error ? error.message : "未检测到 FFmpeg，请配置路径。";
  } finally {
    isChecking.value = false;
  }
}

async function importVideos() {
  if (await importMaterialVideos()) {
    resetAiRemixState(true);
    resetRandomPickState();
  }
}

async function importVideoFolder() {
  if (await importMaterialFolder()) {
    resetAiRemixState(true);
    resetRandomPickState();
  }
}

function selectVideo(video: ImportedVideo) {
  selectMaterialVideo(video);
  resetAiRemixState(true);
  resetRandomPickState();
}

async function selectOutputDirectory() {
  outputDirectoryError.value = null;
  fileManagerError.value = null;

  try {
    const selected = await open({
      directory: true,
      multiple: false,
    });

    if (!selected || Array.isArray(selected)) {
      return false;
    }

    outputDirectory.value = selected;
    return true;
  } catch (error) {
    outputDirectoryError.value =
      error instanceof Error
        ? error.message
        : String(error ?? "输出目录选择失败。");
    return false;
  }
}

async function splitSelectedVideo() {
  if (!(await ensureSplitOutputDirectory())) {
    return;
  }

  resetRandomPickState();
  resetAiRemixState(false);
  await splitAllMaterials(prepareSegmentAssets);
}

async function ensureSplitOutputDirectory() {
  if (outputDirectory.value) {
    try {
      if (await isExistingDirectory(outputDirectory.value)) {
        return true;
      }
    } catch (error) {
      outputDirectoryError.value =
        error instanceof Error ? error.message : String(error ?? "无法检查输出目录。");
      return false;
    }

    outputDirectory.value = null;
    outputDirectoryError.value = "原来的输出目录已经不存在，请重新选择一个文件夹。";
  }

  const selected = await selectOutputDirectory();
  if (!selected && !outputDirectoryError.value) {
    outputDirectoryError.value = "切片前必须先选择一个有效的输出目录。";
  }
  return selected;
}

function validatePlaybackSpeed() {
  if (!Number.isFinite(playbackSpeed.value)) {
    return "变速倍数必须是有效数字。";
  }

  if (playbackSpeed.value < 0.5 || playbackSpeed.value > 2) {
    return "变速倍数暂时只支持 0.5 到 2.0。";
  }

  return null;
}

function validatePictureInPictureSettings() {
  if (!pipEnabled.value) {
    return null;
  }

  if (!pipOverlayFilePath.value) {
    return "请先选择画中画素材，或关闭画中画。";
  }

  if (!Number.isFinite(pipSizeRatio.value) || pipSizeRatio.value < 0.2 || pipSizeRatio.value > 0.5) {
    return "画中画大小比例必须在 0.2 到 0.5 之间。";
  }

  if (!Number.isFinite(pipOpacity.value) || pipOpacity.value < 0 || pipOpacity.value > 1) {
    return "画中画透明度必须在 0 到 1 之间。";
  }

  if (!Number.isFinite(pipMargin.value) || pipMargin.value < 0 || pipMargin.value > 240) {
    return "画中画边距必须在 0 到 240 之间。";
  }

  return null;
}

function validateBgmSettings() {
  if (!bgmEnabled.value) {
    return null;
  }

  if (!bgmAudioFilePath.value) {
    return "请先选择 BGM 音频文件，或关闭 BGM。";
  }

  if (!Number.isFinite(originalVolume.value) || originalVolume.value < 0 || originalVolume.value > 2) {
    return "原视频音量必须在 0 到 2 之间。";
  }

  if (!Number.isFinite(bgmVolume.value) || bgmVolume.value < 0 || bgmVolume.value > 2) {
    return "BGM 音量必须在 0 到 2 之间。";
  }

  if (!Number.isFinite(bgmFadeInSeconds.value) || bgmFadeInSeconds.value < 0 || bgmFadeInSeconds.value > 10) {
    return "BGM 淡入秒数必须在 0 到 10 之间。";
  }

  if (!Number.isFinite(bgmFadeOutSeconds.value) || bgmFadeOutSeconds.value < 0 || bgmFadeOutSeconds.value > 10) {
    return "BGM 淡出秒数必须在 0 到 10 之间。";
  }

  return null;
}

function resetToolSettings(tool: ToolKey) {
  if (tool === "remix") {
    segmentDurationSeconds.value = 5;
    randomPickCount.value = 1;
    batchGenerateCount.value = 3;
    return;
  }

  if (tool === "canvas") {
    canvasAspectRatio.value = "original";
    canvasBackgroundMode.value = "black";
    return;
  }

  if (tool === "transition") {
    smoothRemixEnabled.value = false;
    return;
  }

  if (tool === "mirror") {
    applyHorizontalMirror.value = false;
    applyVerticalMirror.value = false;
    return;
  }

  if (tool === "speed") {
    playbackSpeed.value = 1;
    return;
  }

  if (tool === "rotate") {
    rotationMode.value = "none";
    return;
  }

  if (tool === "effects" || tool === "adjust") {
    brightness.value = 0;
    contrast.value = 1;
    saturation.value = 1;
    return;
  }

  if (tool === "zoom") {
    effectScale.value = 1;
    return;
  }

  if (tool === "pip") {
    pipEnabled.value = false;
    pipOverlayFilePath.value = null;
    pipPosition.value = "topRight";
    pipSizeRatio.value = 0.3;
    pipOpacity.value = 1;
    pipMargin.value = 24;
    return;
  }

  if (tool === "bgm" || tool === "audio") {
    bgmEnabled.value = false;
    bgmAudioFilePath.value = null;
    originalVolume.value = 1;
    bgmVolume.value = 0.35;
    bgmFadeInSeconds.value = 0.5;
    bgmFadeOutSeconds.value = 0.5;
    return;
  }

  if (tool === "cover" || tool === "frame") {
    coverFrameSeconds.value = 1;
    coverError.value = null;
    return;
  }

  if (tool === "tts") {
    resetTtsSettings();
  }
}

async function selectPipOverlayFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "画中画素材",
          extensions: ["mp4", "mov", "avi", "mkv", "png", "jpg", "jpeg", "webp", "bmp"],
        },
      ],
    });

    if (!selected || Array.isArray(selected)) {
      return;
    }

    pipOverlayFilePath.value = selected;
    pipEnabled.value = true;
  } catch (error) {
    mixError.value =
      error instanceof Error ? error.message : String(error ?? "选择画中画素材失败。");
  }
}

async function openOutputDirectory() {
  fileManagerError.value = null;

  if (!outputDirectory.value) {
    fileManagerError.value = "请先选择输出目录。";
    return;
  }

  await openPath(outputDirectory.value);
}

async function openResultLocation(path: string) {
  fileManagerError.value = null;
  await openPath(path);
}

async function openPath(path: string) {
  try {
    await openPathInFileManager(path);
  } catch (error) {
    fileManagerError.value =
      error instanceof Error ? error.message : String(error ?? "打开目录失败。");
  }
}

function getCanvasAspectRatioValue(aspectRatio: CanvasAspectRatio) {
  if (aspectRatio === "portrait916") {
    return "9 / 16";
  }

  if (aspectRatio === "square11") {
    return "1 / 1";
  }

  if (aspectRatio === "landscape169") {
    return "16 / 9";
  }

  if (selectedVideo.value?.width && selectedVideo.value?.height) {
    return `${selectedVideo.value.width} / ${selectedVideo.value.height}`;
  }

  return "16 / 9";
}

function getCanvasRatioNumber(aspectRatio: CanvasAspectRatio) {
  if (aspectRatio === "portrait916") {
    return 9 / 16;
  }

  if (aspectRatio === "square11") {
    return 1;
  }

  if (aspectRatio === "landscape169") {
    return 16 / 9;
  }

  if (selectedVideo.value?.width && selectedVideo.value?.height) {
    return selectedVideo.value.width / selectedVideo.value.height;
  }

  return 16 / 9;
}

function syncPreviewBackground() {
  const foregroundVideo = previewVideoRef.value;
  const backgroundVideo = previewBackgroundVideoRef.value;

  if (!foregroundVideo || !backgroundVideo) {
    return;
  }

  if (Math.abs(backgroundVideo.currentTime - foregroundVideo.currentTime) > 0.08) {
    backgroundVideo.currentTime = foregroundVideo.currentTime;
  }

  backgroundVideo.playbackRate = foregroundVideo.playbackRate;

  if (foregroundVideo.paused) {
    backgroundVideo.pause();
    return;
  }

  void backgroundVideo.play().catch(() => {
    backgroundVideo.pause();
  });
}

watch([previewUrl, shouldShowBlurBackground], () => {
  requestAnimationFrame(syncPreviewBackground);
});

async function selectBgmAudioFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "音频文件",
          extensions: ["mp3", "wav", "m4a", "aac", "flac", "ogg"],
        },
      ],
    });

    if (!selected || Array.isArray(selected)) {
      return;
    }

    bgmAudioFilePath.value = selected;
    bgmEnabled.value = true;
  } catch (error) {
    mixError.value =
      error instanceof Error ? error.message : String(error ?? "选择 BGM 音频失败。");
  }
}

function formatDuration(durationSeconds: number | null) {
  if (durationSeconds === null) {
    return "未知";
  }

  const totalSeconds = Math.round(durationSeconds);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;

  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

function formatResolution(video: ImportedVideo) {
  if (video.width === null || video.height === null) {
    return "未知";
  }

  return `${video.width} x ${video.height}`;
}

function formatFrameRate(frameRate: number | null) {
  if (frameRate === null) {
    return "未知";
  }

  return `${frameRate.toFixed(2)} fps`;
}

function formatFileSize(fileSizeBytes: number) {
  if (fileSizeBytes < 1024 * 1024) {
    return `${(fileSizeBytes / 1024).toFixed(1)} KB`;
  }

  return `${(fileSizeBytes / 1024 / 1024).toFixed(1)} MB`;
}

function formatFileName(filePath: string) {
  return filePath.split(/[\\/]/).pop() ?? filePath;
}

onMounted(() => {
  void runEnvironmentCheck();
  void loadTtsConfig();
  void cleanupTaskTempFiles(true);
});
</script>

<template>
  <main class="app-shell" :class="{ 'app-shell--home': !isWorkspaceVisible }">
    <header class="top-bar">
      <div class="product-mark">
        <button v-if="isWorkspaceVisible" class="back-button" type="button" @click="isWorkspaceVisible = false">
          返回首页
        </button>
        <p v-else class="eyebrow">V0.3.5 分类混剪版</p>
        <h1>本地短视频批量混剪工作台</h1>
      </div>
      <div class="top-status">
        <span class="mode-pill">本地处理 / 批量混剪 / 模块工作台</span>
        <span class="health-pill" :class="{ 'health-pill--ok': environment?.available }">
          {{ environment?.available ? "FFmpeg 就绪" : "FFmpeg 未就绪" }}
        </span>
        <ProjectSaveStatus
          :status="projectSaveStatus"
          :text="projectSaveStatusText"
          :error="projectSaveError"
        />
        <button class="title-icon" type="button" aria-label="用户中心">人</button>
        <button class="title-icon" type="button" aria-label="菜单">≡</button>
        <div class="window-controls" aria-label="窗口控制区">
          <span></span>
          <span></span>
          <span></span>
        </div>
      </div>
    </header>

    <HomePage
      v-if="!isWorkspaceVisible"
      :module-cards="moduleCards"
      :show-welcome="isWelcomeVisible"
      @open-workspace="isWorkspaceVisible = true"
      @close-welcome="isWelcomeVisible = false"
    />

    <section v-else class="workbench" :class="{ 'workbench--advanced': isAdvancedMode }">
      <MaterialPanel
        :is-advanced-mode="isAdvancedMode"
        :imported-videos="importedVideos"
        :selected-video="selectedVideo"
        :is-importing="isImporting"
        :import-error="importError"
        :output-directory="outputDirectory"
        :output-directory-error="outputDirectoryError"
        :split-segment-paths="splitSegmentPaths"
        :selected-segment-path="selectedSegmentPath"
        :random-selected-segments="randomSelectedSegments"
        :segment-categories="segmentCategories"
        :segment-category-options="segmentCategoryOptions"
        :video-cover-urls="videoCoverUrls"
        :segment-thumbnail-urls="segmentThumbnailUrls"
        :format-duration="formatDuration"
        :format-resolution="formatResolution"
        :format-file-name="formatFileName"
        @import-videos="importVideos"
        @import-video-folder="importVideoFolder"
        @select-video="selectVideo"
        @select-segment="selectSegment"
        @select-output-directory="selectOutputDirectory"
        @open-output-directory="openOutputDirectory"
        @update-segment-category="updateSegmentCategory"
      />

      <PreviewPanel
        v-model:preview-video-ref="previewVideoRef"
        v-model:preview-background-video-ref="previewBackgroundVideoRef"
        v-model:ai-script="aiScript"
        v-model:ai-generate-count="aiGenerateCount"
        :is-advanced-mode="isAdvancedMode"
        :imported-video-count="importedVideos.length"
        :selected-video="selectedVideo"
        :preview-title="previewTitle"
        :preview-url="previewUrl"
        :selected-cover-url="selectedCoverUrl"
        :canvas-aspect-ratio="canvasAspectRatio"
        :should-show-blur-background="shouldShowBlurBackground"
        :preview-canvas-style="previewCanvasStyle"
        :is-splitting="isSplitting"
        :is-mixing="isMixing"
        :is-batch-mixing="isBatchMixing"
        :is-exporting="isExporting"
        :split-segment-count="splitSegmentCount"
        :split-error="splitError || outputDirectoryError"
        :random-selected-count="randomSelectedSegments.length"
        :batch-mix-result-count="batchMixResults.length"
        :mix-error="mixError"
        :batch-mix-error="batchMixError"
        :ai-prepared-segments="aiPreparedSegments"
        :ai-planned-shots="aiPlannedShots"
        :is-preparing-ai-segments="isPreparingAiSegments"
        :ai-preparation-error="aiPreparationError"
        :is-planning-ai-remix="isPlanningAiRemix"
        :ai-planning-progress-text="aiPlanningProgressText"
        :is-generating-ai-remix="isGeneratingCurrentAiVideo"
        :tts-video-enabled="ttsVideoEnabled"
        :generation-progress-text="currentAiGenerationProgressText"
        :generation-summary-text="currentAiGenerationSummaryText"
        :generation-success-count="currentAiGenerationSuccessCount"
        :generation-failure-count="currentAiGenerationFailureCount"
        :ai-plan-error="aiPlanError"
        :ai-generate-error="currentAiGenerateError"
        :format-duration="formatDuration"
        :format-resolution="formatResolution"
        :format-frame-rate="formatFrameRate"
        :format-file-size="formatFileSize"
        @split-selected-video="splitSelectedVideo"
        @pick-segments-randomly="pickSegmentsRandomly"
        @concat-random-segments="concatRandomSegments"
        @concat-categorized-segments="concatCategorizedSegments"
        @generate-batch-mixes="generateBatchMixes"
        @export-selected-video="exportSelectedVideo"
        @sync-preview-background="syncPreviewBackground"
        @open-drawer="activeDrawer = $event"
        @open-tool="activeTool = $event"
        @plan-ai-remix="requestAiRemixPlan"
        @move-ai-shot="moveAiRemixShot"
        @remove-ai-shot="removeAiRemixShot"
        @replace-ai-shot-segment="replaceAiRemixShotSegment"
        @generate-ai-remix="generateCurrentAiRemixVideo"
      />

      <RightToolPanel
        :is-advanced-mode="isAdvancedMode"
        :environment="environment"
        :status-text="statusText"
        @open-tool="activeTool = $event"
        @open-drawer="activeDrawer = $event"
        @toggle-advanced-mode="isAdvancedMode = $event"
      />

      <TaskControlBar
        :is-advanced-mode="isAdvancedMode"
        :total-videos="importedVideos.length"
        :completed-count="exportResultItems.length"
        :failed-count="totalFailedTaskCount"
        :output-directory="outputDirectory"
        :is-processing="isAnyProcessing"
        :primary-action-label="primaryTaskActionLabel"
        :primary-action-disabled="primaryTaskActionDisabled"
        :active-task="activeTask"
        :can-retry-failures="canRetryFailures"
        :is-cleaning-temp-files="isCleaning"
        :temp-cleanup-feedback="tempCleanupFeedback"
        @start-processing="runPrimaryTaskAction"
        @cancel-task="cancelActiveTask"
        @retry-failures="retryFailedTasks"
        @cleanup-temp-files="cleanupTaskTempFiles"
        @open-drawer="activeDrawer = $event"
      />
    </section>

    <ToolSettingModal
      :active-tool="activeTool"
      :is-splitting="isSplitting"
      :is-mixing="isMixing"
      :is-batch-mixing="isBatchMixing"
      :is-exporting="isExporting"
      :split-error="splitError"
      :split-output-directory="splitOutputDirectory"
      :split-segment-count="splitSegmentCount"
      :random-pick-error="randomPickError"
      :output-directory="outputDirectory"
      :output-directory-error="outputDirectoryError"
      :segment-duration-seconds="segmentDurationSeconds"
      :random-pick-count="randomPickCount"
      :batch-generate-count="batchGenerateCount"
      :canvas-aspect-ratio="canvasAspectRatio"
      :canvas-background-mode="canvasBackgroundMode"
      :apply-horizontal-mirror="applyHorizontalMirror"
      :apply-vertical-mirror="applyVerticalMirror"
      :smooth-remix-enabled="smoothRemixEnabled"
      :playback-speed="playbackSpeed"
      :rotation-mode="rotationMode"
      :brightness="brightness"
      :contrast="contrast"
      :saturation="saturation"
      :effect-scale="effectScale"
      :pip-enabled="pipEnabled"
      :pip-overlay-file-path="pipOverlayFilePath"
      :pip-position="pipPosition"
      :pip-size-ratio="pipSizeRatio"
      :pip-opacity="pipOpacity"
      :pip-margin="pipMargin"
      :bgm-enabled="bgmEnabled"
      :bgm-audio-file-path="bgmAudioFilePath"
      :original-volume="originalVolume"
      :bgm-volume="bgmVolume"
      :bgm-fade-in-seconds="bgmFadeInSeconds"
      :bgm-fade-out-seconds="bgmFadeOutSeconds"
      :selected-cover-url="selectedCoverUrl"
      :cover-frame-seconds="coverFrameSeconds"
      :is-generating-cover="isGeneratingCover"
      :cover-error="coverError"
      :tts-text="aiScript"
      :tts-speaker="ttsSpeaker"
      :tts-resource-id="ttsConfig.resourceId"
      :tts-configured="ttsConfig.configured"
      :is-loading-tts-config="isLoadingTtsConfig"
      :is-generating-tts="isGeneratingTts"
      :is-generating-narrated-video="isGeneratingNarratedVideo"
      :tts-config-error="ttsConfigError"
      :tts-error="ttsError"
      :narrated-video-error="narratedVideoError"
      :narration-progress-text="narrationProgressText"
      :tts-video-enabled="ttsVideoEnabled"
      :tts-keep-original-audio="ttsKeepOriginalAudio"
      :tts-original-audio-volume="ttsOriginalAudioVolume"
      :tts-subtitle-enabled="ttsSubtitleEnabled"
      :tts-subtitle-position="ttsSubtitlePosition"
      :tts-subtitle-size="ttsSubtitleSize"
      :tts-result="ttsResult"
      :tts-audio-url="ttsAudioUrl"
      @close="activeTool = null"
      @reset="resetToolSettings"
      @split-selected-video="splitSelectedVideo"
      @pick-segments-randomly="pickSegmentsRandomly"
      @concat-random-segments="concatRandomSegments"
      @generate-batch-mixes="generateBatchMixes"
      @select-output-directory="selectOutputDirectory"
      @open-output-directory="openOutputDirectory"
      @export-selected-video="exportSelectedVideo"
      @select-pip-overlay-file="selectPipOverlayFile"
      @select-bgm-audio-file="selectBgmAudioFile"
      @generate-cover-frame="generateCoverFrame"
      @generate-tts="generateTts"
      @api-config-changed="loadTtsConfig"
      @update:segment-duration-seconds="segmentDurationSeconds = $event"
      @update:random-pick-count="randomPickCount = $event"
      @update:batch-generate-count="batchGenerateCount = $event"
      @update:canvas-aspect-ratio="canvasAspectRatio = $event"
      @update:canvas-background-mode="canvasBackgroundMode = $event"
      @update:apply-horizontal-mirror="applyHorizontalMirror = $event"
      @update:apply-vertical-mirror="applyVerticalMirror = $event"
      @update:smooth-remix-enabled="smoothRemixEnabled = $event"
      @update:playback-speed="playbackSpeed = $event"
      @update:rotation-mode="rotationMode = $event"
      @update:brightness="brightness = $event"
      @update:contrast="contrast = $event"
      @update:saturation="saturation = $event"
      @update:effect-scale="effectScale = $event"
      @update:pip-enabled="pipEnabled = $event"
      @update:pip-position="pipPosition = $event"
      @update:pip-size-ratio="pipSizeRatio = $event"
      @update:pip-opacity="pipOpacity = $event"
      @update:pip-margin="pipMargin = $event"
      @update:bgm-enabled="bgmEnabled = $event"
      @update:original-volume="originalVolume = $event"
      @update:bgm-volume="bgmVolume = $event"
      @update:bgm-fade-in-seconds="bgmFadeInSeconds = $event"
      @update:bgm-fade-out-seconds="bgmFadeOutSeconds = $event"
      @update:cover-frame-seconds="coverFrameSeconds = $event"
      @update:tts-text="aiScript = $event"
      @update:tts-speaker="ttsSpeaker = $event"
      @update:tts-video-enabled="ttsVideoEnabled = $event"
      @update:tts-keep-original-audio="ttsKeepOriginalAudio = $event"
      @update:tts-original-audio-volume="ttsOriginalAudioVolume = $event"
      @update:tts-subtitle-enabled="ttsSubtitleEnabled = $event"
      @update:tts-subtitle-position="ttsSubtitlePosition = $event"
      @update:tts-subtitle-size="ttsSubtitleSize = $event"
    />

    <TaskLogDrawer
      :open="activeDrawer === 'logs'"
      :task-logs="taskLogs"
      @close="activeDrawer = null"
    />

    <ProjectRecoveryDialog
      :result="pendingProjectSnapshot"
      :load-error="projectRecoveryLoadError"
      :restore-error="projectRecoveryError"
      :is-restoring="isRestoringProject"
      @restore="restorePendingSnapshot"
      @discard="discardPendingSnapshot"
    />
    <ExportResultDrawer
      :open="activeDrawer === 'exports'"
      :export-result-items="exportResultItems"
      :file-manager-error="fileManagerError"
      :format-file-name="formatFileName"
      @close="activeDrawer = null"
      @open-result-location="openResultLocation"
    />
    <BatchResultDrawer
      :open="activeDrawer === 'batch'"
      :batch-mix-results="batchMixResults"
      :format-file-name="formatFileName"
      @close="activeDrawer = null"
    />
  </main>
</template>
