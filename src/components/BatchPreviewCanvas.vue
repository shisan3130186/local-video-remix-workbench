<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch, type CSSProperties } from "vue";
import type { ImportedVideo } from "../types/videoProbe";
import type { CanvasAspectRatio, CanvasBackgroundMode, CanvasCropSettings } from "../services/videoMixService";
import ToolIcon from "./ToolIcon.vue";

const props = defineProps<{
  previewUrl: string | null;
  selectedVideo: ImportedVideo | null;
}>();

const emit = defineEmits<{
  "update:canvasAspectRatio": [value: CanvasAspectRatio];
  "update:canvasBackgroundMode": [value: CanvasBackgroundMode];
  "open-guide": [];
}>();

const aspectRatio = defineModel<CanvasAspectRatio>("canvasAspectRatio", { required: true });
const backgroundMode = defineModel<CanvasBackgroundMode>("canvasBackgroundMode", { required: true });
const effectScale = defineModel<number>("effectScale", { required: true });
const cropSettings = defineModel<CanvasCropSettings>("cropSettings", { required: true });
const watermarkEnabled = defineModel<boolean>("watermarkEnabled", { required: true });
const watermarkKind = defineModel<"text" | "image">("watermarkKind", { required: true });
const watermarkText = defineModel<string>("watermarkText", { required: true });
const watermarkPosition = defineModel<"topLeft" | "topRight" | "bottomLeft" | "bottomRight" | "center">("watermarkPosition", { required: true });
const cropOpen = ref(false);
const textOpen = ref(false);
const cropRect = computed({
  get: () => cropSettings.value,
  set: (value: CanvasCropSettings) => {
    cropSettings.value = {
      ...value,
      enabled: value.x > 0 || value.y > 0 || value.width < 100 || value.height < 100,
    };
  },
});
const cropPointer = ref<{ kind: "move" | "resize"; handle?: "nw" | "ne" | "sw" | "se"; startX: number; startY: number; startRect: typeof cropRect.value } | null>(null);
const canvasRef = ref<HTMLElement | null>(null);
const canvasSize = ref({ width: 0, height: 0 });
const nativeVideoSize = ref({ width: 0, height: 0 });
let canvasResizeObserver: ResizeObserver | null = null;

const aspectOptions: Array<{ value: CanvasAspectRatio; label: string }> = [
  { value: "original", label: "原比例" },
  { value: "portrait916", label: "9:16" },
  { value: "landscape169", label: "16:9" },
];

const fillOptions: Array<{ value: CanvasBackgroundMode; label: string }> = [
  { value: "black", label: "适应填充" },
  { value: "blur", label: "模糊填充" },
];

const detectedSourceRatio = computed(() => {
  if (nativeVideoSize.value.width && nativeVideoSize.value.height) {
    return nativeVideoSize.value.width / nativeVideoSize.value.height;
  }
  if (props.selectedVideo?.width && props.selectedVideo?.height) {
    return props.selectedVideo.width / props.selectedVideo.height;
  }
  return 16 / 9;
});
const previewPlayerRatio = computed(() => {
  if (aspectRatio.value === "portrait916") return 9 / 16;
  if (aspectRatio.value === "landscape169") return 16 / 9;
  return detectedSourceRatio.value;
});

const isPortrait = computed(() => {
  if (aspectRatio.value === "portrait916") return true;
  if (aspectRatio.value !== "original") return false;
  return detectedSourceRatio.value < 1;
});

const targetRatio = computed(() => {
  if (aspectRatio.value === "portrait916") return 9 / 16;
  if (aspectRatio.value === "landscape169") return 16 / 9;
  return detectedSourceRatio.value;
});

const sourceRatio = computed(() => {
  return detectedSourceRatio.value;
});

const mediaFrameStyle = computed<CSSProperties>(() => ({
  width: sourceRatio.value >= previewPlayerRatio.value ? "100%" : "auto",
  height: sourceRatio.value >= previewPlayerRatio.value ? "auto" : "100%",
  maxWidth: "100%",
  maxHeight: "100%",
  aspectRatio: String(sourceRatio.value),
}));

