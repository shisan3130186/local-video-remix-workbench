<script setup lang="ts">
import type { CSSProperties } from "vue";
import { AiRemixPlanner } from "../features/ai-remix";
import type { AiRemixPlannedShot, AiRemixSegment } from "../features/ai-remix";
import type { ImportedVideo } from "../types/videoProbe";
import type { CanvasAspectRatio } from "../services/videoMixService";
import type { ToolKey } from "../types/workbench";

type PreviewToolKey = Extract<ToolKey, "remix" | "canvas" | "cover">;

defineProps<{
  isAdvancedMode: boolean;
  importedVideoCount: number;
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
  splitError: string | null;
  randomSelectedCount: number;
  batchMixResultCount: number;
  mixError: string | null;
  batchMixError: string | null;
  aiPreparedSegments: AiRemixSegment[];
  aiPlannedShots: AiRemixPlannedShot[];
  isPreparingAiSegments: boolean;
  aiPreparationError: string | null;
  isPlanningAiRemix: boolean;
  aiPlanningProgressText: string | null;
  isGeneratingAiRemix: boolean;
  ttsVideoEnabled: boolean;
  generationProgressText: string | null;
  generationSummaryText: string | null;
  generationSuccessCount: number;
  generationFailureCount: number;
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
  openTool: [tool: PreviewToolKey];
  planAiRemix: [];
  moveAiShot: [index: number, direction: -1 | 1];
  removeAiShot: [index: number];
  replaceAiShotSegment: [index: number, segmentId: string];
  generateAiRemix: [];
}>();

const previewVideoRef = defineModel<HTMLVideoElement | null>("previewVideoRef");
const previewBackgroundVideoRef = defineModel<HTMLVideoElement | null>("previewBackgroundVideoRef");
const aiScript = defineModel<string>("aiScript", { required: true });
const aiGenerateCount = defineModel<number>("aiGenerateCount", { required: true });
</script>

