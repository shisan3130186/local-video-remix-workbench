<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { computed, onMounted, ref, watch } from "vue";
import type { CSSProperties } from "vue";
import BatchResultDrawer from "./components/BatchResultDrawer.vue";
import ExportResultDrawer from "./components/ExportResultDrawer.vue";
import HomePage from "./components/HomePage.vue";
import MaterialPanel from "./components/MaterialPanel.vue";
import PreviewPanel from "./components/PreviewPanel.vue";
import RightToolPanel from "./components/RightToolPanel.vue";
import TaskControlBar from "./components/TaskControlBar.vue";
import TaskLogDrawer from "./components/TaskLogDrawer.vue";
import ToolSettingModal from "./components/ToolSettingModal.vue";
import { analyzeAiRemixSegments, planAiRemix } from "./services/aiRemixService";
import type { AiRemixPlannedShot, AiRemixSegment } from "./services/aiRemixService";
import { listVideoFilesInFolder } from "./services/videoImportService";
import {
  concatSelectedSegments,
  pickCategorizedSegments,
  pickRandomSegments,
} from "./services/videoMixService";
import type {
  BgmSettings,
  CanvasAspectRatio,
  CanvasBackgroundMode,
  MixVideoResult,
  PictureInPictureSettings,
  PipPosition,
  RemixExportSettings,
  RotationMode,
  SegmentCategory,
  SegmentCategoryOption,
  SubtitleSettings,
  VideoEffectSettings,
} from "./services/videoMixService";
import { openPathInFileManager } from "./services/fileManagerService";
import {
  checkFfmpegEnvironment,
  readVideoMetadata,
} from "./services/videoProbeService";
import { exportCurrentVideo } from "./services/videoRenderService";
import { splitCurrentVideo } from "./services/videoSplitService";
import { generateThumbnail } from "./services/videoThumbnailService";
import type { FfmpegEnvironmentResult, ImportedVideo } from "./types/videoProbe";

type TaskLogLevel = "info" | "success" | "error";
type ToolKey =
  | "remix"
  | "canvas"
  | "audio"
  | "bgm"
  | "tts"
  | "subtitleStyle"
  | "watermark"
  | "cover"
  | "entrance"
  | "frame"
  | "effects"
  | "transition"
  | "pip"
  | "adjust"
  | "fusion"
  | "rotate"
  | "mirror"
  | "speed"
  | "zoom"
  | "subtitles"
  | "export";
type DrawerKey = "logs" | "exports" | "batch";

interface TaskLogEntry {
  id: number;
  time: string;
  message: string;
  level: TaskLogLevel;
}

interface ExportResultItem {
  id: number;
  type: "基础导出" | "拼接导出" | "分类混剪" | "批量生成" | "AI 智能混剪";
  path: string;
  time: string;
}

const environment = ref<FfmpegEnvironmentResult | null>(null);
const isChecking = ref(true);
const checkError = ref<string | null>(null);
const importedVideos = ref<ImportedVideo[]>([]);
const selectedVideo = ref<ImportedVideo | null>(null);
const isImporting = ref(false);
const importError = ref<string | null>(null);
const outputDirectory = ref<string | null>(null);
const outputDirectoryError = ref<string | null>(null);
const fileManagerError = ref<string | null>(null);
const isExporting = ref(false);
const exportError = ref<string | null>(null);
const exportResultPath = ref<string | null>(null);
const exportLogs = ref<TaskLogEntry[]>([]);
const segmentDurationSeconds = ref(5);
const isSplitting = ref(false);
const splitError = ref<string | null>(null);
const splitOutputDirectory = ref<string | null>(null);
const splitSegmentCount = ref<number | null>(null);
const splitSegmentPaths = ref<string[]>([]);
const segmentCategories = ref<Record<string, SegmentCategory | "">>({});
const splitLogs = ref<TaskLogEntry[]>([]);
const randomPickCount = ref(1);
const randomPickError = ref<string | null>(null);
const randomSelectedSegments = ref<string[]>([]);
const isMixing = ref(false);
const mixError = ref<string | null>(null);
const mixResultPath = ref<string | null>(null);
const mixLogs = ref<TaskLogEntry[]>([]);
const applyHorizontalMirror = ref(false);
const playbackSpeed = ref(1.0);
const smoothRemixEnabled = ref(false);
const applyVerticalMirror = ref(false);
const rotationMode = ref<RotationMode>("none");
const brightness = ref(0);
const contrast = ref(1);
const saturation = ref(1);
const effectScale = ref(1);
const pipEnabled = ref(false);
const pipOverlayFilePath = ref<string | null>(null);
const pipPosition = ref<PipPosition>("topRight");
const pipSizeRatio = ref(0.3);
const pipOpacity = ref(1);
const pipMargin = ref(24);
const bgmEnabled = ref(false);
const bgmAudioFilePath = ref<string | null>(null);
const originalVolume = ref(1);
const bgmVolume = ref(0.35);
const bgmFadeInSeconds = ref(0.5);
const bgmFadeOutSeconds = ref(0.5);
const videoCoverPaths = ref<Record<string, string>>({});
const segmentThumbnailPaths = ref<Record<string, string>>({});
const aiScript = ref("");
const aiPreparedSegments = ref<AiRemixSegment[]>([]);
const aiPlannedShots = ref<AiRemixPlannedShot[]>([]);
const isPreparingAiSegments = ref(false);
const aiPreparationError = ref<string | null>(null);
const isPlanningAiRemix = ref(false);
const aiPlanningProgressText = ref<string | null>(null);
const isGeneratingAiRemix = ref(false);
const aiPlanError = ref<string | null>(null);
const aiGenerateError = ref<string | null>(null);
const aiRemixLogs = ref<TaskLogEntry[]>([]);
const selectedSegmentPath = ref<string | null>(null);
const selectedCoverPath = ref<string | null>(null);
const coverFrameSeconds = ref(1);
const isGeneratingCover = ref(false);
const coverError = ref<string | null>(null);
const batchGenerateCount = ref(3);
const isBatchMixing = ref(false);
const batchMixError = ref<string | null>(null);
const batchMixResults = ref<string[]>([]);
const batchMixLogs = ref<TaskLogEntry[]>([]);
const exportResultItems = ref<ExportResultItem[]>([]);
const canvasAspectRatio = ref<CanvasAspectRatio>("original");
const canvasBackgroundMode = ref<CanvasBackgroundMode>("black");
const previewVideoRef = ref<HTMLVideoElement | null>(null);
const previewBackgroundVideoRef = ref<HTMLVideoElement | null>(null);
const isWorkspaceVisible = ref(true);
const activeTool = ref<ToolKey | null>(null);
const activeDrawer = ref<DrawerKey | null>(null);
const isWelcomeVisible = ref(true);
const isAdvancedMode = ref(false);