const videoStyle = computed<CSSProperties>(() => ({
  width: "100%",
  height: "100%",
  maxWidth: "none",
  maxHeight: "none",
  objectFit: "contain",
  // 只有裁剪编辑状态才缩放素材，退出裁剪后恢复完整画面。
  transform: cropOpen.value ? `scale(${Math.max(1, Math.min(1.2, effectScale.value))})` : undefined,
}));
const canvasStyle = computed(() => ({
  width: "100%",
  height: "auto",
  maxWidth: "100%",
  maxHeight: "100%",
  aspectRatio: String(previewPlayerRatio.value),
}));
const aspectGuideStyle = computed(() => {
  const { width, height } = canvasSize.value;
  if (!width || !height) return { width: "100%", height: "100%" };
  const canvasRatio = width / height;
  if (canvasRatio >= targetRatio.value) return { width: `${height * targetRatio.value}px`, height: `${height}px` };
  return { width: `${width}px`, height: `${width / targetRatio.value}px` };
});
const cropRectStyle = computed(() => ({
  left: `${cropRect.value.x}%`,
  top: `${cropRect.value.y}%`,
  width: `${cropRect.value.width}%`,
  height: `${cropRect.value.height}%`,
}));

function setAspect(value: CanvasAspectRatio) {
  aspectRatio.value = value;
  emit("update:canvasAspectRatio", value);
}

