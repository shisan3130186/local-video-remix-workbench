<script setup lang="ts">
import type { CanvasAspectRatio, CanvasBackgroundMode } from "../../services/videoMixService";
import { readSelectedValue } from "./inputHelpers";

defineProps<{
  canvasAspectRatio: CanvasAspectRatio;
  canvasBackgroundMode: CanvasBackgroundMode;
}>();

const emit = defineEmits<{
  "update:canvasAspectRatio": [value: CanvasAspectRatio];
  "update:canvasBackgroundMode": [value: CanvasBackgroundMode];
}>();
</script>

<template>
  <label class="field">
    <span>视频比例</span>
    <select
      :value="canvasAspectRatio"
      @change="emit('update:canvasAspectRatio', readSelectedValue($event) as CanvasAspectRatio)"
    >
      <option value="original">原画</option>
      <option value="portrait916">9:16 竖屏</option>
      <option value="square11">1:1 方屏</option>
      <option value="landscape169">16:9 横屏</option>
    </select>
  </label>
  <label class="field">
    <span>背景方式</span>
    <select
      :value="canvasBackgroundMode"
      :disabled="canvasAspectRatio === 'original'"
      @change="emit('update:canvasBackgroundMode', readSelectedValue($event) as CanvasBackgroundMode)"
    >
      <option value="black">黑边</option>
      <option value="blur">模糊背景</option>
    </select>
  </label>
</template>