const moduleCards = [
  {
    title: "AI 智能混剪",
    category: "创作中心",
    description: "导入素材后，按切片、抽取、拼接和批量生成形成本地混剪流程。",
    tags: ["视频混剪", "批量生成", "画布适配"],
    available: true,
  },
  {
    title: "视频效果处理",
    category: "效率工具",
    description: "面向镜像、变速、画布比例和背景填充的本地视频处理入口。",
    tags: ["镜像", "变速", "模糊背景"],
    available: true,
  },
  {
    title: "视频混剪",
    category: "创作中心",
    description: "固定切片、随机抽取、拼接抽中片段和批量导出。",
    tags: ["切片", "随机抽取", "拼接"],
    available: true,
  },
  {
    title: "分类混剪",
    category: "创作中心",
    description: "按素材分类和规则生成不同混剪版本。",
    tags: ["分类素材", "规则混剪"],
    available: true,
  },
  {
    title: "文案改写",
    category: "效率工具",
    description: "后续用于文案多版本整理和创意表达。",
    tags: ["文案", "多版本"],
    available: false,
  },
  {
    title: "视频内容提炼",
    category: "自动化",
    description: "后续用于从素材中提取片段价值和内容标签。",
    tags: ["内容提炼", "素材标签"],
    available: false,
  },
];

const segmentCategoryOptions: SegmentCategoryOption[] = [
  { key: "hook", label: "开头钩子" },
  { key: "product", label: "产品展示" },
  { key: "usage", label: "使用过程" },
  { key: "detail", label: "细节特写" },
  { key: "result", label: "效果展示" },
  { key: "ending", label: "结尾引导" },
];

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

const videoCoverUrls = computed(() => mapFilePathsToUrls(videoCoverPaths.value));

const segmentThumbnailUrls = computed(() => mapFilePathsToUrls(segmentThumbnailPaths.value));

const selectedCoverUrl = computed(() => {
  if (!selectedCoverPath.value) {
    return null;
  }

  return convertFileSrc(selectedCoverPath.value);
});

const taskLogs = computed(() => [
  ...exportLogs.value.map((log) => ({ ...log, group: "基础导出" })),
  ...splitLogs.value.map((log) => ({ ...log, group: "视频切片" })),
  ...mixLogs.value.map((log) => ({ ...log, group: "片段拼接" })),
  ...batchMixLogs.value.map((log) => ({ ...log, group: "批量生成" })),
  ...aiRemixLogs.value.map((log) => ({ ...log, group: "AI 智能混剪" })),
]);

const videoEffectSettings = computed(
  (): VideoEffectSettings => ({
    verticalMirror: applyVerticalMirror.value,
    rotation: rotationMode.value,
    brightness: brightness.value,
    contrast: contrast.value,
    saturation: saturation.value,
    scale: effectScale.value,
  }),
);

const pictureInPictureSettings = computed(
  (): PictureInPictureSettings => ({
    enabled: pipEnabled.value,
    overlayFilePath: pipOverlayFilePath.value,
    position: pipPosition.value,
    sizeRatio: pipSizeRatio.value,
    opacity: pipOpacity.value,
    margin: pipMargin.value,
  }),
);

const bgmSettings = computed(
  (): BgmSettings => ({
    enabled: bgmEnabled.value,
    audioFilePath: bgmAudioFilePath.value,
    originalVolume: originalVolume.value,
    bgmVolume: bgmVolume.value,
    fadeInSeconds: bgmFadeInSeconds.value,
    fadeOutSeconds: bgmFadeOutSeconds.value,
  }),
);

const disabledSubtitleSettings = computed(
  (): SubtitleSettings => ({
    enabled: false,
    text: "",
    position: "bottom",
    fontSize: 36,
    textColor: "#ffffff",
    backgroundEnabled: false,
  }),
);

const remixExportSettings = computed(
  (): RemixExportSettings => ({
    applyHorizontalMirror: applyHorizontalMirror.value,
    playbackSpeed: playbackSpeed.value,
    canvasAspectRatio: canvasAspectRatio.value,
    canvasBackgroundMode: canvasBackgroundMode.value,
    smoothRemixEnabled: smoothRemixEnabled.value,
    videoEffectSettings: videoEffectSettings.value,
    pictureInPictureSettings: pictureInPictureSettings.value,
    bgmSettings: bgmSettings.value,
    subtitleSettings: disabledSubtitleSettings.value,
  }),
);

const isAnyProcessing = computed(
  () =>
    isExporting.value ||
    isMixing.value ||
    isBatchMixing.value ||
    isSplitting.value ||
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

  return "生成当前分镜视频";
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

  await generateAiRemixVideo();
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
  importError.value = null;
  isImporting.value = true;

  try {
    const selected = await open({
      multiple: true,
      filters: [
        {
          name: "视频文件",
          extensions: ["mp4", "mov", "avi", "mkv"],
        },
      ],
    });

    if (!selected) {
      return;
    }

    const filePaths = Array.isArray(selected) ? selected : [selected];
    await loadVideosFromPaths(filePaths);
  } catch (error) {
    importError.value =
      error instanceof Error ? error.message : String(error ?? "视频导入失败。");
  } finally {
    isImporting.value = false;
  }
}

