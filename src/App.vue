<script setup lang="ts">
import { open } from "@tauri-apps/plugin-dialog";
import { computed, onMounted, ref } from "vue";
import {
  checkFfmpegEnvironment,
  readVideoMetadata,
} from "./services/videoProbeService";
import type { FfmpegEnvironmentResult, ImportedVideo } from "./types/videoProbe";

const environment = ref<FfmpegEnvironmentResult | null>(null);
const isChecking = ref(true);
const checkError = ref<string | null>(null);
const importedVideos = ref<ImportedVideo[]>([]);
const isImporting = ref(false);
const importError = ref<string | null>(null);

const statusText = computed(() => {
  if (isChecking.value) {
    return "正在检测 FFmpeg 环境...";
  }

  if (checkError.value) {
    return checkError.value;
  }

  return environment.value?.message ?? "未检测到 FFmpeg，请配置路径。";
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
  } catch (error) {
    importError.value =
      error instanceof Error ? error.message : String(error ?? "视频导入失败。");
  } finally {
    isImporting.value = false;
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

onMounted(() => {
  void runEnvironmentCheck();
});
</script>

<template>
  <main class="app-shell">
    <section class="workspace">
      <header class="workspace__header">
        <div>
          <p class="eyebrow">V0.1 基础可运行版</p>
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
          <button class="ghost-button" type="button" :disabled="isImporting" @click="importVideos">
            {{ isImporting ? "正在导入..." : "导入视频" }}
          </button>
        </div>

        <p v-if="importError" class="error-text">{{ importError }}</p>
        <p v-else-if="importedVideos.length === 0" class="empty-text">
          支持导入 mp4 / mov / avi / mkv。
        </p>

        <div v-else class="asset-list">
          <article v-for="video in importedVideos" :key="video.id" class="asset-card">
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
    </section>
  </main>
</template>
