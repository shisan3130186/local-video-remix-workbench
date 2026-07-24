<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { computed, onMounted, ref, watch } from "vue";
import type { CSSProperties } from "vue";
import ExportResultDrawer from "./components/ExportResultDrawer.vue";
import BatchWorkspacePanel from "./components/BatchWorkspacePanel.vue";
import HomePage from "./components/HomePage.vue";
import PreviewPanel from "./components/PreviewPanel.vue";
import RightToolPanel from "./components/RightToolPanel.vue";
import TaskControlBar from "./components/TaskControlBar.vue";
import TaskLogDrawer from "./components/TaskLogDrawer.vue";
import ToolSettingModal from "./components/ToolSettingModal.vue";
import VideoToolsWorkspace from "./components/VideoToolsWorkspace.vue";
import WorkspaceModeNav from "./components/WorkspaceModeNav.vue";
import {
  SEGMENT_CATEGORY_OPTIONS as segmentCategoryOptions,
} from "./constants/workbench";
import { useRemixSettings } from "./composables/useRemixSettings";
import { useTaskLogs } from "./composables/useTaskLogs";
import { useAiRemix } from "./features/ai-remix";
import type { ApiConfigStatus } from "./features/api-config";
import { getApiConfigStatus } from "./features/api-config/services/apiConfigService";
import { useAsr } from "./features/asr";
import {
  buildMaterialLibraryState,
  sanitizeMaterialLibrarySnapshot,
  useMaterialLibrary,
} from "./features/material-library";
import type { MaterialLibraryState } from "./features/material-library";
import { MaterialPanel, useMaterials } from "./features/materials";
import { MembershipDialog, useMembership } from "./features/membership";
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
import { ScriptLibraryDrawer, useScriptLibrary } from "./features/script-library";
import type { ScriptLibraryEntry } from "./features/script-library";
import { useTts } from "./features/tts";
import { useTaskCenter } from "./features/task-center";
import {
  createDiagnosticReport as requestDiagnosticReport,
  FirstLaunchWizard,
  getDiagnosticInfo,
} from "./features/user-test-package";
import type { DiagnosticInfo } from "./features/user-test-package";
import type {
  CanvasAspectRatio,
  CanvasBackgroundMode,
} from "./services/videoMixService";
import { isExistingDirectory, openPathInFileManager } from "./services/fileManagerService";
import { checkFfmpegEnvironment } from "./services/videoProbeService";
import type { FfmpegEnvironmentResult, ImportedVideo } from "./types/videoProbe";
import type { DrawerKey, ToolKey, WorkspaceMode } from "./types/workbench";

const environment = ref<FfmpegEnvironmentResult | null>(null);
const smartCutIconUrl = "/smartcut-icon.svg";
const isChecking = ref(true);
const checkError = ref<string | null>(null);
const outputDirectory = ref<string | null>(null);
const outputDirectoryError = ref<string | null>(null);
const fileManagerError = ref<string | null>(null);
const previewVideoRef = ref<HTMLVideoElement | null>(null);
const previewBackgroundVideoRef = ref<HTMLVideoElement | null>(null);
const isWorkspaceVisible = ref(false);
const workspaceMode = ref<WorkspaceMode>("ai");
const activeTool = ref<ToolKey | null>(null);
const activeDrawer = ref<DrawerKey | null>(null);
const isWelcomeVisible = ref(false);
const welcomeStartStep = ref(0);
const apiConfigStatus = ref<ApiConfigStatus | null>(null);
const diagnosticInfo = ref<DiagnosticInfo | null>(null);
const isRefreshingReadiness = ref(false);
const readinessError = ref<string | null>(null);
const diagnosticFeedback = ref<string | null>(null);
const isAdvancedMode = ref(false);

const {
  activeAction: membershipActiveAction,
  changePassword: changeMembershipPassword,
  error: membershipError,
  feedback: membershipFeedback,
  isDialogVisible: isMembershipVisible,
  loadStatus: loadMembershipStatus,
  login: loginMembershipAccount,
  logout: logoutMembershipAccount,
  openDialog: openMembershipDialog,
  redeem: redeemMembershipCode,
  refreshStatus: refreshMembershipStatus,
  register: registerMembershipAccount,
  status: membershipStatus,
  topBarText: membershipTopBarText,
} = useMembership();

