<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { computed, onMounted, ref, watch } from "vue";
import type { CSSProperties } from "vue";
import { listVideoFilesInFolder } from "./services/videoImportService";
import {
  concatSelectedSegments,
  pickRandomSegments,
} from "./services/videoMixService";
import type {
  CanvasAspectRatio,
  CanvasBackgroundMode,
} from "./services/videoMixService";
import { openPathInFileManager } from "./services/fileManagerService";
import {
  checkFfmpegEnvironment,
  readVideoMetadata,
} from "./services/videoProbeService";
import { exportCurrentVideo } from "./services/videoRenderService";
import { splitCurrentVideo } from "./services/videoSplitService";
import type { FfmpegEnvironmentResult, ImportedVideo } from "./types/videoProbe";

type TaskLogLevel = "info" | "success" | "error";

interface TaskLogEntry {
  id: number;
  time: string;
  message: string;
  level: TaskLogLevel;
}

interface ExportResultItem {
  id: number;
  type: "基础导出" | "拼接导出" | "批量生成";
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
const isResultPanelExpanded = ref(false);
const isTaskLogExpanded = ref(false);
const isWorkspaceVisible = ref(false);

const moduleCards = [
  {
    title: "AI 智能混剪",
    description: "导入素材后，按切片、抽取、拼接和批量生成形成本地混剪流程。",
    tags: ["视频混剪", "批量生成", "画布适配"],
    available: true,
  },
  {
    title: "视频效果处理",
    description: "面向镜像、变速、画布比例和背景填充的本地视频处理入口。",
    tags: ["镜像", "变速", "模糊背景"],
    available: true,
  },
  {
    title: "视频混剪",
    description: "固定切片、随机抽取、拼接抽中片段和批量导出。",
    tags: ["切片", "随机抽取", "拼接"],
    available: true,
  },
  {
    title: "分类混剪",
    description: "按素材分类和规则生成不同混剪版本。",
    tags: ["分类素材", "规则混剪"],
    available: false,
  },
  {
    title: "文案改写",
    description: "后续用于文案多版本整理和创意表达。",
    tags: ["文案", "多版本"],
    available: false,
  },
  {
    title: "视频内容提炼",
    description: "后续用于从素材中提取片段价值和内容标签。",
    tags: ["内容提炼", "素材标签"],
    available: false,
  },
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
  if (!selectedVideo.value) {
    return null;
  }

  return convertFileSrc(selectedVideo.value.filePath);
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

const taskLogs = computed(() => [
  ...exportLogs.value.map((log) => ({ ...log, group: "基础导出" })),
  ...splitLogs.value.map((log) => ({ ...log, group: "视频切片" })),
  ...mixLogs.value.map((log) => ({ ...log, group: "片段拼接" })),
  ...batchMixLogs.value.map((log) => ({ ...log, group: "批量生成" })),
]);

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

  importedVideos.value = videos;
  selectedVideo.value = videos[0] ?? null;
  resetSplitAndRandomState();
}

function selectVideo(video: ImportedVideo) {
  selectedVideo.value = video;
  resetSplitAndRandomState();
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
  appendMixLog(`拼接中，变速倍数 ${playbackSpeed.value.toFixed(2)}x。`, "info");
  isMixing.value = true;

  try {
    const result = await concatSelectedSegments(
      randomSelectedSegments.value,
      outputDirectory.value,
      applyHorizontalMirror.value,
      playbackSpeed.value,
      canvasAspectRatio.value,
      canvasBackgroundMode.value,
    );
    mixResultPath.value = result.outputPath;
    addExportResult("拼接导出", result.outputPath);
    appendMixLog(buildRemixCanvasLog(result), "info");
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
    `开始批量生成：计划生成 ${batchGenerateCount.value} 条，变速倍数 ${playbackSpeed.value.toFixed(2)}x。`,
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
      );
      batchMixResults.value.push(result.outputPath);
      addExportResult("批量生成", result.outputPath);
      appendBatchMixLog(buildRemixCanvasLog(result), "info");
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

function validatePlaybackSpeed() {
  if (!Number.isFinite(playbackSpeed.value)) {
    return "变速倍数必须是有效数字。";
  }

  if (playbackSpeed.value < 0.5 || playbackSpeed.value > 2) {
    return "变速倍数暂时只支持 0.5 到 2.0。";
  }

  return null;
}

function buildMixOptionSummary() {
  const options = [];

  if (applyHorizontalMirror.value) {
    options.push("已应用水平镜像");
  }

  if (Math.abs(playbackSpeed.value - 1) > 0.001) {
    options.push(`已应用 ${playbackSpeed.value.toFixed(2)}x 变速`);
  }

  return options.length > 0 ? `，${options.join("，")}。` : "。";
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
});
</script>

<template>
  <main
    class="app-shell"
    :class="{
      'app-shell--home': !isWorkspaceVisible,
      'app-shell--log-collapsed': isWorkspaceVisible && !isTaskLogExpanded,
    }"
  >
    <header class="top-bar">
      <div class="product-mark">
        <button v-if="isWorkspaceVisible" class="back-button" type="button" @click="isWorkspaceVisible = false">
          返回首页
        </button>
        <p v-else class="eyebrow">V0.2 基础混剪版</p>
        <h1>本地短视频批量混剪工作台</h1>
      </div>
      <div class="top-status">
        <span class="mode-pill">本地处理 / 批量混剪 / API Key 自带</span>
        <span class="health-pill" :class="{ 'health-pill--ok': environment?.available }">
          {{ environment?.available ? "FFmpeg 就绪" : "FFmpeg 未就绪" }}
        </span>
        <span class="title-icon" aria-hidden="true">人</span>
        <span class="title-icon" aria-hidden="true">≡</span>
      </div>
    </header>

    <section v-if="!isWorkspaceVisible" class="home-screen">
      <div class="home-hero">
        <p class="eyebrow">本地桌面端视频处理</p>
        <h2>本地短视频批量混剪工作台</h2>
        <p>本地处理 / 批量混剪 / API Key 自带</p>
      </div>

      <div class="module-grid">
        <article
          v-for="card in moduleCards"
          :key="card.title"
          class="module-card"
          :class="{ 'module-card--disabled': !card.available }"
        >
          <div class="module-card__header">
            <h3>{{ card.title }}</h3>
            <span>{{ card.available ? "功能可用" : "敬请期待" }}</span>
          </div>
          <p>{{ card.description }}</p>
          <div class="module-card__tags">
            <span v-for="tag in card.tags" :key="tag">{{ tag }}</span>
          </div>
          <button
            class="primary-button primary-button--full"
            type="button"
            :disabled="!card.available"
            @click="isWorkspaceVisible = true"
          >
            {{ card.available ? "进入工作台" : "暂未开放" }}
          </button>
        </article>
      </div>
    </section>

    <section v-else class="workbench">
      <aside class="left-rail" aria-label="素材和片段">
        <section class="panel panel--stretch">
          <div class="panel__header">
            <div>
              <p class="panel__label">素材</p>
              <h2>视频素材列表</h2>
            </div>
            <span class="count-badge">{{ importedVideos.length }}</span>
          </div>

          <div class="button-row">
            <button class="ghost-button" type="button" :disabled="isImporting" @click="importVideos">
              {{ isImporting ? "正在导入..." : "导入视频" }}
            </button>
            <button
              class="ghost-button"
              type="button"
              :disabled="isImporting"
              @click="importVideoFolder"
            >
              导入文件夹
            </button>
          </div>

          <p v-if="importError" class="error-text">{{ importError }}</p>
          <p v-else-if="importedVideos.length === 0" class="empty-text">
            支持 mp4 / mov / avi / mkv。
          </p>

          <div v-else class="asset-list">
            <article
              v-for="video in importedVideos"
              :key="video.id"
              class="asset-card"
              :class="{ 'asset-card--active': selectedVideo?.id === video.id }"
              tabindex="0"
              role="button"
              @click="selectVideo(video)"
              @keydown.enter="selectVideo(video)"
              @keydown.space.prevent="selectVideo(video)"
            >
              <div class="asset-card__title">
                <h3>{{ video.fileName }}</h3>
                <span>{{ video.hasAudio ? "有音频" : "无音频" }}</span>
              </div>
              <p class="asset-card__path">{{ video.filePath }}</p>
              <dl class="asset-card__meta">
                <div>
                  <dt>时长</dt>
                  <dd>{{ formatDuration(video.durationSeconds) }}</dd>
                </div>
                <div>
                  <dt>分辨率</dt>
                  <dd>{{ formatResolution(video) }}</dd>
                </div>
              </dl>
            </article>
          </div>
        </section>

        <section class="panel">
          <div class="panel__header">
            <div>
              <p class="panel__label">片段</p>
              <h2>切片和抽取结果</h2>
            </div>
            <span class="count-badge">{{ splitSegmentPaths.length }}</span>
          </div>

          <p v-if="splitSegmentPaths.length === 0" class="empty-text">
            完成固定切片后，这里会出现片段列表。
          </p>
          <ol v-else class="compact-list">
            <li v-for="segmentPath in splitSegmentPaths" :key="segmentPath">
              <span>{{ formatFileName(segmentPath) }}</span>
            </li>
          </ol>

          <div v-if="randomSelectedSegments.length > 0" class="sub-block">
            <div class="section-title">
              <span>已抽取</span>
              <strong>{{ randomSelectedSegments.length }}</strong>
            </div>
            <ol class="compact-list compact-list--selected">
              <li v-for="segmentPath in randomSelectedSegments" :key="segmentPath">
                <span>{{ formatFileName(segmentPath) }}</span>
              </li>
            </ol>
          </div>
        </section>
      </aside>

      <section class="center-stage" aria-label="预览和结果">
        <section class="panel preview-panel">
          <div class="panel__header">
            <div>
              <p class="panel__label">预览</p>
              <h2>{{ selectedVideo?.fileName ?? "请选择一个素材" }}</h2>
            </div>
          </div>

          <div v-if="previewUrl" class="video-frame">
            <div
              class="video-frame__canvas"
              :class="{
                'video-frame__canvas--fit': canvasAspectRatio !== 'original',
                'video-frame__canvas--blur': shouldShowBlurBackground,
              }"
              :style="previewCanvasStyle"
            >
              <video
                v-if="shouldShowBlurBackground"
                ref="previewBackgroundVideoRef"
                class="video-frame__background"
                :src="previewUrl"
                muted
                playsinline
                preload="metadata"
                tabindex="-1"
                aria-hidden="true"
              ></video>
              <video
                :key="selectedVideo?.id"
                ref="previewVideoRef"
                class="video-frame__foreground"
                :src="previewUrl"
                controls
                preload="metadata"
                @play="syncPreviewBackground"
                @pause="syncPreviewBackground"
                @seeked="syncPreviewBackground"
                @timeupdate="syncPreviewBackground"
                @ratechange="syncPreviewBackground"
              ></video>
            </div>
          </div>
          <div v-else class="video-placeholder">
            导入素材后，点击左侧视频即可预览。
          </div>
        </section>

        <section class="panel info-grid">
          <div class="info-card">
            <p>时长</p>
            <strong>{{ selectedVideo ? formatDuration(selectedVideo.durationSeconds) : "未知" }}</strong>
          </div>
          <div class="info-card">
            <p>分辨率</p>
            <strong>{{ selectedVideo ? formatResolution(selectedVideo) : "未知" }}</strong>
          </div>
          <div class="info-card">
            <p>帧率</p>
            <strong>{{ selectedVideo ? formatFrameRate(selectedVideo.frameRate) : "未知" }}</strong>
          </div>
          <div class="info-card">
            <p>大小</p>
            <strong>{{ selectedVideo ? formatFileSize(selectedVideo.fileSizeBytes) : "未知" }}</strong>
          </div>
        </section>

        <section class="panel result-panel" :class="{ 'result-panel--collapsed': !isResultPanelExpanded }">
          <div class="panel__header">
            <div>
              <p class="panel__label">结果</p>
              <h2>混剪输出</h2>
            </div>
            <button class="panel-toggle" type="button" @click="isResultPanelExpanded = !isResultPanelExpanded">
              {{ isResultPanelExpanded ? "收起" : `展开 ${exportResultItems.length}` }}
            </button>
          </div>

          <div v-if="isResultPanelExpanded" class="collapsible-content">
            <div class="result-grid">
              <p v-if="exportError" class="error-text">{{ exportError }}</p>
              <p v-if="mixError" class="error-text">{{ mixError }}</p>
              <p v-if="batchMixError" class="error-text">{{ batchMixError }}</p>
            </div>

            <div class="sub-block">
              <div class="section-title">
                <span>导出结果列表</span>
                <strong>{{ exportResultItems.length }}</strong>
              </div>
              <ol v-if="exportResultItems.length > 0" class="compact-list result-list">
                <li v-for="item in exportResultItems" :key="item.id">
                  <span>{{ item.type }} · {{ formatFileName(item.path) }}</span>
                  <button class="ghost-button result-action" type="button" @click="openResultLocation(item.path)">
                    打开位置
                  </button>
                  <small>{{ item.time }} · {{ item.path }}</small>
                </li>
              </ol>
              <p v-if="fileManagerError" class="error-text">{{ fileManagerError }}</p>
              <p v-if="exportResultItems.length === 0" class="empty-text">暂无导出结果。</p>
            </div>

            <div v-if="batchMixResults.length > 0" class="sub-block">
              <div class="section-title">
                <span>批量生成结果</span>
                <strong>{{ batchMixResults.length }}</strong>
              </div>
              <ol class="compact-list">
                <li v-for="resultPath in batchMixResults" :key="resultPath">
                  <span>{{ formatFileName(resultPath) }}</span>
                  <small>{{ resultPath }}</small>
                </li>
              </ol>
            </div>
          </div>
        </section>
      </section>

      <aside class="right-rail" aria-label="参数设置">
        <section class="panel">
          <div class="panel__header">
            <div>
              <p class="panel__label">环境</p>
              <h2>本地引擎</h2>
            </div>
          </div>
          <p class="engine-message">{{ statusText }}</p>
          <div class="engine-list">
            <div>
              <span>ffmpeg</span>
              <strong>{{ environment?.ffmpeg.available ? "已检测到" : "未检测到" }}</strong>
            </div>
            <div>
              <span>ffprobe</span>
              <strong>{{ environment?.ffprobe.available ? "已检测到" : "未检测到" }}</strong>
            </div>
          </div>
        </section>

        <section class="panel">
          <div class="panel__header">
            <div>
              <p class="panel__label">切片</p>
              <h2>固定时长切片</h2>
            </div>
          </div>
          <label class="field">
            <span>切片秒数</span>
            <input
              v-model.number="segmentDurationSeconds"
              type="number"
              min="1"
              step="1"
              :disabled="isSplitting"
            />
          </label>
          <button
            class="primary-button primary-button--full"
            type="button"
            :disabled="isSplitting"
            @click="splitSelectedVideo"
          >
            {{ isSplitting ? "正在切片..." : "切片当前视频" }}
          </button>
          <p v-if="splitError" class="error-text">{{ splitError }}</p>
          <p v-else-if="splitOutputDirectory" class="success-text">
            已生成 {{ splitSegmentCount }} 个片段。
          </p>
        </section>

        <section class="panel">
          <div class="panel__header">
            <div>
              <p class="panel__label">混剪</p>
              <h2>随机和批量</h2>
            </div>
          </div>
          <label class="field">
            <span>抽取数量</span>
            <input v-model.number="randomPickCount" type="number" min="1" step="1" />
          </label>
          <button class="ghost-button ghost-button--full" type="button" @click="pickSegmentsRandomly">
            随机抽取
          </button>
          <p v-if="randomPickError" class="error-text">{{ randomPickError }}</p>

          <label class="field">
            <span>视频比例</span>
            <select v-model="canvasAspectRatio">
              <option value="original">原画</option>
              <option value="portrait916">9:16 竖屏</option>
              <option value="square11">1:1 方屏</option>
              <option value="landscape169">16:9 横屏</option>
            </select>
          </label>

          <label class="field">
            <span>背景方式</span>
            <select v-model="canvasBackgroundMode" :disabled="canvasAspectRatio === 'original'">
              <option value="black">黑边</option>
              <option value="blur">模糊背景</option>
            </select>
          </label>

          <label class="option-toggle">
            <input v-model="applyHorizontalMirror" type="checkbox" />
            <span>水平镜像</span>
          </label>

          <label class="field">
            <span>变速倍数</span>
            <input
              v-model.number="playbackSpeed"
              type="number"
              min="0.5"
              max="2"
              step="0.1"
              :disabled="isMixing || isBatchMixing"
            />
          </label>

          <button
            class="primary-button primary-button--full"
            type="button"
            :disabled="isMixing"
            @click="concatRandomSegments"
          >
            {{ isMixing ? "正在拼接..." : "拼接抽中片段" }}
          </button>

          <div class="divider"></div>

          <label class="field">
            <span>批量生成数量</span>
            <input
              v-model.number="batchGenerateCount"
              type="number"
              min="1"
              step="1"
              :disabled="isBatchMixing"
            />
          </label>
          <button
            class="primary-button primary-button--full"
            type="button"
            :disabled="isBatchMixing"
            @click="generateBatchMixes"
          >
            {{ isBatchMixing ? "正在批量生成..." : "批量生成混剪" }}
          </button>
        </section>

        <section class="panel">
          <div class="panel__header">
            <div>
              <p class="panel__label">导出</p>
              <h2>输出设置</h2>
            </div>
          </div>
          <button class="ghost-button ghost-button--full" type="button" @click="selectOutputDirectory">
            选择输出目录
          </button>
          <button
            class="ghost-button ghost-button--full"
            type="button"
            :disabled="!outputDirectory"
            @click="openOutputDirectory"
          >
            打开输出目录
          </button>
          <button
            class="primary-button primary-button--full"
            type="button"
            :disabled="isExporting"
            @click="exportSelectedVideo"
          >
            {{ isExporting ? "正在导出..." : "导出当前视频" }}
          </button>
          <p v-if="outputDirectoryError" class="error-text">{{ outputDirectoryError }}</p>
        </section>
      </aside>
    </section>

    <footer v-if="isWorkspaceVisible" class="bottom-console" aria-label="任务日志和输出">
      <section class="output-strip">
        <p class="panel__label">输出目录</p>
        <p v-if="outputDirectory" class="output-path">{{ outputDirectory }}</p>
        <p v-else class="empty-text">请选择导出结果保存位置。</p>
      </section>

      <section class="task-console">
          <div class="console-header">
            <div>
              <p class="panel__label">任务日志</p>
              <h2>当前处理记录</h2>
            </div>
            <button class="panel-toggle" type="button" @click="isTaskLogExpanded = !isTaskLogExpanded">
              {{ isTaskLogExpanded ? "收起" : `展开 ${taskLogs.length}` }}
            </button>
          </div>
          <div v-if="isTaskLogExpanded" class="collapsible-content">
            <p v-if="taskLogs.length === 0" class="empty-text">
              开始导出、切片、拼接或批量生成后，这里会显示任务过程。
            </p>
            <ol v-else class="log-list">
              <li
                v-for="log in taskLogs"
                :key="`${log.group}-${log.id}`"
                class="log-item"
                :class="`log-item--${log.level}`"
              >
                <span class="log-item__time">{{ log.time }}</span>
                <span class="log-item__group">{{ log.group }}</span>
                <span class="log-item__message">{{ log.message }}</span>
              </li>
            </ol>
          </div>
      </section>
    </footer>
  </main>
</template>
