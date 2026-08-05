<script setup lang="ts">
import { computed, type CSSProperties } from "vue";
import { AiRemixPlanner } from "../features/ai-remix";
import type { AiRemixPlannedShot, AiRemixSegment } from "../features/ai-remix";
import type { ImportedVideo } from "../types/videoProbe";
import type { CanvasAspectRatio } from "../services/videoMixService";
import type { DrawerKey, ToolKey } from "../types/workbench";

type PreviewToolKey = Extract<ToolKey, "remix" | "canvas" | "cover">;

const props = defineProps<{
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
  openDrawer: [drawer: DrawerKey];
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

const isPortraitPreview = computed(() => {
  if (props.canvasAspectRatio === "portrait916") {
    return true;
  }

  if (props.canvasAspectRatio !== "original") {
    return false;
  }

  const width = props.selectedVideo?.width;
  const height = props.selectedVideo?.height;
  return width !== null && width !== undefined && height !== null && height !== undefined && height > width;
});
</script>

<template>
  <section class="center-stage replica-ai-stage" aria-label="AI 混剪工作区">
    <section class="panel preview-panel preview-panel--focused replica-video-preview">
      <div class="replica-pane-heading"><strong>ⓘ 使用指引</strong></div>

      <div
        v-if="previewUrl"
        class="video-frame"
        :class="{ 'video-frame--portrait': isPortraitPreview }"
      >
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
        <span aria-hidden="true">▧</span>
        <strong>选择素材后在此预览</strong>
      </div>
    </section>

    <section class="panel ai-workflow-panel replica-script-workspace">
      <div class="replica-script-heading">
        <strong>视频文案</strong>
        <div v-if="selectedVideo" class="replica-script-actions">
          <button type="button" @click="$emit('openDrawer', 'scripts')">文案库</button>
          <button type="button" @click="$emit('openDrawer', 'exports')">结果 {{ batchMixResultCount }}</button>
        </div>
      </div>

      <div v-if="selectedVideo && aiPreparedSegments.length < 2" class="replica-prepare-row">
        <span>
          <strong>{{ isSplitting || isPreparingAiSegments ? "正在准备素材" : "素材尚未切片" }}</strong>
          <small>{{ isSplitting || isPreparingAiSegments ? "正在生成可匹配的画面片段。" : `共 ${importedVideoCount} 个视频，切片后即可按文案匹配画面。` }}</small>
          <small v-if="splitError" class="workflow-inline-error" role="alert">{{ splitError }}</small>
          <small v-if="aiPreparationError" class="workflow-inline-error" role="alert">{{ aiPreparationError }}</small>
        </span>
        <button
          class="primary-button"
          type="button"
          :disabled="isSplitting || isPreparingAiSegments"
          @click="$emit('splitSelectedVideo')"
        >
          {{ isSplitting || isPreparingAiSegments ? "正在准备..." : "生成素材片段" }}
        </button>
      </div>

      <label
        v-if="aiPreparedSegments.length < 2"
        class="ai-remix-script-field ai-remix-script-field--draft replica-script-field"
        for="ai-remix-script-draft"
      >
        <span v-if="selectedVideo">输入文案或从文案库选择内容</span>
        <textarea
          id="ai-remix-script-draft"
          v-model="aiScript"
          rows="5"
          maxlength="4000"
          :placeholder="selectedVideo ? '在这里输入需要生成视频的文案…' : ''"
          aria-describedby="ai-remix-script-draft-status"
        ></textarea>
        <small v-if="selectedVideo" id="ai-remix-script-draft-status">
          {{ aiScript.length }} / 4000 字 · {{ aiScript.trim() ? "文案已保留，准备好素材后即可生成分镜" : "等待输入文案" }}
        </small>
      </label>

      <div v-if="!selectedVideo" class="replica-script-empty">
        <span aria-hidden="true">▱</span>
        <strong>请在左侧选择一个文件夹</strong>
        <small>选中文件夹后即可输入文案或添加音频</small>
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

      <details v-if="isAdvancedMode && selectedVideo" class="secondary-workflows">
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
