<script setup lang="ts">
import { computed, ref } from "vue";
import type { ImportedVideo } from "../types/videoProbe";
import type { CanvasAspectRatio, CanvasBackgroundMode } from "../services/videoMixService";

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
const cropOpen = ref(false);
const textOpen = ref(false);
const overlayText = ref("");
const cropScale = ref(1);

const aspectOptions: Array<{ value: CanvasAspectRatio; label: string }> = [
  { value: "original", label: "原比例" },
  { value: "portrait916", label: "9:16" },
  { value: "landscape169", label: "16:9" },
];

const fillOptions: Array<{ value: CanvasBackgroundMode; label: string }> = [
  { value: "black", label: "黑边填充" },
  { value: "blur", label: "模糊填充" },
];

const isPortrait = computed(() => {
  if (aspectRatio.value === "portrait916") return true;
  if (aspectRatio.value !== "original") return false;
  return Boolean(props.selectedVideo?.height && props.selectedVideo?.width && props.selectedVideo.height > props.selectedVideo.width);
});

const canvasStyle = computed(() => {
  if (aspectRatio.value === "portrait916") return { aspectRatio: "9 / 16" };
  if (aspectRatio.value === "landscape169") return { aspectRatio: "16 / 9" };
  return { aspectRatio: props.selectedVideo?.width && props.selectedVideo?.height ? `${props.selectedVideo.width} / ${props.selectedVideo.height}` : "16 / 9" };
});

const videoStyle = computed(() => ({ transform: `scale(${cropScale.value})` }));

function setAspect(value: CanvasAspectRatio) {
  aspectRatio.value = value;
  emit("update:canvasAspectRatio", value);
}

function setBackground(value: CanvasBackgroundMode) {
  backgroundMode.value = value;
  emit("update:canvasBackgroundMode", value);
}
</script>

<template>
  <section class="replica-batch-preview replica-batch-preview--canvas" aria-label="视频预览与画布设置">
    <header class="replica-batch-canvas-toolbar">
      <button type="button" class="replica-guide-trigger" @click="$emit('open-guide')">ⓘ 使用指引</button>
      <div class="replica-canvas-controls">
        <div class="replica-control-group"><span>视频比例</span><button v-for="option in aspectOptions" :key="option.value" type="button" :class="{ 'is-active': aspectRatio === option.value }" @click="setAspect(option.value)">{{ option.label }}</button></div>
        <div class="replica-control-group"><span>填充方式</span><button v-for="option in fillOptions" :key="option.value" type="button" :class="{ 'is-active': backgroundMode === option.value }" :disabled="aspectRatio === 'original'" @click="setBackground(option.value)">{{ option.label }}</button></div>
        <button class="replica-inline-tool" type="button" :class="{ 'is-active': cropOpen }" @click="cropOpen = !cropOpen">视频裁剪</button>
        <button class="replica-inline-tool" type="button" :class="{ 'is-active': textOpen }" @click="textOpen = !textOpen">添加文本</button>
      </div>
    </header>

    <div v-if="cropOpen" class="replica-batch-inline-editor">
      <label for="batch-crop-scale">裁剪缩放</label>
      <input id="batch-crop-scale" v-model.number="cropScale" type="range" min="1" max="1.3" step="0.01" />
      <output>{{ cropScale.toFixed(2) }}×</output>
      <button type="button" @click="cropScale = 1">重置</button>
    </div>
    <div v-if="textOpen" class="replica-batch-inline-editor replica-batch-inline-editor--text">
      <label for="batch-overlay-text">画面文本</label>
      <input id="batch-overlay-text" v-model="overlayText" maxlength="80" placeholder="输入要显示在画面中的文字" />
      <button type="button" @click="overlayText = ''">清空</button>
    </div>

    <div class="replica-batch-screen replica-batch-screen--canvas" :class="{ 'is-blur-fill': backgroundMode === 'blur' && aspectRatio !== 'original' }">
      <div class="replica-batch-canvas" :style="canvasStyle">
        <video v-if="previewUrl" :src="previewUrl" :class="{ 'is-portrait': isPortrait }" :style="videoStyle" controls playsinline preload="metadata"></video>
        <div v-if="aspectRatio !== 'original'" class="replica-batch-aspect-guide" aria-hidden="true"></div>
        <div v-if="cropOpen" class="replica-batch-crop-guide" aria-label="裁剪预览框"><span>裁剪范围</span></div>
        <div v-if="overlayText" class="replica-batch-text-overlay">{{ overlayText }}</div>
      </div>
      <div v-if="!previewUrl" class="replica-batch-screen--empty"><span>▶</span><strong>选择素材后在此预览</strong></div>
    </div>
  </section>
</template>