async function importVideoFolder() {
  importError.value = null;
  isImporting.value = true;

  try {
    const selected = await open({
      directory: true,
      multiple: false,
    });

    if (!selected || Array.isArray(selected)) {
      return;
    }

    const filePaths = await listVideoFilesInFolder(selected);

    if (filePaths.length === 0) {
      importedVideos.value = [];
      selectedVideo.value = null;
      importError.value = "该文件夹中没有检测到 mp4 / mov / avi / mkv 视频文件。";
      return;
    }

    await loadVideosFromPaths(filePaths);
  } catch (error) {
    importError.value =
      error instanceof Error ? error.message : String(error ?? "文件夹导入失败。");
  } finally {
    isImporting.value = false;
  }
}

async function loadVideosFromPaths(filePaths: string[]) {
  const videos = await Promise.all(
    filePaths.map(async (filePath) => {
      const metadata = await readVideoMetadata(filePath);

      return {
        ...metadata,
        id: `${metadata.filePath}-${metadata.fileSizeBytes}`,
      };
    }),
  );

  videoCoverPaths.value = {};
  importedVideos.value = videos;
  selectedVideo.value = videos[0] ?? null;
  selectedCoverPath.value = null;
  resetSplitAndRandomState();

  if (selectedVideo.value) {
    void ensureVideoCover(selectedVideo.value);
  }
}

function selectVideo(video: ImportedVideo) {
  selectedVideo.value = video;
  selectedSegmentPath.value = null;
  selectedCoverPath.value = videoCoverPaths.value[video.id] ?? null;
  coverError.value = null;
  resetSplitAndRandomState();
  void ensureVideoCover(video);
}

function selectSegment(segmentPath: string) {
  selectedSegmentPath.value = segmentPath;
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
      return;
    }

    outputDirectory.value = selected;
  } catch (error) {
    outputDirectoryError.value =
      error instanceof Error
        ? error.message
        : String(error ?? "输出目录选择失败。");
  }
}

async function exportSelectedVideo() {
  exportError.value = null;
  exportResultPath.value = null;
  exportLogs.value = [];

  if (!selectedVideo.value) {
    exportError.value = "请先选择一个要导出的视频。";
    appendExportLog(`导出失败：${exportError.value}`, "error");
    return;
  }

  if (!outputDirectory.value) {
    exportError.value = "请先选择输出目录。";
    appendExportLog(`导出失败：${exportError.value}`, "error");
    return;
  }

  appendExportLog("开始导出。", "info");
  appendExportLog("导出中。", "info");
  isExporting.value = true;

  try {
    const result = await exportCurrentVideo(
      selectedVideo.value.filePath,
      outputDirectory.value,
      canvasAspectRatio.value,
      canvasBackgroundMode.value,
    );
    exportResultPath.value = result.outputPath;
    addExportResult("基础导出", result.outputPath);
    appendExportLog(`导出成功：${result.outputPath}`, "success");
  } catch (error) {
    exportError.value =
      error instanceof Error ? error.message : String(error ?? "视频导出失败。");
    appendExportLog(`导出失败：${exportError.value}`, "error");
  } finally {
    isExporting.value = false;
  }
}

async function splitSelectedVideo() {
  splitError.value = null;
  splitOutputDirectory.value = null;
  splitSegmentCount.value = null;
  splitSegmentPaths.value = [];
  splitLogs.value = [];
  resetRandomPickState();
  resetAiRemixState(false);

  if (importedVideos.value.length === 0) {
    splitError.value = "请先导入至少一个要切片的视频。";
    appendSplitLog(`切片失败：${splitError.value}`, "error");
    return;
  }

  if (!outputDirectory.value) {
    splitError.value = "请先选择输出目录。";
    appendSplitLog(`切片失败：${splitError.value}`, "error");
    return;
  }

  if (!Number.isFinite(segmentDurationSeconds.value) || segmentDurationSeconds.value <= 0) {
    splitError.value = "切片秒数必须大于 0。";
    appendSplitLog(`切片失败：${splitError.value}`, "error");
    return;
  }

  appendSplitLog(`开始切片全部素材，共 ${importedVideos.value.length} 个视频。`, "info");
  isSplitting.value = true;

  try {
    const allSegmentPaths: string[] = [];
    const failedVideos: string[] = [];

    for (const [index, video] of importedVideos.value.entries()) {
      appendSplitLog(
        `正在切片 ${index + 1}/${importedVideos.value.length}：${video.fileName}`,
        "info",
      );

      try {
        const result = await splitCurrentVideo(
          video.filePath,
          outputDirectory.value,
          segmentDurationSeconds.value,
        );
        allSegmentPaths.push(...result.segmentPaths);
        appendSplitLog(
          `${video.fileName} 切片完成：${result.segmentCount} 个片段。`,
          "success",
        );
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error ?? "视频切片失败");
        failedVideos.push(video.fileName);
        appendSplitLog(`${video.fileName} 切片失败：${message}`, "error");
      }
    }

    if (allSegmentPaths.length < 2) {
      throw new Error("成功生成的片段不足 2 个，无法继续 AI 混剪。请检查任务日志。 ");
    }

    splitOutputDirectory.value = outputDirectory.value;
    splitSegmentCount.value = allSegmentPaths.length;
    splitSegmentPaths.value = allSegmentPaths;
    segmentCategories.value = buildEmptySegmentCategoryMap(allSegmentPaths);
    await prepareSegmentAssets(allSegmentPaths);
    appendSplitLog(
      `全部素材切片完成：${importedVideos.value.length - failedVideos.length} 个成功，共 ${allSegmentPaths.length} 个片段。`,
      failedVideos.length > 0 ? "error" : "success",
    );

    if (failedVideos.length > 0) {
      splitError.value = `${failedVideos.length} 个视频切片失败，已保留其他视频生成的片段。`;
    }
  } catch (error) {
    splitError.value =
      error instanceof Error ? error.message : String(error ?? "视频切片失败。");
    appendSplitLog(`切片失败：${splitError.value}`, "error");
  } finally {
    isSplitting.value = false;
  }
}

