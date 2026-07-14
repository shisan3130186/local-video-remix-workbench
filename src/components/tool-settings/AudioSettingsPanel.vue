<script setup lang="ts">
import { clampNumber, formatFileName, readChecked, readNumber } from "./inputHelpers";

defineProps<{
  bgmEnabled: boolean;
  bgmAudioFilePath: string | null;
  originalVolume: number;
  bgmVolume: number;
  bgmFadeInSeconds: number;
  bgmFadeOutSeconds: number;
  isMixing: boolean;
  isBatchMixing: boolean;
}>();

const emit = defineEmits<{
  selectBgmAudioFile: [];
  "update:bgmEnabled": [value: boolean];
  "update:originalVolume": [value: number];
  "update:bgmVolume": [value: number];
  "update:bgmFadeInSeconds": [value: number];
  "update:bgmFadeOutSeconds": [value: number];
}>();
</script>

<template>
  <label class="option-toggle">
    <input :checked="bgmEnabled" type="checkbox" @change="emit('update:bgmEnabled', readChecked($event))" />
    <span>启用 BGM</span>
  </label>
  <p class="output-path">{{ formatFileName(bgmAudioFilePath) }}</p>
  <button class="ghost-button ghost-button--full" type="button" @click="emit('selectBgmAudioFile')">
    选择本地音乐
  </button>
  <label class="field">
    <span>原视频音量</span>
    <input
      :value="originalVolume"
      type="number"
      min="0"
      max="2"
      step="0.05"
      :disabled="isMixing || isBatchMixing"
      @input="emit('update:originalVolume', clampNumber(readNumber($event), 0, 2))"
    />
  </label>
  <label class="field">
    <span>BGM 音量</span>
    <input
      :value="bgmVolume"
      type="number"
      min="0"
      max="2"
      step="0.05"
      :disabled="isMixing || isBatchMixing"
      @input="emit('update:bgmVolume', clampNumber(readNumber($event), 0, 2))"
    />
  </label>
  <label class="field">
    <span>BGM 淡入秒数</span>
    <input
      :value="bgmFadeInSeconds"
      type="number"
      min="0"
      max="10"
      step="0.1"
      :disabled="isMixing || isBatchMixing"
      @input="emit('update:bgmFadeInSeconds', clampNumber(readNumber($event), 0, 10))"
    />
  </label>
  <label class="field">
    <span>BGM 淡出秒数</span>
    <input
      :value="bgmFadeOutSeconds"
      type="number"
      min="0"
      max="10"
      step="0.1"
      :disabled="isMixing || isBatchMixing"
      @input="emit('update:bgmFadeOutSeconds', clampNumber(readNumber($event), 0, 10))"
    />
  </label>
  <p class="empty-text">BGM 会自动适配导出视频长度：短了会循环，长了会截断；淡入淡出只影响背景音乐。</p>
</template>