<template>
  <section class="center-stage" aria-label="AI 混剪工作区">
    <nav class="workflow-guide" aria-label="AI 混剪步骤">
      <span :class="{ 'workflow-guide__step--done': selectedVideo }">
        <b>1</b><small>导入素材</small>
      </span>
      <i aria-hidden="true"></i>
      <span :class="{ 'workflow-guide__step--done': aiPreparedSegments.length >= 2 }">
        <b>2</b><small>生成片段</small>
      </span>
      <i aria-hidden="true"></i>
      <span :class="{ 'workflow-guide__step--done': aiPlannedShots.length >= 2 }">
        <b>3</b><small>文案分镜</small>
      </span>
      <i aria-hidden="true"></i>
      <span :class="{ 'workflow-guide__step--active': aiPlannedShots.length >= 2 }">
        <b>4</b><small>生成视频</small>
      </span>
    </nav>

    <section class="panel preview-panel preview-panel--focused">
      <div class="panel__header preview-panel__header">
        <div>
          <p class="panel__label">当前画面</p>
          <h2>{{ previewTitle }}</h2>
        </div>
        <div class="preview-actions">
          <button class="panel-toggle" type="button" @click="$emit('openTool', 'canvas')">画布</button>
          <button class="panel-toggle" type="button" @click="$emit('openTool', 'cover')">
            {{ selectedCoverUrl ? "封面已设置" : "设置封面" }}
          </button>
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
      <div v-else class="video-placeholder preview-empty-state">
        <strong>从左侧导入并选择一个视频</strong>
        <span>视频会在这里预览，之后即可切片并生成 AI 分镜。</span>
      </div>
    </section>

    <section class="panel ai-workflow-panel">
      <div class="ai-workflow-panel__toolbar">
        <div>
          <p class="panel__label">核心流程</p>
          <h2>文案与画面分镜</h2>
        </div>
        <div class="workflow-stats" aria-label="当前素材统计">
          <span>片段 {{ splitSegmentCount ?? 0 }}</span>
          <span>分镜 {{ aiPlannedShots.length }}</span>
          <button type="button" @click="$emit('openDrawer', 'exports')">结果 {{ batchMixResultCount }}</button>
        </div>
      </div>

      <div v-if="!selectedVideo" class="ai-preparation-callout">
        <span><strong>第一步：选择视频</strong><small>从左侧素材库中选择要处理的视频。</small></span>
      </div>
      <div v-else-if="aiPreparedSegments.length < 2" class="ai-preparation-callout">
        <span>
          <strong>{{ isSplitting || isPreparingAiSegments ? "正在准备全部素材片段" : "第二步：把全部素材切成片段" }}</strong>
          <small>{{ isSplitting || isPreparingAiSegments ? "会依次切片所有已导入视频，并生成预览图和时长。" : `当前已导入 ${importedVideoCount} 个视频，AI 会在它们的全部片段中匹配画面。` }}</small>
          <small v-if="splitError" class="workflow-inline-error" role="alert">{{ splitError }}</small>
          <small v-if="aiPreparationError" class="workflow-inline-error" role="alert">{{ aiPreparationError }}</small>
        </span>
        <button
          class="primary-button"
          type="button"
          :disabled="isSplitting || isPreparingAiSegments"
          @click="$emit('splitSelectedVideo')"
        >
          {{ isSplitting || isPreparingAiSegments ? "正在准备..." : importedVideoCount > 1 ? `切片全部素材（${importedVideoCount} 个）` : "切片当前视频" }}
        </button>
      </div>

      <AiRemixPlanner
        v-if="aiPreparedSegments.length >= 2"
        v-model:script="aiScript"
        v-model:generate-count="aiGenerateCount"
        :prepared-segments="aiPreparedSegments"
        :planned-shots="aiPlannedShots"
        :is-preparing="isPreparingAiSegments"
        :preparation-error="aiPreparationError"
        :is-planning="isPlanningAiRemix"
        :planning-progress-text="aiPlanningProgressText"
        :is-generating="isGeneratingAiRemix"
        :tts-video-enabled="ttsVideoEnabled"
        :generation-progress-text="generationProgressText"
        :generation-summary-text="generationSummaryText"
        :generation-success-count="generationSuccessCount"
        :generation-failure-count="generationFailureCount"
        :plan-error="aiPlanError"
        :generate-error="aiGenerateError"
        @plan="$emit('planAiRemix')"
        @move="(index, direction) => $emit('moveAiShot', index, direction)"
        @remove="(index) => $emit('removeAiShot', index)"
        @replace="(index, segmentId) => $emit('replaceAiShotSegment', index, segmentId)"
        @generate="$emit('generateAiRemix')"
      />

      <details v-if="isAdvancedMode" class="secondary-workflows">
        <summary>其他混剪方式与高级操作</summary>
        <div class="secondary-workflows__actions">
          <button class="ghost-button" type="button" :disabled="isSplitting" @click="$emit('splitSelectedVideo')">重新切片</button>
          <button class="ghost-button" type="button" @click="$emit('pickSegmentsRandomly')">随机抽取</button>
          <button class="ghost-button" type="button" :disabled="isMixing" @click="$emit('concatRandomSegments')">拼接抽中片段</button>
          <button class="ghost-button" type="button" :disabled="isMixing" @click="$emit('concatCategorizedSegments')">分类混剪</button>
          <button class="ghost-button" type="button" :disabled="isBatchMixing" @click="$emit('generateBatchMixes')">随机批量生成</button>
          <button class="ghost-button" type="button" :disabled="isExporting" @click="$emit('exportSelectedVideo')">导出当前视频</button>
          <button class="ghost-button" type="button" @click="$emit('openTool', 'remix')">混剪参数</button>
        </div>
      </details>

      <p v-if="mixError || batchMixError" class="workflow-error" role="alert">{{ mixError || batchMixError }}</p>
    </section>
  </section>
</template>
