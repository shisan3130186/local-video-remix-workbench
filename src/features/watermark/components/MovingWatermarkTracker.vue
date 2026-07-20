<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import type { CSSProperties } from "vue";
import type { WatermarkTrackingKeyframe } from "../types";

const props = defineProps<{
  previewUrl: string | null;
  keyframes: WatermarkTrackingKeyframe[];
  regionWidthRatio: number;
  regionHeightRatio: number;
  disabled: boolean;
}>();

const emit = defineEmits<{
  "update:keyframes": [value: WatermarkTrackingKeyframe[]];
  "update:regionWidthRatio": [value: number];
  "update:regionHeightRatio": [value: number];
}>();

const videoRef = ref<HTMLVideoElement | null>(null);
const stageRef = ref<HTMLElement | null>(null);
const currentTime = ref(0);
const duration = ref(0);
const isPlaying = ref(false);
const videoAspectRatio = ref(16 / 9);
const draftXRatio = ref(0.36);
const draftYRatio = ref(0.68);
const selectedKeyframeTime = ref<number | null>(null);
const isMovingRegion = ref(false);

const stageStyle = computed(
  (): CSSProperties => ({
    aspectRatio: String(videoAspectRatio.value),
    maxWidth: `${Math.min(640, 420 * videoAspectRatio.value)}px`,
    "--tracking-aspect": String(videoAspectRatio.value),
  } as CSSProperties),
);

const regionStyle = computed(
  (): CSSProperties => ({
    left: `${draftXRatio.value * 100}%`,
    top: `${draftYRatio.value * 100}%`,
    width: `${props.regionWidthRatio * 100}%`,
    height: `${props.regionHeightRatio * 100}%`,
  }),
);

const sortedKeyframes = computed(() =>
  [...props.keyframes].sort((left, right) => left.timeSeconds - right.timeSeconds),
);

const trajectoryPoints = computed(() =>
  sortedKeyframes.value
    .map((keyframe) => {
      const x = (keyframe.xRatio + props.regionWidthRatio / 2) * 100;
      const y = (keyframe.yRatio + props.regionHeightRatio / 2) * 100;
      return `${x},${y}`;
    })
    .join(" "),
);

const currentKeyframeIndex = computed(() =>
  sortedKeyframes.value.findIndex(
    (keyframe) => Math.abs(keyframe.timeSeconds - currentTime.value) < 0.05,
  ),
);

const canSaveKeyframe = computed(
  () =>
    Boolean(props.previewUrl) &&
    !props.disabled &&
    (sortedKeyframes.value.length < 8 || currentKeyframeIndex.value >= 0),
);

watch(
  () => props.previewUrl,
  () => {
    currentTime.value = 0;
    duration.value = 0;
    isPlaying.value = false;
    selectedKeyframeTime.value = null;
  },
);

watch(
  () => [props.regionWidthRatio, props.regionHeightRatio],
  () => clampDraftPosition(),
);

function readVideoMetadata(event: Event) {
  const video = event.target as HTMLVideoElement;
  duration.value = Number.isFinite(video.duration) ? video.duration : 0;
  videoAspectRatio.value = video.videoWidth > 0 && video.videoHeight > 0
    ? video.videoWidth / video.videoHeight
    : 16 / 9;
}

function updateCurrentTime(event: Event) {
  currentTime.value = (event.target as HTMLVideoElement).currentTime;
}

async function togglePlayback() {
  const video = videoRef.value;
  if (!video) return;
  if (video.paused) {
    await video.play();
    isPlaying.value = true;
  } else {
    video.pause();
    isPlaying.value = false;
  }
}

function seekFromInput(event: Event) {
  seekTo(Number((event.target as HTMLInputElement).value));
}

function seekTo(timeSeconds: number) {
  const video = videoRef.value;
  if (!video || !Number.isFinite(timeSeconds)) return;
  const safeTime = clamp(timeSeconds, 0, duration.value || timeSeconds);
  video.currentTime = safeTime;
  currentTime.value = safeTime;
}

function beginMove(event: PointerEvent) {
  if (props.disabled || !stageRef.value) return;
  isMovingRegion.value = true;
  stageRef.value.setPointerCapture(event.pointerId);
  updateDraftPosition(event);
}

function continueMove(event: PointerEvent) {
  if (!isMovingRegion.value) return;
  updateDraftPosition(event);
}

function endMove(event: PointerEvent) {
  isMovingRegion.value = false;
  if (stageRef.value?.hasPointerCapture(event.pointerId)) {
    stageRef.value.releasePointerCapture(event.pointerId);
  }
}

