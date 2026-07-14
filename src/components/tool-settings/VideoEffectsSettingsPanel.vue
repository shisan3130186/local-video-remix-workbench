<script setup lang="ts">
import type { RotationMode } from "../../services/videoMixService";
import type { ToolKey } from "../../types/workbench";
import { readChecked, readNumber, readSelectedValue } from "./inputHelpers";

defineProps<{
  activeTool: ToolKey;
  applyHorizontalMirror: boolean;
  applyVerticalMirror: boolean;
  playbackSpeed: number;
  rotationMode: RotationMode;
  brightness: number;
  contrast: number;
  saturation: number;
  effectScale: number;
  isMixing: boolean;
  isBatchMixing: boolean;
}>();

const emit = defineEmits<{
  "update:applyHorizontalMirror": [value: boolean];
  "update:applyVerticalMirror": [value: boolean];
  "update:playbackSpeed": [value: number];
  "update:rotationMode": [value: RotationMode];
  "update:brightness": [value: number];
  "update:contrast": [value: number];
  "update:saturation": [value: number];
  "update:effectScale": [value: number];
}>();
</script>

<template>
  <template v-if="activeTool === 'mirror'">
    <label class="option-toggle">
      <input
        :checked="applyHorizontalMirror"
        type="checkbox"
        @change="emit('update:applyHorizontalMirror', readChecked($event))"
      />
      <span>水平镜像</span>
    </label>
    <label class="option-toggle">
      <input
        :checked="applyVerticalMirror"
        type="checkbox"
        @change="emit('update:applyVerticalMirror', readChecked($event))"
      />
      <span>垂直镜像</span>
    </label>
    <p class="empty-text">镜像效果会应用到拼接抽中片段和批量生成混剪。</p>
  </template>

  <label v-else-if="activeTool === 'rotate'" class="field">
    <span>旋转方式</span>
    <select
      :value="rotationMode"
      @change="emit('update:rotationMode', readSelectedValue($event) as RotationMode)"
    >
      <option value="none">不旋转</option>
      <option value="clockwise90">顺时针 90°</option>
      <option value="counterclockwise90">逆时针 90°</option>
      <option value="rotate180">旋转 180°</option>
    </select>
  </label>

  <label v-else-if="activeTool === 'speed'" class="field">
    <span>变速倍数</span>
    <input
      :value="playbackSpeed"
      type="number"
      min="0.5"
      max="2"
      step="0.1"
      :disabled="isMixing || isBatchMixing"
      @input="emit('update:playbackSpeed', readNumber($event))"
    />
  </label>

  <template v-else-if="activeTool === 'effects' || activeTool === 'adjust'">
    <label class="field">
      <span>亮度</span>
      <input
        :value="brightness"
        type="number"
        min="-1"
        max="1"
        step="0.05"
        :disabled="isMixing || isBatchMixing"
        @input="emit('update:brightness', readNumber($event))"
      />
    </label>
    <label class="field">
      <span>对比度</span>
      <input
        :value="contrast"
        type="number"
        min="0"
        max="3"
        step="0.05"
        :disabled="isMixing || isBatchMixing"
        @input="emit('update:contrast', readNumber($event))"
      />
    </label>
    <label class="field">
      <span>饱和度</span>
      <input
        :value="saturation"
        type="number"
        min="0"
        max="3"
        step="0.05"
        :disabled="isMixing || isBatchMixing"
        @input="emit('update:saturation', readNumber($event))"
      />
    </label>
  </template>

  <template v-else-if="activeTool === 'zoom'">
    <label class="field">
      <span>轻微缩放</span>
      <input
        :value="effectScale"
        type="number"
        min="1"
        max="1.2"
        step="0.01"
        :disabled="isMixing || isBatchMixing"
        @input="emit('update:effectScale', readNumber($event))"
      />
    </label>
    <p class="empty-text">建议范围 1.00 到 1.20，用于轻微放大画面。</p>
  </template>
</template>