async function concatCategorizedSegments() {
  mixError.value = null;
  mixResultPath.value = null;
  mixLogs.value = [];

  const pipValidationError = validatePictureInPictureSettings();

  if (pipValidationError) {
    mixError.value = pipValidationError;
    appendMixLog(`分类混剪失败：${mixError.value}`, "error");
    return;
  }

  const bgmValidationError = validateBgmSettings();

  if (bgmValidationError) {
    mixError.value = bgmValidationError;
    appendMixLog(`分类混剪失败：${mixError.value}`, "error");
    return;
  }

  if (!outputDirectory.value) {
    mixError.value = "请先选择输出目录。";
    appendMixLog(`分类混剪失败：${mixError.value}`, "error");
    return;
  }

  const speedValidationError = validatePlaybackSpeed();

  if (speedValidationError) {
    mixError.value = speedValidationError;
    appendMixLog(`分类混剪失败：${mixError.value}`, "error");
    return;
  }

  let categorizedPick;

  try {
    categorizedPick = pickCategorizedSegments(
      splitSegmentPaths.value,
      segmentCategories.value,
      segmentCategoryOptions,
    );
  } catch (error) {
    mixError.value =
      error instanceof Error ? error.message : String(error ?? "分类混剪失败。");
    appendMixLog(`分类混剪失败：${mixError.value}`, "error");
    return;
  }

  randomSelectedSegments.value = categorizedPick.pickedSegments.map(
    (pickedSegment) => pickedSegment.path,
  );

  appendMixLog("开始分类混剪。", "info");
  appendMixLog("分类顺序：开头钩子 -> 产品展示 -> 使用过程 -> 细节特写 -> 效果展示 -> 结尾引导。", "info");

  for (const pickedSegment of categorizedPick.pickedSegments) {
    appendMixLog(
      `${pickedSegment.label}：抽中 ${formatFileName(pickedSegment.path)}。`,
      "info",
    );
  }

  if (categorizedPick.skippedCategories.length > 0) {
    appendMixLog(
      `已自动跳过空分类：${categorizedPick.skippedCategories.join("、")}。`,
      "info",
    );
  }

  appendMixLog(
    `分类混剪中，变速倍数 ${playbackSpeed.value.toFixed(2)}x，平滑混剪${
      smoothRemixEnabled.value ? "已开启" : "未开启"
    }，BGM${bgmEnabled.value ? "已开启" : "未开启"}。`,
    "info",
  );
  isMixing.value = true;

  try {
    const result = await concatSelectedSegments(
      randomSelectedSegments.value,
      outputDirectory.value,
      remixExportSettings.value,
    );
    mixResultPath.value = result.outputPath;
    addExportResult("分类混剪", result.outputPath);
    appendMixLog(buildRemixCanvasLog(result), "info");
    appendMixLog(buildSmoothRemixLog(result), "info");
    appendMixLog(
      `分类混剪成功：已使用 ${result.inputCount} 个片段生成 ${result.outputPath}${
        buildMixOptionSummary()
      }`,
      "success",
    );
  } catch (error) {
    mixError.value =
      error instanceof Error ? error.message : String(error ?? "分类混剪失败。");
    appendMixLog(`分类混剪失败：${mixError.value}`, "error");
  } finally {
    isMixing.value = false;
  }
}

function pickSegmentsRandomly() {
  randomPickError.value = null;
  randomSelectedSegments.value = [];
  resetMixState();

  try {
    randomSelectedSegments.value = pickRandomSegments(
      splitSegmentPaths.value,
      randomPickCount.value,
    );
  } catch (error) {
    randomPickError.value =
      error instanceof Error ? error.message : String(error ?? "随机抽取失败。");
  }
}

async function concatRandomSegments() {
  mixError.value = null;
  mixResultPath.value = null;
  mixLogs.value = [];

  const pipValidationError = validatePictureInPictureSettings();

  if (pipValidationError) {
    mixError.value = pipValidationError;
    appendMixLog(`拼接失败：${mixError.value}`, "error");
    return;
  }

  const bgmValidationError = validateBgmSettings();

  if (bgmValidationError) {
    mixError.value = bgmValidationError;
    appendMixLog(`拼接失败：${mixError.value}`, "error");
    return;
  }

  if (!outputDirectory.value) {
    mixError.value = "请先选择输出目录。";
    appendMixLog(`拼接失败：${mixError.value}`, "error");
    return;
  }

  if (randomSelectedSegments.value.length < 2) {
    mixError.value = "至少需要随机抽取 2 个片段才能拼接。";
    appendMixLog(`拼接失败：${mixError.value}`, "error");
    return;
  }

  const speedValidationError = validatePlaybackSpeed();

  if (speedValidationError) {
    mixError.value = speedValidationError;
    appendMixLog(`拼接失败：${mixError.value}`, "error");
    return;
  }

  appendMixLog("开始拼接。", "info");
  appendMixLog(
    `拼接中，变速倍数 ${playbackSpeed.value.toFixed(2)}x，平滑混剪${
      smoothRemixEnabled.value ? "已开启" : "未开启"
    }，BGM${bgmEnabled.value ? "已开启" : "未开启"}。`,
    "info",
  );
  isMixing.value = true;

  try {
    const result = await concatSelectedSegments(
      randomSelectedSegments.value,
      outputDirectory.value,
      remixExportSettings.value,
    );
    mixResultPath.value = result.outputPath;
    addExportResult("拼接导出", result.outputPath);
    appendMixLog(buildRemixCanvasLog(result), "info");
    appendMixLog(buildSmoothRemixLog(result), "info");
    appendMixLog(
      `拼接成功：已使用 ${result.inputCount} 个片段生成 ${result.outputPath}${
        buildMixOptionSummary()
      }`,
      "success",
    );
  } catch (error) {
    mixError.value =
      error instanceof Error ? error.message : String(error ?? "片段拼接失败。");
    appendMixLog(`拼接失败：${mixError.value}`, "error");
  } finally {
    isMixing.value = false;
  }
}

