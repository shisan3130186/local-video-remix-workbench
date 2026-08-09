<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { computed, onMounted, ref, watch } from "vue";
import type { CSSProperties } from "vue";
import ExportResultDrawer from "./components/ExportResultDrawer.vue";
import TaskQueuePanel from "./components/TaskQueuePanel.vue";
import AiSourceSidebar from "./components/AiSourceSidebar.vue";
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
import type { AiRemixMatchMode } from "./features/ai-remix";
import type { ApiConfigStatus } from "./features/api-config";
import { getApiConfigStatus } from "./features/api-config/services/apiConfigService";
import { useAsr } from "./features/asr";
import { CopyRewriteWorkbench } from "./features/copy-rewrite";
import { rewriteScripts } from "./features/copy-rewrite/services/copyRewriteService";
import { FileRenamerWorkbench } from "./features/file-renamer";
import { ImageToVideoWorkbench } from "./features/image-to-video";
import {
  buildMaterialLibraryState,
  sanitizeMaterialLibrarySnapshot,
  useMaterialLibrary,
} from "./features/material-library";
import type { MaterialLibraryState } from "./features/material-library";
import { useMaterials } from "./features/materials";
import type { FixedMaterialKind } from "./features/materials/types";
import type { MaterialFolderSettings } from "./features/materials/types";
import { AccessRequirementDialog, MembershipDialog, useMembership } from "./features/membership";
import { PosterMakerWorkbench } from "./features/poster-maker";
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
import { SubtitleEditorWorkbench } from "./features/subtitle-editor";
import type { ScriptLibraryEntry } from "./features/script-library";
import { ThemeSettingsDialog, useTheme } from "./features/theme";
import { useTts } from "./features/tts";
import { useTaskCenter } from "./features/task-center";
import { buildSmartEffectPlan } from "./features/video-effects/smartConfig";
import type { SmartEffectPlan } from "./features/video-effects/smartConfig";
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
import {
  createAiRemixOutputDirectory,
  exportJianyingDraftPackage,
  isExistingDirectory,
  openPathInFileManager,
} from "./services/fileManagerService";
import { checkFfmpegEnvironment } from "./services/videoProbeService";
import type { FfmpegEnvironmentResult, ImportedVideo } from "./types/videoProbe";
import type { DrawerKey, FeatureKey, ToolKey, WorkspaceMode } from "./types/workbench";

const environment = ref<FfmpegEnvironmentResult | null>(null);
const smartCutIconUrl = "/smartcut-icon.svg";
const isDesktopRuntime = "__TAURI_INTERNALS__" in window;
const isChecking = ref(true);
const checkError = ref<string | null>(null);
const outputDirectory = ref<string | null>(null);
const generationOutputDirectory = ref<string | null>(null);
const generationVideoOutputDirectory = computed(() =>
  generationOutputDirectory.value
    ? `${generationOutputDirectory.value}\\视频成片`
    : null,
);
const outputDirectoryError = ref<string | null>(null);
const fileManagerError = ref<string | null>(null);
const generationOutputError = ref<string | null>(null);
const aiRemixInputMode = ref<"custom" | "script" | "audio">("custom");
const aiMatchMode = ref<AiRemixMatchMode>("local");
const draftExportFeedback = ref<string | null>(null);
const previewVideoRef = ref<HTMLVideoElement | null>(null);
const previewBackgroundVideoRef = ref<HTMLVideoElement | null>(null);
const isWorkspaceVisible = ref(false);
const workspaceMode = ref<WorkspaceMode>("ai");
const activeFeature = ref<FeatureKey | null>(null);
const activeTool = ref<ToolKey | null>(null);
const frameMaterialFilePath = ref<string | null>(null);
const fusionMaterialFilePath = ref<string | null>(null);
const smartEffectPlan = ref<SmartEffectPlan | null>(null);
const isSmartConfiguring = ref(false);
const smartConfigError = ref<string | null>(null);
const activeDrawer = ref<DrawerKey | null>(null);
const isWelcomeVisible = ref(false);
const welcomeStartStep = ref(0);
const apiConfigStatus = ref<ApiConfigStatus | null>(null);
const diagnosticInfo = ref<DiagnosticInfo | null>(null);
const isRefreshingReadiness = ref(false);
const readinessError = ref<string | null>(null);
const diagnosticFeedback = ref<string | null>(null);
const isAdvancedMode = ref(false);
const isThemeSettingsVisible = ref(false);
const isAccessRequirementVisible = ref(false);
const membershipInitialPage = ref<"profile" | "api">("profile");
const missingAuthorization = ref(false);
const missingAiService = ref(false);
const missingTtsService = ref(false);
const {
  preference: themePreference,
  resolvedTheme,
  setThemePreference,
} = useTheme();

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
  clearTaskHistory,
  isCleaning,
  isTaskRunning,
  runTask,
  taskHistory,
} = useTaskCenter();
const tempCleanupFeedback = ref<string | null>(null);
const isTaskQueuePanelVisible = ref<boolean>(false);
const userClosedTaskQueuePanel = ref<boolean>(false);
watch(isTaskRunning, (running, previous) => {
  if (running && !previous) {
    // 任务一启动就自动展开队列面板（用户已手动关闭过的会话不再强制展开）
    if (!userClosedTaskQueuePanel.value) {
      isTaskQueuePanelVisible.value = true;
    }
  }
});

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
  hslEnabled,
  hue,
  canvasAspectRatio,
  canvasBackgroundMode,
  contrast,
  detectEncoders,
  encoderCapabilities,
  encoderDetectionError,
  effectScale,
  zoomEnabled,
  zoomMode,
  zoomMinScale,
  zoomMaxScale,
  zoomMinDurationSeconds,
  zoomMaxDurationSeconds,
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
  subtitleEnabled,
  subtitlePosition,
  subtitleSize,
  subtitleText,
  resetWatermarkSettings,
  resetWatermarkRemovalSettings,
  validateWatermarkSettings,
  validateWatermarkRemovalSettings,
  watermarkEnabled,
  watermarkAssetType,
  watermarkImageFilePath,
  watermarkImagePositionXRatio,
  watermarkImagePositionYRatio,
  watermarkImageSizeRatio,
  watermarkKind,
  watermarkMargin,
  watermarkOpacity,
  watermarkPosition,
  watermarkSettings,
  watermarkText,
  watermarkTextColor,
  watermarkTextFontSize,
  watermarkTrajectory,
  watermarkRemovalCoverColor,
  watermarkRemovalCoverOpacity,
  watermarkRemovalEnabled,
  watermarkRemovalRegionCount,
  watermarkRemovalManualRegions,
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
  clearImportedVideos,
  generateCoverFrame,
  importError,
  importedVideos,
  importVideoFolder: importMaterialFolder,
  importVideos: importMaterialVideos,
  isGeneratingCover,
  isImporting,
  isSplitting,
  maximumSegmentSeconds,
  materialFolders,
  applyMaterialFolderSettingsToAll,
  removeMaterialFolder,
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
  updateMaterialFolderSettings,
  videoCoverUrls,
  videoCoverPaths,
} = useMaterials({
  outputDirectory,
  appendSplitLog,
  clearSplitLogs,
  runTask,
});