function updateDraftPosition(event: PointerEvent) {
  const stage = stageRef.value;
  if (!stage) return;
  const bounds = stage.getBoundingClientRect();
  const centerX = (event.clientX - bounds.left) / bounds.width;
  const centerY = (event.clientY - bounds.top) / bounds.height;
  draftXRatio.value = clamp(
    centerX - props.regionWidthRatio / 2,
    0,
    1 - props.regionWidthRatio,
  );
  draftYRatio.value = clamp(
    centerY - props.regionHeightRatio / 2,
    0,
    1 - props.regionHeightRatio,
  );
}

function updateDraftX(event: Event) {
  draftXRatio.value = clamp(
    numberValue(event, 0, (1 - props.regionWidthRatio) * 100) / 100,
    0,
    1 - props.regionWidthRatio,
  );
}

function updateDraftY(event: Event) {
  draftYRatio.value = clamp(
    numberValue(event, 0, (1 - props.regionHeightRatio) * 100) / 100,
    0,
    1 - props.regionHeightRatio,
  );
}

function updateRegionWidth(event: Event) {
  const value = numberValue(event, 4, 80) / 100;
  emit("update:regionWidthRatio", value);
  emit(
    "update:keyframes",
    sortedKeyframes.value.map((keyframe) => ({
      ...keyframe,
      xRatio: clamp(keyframe.xRatio, 0, 1 - value),
    })),
  );
  draftXRatio.value = clamp(draftXRatio.value, 0, 1 - value);
}

function updateRegionHeight(event: Event) {
  const value = numberValue(event, 4, 80) / 100;
  emit("update:regionHeightRatio", value);
  emit(
    "update:keyframes",
    sortedKeyframes.value.map((keyframe) => ({
      ...keyframe,
      yRatio: clamp(keyframe.yRatio, 0, 1 - value),
    })),
  );
  draftYRatio.value = clamp(draftYRatio.value, 0, 1 - value);
}

function saveCurrentKeyframe() {
  if (!canSaveKeyframe.value) return;
  const timeSeconds = round(currentTime.value, 3);
  const keyframe: WatermarkTrackingKeyframe = {
    timeSeconds,
    xRatio: round(draftXRatio.value, 5),
    yRatio: round(draftYRatio.value, 5),
  };
  const next = [...sortedKeyframes.value];
  const existingIndex = next.findIndex(
    (item) => Math.abs(item.timeSeconds - timeSeconds) < 0.05,
  );
  if (existingIndex >= 0) {
    next[existingIndex] = keyframe;
  } else {
    next.push(keyframe);
  }
  next.sort((left, right) => left.timeSeconds - right.timeSeconds);
  emit("update:keyframes", next);
  selectedKeyframeTime.value = timeSeconds;
}

async function loadKeyframe(keyframe: WatermarkTrackingKeyframe) {
  draftXRatio.value = keyframe.xRatio;
  draftYRatio.value = keyframe.yRatio;
  selectedKeyframeTime.value = keyframe.timeSeconds;
  await nextTick();
  seekTo(keyframe.timeSeconds);
}

function removeKeyframe(timeSeconds: number) {
  emit(
    "update:keyframes",
    sortedKeyframes.value.filter(
      (keyframe) => Math.abs(keyframe.timeSeconds - timeSeconds) >= 0.001,
    ),
  );
  if (selectedKeyframeTime.value === timeSeconds) {
    selectedKeyframeTime.value = null;
  }
}

function clearKeyframes() {
  emit("update:keyframes", []);
  selectedKeyframeTime.value = null;
}

function clampDraftPosition() {
  draftXRatio.value = clamp(draftXRatio.value, 0, 1 - props.regionWidthRatio);
  draftYRatio.value = clamp(draftYRatio.value, 0, 1 - props.regionHeightRatio);
}

function numberValue(event: Event, minimum: number, maximum: number) {
  const parsed = Number((event.target as HTMLInputElement).value);
  return clamp(Number.isFinite(parsed) ? parsed : minimum, minimum, maximum);
}

function formatTime(seconds: number) {
  const safe = Math.max(0, seconds);
  const minutes = Math.floor(safe / 60);
  const remaining = safe - minutes * 60;
  return `${minutes}:${remaining.toFixed(2).padStart(5, "0")}`;
}

function round(value: number, precision: number) {
  const scale = 10 ** precision;
  return Math.round(value * scale) / scale;
}

function clamp(value: number, minimum: number, maximum: number) {
  return Math.min(maximum, Math.max(minimum, value));
}
</script>