function setBackground(value: CanvasBackgroundMode) {
  backgroundMode.value = value;
  emit("update:canvasBackgroundMode", value);
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

function startCropInteraction(event: PointerEvent, kind: "move" | "resize", handle?: "nw" | "ne" | "sw" | "se") {
  const canvas = (event.currentTarget as HTMLElement).closest(".replica-batch-canvas");
  if (!canvas) return;
  event.preventDefault();
  cropPointer.value = { kind, handle, startX: event.clientX, startY: event.clientY, startRect: { ...cropRect.value } };
  window.addEventListener("pointermove", handleCropPointerMove);
  window.addEventListener("pointerup", stopCropInteraction, { once: true });
}

function handleCropPointerMove(event: PointerEvent) {
  const interaction = cropPointer.value;
  if (!interaction) return;
  const rect = canvasRef.value?.getBoundingClientRect();
  if (!rect) return;
  const deltaX = (event.clientX - interaction.startX) / Math.max(1, rect.width) * 100;
  const deltaY = (event.clientY - interaction.startY) / Math.max(1, rect.height) * 100;
  if (interaction.kind === "move") {
    cropRect.value = {
      ...cropRect.value,
      x: clamp(interaction.startRect.x + deltaX, 0, 100 - interaction.startRect.width),
      y: clamp(interaction.startRect.y + deltaY, 0, 100 - interaction.startRect.height),
    };
    return;
  }
  const start = interaction.startRect;
  let left = start.x;
  let top = start.y;
  let right = start.x + start.width;
  let bottom = start.y + start.height;
  if (interaction.handle?.includes("w")) left = clamp(start.x + deltaX, 0, right - 12);
  if (interaction.handle?.includes("e")) right = clamp(start.x + start.width + deltaX, left + 12, 100);
  if (interaction.handle?.includes("n")) top = clamp(start.y + deltaY, 0, bottom - 12);
  if (interaction.handle?.includes("s")) bottom = clamp(start.y + start.height + deltaY, top + 12, 100);
  cropRect.value = { enabled: true, x: left, y: top, width: right - left, height: bottom - top };
}

function resetCrop() {
  effectScale.value = 1;
  cropRect.value = { enabled: false, x: 0, y: 0, width: 100, height: 100 };
}

function enableTextOverlay() {
  watermarkEnabled.value = true;
  watermarkKind.value = "text";
  watermarkPosition.value = "center";
}

function clearTextOverlay() {
  watermarkText.value = "";
  if (watermarkKind.value === "text") watermarkEnabled.value = false;
}

function syncNativeVideoSize(event: Event) {
  const video = event.currentTarget;
  if (!(video instanceof HTMLVideoElement) || !video.videoWidth || !video.videoHeight) return;
  nativeVideoSize.value = { width: video.videoWidth, height: video.videoHeight };
}

function stopCropInteraction() {
  cropPointer.value = null;
  window.removeEventListener("pointermove", handleCropPointerMove);
}

function updateCanvasSize() {
  const rect = canvasRef.value?.getBoundingClientRect();
  if (rect) canvasSize.value = { width: rect.width, height: rect.height };
}

onMounted(() => {
  updateCanvasSize();
  if (canvasRef.value) {
    canvasResizeObserver = new ResizeObserver(updateCanvasSize);
    canvasResizeObserver.observe(canvasRef.value);
  }
});

watch(() => props.previewUrl, () => {
  nativeVideoSize.value = { width: 0, height: 0 };
});

onUnmounted(() => {
  stopCropInteraction();
  canvasResizeObserver?.disconnect();
});
</script>

<template>
  <section class="replica-batch-preview replica-batch-preview--canvas" aria-label="视频预览与画布设置">
    <header class="replica-batch-canvas-toolbar">
       <button type="button" class="replica-guide-trigger" @click="$emit('open-guide')"><ToolIcon name="info" />使用指引</button>
      <div class="replica-canvas-controls">
        <div class="replica-control-group"><span>视频比例</span><button v-for="option in aspectOptions" :key="option.value" type="button" :class="{ 'is-active': aspectRatio === option.value }" @click="setAspect(option.value)">{{ option.label }}</button></div>
        <div class="replica-control-group"><span>填充方式</span><button v-for="option in fillOptions" :key="option.value" type="button" :class="{ 'is-active': backgroundMode === option.value }" :disabled="aspectRatio === 'original'" @click="setBackground(option.value)">{{ option.label }}</button></div>
        <button class="replica-inline-tool" type="button" :class="{ 'is-active': cropOpen }" @click="cropOpen = !cropOpen">视频裁剪</button>
        <button class="replica-inline-tool" type="button" :class="{ 'is-active': textOpen }" @click="textOpen = !textOpen">添加文本</button>
      </div>
    </header>

    <div v-if="cropOpen" class="replica-batch-inline-editor">
      <label for="batch-crop-scale">裁剪缩放</label>
      <input id="batch-crop-scale" v-model.number="effectScale" type="range" min="1" max="1.2" step="0.01" />
      <output>{{ effectScale.toFixed(2) }}×</output>
      <button type="button" @click="resetCrop">重置</button>
    </div>
    <div v-if="textOpen" class="replica-batch-inline-editor replica-batch-inline-editor--text">
      <label for="batch-overlay-text">画面文本</label>
      <input id="batch-overlay-text" v-model="watermarkText" maxlength="80" placeholder="输入要显示在画面中的文字" @input="enableTextOverlay" />
      <button type="button" @click="clearTextOverlay">清空</button>
    </div>

    <div class="replica-batch-screen replica-batch-screen--canvas" :class="{ 'is-blur-fill': backgroundMode === 'blur' && aspectRatio !== 'original' }">
      <div ref="canvasRef" class="replica-batch-canvas" :class="{ 'is-text-editing': textOpen, 'is-crop-editing': cropOpen }" :style="canvasStyle">
        <video v-if="previewUrl && backgroundMode === 'blur' && aspectRatio !== 'original'" class="replica-batch-canvas__background" :src="previewUrl" muted autoplay loop playsinline preload="metadata" aria-hidden="true"></video>
        <div class="replica-batch-media-frame" :style="mediaFrameStyle">
          <video v-if="previewUrl" class="replica-batch-canvas__foreground" :src="previewUrl" :class="{ 'is-portrait': isPortrait }" :style="videoStyle" controls playsinline preload="metadata" @loadedmetadata="syncNativeVideoSize"></video>
        </div>
        <div v-if="aspectRatio !== 'original'" class="replica-batch-aspect-guide" :style="aspectGuideStyle" aria-label="当前输出比例虚框" aria-hidden="true"></div>
        <div v-if="cropOpen" class="replica-crop-box replica-batch-crop-box" :style="cropRectStyle" aria-label="可调整的视频裁剪区域" @pointerdown="startCropInteraction($event, 'move')">
          <span class="replica-crop-grid replica-crop-grid--vertical" aria-hidden="true"></span><span class="replica-crop-grid replica-crop-grid--horizontal" aria-hidden="true"></span>
          <button v-for="handle in (['nw', 'ne', 'sw', 'se'] as const)" :key="handle" class="replica-crop-handle" :class="`replica-crop-handle--${handle}`" type="button" :aria-label="`调整裁剪框${handle}`" @pointerdown.stop="startCropInteraction($event, 'resize', handle)"></button>
          <span class="replica-crop-size">{{ Math.round(cropRect.width) }}% × {{ Math.round(cropRect.height) }}%</span>
        </div>
        <div v-if="watermarkEnabled && watermarkKind === 'text' && watermarkText" class="replica-batch-text-overlay">{{ watermarkText }}</div>
      </div>
       <div v-if="!previewUrl" class="replica-batch-screen--empty"><span><ToolIcon name="video" /></span><strong>选择素材后在此预览</strong></div>
    </div>
  </section>
</template>