watch(selectedVideo, (video, previousVideo) => {
  if (video?.id === previousVideo?.id) return;
  smartEffectPlan.value = null;
  smartConfigError.value = null;
});

const {
  batchGenerateCount,
  batchMixError,
  batchMixFailures,
  batchMixResults,
  concatCategorizedSegments,
  concatRandomSegments,
  activeProcessingVideoId,
  exportError,
  exportImportedVideos,
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
  videoProcessingStates,
} = useRemixExport({
  importedVideos,
  selectedVideo,
  outputDirectory,
  remixExportSettings,
  segmentPaths: splitSegmentPaths,
  segmentCategories,
  categoryOptions: segmentCategoryOptions,
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
  prepareAiRemixVariants,
  prepareSegmentAssets,
  requestAiRemixPlan,
  resetAiRemixState,
  restoreAiPreparedSegments,
  retryFailedAiRemixVideos,
  restoreAiRemixState,
  updateAiSegmentContentAnalysis,
} = useAiRemix({
  outputDirectory,
  generationOutputDirectory: generationVideoOutputDirectory,
  matchMode: aiMatchMode,
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

const isRewritingAiScript = ref(false);
const aiRewriteFeedback = ref<string | null>(null);

async function rewriteAiScript() {
  const source = aiScript.value.trim();
  if (!source || isRewritingAiScript.value) return;
  isRewritingAiScript.value = true;
  aiRewriteFeedback.value = null;
  try {
    const result = await rewriteScripts({
      scripts: [source],
      style: "conservative",
      targetLength: "similar",
      customPrompt: "保留原意和信息点，润色成自然、适合短视频口播的文案。",
      count: 1,
    });
    const rewritten = result.items[0]?.versions[0]?.trim();
    if (!rewritten) throw new Error("AI 没有返回可用的改写文案。");
    aiScript.value = rewritten;
    aiRewriteFeedback.value = "AI 改写完成";
  } catch (error) {
    aiRewriteFeedback.value = error instanceof Error ? error.message : "AI 改写失败，请检查模型配置。";
  } finally {
    isRewritingAiScript.value = false;
  }
}

watch(
  aiScript,
  (value) => {
    subtitleText.value = value;
  },
  { immediate: true },
);

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
  generationOutputDirectory: generationVideoOutputDirectory,
  narrationOutputDirectory: generationOutputDirectory,
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
      materialFolders: materialFolders.value,
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
  enabled: isDesktopRuntime,
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
  if (activeFeature.value === "copyRewrite") return { title: "文案改写", subtitle: "单条与批量AI文案改写" };
  if (activeFeature.value === "fileRenamer") return { title: "文件批量改名", subtitle: "规则预览与安全批量执行" };
  if (activeFeature.value === "subtitleEditor") return { title: "字幕识别", subtitle: "语音识别、逐句编辑与字幕导出" };
  if (activeFeature.value === "posterMaker") return { title: "大字报设计", subtitle: "营销文字排版与PNG素材导出" };
  if (activeFeature.value === "imageToVideo") return { title: "图片转视频", subtitle: "批量生成可直接混剪的视频素材" };
  if (workspaceMode.value === "batch") {
    return { title: "批量混剪", subtitle: "多素材规则组合与批量生成" };
  }
  if (workspaceMode.value === "tools") {
    return { title: "视频工具", subtitle: "单项处理中心" };
  }
  return { title: "AI 智能成片", subtitle: "文案驱动的智能成片流程" };
});

const batchWorkspaceKind = ref<"remix" | "category">("remix");
const toolsWorkspaceView = ref<"effects" | "extract">("extract");
const homeSection = ref<"creation" | "utilities">("creation");

const productionWorkspaceMode = computed<"ai" | "batch">(() =>
  workspaceMode.value === "batch" ? "batch" : "ai",
);

function openMembershipCenter(page: "profile" | "api" = "profile") {
  membershipInitialPage.value = page;
  openMembershipDialog();
}

function ensureFeatureAccess(): boolean {
  if (!isDesktopRuntime) return true;

  missingAuthorization.value = membershipStatus.value?.memberActive !== true;
  missingAiService.value = apiConfigStatus.value?.aiConfigured !== true;
  missingTtsService.value = apiConfigStatus.value?.ttsConfigured !== true;
  if (!missingAuthorization.value && !missingAiService.value && !missingTtsService.value) {
    return true;
  }

  isAccessRequirementVisible.value = true;
  return false;
}

function confirmAccessRequirement() {
  isAccessRequirementVisible.value = false;
  openMembershipCenter(missingAuthorization.value ? "profile" : "api");
}

function openWorkspace(mode: WorkspaceMode = "ai", batchKind?: "remix" | "category") {
  if (!ensureFeatureAccess()) return;
  activeFeature.value = null;
  workspaceMode.value = mode;
  if (mode === "batch" && batchKind) batchWorkspaceKind.value = batchKind;
  if (mode === "tools") toolsWorkspaceView.value = "extract";
  activeTool.value = null;
  activeDrawer.value = null;
  isWorkspaceVisible.value = true;
}

function openWorkspaceTool(tool: ToolKey) {
  if (!ensureFeatureAccess()) return;
  activeFeature.value = null;
  workspaceMode.value = "tools";
  if (tool === "effects") {
    toolsWorkspaceView.value = "effects";
    activeTool.value = null;
  } else {
    activeTool.value = tool;
  }
  activeDrawer.value = null;
  isWorkspaceVisible.value = true;
}

function changeWorkspaceMode(mode: WorkspaceMode) {
  activeFeature.value = null;
  workspaceMode.value = mode;
  activeTool.value = null;
  activeDrawer.value = null;
}

function openFeature(feature: FeatureKey) {
  if (!ensureFeatureAccess()) return;
  activeFeature.value = feature;
  activeTool.value = null;
  activeDrawer.value = null;
  isWorkspaceVisible.value = true;
}

function goHome() {
  activeFeature.value = null;
  activeTool.value = null;
  activeDrawer.value = null;
  isWorkspaceVisible.value = false;
}

watch(
  [
    () => membershipStatus.value?.memberActive,
    () => apiConfigStatus.value?.aiConfigured,
    () => apiConfigStatus.value?.ttsConfigured,
  ],
  ([memberActive, aiConfigured, ttsConfigured]) => {
    if (!isDesktopRuntime || !isWorkspaceVisible.value) return;
    if (memberActive === true && aiConfigured === true && ttsConfigured === true) return;
    goHome();
    ensureFeatureAccess();
  },
);

function useFeatureText(text: string) {
  aiScript.value = text;
  openWorkspace("ai");
}

async function continueHomeProject() {
  if (!ensureFeatureAccess()) return;
  if (pendingProjectSnapshot.value?.snapshot) {
    await restorePendingSnapshot();
    return;
  }
  openWorkspace("ai");
}

async function restoreProjectWithAccess() {
  if (!ensureFeatureAccess()) return;
  await restorePendingSnapshot();
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

  if (!(await ensureAiGenerationOutputDirectory())) {
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

async function ensureAiGenerationOutputDirectory() {
  generationOutputError.value = null;
  draftExportFeedback.value = null;
  if (generationOutputDirectory.value) {
    try {
      if (await isExistingDirectory(generationOutputDirectory.value)) {
        return true;
      }
    } catch (error) {
      generationOutputError.value = toErrorMessage(error, "无法检查本次任务目录。");
      return false;
    }
    generationOutputDirectory.value = null;
  }

  if (!outputDirectory.value) {
    generationOutputError.value = "请先选择输出目录。";
    return false;
  }

  try {
    generationOutputDirectory.value = await createAiRemixOutputDirectory(
      outputDirectory.value,
      selectedVideo.value?.fileName?.replace(/\.[^.]+$/, "") || "AI混剪任务",
    );
    return true;
  } catch (error) {
    generationOutputError.value = toErrorMessage(error, "无法创建本次 AI 混剪任务目录。");
    return false;
  }
}

async function openAiGenerationDirectory() {
  if (!generationOutputDirectory.value) {
    fileManagerError.value = "本次任务还没有生成结果。";
    return;
  }
  await openPath(generationOutputDirectory.value);
}

async function exportAiJianyingDraft() {
  draftExportFeedback.value = null;
  if (!generationOutputDirectory.value) {
    draftExportFeedback.value = "本次任务还没有生成结果。";
    return;
  }
  const paths = [...aiGeneratedResults.value, ...narratedVideoResults.value];
  try {
    const folder = await exportJianyingDraftPackage(
      generationOutputDirectory.value,
      selectedVideo.value?.fileName?.replace(/\.[^.]+$/, "") || "AI混剪草稿",
      paths,
      aiScript.value,
    );
    draftExportFeedback.value = `剪映草稿已整理到：${folder}`;
  } catch (error) {
    draftExportFeedback.value = toErrorMessage(error, "剪映草稿导出失败。");
  }
}

function useAudioAsRemixScript() {
  if (!asrResult.value?.text.trim()) {
    generationOutputError.value = "请先选择音频并完成语音识别。";
    activeTool.value = "asr";
    return;
  }
  aiScript.value = asrResult.value.text.trim();
  aiRemixInputMode.value = "audio";
  generationOutputError.value = null;
  activeTool.value = null;
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

const watermarkAssetPreviewUrl = computed(() =>
  watermarkImageFilePath.value ? convertFileSrc(watermarkImageFilePath.value) : null,
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

const videoToolsProcessingProgress = computed(() =>
  activeProcessingVideoId.value && isExporting.value ? activeTask.value?.progressPercent ?? 0 : 0,
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
    generationOutputDirectory.value = null;
    resetAiRemixState(true);
    resetRandomPickState();
  }
}

async function importVideoFolder() {
  if (await importMaterialFolder()) {
    generationOutputDirectory.value = null;
    resetAiRemixState(true);
    resetRandomPickState();
  }
}

async function selectFixedMaterial(folderId: string, kind: FixedMaterialKind) {
  const selected = await open(
    kind === "folder"
      ? { directory: true, multiple: false }
      : {
          multiple: false,
          filters: [{ name: "视频文件", extensions: ["mp4", "mov", "avi", "mkv"] }],
        },
  );
  if (!selected || Array.isArray(selected)) return;

  updateMaterialFolderSettings(folderId, {
    fixedFirstMaterialEnabled: true,
    fixedFirstMaterialPath: selected,
    fixedFirstMaterialKind: kind,
  });
}

function updateFolderSettings(
  folderId: string,
  patch: Partial<MaterialFolderSettings>,
) {
  updateMaterialFolderSettings(folderId, patch);
}

function selectVideo(video: ImportedVideo) {
  selectMaterialVideo(video);
  generationOutputDirectory.value = null;
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
    generationOutputDirectory.value = null;
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

async function requestSmartEffectConfig() {
  smartConfigError.value = null;

  if (!selectedVideo.value) {
    smartConfigError.value = "请先选择一个视频素材，再让 AI 分析画面。";
    return;
  }

  if (!outputDirectory.value) {
    smartConfigError.value = "请先选择输出目录，AI 需要把临时分析帧写入该目录。";
    outputDirectoryError.value = smartConfigError.value;
    return;
  }

  isSmartConfiguring.value = true;

  try {
    const plan = await buildSmartEffectPlan(
      selectedVideo.value,
      outputDirectory.value,
      aiMatchMode.value,
    );
    smartEffectPlan.value = plan;
    applySmartEffectDefaults(plan);
  } catch (error) {
    smartEffectPlan.value = null;
    smartConfigError.value =
      error instanceof Error ? error.message : String(error ?? "AI 智能配置失败，请稍后重试。");
  } finally {
    isSmartConfiguring.value = false;
  }
}

function applySmartEffectDefaults(plan: SmartEffectPlan) {
  const enabled = new Set(plan.enabledEffectKeys);

  applyHorizontalMirror.value = false;
  applyVerticalMirror.value = false;
  rotationMode.value = "none";

  hslEnabled.value = enabled.has("effects");
  if (hslEnabled.value) {
    hue.value = 0;
    brightness.value = 0.02;
    contrast.value = 1.03;
    saturation.value = 1.04;
  }

  zoomEnabled.value = enabled.has("zoom");
  if (zoomEnabled.value) {
    zoomMode.value = "random";
    zoomMinScale.value = 1.02;
    zoomMaxScale.value = 1.08;
    zoomMinDurationSeconds.value = 6;
    zoomMaxDurationSeconds.value = 10;
  }

  playbackSpeed.value = enabled.has("speed") ? 1.03 : 1;
  bgmEnabled.value = false;
  pipEnabled.value = false;
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
    zoomEnabled.value = false;
    zoomMode.value = "push";
    zoomMinScale.value = 1.02;
    zoomMaxScale.value = 1.08;
    zoomMinDurationSeconds.value = 8;
    zoomMaxDurationSeconds.value = 10;
    return;
  }

  if (tool === "frame") {
    frameMaterialFilePath.value = null;
    return;
  }

  if (tool === "fusion") {
    fusionMaterialFilePath.value = null;
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

  if (tool === "cover") {
    coverFrameSeconds.value = 1;
    selectedCoverPath.value = null;
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

async function selectFrameMaterialFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: "视频帧操作素材", extensions: ["mp4", "mov", "avi", "mkv", "png", "jpg", "jpeg", "webp", "bmp"] }],
    });
    if (!selected || Array.isArray(selected)) return;
    frameMaterialFilePath.value = selected;
  } catch (error) {
    mixError.value = error instanceof Error ? error.message : String(error ?? "选择视频帧操作素材失败。");
  }
}

async function selectFusionMaterialFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: "像素融合素材", extensions: ["mp4", "mov", "avi", "mkv", "png", "jpg", "jpeg", "webp", "bmp"] }],
    });
    if (!selected || Array.isArray(selected)) return;
    fusionMaterialFilePath.value = selected;
  } catch (error) {
    mixError.value = error instanceof Error ? error.message : String(error ?? "选择像素融合素材失败。");
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

async function selectCoverImageFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "封面图片",
          extensions: ["png", "jpg", "jpeg", "webp", "bmp"],
        },
      ],
    });

    if (!selected || Array.isArray(selected)) {
      return;
    }

    selectedCoverPath.value = selected;
    coverError.value = null;
  } catch (error) {
    coverError.value = error instanceof Error ? error.message : String(error ?? "选择封面图片失败。");
  }
}