<template>
  <section class="moving-watermark-tracker" aria-label="移动水印关键位置">
    <div v-if="previewUrl" class="moving-watermark-stage" :style="stageStyle">
      <video
        ref="videoRef"
        :key="previewUrl"
        :src="previewUrl"
        muted
        playsinline
        preload="metadata"
        @loadedmetadata="readVideoMetadata"
        @timeupdate="updateCurrentTime"
        @play="isPlaying = true"
        @pause="isPlaying = false"
        @ended="isPlaying = false"
      ></video>
      <div
        ref="stageRef"
        class="moving-watermark-stage__interaction"
        :class="{ 'moving-watermark-stage__interaction--moving': isMovingRegion }"
        @pointerdown="beginMove"
        @pointermove="continueMove"
        @pointerup="endMove"
        @pointercancel="endMove"
      >
        <svg class="moving-watermark-stage__trajectory" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
          <polyline v-if="sortedKeyframes.length > 1" :points="trajectoryPoints" />
          <circle
            v-for="(keyframe, index) in sortedKeyframes"
            :key="`${keyframe.timeSeconds}-${index}`"
            :cx="(keyframe.xRatio + regionWidthRatio / 2) * 100"
            :cy="(keyframe.yRatio + regionHeightRatio / 2) * 100"
            r="1.2"
          />
        </svg>
        <span class="moving-watermark-stage__region" :style="regionStyle">移动水印</span>
      </div>
    </div>

    <div v-else class="moving-watermark-empty" role="status">
      请先从左侧选择一个完整视频，再设置移动水印轨迹。
    </div>

    <div class="moving-watermark-player-controls">
      <button type="button" :disabled="!previewUrl || disabled" @click="togglePlayback">
        {{ isPlaying ? "暂停" : "播放" }}
      </button>
      <input
        :value="currentTime"
        type="range"
        min="0"
        :max="duration || 0"
        step="0.01"
        :disabled="!previewUrl || disabled"
        aria-label="移动水印视频时间"
        @input="seekFromInput"
      />
      <span>{{ formatTime(currentTime) }} / {{ formatTime(duration) }}</span>
    </div>

    <p class="watermark-help">暂停到水印位置发生变化的时刻，直接在画面上拖动框，再保存关键位置。</p>

    <div class="moving-watermark-region-fields">
      <label class="field">
        <span>横向位置</span>
        <input
          :value="Math.round(draftXRatio * 100)"
          type="number"
          min="0"
          :max="Math.round((1 - regionWidthRatio) * 100)"
          :disabled="disabled"
          @input="updateDraftX"
        />
      </label>
      <label class="field">
        <span>纵向位置</span>
        <input
          :value="Math.round(draftYRatio * 100)"
          type="number"
          min="0"
          :max="Math.round((1 - regionHeightRatio) * 100)"
          :disabled="disabled"
          @input="updateDraftY"
        />
      </label>
      <label class="field">
        <span>框宽度</span>
        <input
          :value="Math.round(regionWidthRatio * 100)"
          type="number"
          min="4"
          max="80"
          :disabled="disabled"
          @input="updateRegionWidth"
        />
      </label>
      <label class="field">
        <span>框高度</span>
        <input
          :value="Math.round(regionHeightRatio * 100)"
          type="number"
          min="4"
          max="80"
          :disabled="disabled"
          @input="updateRegionHeight"
        />
      </label>
    </div>

    <div class="moving-watermark-actions">
      <button class="primary-button" type="button" :disabled="!canSaveKeyframe" @click="saveCurrentKeyframe">
        {{ currentKeyframeIndex >= 0 ? "更新当前关键位置" : "添加当前关键位置" }}
      </button>
      <button type="button" :disabled="keyframes.length === 0 || disabled" @click="clearKeyframes">清空轨迹</button>
    </div>

    <ol v-if="sortedKeyframes.length" class="moving-watermark-keyframes" aria-label="移动水印关键位置列表">
      <li
        v-for="(keyframe, index) in sortedKeyframes"
        :key="keyframe.timeSeconds"
        :class="{ 'moving-watermark-keyframes__item--active': selectedKeyframeTime === keyframe.timeSeconds }"
      >
        <button type="button" :disabled="disabled" @click="loadKeyframe(keyframe)">
          <strong>关键位置 {{ index + 1 }}</strong>
          <small>{{ formatTime(keyframe.timeSeconds) }} · X {{ Math.round(keyframe.xRatio * 100) }}% · Y {{ Math.round(keyframe.yRatio * 100) }}%</small>
        </button>
        <button type="button" :disabled="disabled" aria-label="删除这个关键位置" @click="removeKeyframe(keyframe.timeSeconds)">删除</button>
      </li>
    </ol>
    <p v-else class="moving-watermark-keyframes-empty" role="status">至少添加2个关键位置，最多支持8个。</p>
  </section>
</template>