const {
  aiRemixLogs,
  appendAsrLog,
  appendAiRemixLog,
  appendBatchMixLog,
  appendExportLog,
  appendMixLog,
  appendSplitLog,
  appendTtsLog,
  addExportResult,
  clearAiRemixLogs,
  clearAsrLogs,
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
  asrConfig,
  asrError,
  asrResult,
  asrSourceFileName,
  asrSourceFilePath,
  clearAsrSelection,
  isLoadingAsrConfig,
  isRecognizingAsr,
  loadAsrConfig,
  recognizeAsr,
  selectAsrSourceFile,
} = useAsr({
  appendLog: appendAsrLog,
  clearLogs: clearAsrLogs,
  runTask,
});

const {
  deleteScriptEntry,
  deletingScriptId,
  isLoadingScriptLibrary,
  isSavingScript,
  loadScriptLibrary,
  saveAsrToScriptLibrary,
  scriptLibraryEntries,
  scriptLibraryError,
  scriptLibraryFeedback,
} = useScriptLibrary();

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
  detectEncoders,
  encoderCapabilities,
  encoderDetectionError,
  effectScale,
  isDetectingEncoders,
  originalVolume,
  outputEncoder,
  outputFrameRate,
  outputQuality,
  outputResolution,
  outputSettings,
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
  resetWatermarkSettings,
  resetWatermarkRemovalSettings,
  validateWatermarkSettings,
  validateWatermarkRemovalSettings,
  watermarkEnabled,
  watermarkImageFilePath,
  watermarkImageSizeRatio,
  watermarkKind,
  watermarkMargin,
  watermarkOpacity,
  watermarkPosition,
  watermarkSettings,
  watermarkText,
  watermarkTextColor,
  watermarkTextFontSize,
  watermarkRemovalCoverColor,
  watermarkRemovalCoverOpacity,
  watermarkRemovalEnabled,
  watermarkRemovalMargin,
  watermarkRemovalMode,
  watermarkRemovalPosition,
  watermarkRemovalSettings,
  watermarkRemovalSize,
  watermarkRemovalStrength,
  watermarkRemovalTrackingEnabled,
  watermarkRemovalTrackingKeyframes,
  watermarkRemovalTrackingRegionHeightRatio,
  watermarkRemovalTrackingRegionWidthRatio,
} = useRemixSettings();