async function selectCoverImageFolder() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
    });

    if (!selected || Array.isArray(selected)) {
      return;
    }

    selectedCoverPath.value = selected;
    coverError.value = null;
  } catch (error) {
    coverError.value =
      error instanceof Error ? error.message : String(error ?? "选择封面图片文件夹失败。");
  }
}

async function selectWatermarkImageFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "图片或视频水印",
          extensions: ["png", "jpg", "jpeg", "webp", "bmp", "mp4", "mov", "webm", "m4v"],
        },
      ],
    });

    if (!selected || Array.isArray(selected)) return;
    watermarkImageFilePath.value = selected;
    watermarkKind.value = "image";
    watermarkAssetType.value = /\.(mp4|mov|webm|m4v)$/i.test(selected) ? "video" : "image";
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
  if (!isDesktopRuntime) {
    isChecking.value = false;
    return;
  }
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
  <main class="app-shell" :class="{ 'app-shell--home': !isWorkspaceVisible, 'app-shell--workspace': isWorkspaceVisible }">
    <header class="top-bar">
      <div class="product-mark">
        <template v-if="!isWorkspaceVisible">
          <img class="product-logo" :src="smartCutIconUrl" alt="" />
          <div class="product-copy">
          <h1>智剪 <span>SmartCut</span></h1>
            <small>本地 AI 视频工作台</small>
          </div>
        </template>
        <button v-if="isWorkspaceVisible" class="back-button" type="button" @click="goHome">
          ← 返回首页
        </button>
      </div>
      <WorkspaceModeNav
        v-if="isWorkspaceVisible"
        :active-mode="workspaceMode"
        @change="changeWorkspaceMode"
      />
      <div class="top-status">
        <span v-if="!isWorkspaceVisible" class="mode-pill">数据只保存在这台电脑</span>
        <ProjectSaveStatus v-if="!isWorkspaceVisible"
          :status="projectSaveStatus"
          :text="projectSaveStatusText"
          :error="projectSaveError"
        />
        <button
          class="top-help-button membership-top-button"
          :class="{ 'membership-top-button--active': membershipStatus?.memberActive }"
          type="button"
          @click="openMembershipCenter('profile')"
        >{{ isWorkspaceVisible ? "♙" : membershipTopBarText }}</button>
        <button
          class="top-help-button theme-top-button"
          type="button"
          aria-label="打开主题设置"
          title="主题设置"
          @click="isThemeSettingsVisible = true"
        ><span aria-hidden="true">◐</span><span v-if="!isWorkspaceVisible">主题</span></button>
        <button v-if="isWorkspaceVisible" class="top-help-button top-menu-button" type="button" aria-label="打开菜单" @click="openWelcomeGuide()">≡</button>
        <button v-else class="top-help-button" type="button" @click="openWelcomeGuide()">新手教程</button>
        <button v-if="!isWorkspaceVisible" class="top-help-button" type="button" @click="openDiagnosticGuide">诊断</button>
        <button
          v-if="isWorkspaceVisible"
          class="top-help-button task-queue-top-button"
          :class="{ 'task-queue-top-button--active': isTaskQueuePanelVisible }"
          type="button"
          aria-label="打开任务队列面板"
          title="任务队列"
          @click="isTaskQueuePanelVisible = !isTaskQueuePanelVisible; if (isTaskQueuePanelVisible) userClosedTaskQueuePanel = false"
        >任务队列</button>
      </div>
    </header>

    <MembershipDialog
      :visible="isMembershipVisible"
      :status="membershipStatus"
      :active-action="membershipActiveAction"
      :error="membershipError"
      :feedback="membershipFeedback"
      :initial-page="membershipInitialPage"
      @close="isMembershipVisible = false"
      @login="loginMembershipAccount"
      @register="registerMembershipAccount"
      @refresh="refreshMembershipStatus(false)"
      @redeem="redeemMembershipCode"
      @change-password="changeMembershipPassword"
      @logout="logoutMembershipAccount"
      @api-config-changed="refreshSpeechConfiguration"
    />

    <AccessRequirementDialog
      :visible="isAccessRequirementVisible"
      :missing-authorization="missingAuthorization"
      :missing-ai-service="missingAiService"
      :missing-tts-service="missingTtsService"
      @close="isAccessRequirementVisible = false"
      @confirm="confirmAccessRequirement"
    />

    <ThemeSettingsDialog
      :visible="isThemeSettingsVisible"
      :preference="themePreference"
      :resolved-theme="resolvedTheme"
      @close="isThemeSettingsVisible = false"
      @select="setThemePreference"
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
      v-model:active-section="homeSection"
      :has-recent-project="hasHomeProject"
      :recent-project-title="homeProjectTitle"
      :recent-project-summary="homeProjectSummary"
      :environment-available="environment?.available ?? null"
      :ai-configured="apiConfigStatus?.aiConfigured ?? null"
      :speech-configured="apiConfigStatus?.ttsConfigured ?? null"
      :diagnostics-available="diagnosticInfo !== null"
      @open-workspace="openWorkspace"
      @open-tool="openWorkspaceTool"
      @open-feature="openFeature"
      @continue-project="continueHomeProject"
      @open-welcome="openWelcomeGuide"
    />

    <CopyRewriteWorkbench v-else-if="activeFeature === 'copyRewrite'" @use-text="useFeatureText" />
    <FileRenamerWorkbench v-else-if="activeFeature === 'fileRenamer'" />
    <SubtitleEditorWorkbench
      v-else-if="activeFeature === 'subtitleEditor'"
      :configured="asrConfig.configured"
      :source-file-path="asrSourceFilePath"
      :source-file-name="asrSourceFileName"
      :is-loading-config="isLoadingAsrConfig"
      :is-recognizing="isRecognizingAsr"
      :error="asrError"
      :result="asrResult"
      @select-source="selectAsrSourceFile"
      @recognize="recognizeAsr"
      @clear="clearAsrSelection"
      @use-text="useFeatureText"
    />
    <PosterMakerWorkbench v-else-if="activeFeature === 'posterMaker'" />
    <ImageToVideoWorkbench v-else-if="activeFeature === 'imageToVideo'" />

    <section
      v-else
      class="workbench"
      :class="[
        { 'workbench--advanced': isAdvancedMode },
        `workbench--${workspaceMode}`,
      ]"
    >
      <AiSourceSidebar
        v-if="workspaceMode === 'ai'"
        :imported-videos="importedVideos"
        :selected-video="selectedVideo"
        :video-cover-urls="videoCoverUrls"
        :split-segment-paths="splitSegmentPaths"
        :selected-segment-path="selectedSegmentPath"
        :segment-thumbnail-urls="segmentThumbnailUrls"
        :format-duration="formatDuration"
        :format-file-name="formatFileName"
        :match-mode="aiMatchMode"
        :material-folders="materialFolders"
        @import-videos="importVideos"
        @import-video-folder="importVideoFolder"
        @clear-videos="clearImportedVideos"
        @select-video="selectVideo"
        @select-segment="selectSegment"
        @update-match-mode="aiMatchMode = $event"
        @remove-material-folder="removeMaterialFolder"
        @update-material-folder-settings="updateFolderSettings"
        @select-fixed-material="selectFixedMaterial"
        @apply-folder-settings-to-all="applyMaterialFolderSettingsToAll"
        @open-tool="activeTool = $event"
        @open-drawer="activeDrawer = $event"
      />

      <PreviewPanel
        v-if="workspaceMode === 'ai'"
        v-model:preview-video-ref="previewVideoRef"
        v-model:preview-background-video-ref="previewBackgroundVideoRef"
        v-model:ai-script="aiScript"
        v-model:ai-generate-count="aiGenerateCount"
        v-model:canvas-aspect-ratio="canvasAspectRatio"
        v-model:canvas-background-mode="canvasBackgroundMode"
        v-model:effect-scale="effectScale"
        v-model:watermark-removal-region-count="watermarkRemovalRegionCount"
        v-model:watermark-removal-manual-regions="watermarkRemovalManualRegions"
        v-model:watermark-enabled="watermarkEnabled"
        v-model:watermark-kind="watermarkKind"
        v-model:watermark-text="watermarkText"
        v-model:watermark-position="watermarkPosition"
        v-model:watermark-opacity="watermarkOpacity"
        v-model:watermark-image-size-ratio="watermarkImageSizeRatio"
        v-model:watermark-image-position-x-ratio="watermarkImagePositionXRatio"
        v-model:watermark-image-position-y-ratio="watermarkImagePositionYRatio"
        v-model:watermark-text-font-size="watermarkTextFontSize"
        v-model:watermark-text-color="watermarkTextColor"
        v-model:tts-subtitle-enabled="ttsSubtitleEnabled"
        :is-advanced-mode="isAdvancedMode"
        :imported-video-count="importedVideos.length"
        :selected-video="selectedVideo"
        :preview-title="previewTitle"
        :preview-url="previewUrl"
        :watermark-asset-preview-url="watermarkAssetPreviewUrl"
        :watermark-asset-type="watermarkAssetType"
        :watermark-trajectory="watermarkTrajectory"
        :watermark-removal-enabled="watermarkRemovalEnabled"
        :hsl-enabled="hslEnabled"
        :hue="hue"
        :brightness="brightness"
        :saturation="saturation"
        :zoom-enabled="zoomEnabled"
        :zoom-mode="zoomMode"
        :zoom-min-scale="zoomMinScale"
        :zoom-max-scale="zoomMaxScale"
        :zoom-min-duration-seconds="zoomMinDurationSeconds"
        :zoom-max-duration-seconds="zoomMaxDurationSeconds"
        :selected-cover-url="selectedCoverUrl"
        :should-show-blur-background="shouldShowBlurBackground"
        :preview-canvas-style="previewCanvasStyle"
        :is-splitting="isSplitting"
        :is-mixing="isMixing"
        :is-batch-mixing="isBatchMixing"
        :is-exporting="isExporting"
        :split-segment-count="splitSegmentCount"
        :split-error="splitError || outputDirectoryError"
        :random-selected-count="randomSelectedSegments.length"
        :batch-mix-result-count="batchMixResults.length + aiGeneratedResults.length + narratedVideoResults.length"
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
        :ai-remix-input-mode="aiRemixInputMode"
        :asr-source-file-name="asrSourceFileName"
        :asr-result-text="asrResult?.text ?? null"
        :generation-output-directory="generationOutputDirectory"
        :generation-output-error="generationOutputError"
        :draft-export-feedback="draftExportFeedback"
        :generation-progress-text="currentAiGenerationProgressText"
        :generation-summary-text="currentAiGenerationSummaryText"
        :generation-success-count="currentAiGenerationSuccessCount"
        :generation-failure-count="currentAiGenerationFailureCount"
        :ai-plan-error="aiPlanError"
        :ai-generate-error="currentAiGenerateError"
        :is-rewriting-ai-script="isRewritingAiScript"
        :ai-rewrite-feedback="aiRewriteFeedback"
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
        @generate-ai-remix="generateCurrentAiRemixVideo"
        @update-ai-remix-input-mode="aiRemixInputMode = $event"
        @select-audio-source="selectAsrSourceFile"
        @recognize-audio="recognizeAsr"
        @use-recognized-audio="useAudioAsRemixScript"
        @rewrite-ai-script="rewriteAiScript"
        @open-generation-directory="openAiGenerationDirectory"
        @export-jianying-draft="exportAiJianyingDraft"
      />

      <BatchWorkspacePanel
        v-else-if="workspaceMode === 'batch'"
        v-model:script="aiScript"
        v-model:original-volume="originalVolume"
        v-model:bgm-enabled="bgmEnabled"
        v-model:bgm-volume="bgmVolume"
        v-model:tts-enabled="ttsVideoEnabled"
        v-model:tts-speaker="ttsSpeaker"
        v-model:speech-volume="ttsOriginalAudioVolume"
        v-model:speech-speed="playbackSpeed"
        v-model:subtitle-enabled="subtitleEnabled"
        v-model:subtitle-position="subtitlePosition"
        v-model:subtitle-size="subtitleSize"
        v-model:watermark-removal-enabled="watermarkRemovalEnabled"
        v-model:watermark-removal-region-count="watermarkRemovalRegionCount"
        v-model:watermark-enabled="watermarkEnabled"
        v-model:watermark-asset-type="watermarkAssetType"
        v-model:watermark-image-file-path="watermarkImageFilePath"
        v-model:watermark-opacity="watermarkOpacity"
        v-model:watermark-image-size-ratio="watermarkImageSizeRatio"
        v-model:watermark-trajectory="watermarkTrajectory"
        v-model:hsl-enabled="hslEnabled"
        v-model:hue="hue"
        v-model:brightness="brightness"
        v-model:saturation="saturation"
        v-model:zoom-enabled="zoomEnabled"
        v-model:zoom-mode="zoomMode"
        v-model:zoom-min-scale="zoomMinScale"
        v-model:zoom-max-scale="zoomMaxScale"
        v-model:zoom-min-duration-seconds="zoomMinDurationSeconds"
        v-model:zoom-max-duration-seconds="zoomMaxDurationSeconds"
        v-model:canvas-aspect-ratio="canvasAspectRatio"
        v-model:canvas-background-mode="canvasBackgroundMode"
        :kind="batchWorkspaceKind"
        :imported-videos="importedVideos"
        :selected-video="selectedVideo"
        :source-preview-url="watermarkPreviewUrl"
        :preview-url="previewUrl"
        :video-cover-urls="videoCoverUrls"
        :split-segment-paths="splitSegmentPaths"
        :selected-segment-path="selectedSegmentPath"
        :segment-thumbnail-urls="segmentThumbnailUrls"
        :prepared-segments="aiPreparedSegments"
        :random-selected-count="randomSelectedSegments.length"
        :batch-generate-count="batchGenerateCount"
        :batch-mix-result-count="batchMixResults.length"
        :is-splitting="isSplitting"
        :is-mixing="isMixing"
        :is-batch-mixing="isBatchMixing"
        :split-error="splitError || outputDirectoryError"
        :mix-error="mixError"
        :batch-mix-error="batchMixError"
        :material-folders="materialFolders"
        :production-workspace-mode="productionWorkspaceMode"
        :is-advanced-mode="isAdvancedMode"
        :environment="environment"
        :status-text="statusText"
        :bgm-audio-file-path="bgmAudioFilePath"
        :is-processing="isAnyProcessing"
        :format-duration="formatDuration"
        :format-resolution="formatResolution"
        :format-file-name="formatFileName"
        @import-videos="importVideos"
        @import-video-folder="importVideoFolder"
        @select-video="selectVideo"
        @select-segment="selectSegment"
        @update:batch-generate-count="batchGenerateCount = $event"
        @split-selected-video="splitSelectedVideo"
        @pick-segments-randomly="pickSegmentsRandomly"
        @concat-random-segments="concatRandomSegments"
        @concat-categorized-segments="concatCategorizedSegments"
        @generate-batch-mixes="generateBatchMixes"
        @open-drawer="activeDrawer = $event"
        @open-tool="activeTool = $event"
        @toggle-advanced-mode="isAdvancedMode = $event"
        @select-bgm-audio-file="selectBgmAudioFile"
        @select-watermark-asset="selectWatermarkImageFile"
        @update-material-folder-settings="updateFolderSettings"
        @select-fixed-material="selectFixedMaterial"
        @apply-folder-settings-to-all="applyMaterialFolderSettingsToAll"
      />

      <VideoToolsWorkspace
        v-else
        v-model:output-resolution="outputResolution"
        v-model:output-frame-rate="outputFrameRate"
        v-model:output-quality="outputQuality"
        v-model:output-encoder="outputEncoder"
        :view="toolsWorkspaceView"
        :imported-videos="importedVideos"
        :selected-video="selectedVideo"
        :preview-url="previewUrl"
        :imported-video-count="importedVideos.length"
        :video-cover-urls="videoCoverUrls"
        :split-segment-paths="splitSegmentPaths"
        :selected-segment-path="selectedSegmentPath"
        :segment-thumbnail-urls="segmentThumbnailUrls"
        :prepared-segments="aiPreparedSegments"
        :output-directory="outputDirectory"
        :is-processing="isSplitting || isExporting || isAnalyzingAiContent"
        :progress-text="aiContentAnalysisProgressText"
        :error="splitError || aiContentAnalysisError || exportError || outputDirectoryError"
        :video-processing-states="videoProcessingStates"
        :active-processing-video-id="activeProcessingVideoId"
        :processing-progress="videoToolsProcessingProgress"
        v-model:smart-match-mode="aiMatchMode"
        :smart-effect-plan="smartEffectPlan"
        :is-smart-configuring="isSmartConfiguring"
        :smart-config-error="smartConfigError"
        :frame-material-file-path="frameMaterialFilePath"
        :fusion-material-file-path="fusionMaterialFilePath"
        :pip-enabled="pipEnabled"
        :pip-overlay-file-path="pipOverlayFilePath"
        :pip-position="pipPosition"
        :pip-size-ratio="pipSizeRatio"
        :pip-opacity="pipOpacity"
        :watermark-settings="watermarkSettings"
        :watermark-enabled="watermarkEnabled"
        :watermark-asset-type="watermarkAssetType"
        :watermark-image-file-path="watermarkImageFilePath"
        :watermark-opacity="watermarkOpacity"
        :watermark-image-size-ratio="watermarkImageSizeRatio"
        :watermark-trajectory="watermarkTrajectory"
        :watermark-removal-settings="watermarkRemovalSettings"
        :subtitle-enabled="subtitleEnabled"
        :subtitle-position="subtitlePosition"
        :subtitle-size="subtitleSize"
        :format-duration="formatDuration"
        :format-file-name="formatFileName"
        @import-videos="importVideos"
        @import-video-folder="importVideoFolder"
        @clear-videos="clearImportedVideos"
        @select-video="selectVideo"
        @select-segment="selectSegment"
        @select-output-directory="selectOutputDirectory"
        @split-selected-video="splitSelectedVideo"
        @analyze-content="analyzePreparedSegmentContent"
        @generate-categorized-segments="concatCategorizedSegments"
        @start-processing="exportImportedVideos"
        @request-smart-config="requestSmartEffectConfig"
        @open-drawer="activeDrawer = $event"
        @open-tool="activeTool = $event"
        @select-pip-overlay-file="selectPipOverlayFile"
        @select-watermark-asset="selectWatermarkImageFile"
        @update:pip-enabled="pipEnabled = $event"
        @update:pip-position="pipPosition = $event"
        @update:pip-size-ratio="pipSizeRatio = $event"
        @update:pip-opacity="pipOpacity = $event"
        @update:watermark-text-enabled="watermarkEnabled = $event"
        @update:watermark-kind="watermarkKind = $event"
        @update:watermark-text="watermarkText = $event"
        @update:watermark-position="watermarkPosition = $event"
        @update:watermark-opacity="watermarkOpacity = $event"
        @update:watermark-text-font-size="watermarkTextFontSize = $event"
        @update:watermark-text-color="watermarkTextColor = $event"
        @update:watermark-removal-enabled="watermarkRemovalEnabled = $event"
        @update:watermark-removal-region-count="watermarkRemovalRegionCount = $event"
        @update:subtitle-enabled="subtitleEnabled = $event"
        @update:subtitle-position="subtitlePosition = $event"
        @update:subtitle-size="subtitleSize = $event"
        @update:watermark-asset-enabled="watermarkEnabled = $event"
        @update:watermark-asset-type="watermarkAssetType = $event"
        @update:watermark-opacity-asset="watermarkOpacity = $event"
        @update:watermark-image-size-ratio="watermarkImageSizeRatio = $event"
        @update:watermark-trajectory="watermarkTrajectory = $event"
      />

      <RightToolPanel
        v-if="workspaceMode === 'ai'"
        :workspace-mode="productionWorkspaceMode"
        :is-advanced-mode="isAdvancedMode"
        :environment="environment"
        :status-text="statusText"
        :bgm-audio-file-path="bgmAudioFilePath"
        :is-processing="isAnyProcessing"
        v-model:original-volume="originalVolume"
        v-model:bgm-enabled="bgmEnabled"
        v-model:bgm-volume="bgmVolume"
        v-model:tts-enabled="ttsVideoEnabled"
        v-model:tts-speaker="ttsSpeaker"
        v-model:speech-volume="ttsOriginalAudioVolume"
        v-model:speech-speed="playbackSpeed"
        v-model:subtitle-enabled="subtitleEnabled"
        v-model:subtitle-position="subtitlePosition"
        v-model:subtitle-size="subtitleSize"
        v-model:watermark-removal-enabled="watermarkRemovalEnabled"
        v-model:watermark-removal-region-count="watermarkRemovalRegionCount"
        v-model:watermark-enabled="watermarkEnabled"
        v-model:watermark-asset-type="watermarkAssetType"
        v-model:watermark-image-file-path="watermarkImageFilePath"
        v-model:watermark-opacity="watermarkOpacity"
        v-model:watermark-image-size-ratio="watermarkImageSizeRatio"
        v-model:watermark-trajectory="watermarkTrajectory"
        v-model:hsl-enabled="hslEnabled"
        v-model:hue="hue"
        v-model:brightness="brightness"
        v-model:saturation="saturation"
        v-model:zoom-enabled="zoomEnabled"
        v-model:zoom-mode="zoomMode"
        v-model:zoom-min-scale="zoomMinScale"
        v-model:zoom-max-scale="zoomMaxScale"
        v-model:zoom-min-duration-seconds="zoomMinDurationSeconds"
        v-model:zoom-max-duration-seconds="zoomMaxDurationSeconds"
        @open-tool="activeTool = $event"
        @open-drawer="activeDrawer = $event"
        @toggle-advanced-mode="isAdvancedMode = $event"
        @select-bgm-audio-file="selectBgmAudioFile"
        @select-watermark-asset="selectWatermarkImageFile"
      />
    </section>

    <TaskControlBar
      v-if="isWorkspaceVisible"
      :is-advanced-mode="isAdvancedMode"
      :total-videos="importedVideos.length"
      :completed-count="currentAiGenerationSuccessCount + batchMixResults.length + exportResultItems.length"
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
      :zoom-enabled="zoomEnabled"
      :zoom-mode="zoomMode"
      :zoom-min-scale="zoomMinScale"
      :zoom-max-scale="zoomMaxScale"
      :zoom-min-duration-seconds="zoomMinDurationSeconds"
      :zoom-max-duration-seconds="zoomMaxDurationSeconds"
      :frame-material-file-path="frameMaterialFilePath"
      :fusion-material-file-path="fusionMaterialFilePath"
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
      :selected-cover-path="selectedCoverPath"
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
      @select-frame-material-file="selectFrameMaterialFile"
      @select-fusion-material-file="selectFusionMaterialFile"
      @clear-frame-material-file="frameMaterialFilePath = null"
      @clear-fusion-material-file="fusionMaterialFilePath = null"
      @select-bgm-audio-file="selectBgmAudioFile"
      @select-cover-image-file="selectCoverImageFile"
      @select-cover-image-folder="selectCoverImageFolder"
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
      @update:zoom-enabled="zoomEnabled = $event"
      @update:zoom-mode="zoomMode = $event"
      @update:zoom-min-scale="zoomMinScale = $event"
      @update:zoom-max-scale="zoomMaxScale = $event"
      @update:zoom-min-duration-seconds="zoomMinDurationSeconds = $event"
      @update:zoom-max-duration-seconds="zoomMaxDurationSeconds = $event"
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
      @restore="restoreProjectWithAccess"
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
      :generated-results="[...aiGeneratedResults, ...narratedVideoResults]"
      :generation-output-directory="generationOutputDirectory"
      :format-file-name="formatFileName"
      @close="activeDrawer = null"
      @open-location="openAiGenerationDirectory"
      @export-jianying-draft="exportAiJianyingDraft"
    />
    <TaskQueuePanel
      :active-task="activeTask"
      :task-history="taskHistory"
      :is-task-running="isTaskRunning"
      :is-visible="isTaskQueuePanelVisible"
      @close="isTaskQueuePanelVisible = false; userClosedTaskQueuePanel = true"
      @clear-history="clearTaskHistory"
    />
  </main>
</template>
