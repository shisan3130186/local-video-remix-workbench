<script setup lang="ts">
import type { PipPosition } from "../../services/videoMixService";
import {
  clampNumber,
  formatFileName,
  readChecked,
  readNumber,
  readSelectedValue,
} from "./inputHelpers";

defineProps<{
  pipEnabled: boolean;
  pipOverlayFilePath: string | null;
  pipPosition: PipPosition;
  pipSizeRatio: number;
  pipOpacity: number;
  pipMargin: number;
}>();

const emit = defineEmits<{
  selectPipOverlayFile: [];
  "update:pipEnabled": [value: boolean];
  "update:pipPosition": [value: PipPosition];
  "update:pipSizeRatio": [value: number];
  "update:pipOpacity": [value: number];
  "update:pipMargin": [value: number];
}>();
</script>

<template>
  <label class="option-toggle">
    <input :checked="pipEnabled" type="checkbox" @change="emit('update:pipEnabled', readChecked($event))" />
    <span>启用画中画</span>
  </label>
  <p class="output-path">{{ formatFileName(pipOverlayFilePath) }}</p>
  <button class="ghost-button ghost-button--full" type="button" @click="emit('selectPipOverlayFile')">
    选择叠加视频 / 图片
  </button>
  <label class="field">
    <span>位置</span>
    <select
      :value="pipPosition"
      @change="emit('update:pipPosition', readSelectedValue($event) as PipPosition)"
    >
      <option value="topLeft">左上</option>
      <option value="topRight">右上</option>
      <option value="bottomLeft">左下</option>
      <option value="bottomRight">右下</option>
      <option value="center">居中</option>
    </select>
  </label>
  <label class="field">
    <span>大小比例</span>
    <input
      :value="pipSizeRatio"
      type="number"
      min="0.2"
      max="0.5"
      step="0.05"
      @input="emit('update:pipSizeRatio', clampNumber(readNumber($event), 0.2, 0.5))"
    />
  </label>
  <label class="field">
    <span>透明度</span>
    <input
      :value="pipOpacity"
      type="number"
      min="0"
      max="1"
      step="0.05"
      @input="emit('update:pipOpacity', clampNumber(readNumber($event), 0, 1))"
    />
  </label>
  <label class="field">
    <span>边距</span>
    <input
      :value="pipMargin"
      type="number"
      min="0"
      max="240"
      step="4"
      @input="emit('update:pipMargin', clampNumber(readNumber($event), 0, 240))"
    />
  </label>
</template>
