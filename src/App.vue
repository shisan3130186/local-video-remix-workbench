<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { computed, onMounted, ref } from "vue";
import { listVideoFilesInFolder } from "./services/videoImportService";
import { pickRandomSegments } from "./services/videoMixService";
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

const environment = ref<FfmpegEnvironmentResult | null>(null);
const isChecking = ref(true);
const checkError = ref<string | null>(null);
const importedVideos = ref<ImportedVideo[]>([]);
const selectedVideo = ref<ImportedVideo | null>(null);
const isImporting = ref(false);
const importError = ref<string | null>(null);
const outputDirectory = ref<string | null>(null);
const outputDirectoryError = ref<string | null>(null);
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
    );
    exportResultPath.value = result.outputPath;
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
}

function appendSplitLog(message: string, level: TaskLogLevel) {
  appendTaskLog(splitLogs.value, message, level);
}

function appendExportLog(message: string, level: TaskLogLevel) {
  appendTaskLog(exportLogs.value, message, level);
}

function appendTaskLog(logs: TaskLogEntry[], message: string, level: TaskLogLevel) {
  logs.push({
    id: Date.now() + logs.length,
    time: new Date().toLocaleTimeString("zh-CN", { hour12: false }),
    message,
    level,
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
  <main class="app-shell">
    <section class="workspace">
      <header class="workspace__header">
        <div>
          <p class="eyebrow">V0.2 基础混剪版</p>
          <h1>本地短视频批量混剪工作台</h1>
        </div>
        <button class="ghost-button" type="button" @click="runEnvironmentCheck">
          重新检测
        </button>
      </header>

      <section class="status-panel" :class="{ 'status-panel--ok': environment?.available }">
        <div>
          <p class="status-panel__label">FFmpeg 环境</p>
          <h2>{{ statusText }}</h2>
        </div>
      </section>

      <section class="tool-grid" aria-label="FFmpeg 检测结果">
        <article class="tool-card">
          <p class="tool-card__name">ffmpeg</p>
          <p class="tool-card__state">
            {{ environment?.ffmpeg.available ? "已检测到" : "未检测到" }}
          </p>
          <p class="tool-card__version">
            {{ environment?.ffmpeg.version ?? "未检测到 FFmpeg，请配置路径。" }}
          </p>
        </article>

        <article class="tool-card">
          <p class="tool-card__name">ffprobe</p>
          <p class="tool-card__state">
            {{ environment?.ffprobe.available ? "已检测到" : "未检测到" }}
          </p>
          <p class="tool-card__version">
            {{ environment?.ffprobe.version ?? "未检测到 FFmpeg，请配置路径。" }}
          </p>
        </article>
      </section>

      <section class="asset-panel" aria-label="素材列表">
        <div class="asset-panel__header">
          <div>
            <p class="asset-panel__label">素材列表</p>
            <h2>导入视频并读取信息</h2>
          </div>
          <div class="asset-panel__actions">
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
        </div>

        <p v-if="importError" class="error-text">{{ importError }}</p>
        <p v-else-if="importedVideos.length === 0" class="empty-text">
          支持导入 mp4 / mov / avi / mkv 文件，或选择一个包含视频的文件夹。
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
              <div>
                <dt>帧率</dt>
                <dd>{{ formatFrameRate(video.frameRate) }}</dd>
              </div>
              <div>
                <dt>大小</dt>
                <dd>{{ formatFileSize(video.fileSizeBytes) }}</dd>
              </div>
            </dl>
          </article>
        </div>
      </section>

      <section class="preview-panel" aria-label="视频预览">
        <div class="preview-panel__header">
          <div>
            <p class="preview-panel__label">视频预览</p>
            <h2>{{ selectedVideo?.fileName ?? "请选择一个素材" }}</h2>
          </div>
        </div>

        <div v-if="previewUrl" class="video-frame">
          <video :key="selectedVideo?.id" :src="previewUrl" controls preload="metadata"></video>
        </div>
        <p v-else class="empty-text">导入素材后，点击列表中的视频即可预览。</p>
      </section>

      <section class="output-panel" aria-label="输出设置">
        <div class="output-panel__header">
          <div>
            <p class="output-panel__label">输出设置</p>
            <h2>导出保存位置</h2>
          </div>
          <div class="output-panel__actions">
            <button class="ghost-button" type="button" @click="selectOutputDirectory">
              选择输出目录
            </button>
            <button
              class="primary-button"
              type="button"
              :disabled="isExporting"
              @click="exportSelectedVideo"
            >
              {{ isExporting ? "正在导出..." : "导出当前视频" }}
            </button>
          </div>
        </div>

        <p v-if="outputDirectoryError" class="error-text">{{ outputDirectoryError }}</p>
        <p v-else-if="outputDirectory" class="output-path">{{ outputDirectory }}</p>
        <p v-else class="empty-text">请选择后续导出视频的保存目录。</p>

        <p v-if="exportError" class="error-text">{{ exportError }}</p>
        <p v-else-if="exportResultPath" class="success-text">
          导出完成：{{ exportResultPath }}
        </p>

        <div class="export-log-panel" aria-label="导出日志">
          <div class="export-log-panel__header">
            <p class="output-panel__label">导出日志</p>
            <span>当前单个导出任务</span>
          </div>
          <p v-if="exportLogs.length === 0" class="empty-text">
            点击导出后，这里会显示本次导出的过程和结果。
          </p>
          <ol v-else class="export-log-list">
            <li
              v-for="log in exportLogs"
              :key="log.id"
              class="export-log-item"
              :class="`export-log-item--${log.level}`"
            >
              <span class="export-log-item__time">{{ log.time }}</span>
              <span class="export-log-item__message">{{ log.message }}</span>
            </li>
          </ol>
        </div>
      </section>

      <section class="split-panel" aria-label="视频切片">
        <div class="split-panel__header">
          <div>
            <p class="split-panel__label">视频切片</p>
            <h2>固定时长切片</h2>
          </div>
          <div class="split-panel__actions">
            <label class="duration-field">
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
              class="primary-button"
              type="button"
              :disabled="isSplitting"
              @click="splitSelectedVideo"
            >
              {{ isSplitting ? "正在切片..." : "切片当前视频" }}
            </button>
          </div>
        </div>

        <p class="empty-text">
          选择一个视频和输出目录后，可以按固定秒数生成多个 mp4 片段。
        </p>

        <p v-if="splitError" class="error-text">{{ splitError }}</p>
        <p v-else-if="splitOutputDirectory" class="success-text">
          切片完成：共生成 {{ splitSegmentCount }} 个片段，保存到 {{ splitOutputDirectory }}
        </p>

        <div class="export-log-panel" aria-label="切片日志">
          <div class="export-log-panel__header">
            <p class="split-panel__label">切片日志</p>
            <span>当前单个切片任务</span>
          </div>
          <p v-if="splitLogs.length === 0" class="empty-text">
            点击切片后，这里会显示本次切片的过程和结果。
          </p>
          <ol v-else class="export-log-list">
            <li
              v-for="log in splitLogs"
              :key="log.id"
              class="export-log-item"
              :class="`export-log-item--${log.level}`"
            >
              <span class="export-log-item__time">{{ log.time }}</span>
              <span class="export-log-item__message">{{ log.message }}</span>
            </li>
          </ol>
        </div>
      </section>

      <section class="mix-panel" aria-label="随机片段抽取">
        <div class="mix-panel__header">
          <div>
            <p class="mix-panel__label">基础混剪</p>
            <h2>随机片段抽取</h2>
          </div>
          <div class="mix-panel__actions">
            <label class="duration-field">
              <span>抽取数量</span>
              <input v-model.number="randomPickCount" type="number" min="1" step="1" />
            </label>
            <button class="primary-button" type="button" @click="pickSegmentsRandomly">
              随机抽取
            </button>
          </div>
        </div>

        <p class="empty-text">
          固定时长切片完成后，可以从当前片段列表里随机抽取素材。此步骤不会修改或拼接视频文件。
        </p>

        <p v-if="randomPickError" class="error-text">{{ randomPickError }}</p>
        <p v-else-if="splitSegmentPaths.length > 0" class="output-path">
          当前可抽取片段：{{ splitSegmentPaths.length }} 个
        </p>

        <div v-if="randomSelectedSegments.length > 0" class="random-result">
          <div class="random-result__header">
            <p class="mix-panel__label">抽取结果</p>
            <span>共 {{ randomSelectedSegments.length }} 个片段</span>
          </div>
          <ol class="random-result__list">
            <li v-for="segmentPath in randomSelectedSegments" :key="segmentPath">
              <span>{{ formatFileName(segmentPath) }}</span>
              <small>{{ segmentPath }}</small>
            </li>
          </ol>
        </div>
      </section>
    </section>
  </main>
</template>
