<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { checkFfmpegEnvironment } from "./services/videoProbeService";
import type { FfmpegEnvironmentResult } from "./types/videoProbe";

const environment = ref<FfmpegEnvironmentResult | null>(null);
const isChecking = ref(true);
const checkError = ref<string | null>(null);

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
    </section>
  </main>
</template>