async function generateBatchMixes() {
  batchMixError.value = null;
  batchMixResults.value = [];
  batchMixLogs.value = [];

  const pipValidationError = validatePictureInPictureSettings();

  if (pipValidationError) {
    batchMixError.value = pipValidationError;
    appendBatchMixLog(`批量生成失败：${batchMixError.value}`, "error");
    return;
  }

  const bgmValidationError = validateBgmSettings();

  if (bgmValidationError) {
    batchMixError.value = bgmValidationError;
    appendBatchMixLog(`批量生成失败：${batchMixError.value}`, "error");
    return;
  }

  if (!outputDirectory.value) {
    batchMixError.value = "请先选择输出目录。";
    appendBatchMixLog(`批量生成失败：${batchMixError.value}`, "error");
    return;
  }

  if (!Number.isInteger(batchGenerateCount.value) || batchGenerateCount.value <= 0) {
    batchMixError.value = "批量生成数量必须大于 0。";
    appendBatchMixLog(`批量生成失败：${batchMixError.value}`, "error");
    return;
  }

  if (!Number.isInteger(randomPickCount.value) || randomPickCount.value < 2) {
    batchMixError.value = "每条混剪至少需要抽取 2 个片段。";
    appendBatchMixLog(`批量生成失败：${batchMixError.value}`, "error");
    return;
  }

  const speedValidationError = validatePlaybackSpeed();

  if (speedValidationError) {
    batchMixError.value = speedValidationError;
    appendBatchMixLog(`批量生成失败：${batchMixError.value}`, "error");
    return;
  }

  appendBatchMixLog(
    `开始批量生成：计划生成 ${batchGenerateCount.value} 条，变速倍数 ${playbackSpeed.value.toFixed(2)}x，平滑混剪${
      smoothRemixEnabled.value ? "已开启" : "未开启"
    }，BGM${bgmEnabled.value ? "已开启" : "未开启"}。`,
    "info",
  );
  isBatchMixing.value = true;

  try {
    for (let index = 0; index < batchGenerateCount.value; index += 1) {
      const pickedSegments = pickRandomSegments(splitSegmentPaths.value, randomPickCount.value);
      appendBatchMixLog(
        `正在生成第 ${index + 1} 条，使用 ${pickedSegments.length} 个随机片段。`,
        "info",
      );

      const result = await concatSelectedSegments(
        pickedSegments,
        outputDirectory.value,
        remixExportSettings.value,
      );
      batchMixResults.value.push(result.outputPath);
      addExportResult("批量生成", result.outputPath);
      appendBatchMixLog(buildRemixCanvasLog(result), "info");
      appendBatchMixLog(buildSmoothRemixLog(result), "info");
      appendBatchMixLog(
        `第 ${index + 1} 条生成成功：${result.outputPath}${
          buildMixOptionSummary()
        }`,
        "success",
      );
    }

    appendBatchMixLog(`批量生成完成：共生成 ${batchMixResults.value.length} 条。`, "success");
  } catch (error) {
    batchMixError.value =
      error instanceof Error ? error.message : String(error ?? "批量生成失败。");
    appendBatchMixLog(`批量生成失败：${batchMixError.value}`, "error");
  } finally {
    isBatchMixing.value = false;
  }
}

function resetSplitAndRandomState() {
  splitError.value = null;
  splitOutputDirectory.value = null;
  splitSegmentCount.value = null;
  splitSegmentPaths.value = [];
  segmentCategories.value = {};
  segmentThumbnailPaths.value = {};
  selectedSegmentPath.value = null;
  splitLogs.value = [];
  resetAiRemixState(true);
  resetRandomPickState();
}

function resetAiRemixState(clearScript: boolean) {
  if (clearScript) {
    aiScript.value = "";
  }

  aiPreparedSegments.value = [];
  aiPlannedShots.value = [];
  isPreparingAiSegments.value = false;
  aiPreparationError.value = null;
  isPlanningAiRemix.value = false;
  isGeneratingAiRemix.value = false;
  aiPlanError.value = null;
  aiGenerateError.value = null;
  aiRemixLogs.value = [];
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
  mixLogs.value = [];
  resetBatchMixState();
}

function resetBatchMixState() {
  isBatchMixing.value = false;
  batchMixError.value = null;
  batchMixResults.value = [];
  batchMixLogs.value = [];
}

function appendSplitLog(message: string, level: TaskLogLevel) {
  appendTaskLog(splitLogs.value, message, level);
}

function appendMixLog(message: string, level: TaskLogLevel) {
  appendTaskLog(mixLogs.value, message, level);
}

function appendBatchMixLog(message: string, level: TaskLogLevel) {
  appendTaskLog(batchMixLogs.value, message, level);
}

function appendAiRemixLog(message: string, level: TaskLogLevel) {
  appendTaskLog(aiRemixLogs.value, message, level);
}

function appendExportLog(message: string, level: TaskLogLevel) {
  appendTaskLog(exportLogs.value, message, level);
}

