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
  RotationMode,
  SegmentCategory,
  SegmentCategoryOption,
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
  type: "基础导出" | "拼接导出" | "分类混剪" | "批量生成";
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

  if (!selectedVideo.value) {
    splitError.value = "请先选择一个要切片的视频。";
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

  appendSplitLog("开始切片。", "info");
  appendSplitLog("切片中。", "info");
  isSplitting.value = true;

  try {
    const result = await splitCurrentVideo(
      selectedVideo.value.filePath,
      outputDirectory.value,
      segmentDurationSeconds.value,
    );
    splitOutputDirectory.value = result.outputDirectory;
    splitSegmentCount.value = result.segmentCount;
    splitSegmentPaths.value = result.segmentPaths;
    segmentCategories.value = buildEmptySegmentCategoryMap(result.segmentPaths);
    void generateSegmentThumbnails(result.segmentPaths);
    appendSplitLog(
      `切片成功：共生成 ${result.segmentCount} 个片段，保存到 ${result.outputDirectory}`,
      "success",
    );
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
      applyHorizontalMirror.value,
      playbackSpeed.value,
      canvasAspectRatio.value,
      canvasBackgroundMode.value,
      smoothRemixEnabled.value,
      videoEffectSettings.value,
      pictureInPictureSettings.value,
      bgmSettings.value,
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
      applyHorizontalMirror.value,
      playbackSpeed.value,
      canvasAspectRatio.value,
      canvasBackgroundMode.value,
      smoothRemixEnabled.value,
      videoEffectSettings.value,
      pictureInPictureSettings.value,
      bgmSettings.value,
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
        applyHorizontalMirror.value,
        playbackSpeed.value,
        canvasAspectRatio.value,
        canvasBackgroundMode.value,
        smoothRemixEnabled.value,
        videoEffectSettings.value,
        pictureInPictureSettings.value,
        bgmSettings.value,
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
  resetRandomPickState();
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

async function generateSegmentThumbnails(segmentPaths: string[]) {
  const thumbnailEntries: Record<string, string> = {};

  for (const [index, segmentPath] of segmentPaths.entries()) {
    try {
      const result = await generateThumbnail(
        segmentPath,
        outputDirectory.value,
        0.1,
        `segment_${index + 1}`,
      );
      thumbnailEntries[segmentPath] = result.thumbnailPath;
    } catch (error) {
      appendSplitLog(
        `片段预览图生成失败：${formatFileName(segmentPath)}，${
          error instanceof Error ? error.message : String(error ?? "未知错误")
        }`,
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
        :is-advanced-mode="isAdvancedMode"
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
        :is-processing="isExporting || isMixing || isBatchMixing || isSplitting"
        @start-processing="generateBatchMixes"
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
