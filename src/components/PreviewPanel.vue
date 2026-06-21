<script setup lang="ts">
import { computed, ref } from "vue";
import type { CSSProperties } from "vue";
import type { ImportedVideo } from "../types/videoProbe";
import type { CanvasAspectRatio } from "../services/videoMixService";

type ToolKey = "canvas" | "cover" | "subtitles";

defineProps<{
  selectedVideo: ImportedVideo | null;
  previewUrl: string | null;
  selectedCoverUrl: string | null;
  canvasAspectRatio: CanvasAspectRatio;
  shouldShowBlurBackground: boolean;
  previewCanvasStyle: CSSProperties;
  isSplitting: boolean;
  isMixing: boolean;
  isBatchMixing: boolean;
  isExporting: boolean;
  splitSegmentCount: number | null;
  randomSelectedCount: number;
  batchMixResultCount: number;
  formatDuration: (durationSeconds: number | null) => string;
  formatResolution: (video: ImportedVideo) => string;
  formatFrameRate: (frameRate: number | null) => string;
  formatFileSize: (fileSizeBytes: number) => string;
}>();

defineEmits<{
  splitSelectedVideo: [];
  pickSegmentsRandomly: [];
  concatRandomSegments: [];
  concatCategorizedSegments: [];
  generateBatchMixes: [];
  exportSelectedVideo: [];
  syncPreviewBackground: [];
  openDrawer: [drawer: "logs" | "exports" | "batch"];
  openTool: [tool: ToolKey];
}>();

const previewVideoRef = defineModel<HTMLVideoElement | null>("previewVideoRef");
const previewBackgroundVideoRef = defineModel<HTMLVideoElement | null>("previewBackgroundVideoRef");
const scriptText = ref("");
const scriptTextLength = computed(() => scriptText.value.trim().length);
</script>

<template>
  <section class="center-stage" aria-label="预览和核心操作">
    <section class="panel preview-panel">
      <div class="panel__header">
        <div>
          <p class="panel__label">预览</p>
          <h2>{{ selectedVideo?.fileName ?? "请选择一个素材" }}</h2>
        </div>
        <div class="preview-actions">
          <button class="panel-toggle" type="button" @click="$emit('openTool', 'canvas')">视频裁剪</button>
          <button class="panel-toggle" type="button" @click="$emit('openTool', 'cover')">视频封面</button>
          <button class="panel-toggle" type="button" @click="$emit('openTool', 'subtitles')">添加文本</button>
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
            @play="$emit('syncPreviewBackground')"
            @pause="$emit('syncPreviewBackground')"
            @seeked="$emit('syncPreviewBackground')"
            @timeupdate="$emit('syncPreviewBackground')"
            @ratechange="$emit('syncPreviewBackground')"
          ></video>
        </div>
      </div>
      <div v-else class="video-placeholder">导入素材后，点击左侧视频即可预览。</div>
    </section>

    <section class="panel copy-panel">
      <div class="copy-panel__header">
        <div>
          <p class="panel__label">文案</p>
          <h2>视频文案</h2>
        </div>
        <div class="subtitle-mode">
          <span>视频字幕</span>
          <button class="subtitle-mode__active" type="button">自动识别</button>
          <button type="button">不启用</button>
        </div>
      </div>
      <div class="copy-tabs">
        <button class="copy-tab copy-tab--active" type="button">文案 1</button>
        <button class="copy-tab" type="button">＋</button>
      </div>
      <textarea
        v-model="scriptText"
        class="copy-textarea"
        placeholder="在这里输入本条视频的口播文案、字幕草稿或混剪说明。"
      ></textarea>
      <div class="copy-footer">
        <span>字数：{{ scriptTextLength }}</span>
        <span>文案仅用于 UI 入口，本次不接入真实字幕生成。</span>
      </div>
    </section>

    <section class="panel core-actions-panel">
      <div class="panel__header">
        <div>
          <p class="panel__label">核心操作</p>
          <h2>切片、抽取、混剪、导出</h2>
        </div>
      </div>
      <div class="core-action-grid">
        <button class="primary-button" type="button" :disabled="isSplitting" @click="$emit('splitSelectedVideo')">
          {{ isSplitting ? "正在切片..." : "切片当前视频" }}
        </button>
        <button class="ghost-button" type="button" @click="$emit('pickSegmentsRandomly')">随机抽取</button>
        <button class="primary-button" type="button" :disabled="isMixing" @click="$emit('concatRandomSegments')">
          {{ isMixing ? "正在拼接..." : "拼接抽中片段" }}
        </button>
        <button class="primary-button" type="button" :disabled="isMixing" @click="$emit('concatCategorizedSegments')">
          {{ isMixing ? "正在分类混剪..." : "分类混剪" }}
        </button>
        <button class="primary-button" type="button" :disabled="isBatchMixing" @click="$emit('generateBatchMixes')">
          {{ isBatchMixing ? "正在批量生成..." : "批量生成混剪" }}
        </button>
        <button class="ghost-button" type="button" :disabled="isExporting" @click="$emit('exportSelectedVideo')">
          {{ isExporting ? "正在导出..." : "导出当前视频" }}
        </button>
      </div>
      <div class="status-grid">
        <button type="button" class="status-chip" @click="$emit('openDrawer', 'batch')">
          切片 {{ splitSegmentCount ?? 0 }} / 已抽取 {{ randomSelectedCount }}
        </button>
        <button type="button" class="status-chip" @click="$emit('openDrawer', 'exports')">
          批量结果 {{ batchMixResultCount }}
        </button>
        <button type="button" class="status-chip" @click="$emit('openDrawer', 'logs')">任务日志</button>
      </div>
    </section>
    <section class="panel info-grid">
      <div class="info-card info-card--cover">
        <p>封面帧</p>
        <img v-if="selectedCoverUrl" :src="selectedCoverUrl" alt="" />
        <strong v-else>未生成</strong>
      </div>
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
  </section>
</template>