function updateSegmentCategory(segmentPath: string, category: SegmentCategory | "") {
  segmentCategories.value = {
    ...segmentCategories.value,
    [segmentPath]: category,
  };
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

function buildMixOptionSummary() {
  const options = [];

  if (applyHorizontalMirror.value) {
    options.push("已应用水平镜像");
  }

  if (applyVerticalMirror.value) {
    options.push("已应用垂直镜像");
  }

  if (smoothRemixEnabled.value) {
    options.push("已应用平滑混剪");
  }

  if (rotationMode.value !== "none") {
    options.push("已应用旋转");
  }

  if (
    Math.abs(brightness.value) > 0.001 ||
    Math.abs(contrast.value - 1) > 0.001 ||
    Math.abs(saturation.value - 1) > 0.001
  ) {
    options.push("已应用画面调整");
  }

  if (Math.abs(effectScale.value - 1) > 0.001) {
    options.push(`已应用 ${effectScale.value.toFixed(2)}x 轻微缩放`);
  }

  if (pipEnabled.value) {
    options.push("已应用画中画");
  }

  if (bgmEnabled.value) {
    const fadeSummary =
      bgmFadeInSeconds.value > 0 || bgmFadeOutSeconds.value > 0
        ? `，淡入 ${bgmFadeInSeconds.value.toFixed(1)} 秒，淡出 ${bgmFadeOutSeconds.value.toFixed(1)} 秒`
        : "";
    options.push(`已添加 BGM${fadeSummary}`);
  }

  if (Math.abs(playbackSpeed.value - 1) > 0.001) {
    options.push(`已应用 ${playbackSpeed.value.toFixed(2)}x 变速`);
  }

  return options.length > 0 ? `，${options.join("，")}。` : "。";
}

function buildSmoothRemixLog(result: MixVideoResult) {
  if (!result.smoothRemixEnabled) {
    return "平滑混剪：未开启。";
  }

  return `平滑混剪：已开启，过滤过短片段 ${result.skippedShortSegmentCount} 个。`;
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

async function ensureVideoCover(video: ImportedVideo) {
  if (videoCoverPaths.value[video.id]) {
    selectedCoverPath.value = videoCoverPaths.value[video.id];
    return;
  }

  try {
    const result = await generateThumbnail(
      video.filePath,
      outputDirectory.value,
      getDefaultCoverTime(video),
      "cover",
    );
    videoCoverPaths.value = {
      ...videoCoverPaths.value,
      [video.id]: result.thumbnailPath,
    };

    if (selectedVideo.value?.id === video.id) {
      selectedCoverPath.value = result.thumbnailPath;
    }
  } catch (error) {
    if (selectedVideo.value?.id === video.id) {
      coverError.value =
        error instanceof Error ? error.message : String(error ?? "生成封面帧失败。");
    }
  }
}

async function generateCoverFrame() {
  coverError.value = null;

  if (!selectedVideo.value) {
    coverError.value = "请先选择一个视频素材。";
    return;
  }

  if (!Number.isFinite(coverFrameSeconds.value) || coverFrameSeconds.value < 0) {
    coverError.value = "抽帧时间必须大于或等于 0。";
    return;
  }

  isGeneratingCover.value = true;

  try {
    const result = await generateThumbnail(
      selectedVideo.value.filePath,
      outputDirectory.value,
      coverFrameSeconds.value,
      "selected_cover",
    );
    videoCoverPaths.value = {
      ...videoCoverPaths.value,
      [selectedVideo.value.id]: result.thumbnailPath,
    };
    selectedCoverPath.value = result.thumbnailPath;
  } catch (error) {
    coverError.value =
      error instanceof Error ? error.message : String(error ?? "生成封面帧失败。");
  } finally {
    isGeneratingCover.value = false;
  }
}

async function prepareSegmentAssets(segmentPaths: string[]) {
  isPreparingAiSegments.value = true;
  aiPreparationError.value = null;
  aiPreparedSegments.value = [];
  aiPlannedShots.value = [];
  const thumbnailEntries: Record<string, string> = {};
  const preparedSegments: AiRemixSegment[] = [];
  const preparationErrors: string[] = [];

  for (const [index, segmentPath] of segmentPaths.entries()) {
    try {
      const [thumbnailResult, metadata] = await Promise.all([
        generateThumbnail(
          segmentPath,
          outputDirectory.value,
          0.1,
          `segment_${index + 1}`,
        ),
        readVideoMetadata(segmentPath),
      ]);
      const durationSeconds = metadata.durationSeconds;

      if (durationSeconds === null || !Number.isFinite(durationSeconds) || durationSeconds <= 0) {
        throw new Error("无法读取有效片段时长。");
      }

      thumbnailEntries[segmentPath] = thumbnailResult.thumbnailPath;
      preparedSegments.push({
        segmentId: `segment-${String(index + 1).padStart(3, "0")}`,
        path: segmentPath,
        durationSeconds,
        thumbnailPath: thumbnailResult.thumbnailPath,
        thumbnailUrl: convertFileSrc(thumbnailResult.thumbnailPath),
        description: null,
      });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error ?? "未知错误");
      preparationErrors.push(`${formatFileName(segmentPath)}：${errorMessage}`);
      appendSplitLog(
        `片段 AI 信息准备失败：${formatFileName(segmentPath)}，${errorMessage}`,
        "error",
      );
    }
  }

  segmentThumbnailPaths.value = {
    ...segmentThumbnailPaths.value,
    ...thumbnailEntries,
  };

  if (Object.keys(thumbnailEntries).length > 0) {
    appendSplitLog(`片段预览图生成完成：${Object.keys(thumbnailEntries).length} 张。`, "success");
  }

  if (preparationErrors.length === 0 && preparedSegments.length === segmentPaths.length) {
    aiPreparedSegments.value = preparedSegments;
    appendSplitLog(`AI 片段信息准备完成：${preparedSegments.length} 个片段。`, "success");
  } else {
    aiPreparationError.value = `有 ${preparationErrors.length} 个片段缺少预览图或时长，请重新切片后再试。`;
  }

  isPreparingAiSegments.value = false;
}

async function ensureAiSegmentDescriptions() {
  const pendingSegments = aiPreparedSegments.value.filter(
    (segment) => !segment.description?.trim(),
  );

  if (pendingSegments.length === 0) {
    appendAiRemixLog("复用当前切片已缓存的画面理解结果。", "info");
    return;
  }

  const batches: AiRemixSegment[][] = [];

  for (let index = 0; index < pendingSegments.length; index += 1) {
    batches.push(pendingSegments.slice(index, index + 1));
  }

  let completedCount = aiPreparedSegments.value.length - pendingSegments.length;
  aiPlanningProgressText.value = `正在理解片段画面 ${completedCount}/${aiPreparedSegments.value.length}`;
  appendAiRemixLog(
    `开始分批理解 ${pendingSegments.length} 个片段画面：每次只发送 1 张预览图，最多同时处理 2 张。`,
    "info",
  );

  for (let waveIndex = 0; waveIndex < batches.length; waveIndex += 2) {
    const wave = batches.slice(waveIndex, waveIndex + 2);
    const waveSegmentIds = wave.flatMap((batch) =>
      batch.map((segment) => segment.segmentId),
    );
    appendAiRemixLog(
      `正在理解 ${waveSegmentIds.join("、")}（第 ${waveIndex + 1}-${waveIndex + wave.length}/${batches.length} 批）。`,
      "info",
    );
    const results = await Promise.allSettled(
      wave.map((batch) =>
        analyzeAiRemixSegments(
          batch.map((segment) => ({
            segmentId: segment.segmentId,
            durationSeconds: segment.durationSeconds,
            thumbnailPath: segment.thumbnailPath,
          })),
        ),
      ),
    );
    const descriptions = new Map<string, string>();
    const errors: string[] = [];

    results.forEach((result, resultIndex) => {
      const batch = wave[resultIndex];

      if (result.status === "fulfilled") {
        result.value.segments.forEach((analysis) => {
          descriptions.set(analysis.segmentId, analysis.description.trim());
        });
        completedCount += batch.length;
      } else {
        const message =
          result.reason instanceof Error
            ? result.reason.message
            : String(result.reason ?? "片段画面理解失败");
        errors.push(`${batch.map((segment) => segment.segmentId).join("、")}：${message}`);
      }
    });

    if (descriptions.size > 0) {
      aiPreparedSegments.value = aiPreparedSegments.value.map((segment) => ({
        ...segment,
        description: descriptions.get(segment.segmentId) ?? segment.description,
      }));
    }

    aiPlanningProgressText.value = `正在理解片段画面 ${completedCount}/${aiPreparedSegments.value.length}`;
    appendAiRemixLog(
      `片段画面理解进度：${completedCount}/${aiPreparedSegments.value.length}。`,
      errors.length > 0 ? "error" : "success",
    );

    if (errors.length > 0) {
      throw new Error(`部分片段理解失败：${errors[0]}`);
    }
  }
}

async function requestAiRemixPlan() {
  aiPlanError.value = null;

  if (aiPreparedSegments.value.length !== splitSegmentPaths.value.length) {
    aiPlanError.value = "片段预览图或时长尚未准备完整，请重新切片后再试。";
    return;
  }

  if (aiScript.value.trim().length === 0) {
    aiPlanError.value = "请先输入用于规划混剪的文案。";
    return;
  }

  isPlanningAiRemix.value = true;
  aiPlanningProgressText.value = null;
  appendAiRemixLog(
    `开始准备 AI 逐句分镜，共 ${aiPreparedSegments.value.length} 个候选片段。`,
    "info",
  );

  try {
    await ensureAiSegmentDescriptions();
    const segmentsWithDescriptions = aiPreparedSegments.value.filter(
      (segment) => segment.description?.trim(),
    );

    if (segmentsWithDescriptions.length !== aiPreparedSegments.value.length) {
      throw new Error("仍有片段缺少画面描述，请重试。已成功识别的片段会继续保留。 ");
    }

    aiPlanningProgressText.value = "正在根据文案和画面描述生成分镜...";
    appendAiRemixLog("片段画面理解完成，开始生成纯文字分镜规划。", "info");
    const result = await planAiRemix(
      aiScript.value,
      segmentsWithDescriptions.map((segment) => ({
        segmentId: segment.segmentId,
        durationSeconds: segment.durationSeconds,
        description: segment.description as string,
      })),
    );
    const segmentMap = new Map(
      aiPreparedSegments.value.map((segment) => [segment.segmentId, segment]),
    );
    const primarySegmentIds = new Set(result.shots.map((shot) => shot.segmentId));
    const plannedShots = result.shots.map((shot, index) => {
      const segment = segmentMap.get(shot.segmentId);
      const alternativeSegments = shot.alternativeSegmentIds
        .filter((segmentId) => !primarySegmentIds.has(segmentId))
        .map((segmentId) => segmentMap.get(segmentId));

      if (!segment || alternativeSegments.some((alternative) => !alternative)) {
        throw new Error("AI 分镜结果包含无法识别的片段编号。");
      }

      return {
        shotId: `shot-${Date.now()}-${index + 1}`,
        text: shot.text.trim(),
        segment,
        alternativeSegments: alternativeSegments as AiRemixSegment[],
      };
    });

    aiPlannedShots.value = plannedShots;
    appendAiRemixLog(
      `AI 分镜完成：${plannedShots.map((shot) => shot.segment.segmentId).join(" -> ")}。`,
      "success",
    );
  } catch (error) {
    aiPlanError.value =
      error instanceof Error ? error.message : String(error ?? "AI 分镜规划失败。");
    appendAiRemixLog(`AI 分镜规划失败：${aiPlanError.value}`, "error");
  } finally {
    isPlanningAiRemix.value = false;
    aiPlanningProgressText.value = null;
  }
}

function moveAiRemixShot(index: number, direction: -1 | 1) {
  const targetIndex = index + direction;

  if (index < 0 || targetIndex < 0 || targetIndex >= aiPlannedShots.value.length) {
    return;
  }

  const reordered = [...aiPlannedShots.value];
  [reordered[index], reordered[targetIndex]] = [reordered[targetIndex], reordered[index]];
  aiPlannedShots.value = reordered;
}

function removeAiRemixShot(index: number) {
  if (index < 0 || index >= aiPlannedShots.value.length) {
    return;
  }

  aiPlannedShots.value = aiPlannedShots.value.filter(
    (_shot, shotIndex) => shotIndex !== index,
  );
}

function replaceAiRemixShotSegment(index: number, segmentId: string) {
  const shot = aiPlannedShots.value[index];
  const replacement = aiPreparedSegments.value.find((segment) => segment.segmentId === segmentId);

  if (!shot || !replacement || replacement.segmentId === shot.segment.segmentId) {
    return;
  }

  const alternativeMap = new Map(
    [shot.segment, ...shot.alternativeSegments]
      .filter((segment) => segment.segmentId !== replacement.segmentId)
      .map((segment) => [segment.segmentId, segment]),
  );
  const updatedShots = [...aiPlannedShots.value];
  updatedShots[index] = {
    ...shot,
    segment: replacement,
    alternativeSegments: Array.from(alternativeMap.values()).slice(0, 3),
  };
  aiPlannedShots.value = updatedShots;
}

async function generateAiRemixVideo() {
  aiGenerateError.value = null;

  if (aiPlannedShots.value.length < 2) {
    aiGenerateError.value = "至少保留 2 个分镜才能生成 AI 混剪视频。";
    return;
  }

  if (!outputDirectory.value) {
    aiGenerateError.value = "请先选择输出目录。";
    return;
  }

  const pipValidationError = validatePictureInPictureSettings();
  const bgmValidationError = validateBgmSettings();
  const speedValidationError = validatePlaybackSpeed();
  const validationError = pipValidationError ?? bgmValidationError ?? speedValidationError;

  if (validationError) {
    aiGenerateError.value = validationError;
    return;
  }

  isGeneratingAiRemix.value = true;
  isMixing.value = true;
  appendAiRemixLog(
    `开始生成 AI 混剪视频，使用 ${aiPlannedShots.value.length} 个分镜。`,
    "info",
  );

  try {
    const result = await concatSelectedSegments(
      aiPlannedShots.value.map((shot) => shot.segment.path),
      outputDirectory.value,
      remixExportSettings.value,
    );
    mixResultPath.value = result.outputPath;
    addExportResult("AI 智能混剪", result.outputPath);
    appendAiRemixLog(buildRemixCanvasLog(result), "info");
    appendAiRemixLog(buildSmoothRemixLog(result), "info");
    appendAiRemixLog(`AI 智能混剪生成成功：${result.outputPath}`, "success");
  } catch (error) {
    aiGenerateError.value =
      error instanceof Error ? error.message : String(error ?? "AI 混剪视频生成失败。");
    appendAiRemixLog(`AI 混剪生成失败：${aiGenerateError.value}`, "error");
  } finally {
    isGeneratingAiRemix.value = false;
    isMixing.value = false;
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

function buildRemixCanvasLog(result: {
  outputAspectRatio: string;
  outputResolution: string;
  backgroundMode: string;
  appliedToRemixExport: boolean;
}) {
  return `混剪画布：输出比例 ${result.outputAspectRatio}，输出分辨率 ${result.outputResolution}，背景方式 ${result.backgroundMode}，已应用到混剪导出：${
    result.appliedToRemixExport ? "是" : "否"
  }。`;
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

function appendTaskLog(logs: TaskLogEntry[], message: string, level: TaskLogLevel) {
  logs.push({
    id: Date.now() + logs.length,
    time: new Date().toLocaleTimeString("zh-CN", { hour12: false }),
    message,
    level,
  });
}

function addExportResult(type: ExportResultItem["type"], path: string) {
  exportResultItems.value.unshift({
    id: Date.now() + exportResultItems.value.length,
    type,
    path,
    time: new Date().toLocaleTimeString("zh-CN", { hour12: false }),
  });
}

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

function buildEmptySegmentCategoryMap(segmentPaths: string[]) {
  return Object.fromEntries(segmentPaths.map((segmentPath) => [segmentPath, ""])) as Record<
    string,
    SegmentCategory | ""
  >;
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

function getDefaultCoverTime(video: ImportedVideo) {
  if (video.durationSeconds === null) {
    return 0.1;
  }

  return Math.min(1, Math.max(0.1, video.durationSeconds / 10));
}

function mapFilePathsToUrls(paths: Record<string, string>) {
  return Object.fromEntries(
    Object.entries(paths).map(([key, path]) => [key, convertFileSrc(path)]),
  );
}

onMounted(() => {
  void runEnvironmentCheck();
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
        :is-generating-ai-remix="isGeneratingAiRemix"
        :ai-plan-error="aiPlanError"
        :ai-generate-error="aiGenerateError"
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
        @generate-ai-remix="generateAiRemixVideo"
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
        :failed-count="0"
        :output-directory="outputDirectory"
        :is-processing="isAnyProcessing"
        :primary-action-label="primaryTaskActionLabel"
        :primary-action-disabled="primaryTaskActionDisabled"
        @start-processing="runPrimaryTaskAction"
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
    />

    <TaskLogDrawer
      :open="activeDrawer === 'logs'"
      :task-logs="taskLogs"
      @close="activeDrawer = null"
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
