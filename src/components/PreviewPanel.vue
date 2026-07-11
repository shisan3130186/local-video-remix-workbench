<script setup lang="ts">
import type { CSSProperties } from "vue";
import AiRemixPlanner from "./AiRemixPlanner.vue";
import type { AiRemixSegment } from "../services/aiRemixService";
import type { ImportedVideo } from "../types/videoProbe";
import type { CanvasAspectRatio } from "../services/videoMixService";

type ToolKey = "remix" | "canvas" | "cover" | "subtitles";

defineProps<{
  isAdvancedMode: boolean;
  selectedVideo: ImportedVideo | null;
  previewTitle: string;
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
  mixError: string | null;
  batchMixError: string | null;
  aiPreparedSegments: AiRemixSegment[];
  aiOrderedSegments: AiRemixSegment[];
  isPreparingAiSegments: boolean;
  aiPreparationError: string | null;
  isPlanningAiRemix: boolean;
  isGeneratingAiRemix: boolean;
  aiPlanError: string | null;
  aiGenerateError: string | null;
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
  planAiRemix: [];
  moveAiSegment: [index: number, direction: -1 | 1];
  removeAiSegment: [index: number];
  generateAiRemix: [];
}>();

const previewVideoRef = defineModel<HTMLVideoElement | null>("previewVideoRef");
const previewBackgroundVideoRef = defineModel<HTMLVideoElement | null>("previewBackgroundVideoRef");
const aiScript = defineModel<string>("aiScript", { required: true });
</script>

<template>
  <section class="center-stage" aria-label="预览和核心操作">
    <section class="panel preview-panel">
      <div class="panel__header">
        <div>
          <p class="panel__label">视频预览</p>
          <h2>{{ previewTitle }}</h2>
        </div>
        <div v-if="isAdvancedMode" class="preview-actions">
          <span class="preview-mode-label">视频比例</span>
          <button class="panel-toggle" type="button" @click="$emit('openTool', 'canvas')">画布比例</button>
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
            :key="previewUrl"
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

    <section class="panel remix-workflow-panel" :class="{ 'remix-workflow-panel--simple': !isAdvancedMode }">
      <div class="remix-workflow-panel__header">
        <div>
          <p class="panel__label">{{ isAdvancedMode ? "混剪流程" : "新手流程" }}</p>
          <h2>{{ isAdvancedMode ? "切片、分类和生成" : "导入素材 -> 预览视频 -> 开始处理 -> 查看结果" }}</h2>
        </div>
        <div class="workflow-stats">
          <button type="button" class="status-chip" @click="$emit('openDrawer', 'batch')">
            切片 {{ splitSegmentCount ?? 0 }}
          </button>
          <button type="button" class="status-chip" @click="$emit('openDrawer', 'batch')">
            已抽取 {{ randomSelectedCount }}
          </button>
          <button type="button" class="status-chip" @click="$emit('openDrawer', 'exports')">
            结果 {{ batchMixResultCount }}
          </button>
        </div>
      </div>

      <div v-if="!isAdvancedMode" class="simple-workflow">
        <div class="simple-workflow__steps" aria-label="新手四步流程">
          <span>1 导入素材</span>
          <span>2 预览视频</span>
          <span>3 开始处理</span>
          <span>4 查看结果</span>
        </div>
        <div class="simple-workflow__actions">
          <button class="primary-button" type="button" :disabled="isBatchMixing" @click="$emit('generateBatchMixes')">
            {{ isBatchMixing ? "正在处理..." : "一键混剪" }}
          </button>
          <button class="ghost-button" type="button" @click="$emit('openTool', 'remix')">高级设置</button>
          <button class="ghost-button" type="button" @click="$emit('openDrawer', 'exports')">查看结果</button>
        </div>
      </div>

      <div v-else class="workflow-actions">
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
          {{ isBatchMixing ? "正在批量生成..." : "批量生成" }}
        </button>
        <button class="ghost-button" type="button" :disabled="isExporting" @click="$emit('exportSelectedVideo')">
          {{ isExporting ? "正在导出..." : "导出当前视频" }}
        </button>
      </div>
      <div v-if="isAdvancedMode && (mixError || batchMixError)" class="workflow-error">
        {{ mixError || batchMixError }}
      </div>

      <AiRemixPlanner
        v-model:script="aiScript"
        :prepared-segments="aiPreparedSegments"
        :ordered-segments="aiOrderedSegments"
        :is-preparing="isPreparingAiSegments"
        :preparation-error="aiPreparationError"
        :is-planning="isPlanningAiRemix"
        :is-generating="isGeneratingAiRemix"
        :plan-error="aiPlanError"
        :generate-error="aiGenerateError"
        @plan="$emit('planAiRemix')"
        @move="(index, direction) => $emit('moveAiSegment', index, direction)"
        @remove="(index) => $emit('removeAiSegment', index)"
        @generate="$emit('generateAiRemix')"
      />

      <div v-if="isAdvancedMode" class="workflow-meta">
        <span>时长：{{ selectedVideo ? formatDuration(selectedVideo.durationSeconds) : "未知" }}</span>
        <span>分辨率：{{ selectedVideo ? formatResolution(selectedVideo) : "未知" }}</span>
        <span>帧率：{{ selectedVideo ? formatFrameRate(selectedVideo.frameRate) : "未知" }}</span>
        <span>大小：{{ selectedVideo ? formatFileSize(selectedVideo.fileSizeBytes) : "未知" }}</span>
        <button type="button" class="panel-toggle" @click="$emit('openTool', 'subtitles')">文案 / 字幕入口</button>
        <button type="button" class="panel-toggle" @click="$emit('openTool', 'cover')">
          {{ selectedCoverUrl ? "查看封面" : "设置封面" }}
        </button>
      </div>
    </section>
  </section>
</template>