const {
  addRelinkedMaterial,
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
  maximumSegmentSeconds,
  mergeSegmentThumbnailPaths,
  minimumSegmentSeconds,
  restoreMaterials,
  segmentCategories,
  segmentDurationSeconds,
  sceneSensitivity,
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
  splitMode,
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
  outputSettings,
  watermarkSettings,
  watermarkRemovalSettings,
  segmentPaths: splitSegmentPaths,
  segmentCategories,
  categoryOptions: segmentCategoryOptions,
  remixExportSettings,
  validatePictureInPicture: validatePictureInPictureSettings,
  validateBgm: validateBgmSettings,
  validateWatermark: validateWatermarkSettings,
  validateWatermarkRemoval: validateWatermarkRemovalSettings,
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
  aiContentAnalysisError,
  aiContentAnalysisProgressText,
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
  analyzePreparedSegmentContent,
  generateAiRemixVideos,
  isGeneratingAiRemix,
  isAnalyzingAiContent,
  isPlanningAiRemix,
  isPreparingAiSegments,
  moveAiRemixShot,
  prepareAiRemixVariants,
  prepareSegmentAssets,
  removeAiRemixShot,
  replaceAiRemixShotSegment,
  requestAiRemixPlan,
  resetAiRemixState,
  restoreAiPreparedSegments,
  retryFailedAiRemixVideos,
  restoreAiRemixState,
  updateAiSegmentContentAnalysis,
} = useAiRemix({
  outputDirectory,
  remixExportSettings,
  appendAiRemixLog,
  appendSplitLog,
  clearAiRemixLogs,
  onSegmentThumbnailsPrepared(entries) {
    mergeSegmentThumbnailPaths(entries);
  },
  onSegmentCategorySuggested(segmentPath, category) {
    updateSegmentCategory(segmentPath, category);
  },
  validateExportSettings() {
    return (
      validatePictureInPictureSettings() ??
      validateBgmSettings() ??
      validateWatermarkSettings() ??
      validateWatermarkRemovalSettings() ??
      validatePlaybackSpeed()
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
      validatePictureInPictureSettings() ??
      validateBgmSettings() ??
      validateWatermarkSettings() ??
      validateWatermarkRemovalSettings() ??
      validatePlaybackSpeed()
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

async function refreshSpeechConfiguration() {
  await Promise.all([loadTtsConfig(), loadAsrConfig()]);
  await refreshConfigurationAndDiagnostics();
}

async function saveCurrentAsrToLibrary(useAfterSave: boolean) {
  if (!asrResult.value) return;
  const saved = await saveAsrToScriptLibrary(asrResult.value);
  if (!saved || !useAfterSave) return;
  if (useScriptEntry(saved.entry)) {
    scriptLibraryFeedback.value = "文案已保存，并填入当前AI分镜文案。";
    activeTool.value = null;
  }
}

function useScriptEntry(entry: ScriptLibraryEntry) {
  if (entry.text.length > 4000) {
    window.alert("这条文案超过4000字，请先复制并精简后再用于AI分镜。");
    return false;
  }
  const currentText = aiScript.value.trim();
  if (
    currentText &&
    currentText !== entry.text.trim() &&
    !window.confirm("当前文案框中已有内容，确定要替换为这条文案吗？")
  ) {
    return false;
  }
  aiScript.value = entry.text;
  activeDrawer.value = null;
  return true;
}

async function removeScriptEntry(entry: ScriptLibraryEntry) {
  if (!window.confirm(`确定删除“${entry.title}”吗？`)) return;
  await deleteScriptEntry(entry.id);
}

const materialLibraryState = computed(
  (): MaterialLibraryState =>
    buildMaterialLibraryState({
      importedVideos: importedVideos.value,
      selectedVideoPath: selectedVideo.value?.filePath ?? null,
      segmentDurationSeconds: segmentDurationSeconds.value,
      splitOutputDirectory: splitOutputDirectory.value,
      selectedSegmentPath: selectedSegmentPath.value,
      coverFrameSeconds: coverFrameSeconds.value,
      videoCoverPaths: videoCoverPaths.value,
      splitSegmentPaths: splitSegmentPaths.value,
      segmentCategories: segmentCategories.value,
      segmentThumbnailPaths: segmentThumbnailPaths.value,
      preparedSegments: aiPreparedSegments.value,
    }),
);

const {
  activateWithCurrentState: activateMaterialLibrary,
  hasAvailableLibrary,
  inspectLibrary: inspectMaterialLibrary,
  isLoadingLibrary: isLoadingMaterialLibrary,
  loadAvailableLibrary,
  loadError: materialLibraryLoadError,
  missingMaterials,
  missingSegmentCount,
  relinkingPath: relinkingMaterialPath,
  relinkMissingMaterial,
  saveError: materialLibrarySaveError,
  statusText: materialLibraryStatusText,
} = useMaterialLibrary({
  currentState: materialLibraryState,
  applySnapshot(snapshot, missingFilePaths) {
    const sanitized = sanitizeMaterialLibrarySnapshot(snapshot, missingFilePaths);
    restoreMaterials(sanitized.materials);
    restoreAiPreparedSegments(sanitized.preparedSegments);
    appendSplitLog(
      `本地素材库已载入：${sanitized.restoredMaterials} 个视频、${sanitized.restoredSegments} 个片段。`,
      "success",
    );
    return {
      restoredMaterials: sanitized.restoredMaterials,
      restoredSegments: sanitized.restoredSegments,
    };
  },
  onMaterialRelinked(_missingMaterial, replacement) {
    addRelinkedMaterial(replacement.video, replacement.coverPath);
    appendSplitLog(`失效素材已重新定位：${replacement.video.fileName}`, "success");
  },
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
    activateMaterialLibrary();
    resetRandomPickState();
    workspaceMode.value = "ai";
    isWorkspaceVisible.value = true;

    appendSplitLog("已恢复上次项目进度。", "success");
    sanitized.notices.forEach((notice) => appendSplitLog(notice, "error"));
    return sanitized.notices;
  },
  onNoSnapshot: inspectMaterialLibrary,
});

async function discardProjectSnapshotAndStartBlank() {
  await discardPendingSnapshot();
  if (!pendingProjectSnapshot.value) {
    await inspectMaterialLibrary();
  }
}

const hasHomeProject = computed(
  () => importedVideos.value.length > 0 || Boolean(pendingProjectSnapshot.value?.snapshot),
);

const homeProjectTitle = computed(() => {
  if (selectedVideo.value?.fileName) return selectedVideo.value.fileName;
  return pendingProjectSnapshot.value?.snapshot?.project.materials.importedVideos[0]?.fileName
    ?? "上次自动保存的项目";
});

const homeProjectSummary = computed(() => {
  if (importedVideos.value.length > 0) {
    return `${importedVideos.value.length} 个素材 · ${aiPlannedShots.value.length} 个分镜 · 项目会自动保存`;
  }

  const snapshot = pendingProjectSnapshot.value?.snapshot;
  if (!snapshot) return "素材、文案和设置会自动保存";
  const videoCount = snapshot.project.materials.importedVideos.length;
  const shotCount = snapshot.project.ai.plannedShots.length;
  return `${videoCount} 个素材 · ${shotCount} 个分镜 · ${new Date(snapshot.savedAt).toLocaleString("zh-CN")}`;
});

const workspaceModeCopy = computed(() => {
  if (workspaceMode.value === "batch") {
    return { title: "批量混剪", subtitle: "多素材规则组合与批量生成" };
  }
  if (workspaceMode.value === "tools") {
    return { title: "视频工具", subtitle: "单项处理中心" };
  }
  return { title: "AI 智能成片", subtitle: "文案驱动的智能成片流程" };
});

const productionWorkspaceMode = computed<"ai" | "batch">(() =>
  workspaceMode.value === "batch" ? "batch" : "ai",
);

function openWorkspace(mode: WorkspaceMode = "ai") {
  workspaceMode.value = mode;
  activeTool.value = null;
  activeDrawer.value = null;
  isWorkspaceVisible.value = true;
}

function changeWorkspaceMode(mode: WorkspaceMode) {
  workspaceMode.value = mode;
  activeTool.value = null;
  activeDrawer.value = null;
}

async function continueHomeProject() {
  if (pendingProjectSnapshot.value?.snapshot) {
    await restorePendingSnapshot();
    return;
  }
  openWorkspace("ai");
}

function openWelcomeGuide(startStep = 0) {
  welcomeStartStep.value = startStep;
  isWelcomeVisible.value = true;
}

function openDiagnosticGuide() {
  openWelcomeGuide(3);
}

function closeWelcomeGuide() {
  isWelcomeVisible.value = false;
  try {
    window.localStorage.setItem("smartcut:first-launch-v2", "true");
  } catch {
    // 本地存储不可用时不影响软件继续使用。
  }
}

function openApiConfigFromWizard() {
  activeTool.value = "apiKeys";
}

async function refreshConfigurationAndDiagnostics() {
  const [apiResult, diagnosticResult] = await Promise.allSettled([
    getApiConfigStatus(),
    getDiagnosticInfo(),
  ]);

  if (apiResult.status === "fulfilled") {
    apiConfigStatus.value = apiResult.value;
  }
  if (diagnosticResult.status === "fulfilled") {
    diagnosticInfo.value = diagnosticResult.value;
  }

  const failure = [apiResult, diagnosticResult].find(
    (result): result is PromiseRejectedResult => result.status === "rejected",
  );
  if (failure) {
    readinessError.value = toErrorMessage(failure.reason, "读取配置和诊断信息失败。");
  }
}

async function refreshUserTestReadiness() {
  isRefreshingReadiness.value = true;
  readinessError.value = null;
  diagnosticFeedback.value = null;
  try {
    await Promise.all([runEnvironmentCheck(), refreshConfigurationAndDiagnostics()]);
  } finally {
    isRefreshingReadiness.value = false;
  }
}

async function generateDiagnosticReport() {
  diagnosticFeedback.value = null;
  readinessError.value = null;
  try {
    const path = await requestDiagnosticReport();
    diagnosticFeedback.value = `诊断报告已生成：${path}`;
    diagnosticInfo.value = await getDiagnosticInfo();
  } catch (error) {
    readinessError.value = toErrorMessage(error, "生成诊断报告失败。");
  }
}

async function openDiagnosticsDirectory() {
  const path = diagnosticInfo.value?.logsDirectory;
  if (!path) {
    readinessError.value = "诊断目录尚未准备完成，请先刷新状态。";
    return;
  }
  try {
    await openPathInFileManager(path);
  } catch (error) {
    readinessError.value = toErrorMessage(error, "无法打开诊断目录。");
  }
}

async function openLegalDirectory() {
  const path = diagnosticInfo.value?.legalDirectory;
  if (!path) {
    readinessError.value = "完整说明目录尚未找到，请先刷新状态。";
    return;
  }
  try {
    await openPathInFileManager(path);
  } catch (error) {
    readinessError.value = toErrorMessage(error, "无法打开隐私和许可证目录。");
  }
}

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

const watermarkPreviewUrl = computed(() =>
  selectedVideo.value ? convertFileSrc(selectedVideo.value.filePath) : null,
);

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
    isAnalyzingAiContent.value ||
    isGeneratingAiRemix.value,
);

const primaryTaskActionLabel = computed(() => {
  if (workspaceMode.value === "tools") {
    return importedVideos.value.length > 0 ? "请从上方选择一个工具" : "可直接打开语音或密钥工具";
  }

  if (workspaceMode.value === "batch") {
    if (importedVideos.value.length === 0) return "请先导入一组素材";
    if (aiPreparedSegments.value.length < 2) return "智能切片全部素材";
    if (!outputDirectory.value) return "选择批量输出目录";
    return `生成 ${batchGenerateCount.value} 条差异视频`;
  }

  if (importedVideos.value.length === 0) {
    return "请先导入素材";
  }

  if (aiPreparedSegments.value.length < 2) {
    return importedVideos.value.length > 1
      ? `${splitMode.value === "scene" ? "智能切片" : "切片"}全部素材（${importedVideos.value.length} 个）`
      : splitMode.value === "scene" ? "智能切片当前视频" : "切片当前视频";
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
    workspaceMode.value === "tools" ||
    isAnyProcessing.value ||
    importedVideos.value.length === 0 ||
    (workspaceMode.value === "ai" &&
      aiPreparedSegments.value.length >= 2 &&
      aiPlannedShots.value.length < 2 &&
      aiScript.value.trim().length === 0),
);

async function runPrimaryTaskAction() {
  if (primaryTaskActionDisabled.value) {
    return;
  }

  if (workspaceMode.value === "batch") {
    if (aiPreparedSegments.value.length < 2) {
      await splitSelectedVideo();
      return;
    }
    if (!outputDirectory.value) {
      await selectOutputDirectory();
      return;
    }
    await generateBatchMixes();
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
    return environment.value;
  } catch (error) {
    checkError.value =
      error instanceof Error ? error.message : "未检测到 FFmpeg，请配置路径。";
    return null;
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
    splitMode.value = "scene";
    sceneSensitivity.value = "balanced";
    minimumSegmentSeconds.value = 2;
    maximumSegmentSeconds.value = 10;
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

  if (tool === "watermark") {
    resetWatermarkSettings();
    resetWatermarkRemovalSettings();
    return;
  }

  if (tool === "cover" || tool === "frame") {
    coverFrameSeconds.value = 1;
    coverError.value = null;
    return;
  }

  if (tool === "export") {
    outputResolution.value = "followCanvas";
    outputFrameRate.value = "source";
    outputQuality.value = "standard";
    outputEncoder.value = "auto";
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

async function selectWatermarkImageFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "图片水印",
          extensions: ["png", "jpg", "jpeg", "webp", "bmp"],
        },
      ],
    });

    if (!selected || Array.isArray(selected)) return;
    watermarkImageFilePath.value = selected;
    watermarkKind.value = "image";
    watermarkEnabled.value = true;
  } catch (error) {
    mixError.value = error instanceof Error ? error.message : String(error ?? "选择图片水印失败。");
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

function toErrorMessage(error: unknown, fallback: string) {
  if (error instanceof Error) return error.message;
  const message = String(error ?? "").trim();
  return message || fallback;
}

onMounted(() => {
  try {
    isWelcomeVisible.value = window.localStorage.getItem("smartcut:first-launch-v2") !== "true";
  } catch {
    isWelcomeVisible.value = true;
  }
  void refreshUserTestReadiness();
  void loadTtsConfig();
  void loadAsrConfig();
  void loadScriptLibrary();
  void detectEncoders(false);
  void cleanupTaskTempFiles(true);
  void loadMembershipStatus().then((status) => {
    if (status?.signedIn) void refreshMembershipStatus(true);
  });
});
</script>

<template>
  <main class="app-shell" :class="{ 'app-shell--home': !isWorkspaceVisible }">
    <header class="top-bar">
      <div class="product-mark">
        <img class="product-logo" :src="smartCutIconUrl" alt="" />
        <div class="product-copy">
          <h1>智剪 <span>SmartCut</span></h1>
          <small>{{ isWorkspaceVisible ? workspaceModeCopy.subtitle : "本地 AI 视频工作台" }}</small>
        </div>
        <button v-if="isWorkspaceVisible" class="back-button" type="button" @click="isWorkspaceVisible = false">
          ← 首页
        </button>
      </div>
      <WorkspaceModeNav
        v-if="isWorkspaceVisible"
        :active-mode="workspaceMode"
        @change="changeWorkspaceMode"
      />
      <div class="top-status">
        <span class="mode-pill">{{ isWorkspaceVisible ? workspaceModeCopy.title : "数据只保存在这台电脑" }}</span>
        <span class="health-pill" :class="{ 'health-pill--ok': environment?.available }">
          {{ environment?.available ? "FFmpeg 就绪" : "FFmpeg 未就绪" }}
        </span>
        <ProjectSaveStatus
          :status="projectSaveStatus"
          :text="projectSaveStatusText"
          :error="projectSaveError"
        />
        <button
          class="top-help-button membership-top-button"
          :class="{ 'membership-top-button--active': membershipStatus?.memberActive }"
          type="button"
          @click="openMembershipDialog"
        >{{ membershipTopBarText }}</button>
        <button class="top-help-button" type="button" @click="openWelcomeGuide()">新手教程</button>
        <button class="top-help-button" type="button" @click="openDiagnosticGuide">诊断</button>
      </div>
    </header>

    <MembershipDialog
      :visible="isMembershipVisible"
      :status="membershipStatus"
      :active-action="membershipActiveAction"
      :error="membershipError"
      :feedback="membershipFeedback"
      @close="isMembershipVisible = false"
      @login="loginMembershipAccount"
      @register="registerMembershipAccount"
      @refresh="refreshMembershipStatus(false)"
      @redeem="redeemMembershipCode"
      @change-password="changeMembershipPassword"
      @logout="logoutMembershipAccount"
      @api-config-changed="refreshSpeechConfiguration"
    />

    <FirstLaunchWizard
      :visible="isWelcomeVisible"
      :start-step="welcomeStartStep"
      :environment="environment"
      :api-status="apiConfigStatus"
      :diagnostics="diagnosticInfo"
      :is-refreshing="isRefreshingReadiness || isChecking"
      :error="readinessError ?? checkError"
      :report-feedback="diagnosticFeedback"
      @close="closeWelcomeGuide"
      @refresh="refreshUserTestReadiness"
      @open-api-config="openApiConfigFromWizard"
      @create-report="generateDiagnosticReport"
      @open-logs="openDiagnosticsDirectory"
      @open-legal="openLegalDirectory"
    />

    <HomePage
      v-if="!isWorkspaceVisible"
      :has-recent-project="hasHomeProject"
      :recent-project-title="homeProjectTitle"
      :recent-project-summary="homeProjectSummary"
      :environment-available="environment?.available ?? null"
      :ai-configured="apiConfigStatus?.aiConfigured ?? null"
      :speech-configured="apiConfigStatus?.ttsConfigured ?? null"
      :diagnostics-available="diagnosticInfo !== null"
      @open-workspace="openWorkspace"
      @continue-project="continueHomeProject"
      @open-welcome="openWelcomeGuide"
    />

    <section
      v-else
      class="workbench"
      :class="[
        { 'workbench--advanced': isAdvancedMode },
        `workbench--${workspaceMode}`,
      ]"
    >
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
        :prepared-segments="aiPreparedSegments"
        :is-analyzing-ai-content="isAnalyzingAiContent"
        :ai-content-analysis-progress-text="aiContentAnalysisProgressText"
        :ai-content-analysis-error="aiContentAnalysisError"
        :video-cover-urls="videoCoverUrls"
        :segment-thumbnail-urls="segmentThumbnailUrls"
        :material-library-status-text="materialLibraryStatusText"
        :material-library-error="materialLibraryLoadError ?? materialLibrarySaveError"
        :has-available-material-library="hasAvailableLibrary"
        :is-loading-material-library="isLoadingMaterialLibrary"
        :missing-materials="missingMaterials"
        :missing-segment-count="missingSegmentCount"
        :relinking-material-path="relinkingMaterialPath"
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
        @analyze-ai-content="analyzePreparedSegmentContent"
        @update-segment-content-analysis="updateAiSegmentContentAnalysis"
        @relink-missing-material="relinkMissingMaterial"
        @load-material-library="loadAvailableLibrary"
      />

      <PreviewPanel
        v-if="workspaceMode === 'ai'"
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

      <BatchWorkspacePanel
        v-else-if="workspaceMode === 'batch'"
        :imported-video-count="importedVideos.length"
        :selected-video="selectedVideo"
        :preview-url="previewUrl"
        :split-segment-count="splitSegmentCount"
        :random-selected-count="randomSelectedSegments.length"
        :batch-generate-count="batchGenerateCount"
        :batch-mix-result-count="batchMixResults.length"
        :is-splitting="isSplitting"
        :is-mixing="isMixing"
        :is-batch-mixing="isBatchMixing"
        :split-error="splitError || outputDirectoryError"
        :mix-error="mixError"
        :batch-mix-error="batchMixError"
        @update:batch-generate-count="batchGenerateCount = $event"
        @split-selected-video="splitSelectedVideo"
        @pick-segments-randomly="pickSegmentsRandomly"
        @concat-random-segments="concatRandomSegments"
        @concat-categorized-segments="concatCategorizedSegments"
        @generate-batch-mixes="generateBatchMixes"
        @open-drawer="activeDrawer = $event"
        @open-tool="activeTool = $event"
      />

      <VideoToolsWorkspace
        v-else
        :selected-video="selectedVideo"
        :preview-url="previewUrl"
        :imported-video-count="importedVideos.length"
        @open-drawer="activeDrawer = $event"
        @open-tool="activeTool = $event"
      />

      <RightToolPanel
        v-if="workspaceMode !== 'tools'"
        :workspace-mode="productionWorkspaceMode"
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
      :split-mode="splitMode"
      :scene-sensitivity="sceneSensitivity"
      :minimum-segment-seconds="minimumSegmentSeconds"
      :maximum-segment-seconds="maximumSegmentSeconds"
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
      :asr-configured="asrConfig.configured"
      :asr-resource-id="asrConfig.resourceId"
      :asr-source-file-path="asrSourceFilePath"
      :asr-source-file-name="asrSourceFileName"
      :is-loading-asr-config="isLoadingAsrConfig"
      :is-recognizing-asr="isRecognizingAsr"
      :asr-error="asrError"
      :asr-result="asrResult"
      :is-saving-asr-to-library="isSavingScript"
      :asr-library-feedback="scriptLibraryError ?? scriptLibraryFeedback"
      :watermark-settings="watermarkSettings"
      :watermark-removal-settings="watermarkRemovalSettings"
      :watermark-preview-url="watermarkPreviewUrl"
      :output-resolution="outputResolution"
      :output-frame-rate="outputFrameRate"
      :output-quality="outputQuality"
      :output-encoder="outputEncoder"
      :encoder-capabilities="encoderCapabilities"
      :encoder-detection-error="encoderDetectionError"
      :is-detecting-encoders="isDetectingEncoders"
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
      @select-asr-source-file="selectAsrSourceFile"
      @recognize-asr="recognizeAsr"
      @clear-asr-selection="clearAsrSelection"
      @save-asr-to-library="saveCurrentAsrToLibrary(false)"
      @save-and-use-asr-script="saveCurrentAsrToLibrary(true)"
      @select-watermark-image="selectWatermarkImageFile"
      @api-config-changed="refreshSpeechConfiguration"
      @detect-encoders="detectEncoders"
      @update:segment-duration-seconds="segmentDurationSeconds = $event"
      @update:split-mode="splitMode = $event"
      @update:scene-sensitivity="sceneSensitivity = $event"
      @update:minimum-segment-seconds="minimumSegmentSeconds = $event"
      @update:maximum-segment-seconds="maximumSegmentSeconds = $event"
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
      @update:watermark-enabled="watermarkEnabled = $event"
      @update:watermark-kind="watermarkKind = $event"
      @update:watermark-text="watermarkText = $event"
      @update:watermark-position="watermarkPosition = $event"
      @update:watermark-opacity="watermarkOpacity = $event"
      @update:watermark-margin="watermarkMargin = $event"
      @update:watermark-text-font-size="watermarkTextFontSize = $event"
      @update:watermark-text-color="watermarkTextColor = $event"
      @update:watermark-image-size-ratio="watermarkImageSizeRatio = $event"
      @update:watermark-removal-enabled="watermarkRemovalEnabled = $event"
      @update:watermark-removal-mode="watermarkRemovalMode = $event"
      @update:watermark-removal-position="watermarkRemovalPosition = $event"
      @update:watermark-removal-size="watermarkRemovalSize = $event"
      @update:watermark-removal-margin="watermarkRemovalMargin = $event"
      @update:watermark-removal-strength="watermarkRemovalStrength = $event"
      @update:watermark-removal-cover-color="watermarkRemovalCoverColor = $event"
      @update:watermark-removal-cover-opacity="watermarkRemovalCoverOpacity = $event"
      @update:watermark-removal-tracking-enabled="watermarkRemovalTrackingEnabled = $event"
      @update:watermark-removal-tracking-region-width-ratio="watermarkRemovalTrackingRegionWidthRatio = $event"
      @update:watermark-removal-tracking-region-height-ratio="watermarkRemovalTrackingRegionHeightRatio = $event"
      @update:watermark-removal-tracking-keyframes="watermarkRemovalTrackingKeyframes = $event"
      @update:output-resolution="outputResolution = $event"
      @update:output-frame-rate="outputFrameRate = $event"
      @update:output-quality="outputQuality = $event"
      @update:output-encoder="outputEncoder = $event"
    />

    <TaskLogDrawer
      :open="activeDrawer === 'logs'"
      :task-logs="taskLogs"
      @close="activeDrawer = null"
    />

    <ScriptLibraryDrawer
      :open="activeDrawer === 'scripts'"
      :entries="scriptLibraryEntries"
      :is-loading="isLoadingScriptLibrary"
      :deleting-id="deletingScriptId"
      :error="scriptLibraryError"
      :feedback="scriptLibraryFeedback"
      @close="activeDrawer = null"
      @use="useScriptEntry"
      @delete="removeScriptEntry"
    />

    <ProjectRecoveryDialog
      :result="pendingProjectSnapshot"
      :load-error="projectRecoveryLoadError"
      :restore-error="projectRecoveryError"
      :is-restoring="isRestoringProject"
      @restore="restorePendingSnapshot"
      @discard="discardProjectSnapshotAndStartBlank"
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
