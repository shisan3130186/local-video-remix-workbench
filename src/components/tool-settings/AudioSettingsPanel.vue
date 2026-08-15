<script setup lang="ts">
import { clampNumber, formatFileName, readChecked, readNumber } from "./inputHelpers";
import ToolIcon from "../ToolIcon.vue";

const props = defineProps<{
  bgmEnabled: boolean;
  bgmAudioFilePath: string | null;
  originalVolume: number;
  originalVolumeMin: number;
  originalVolumeMax: number;
  originalFadeEnabled: boolean;
  dynamicAdjustEnabled: boolean;
  bgmVolume: number;
  bgmVolumeMin: number;
  bgmVolumeMax: number;
  bgmFadeEnabled: boolean;
  loopPlaybackEnabled: boolean;
  bgmFadeInSeconds: number;
  bgmFadeOutSeconds: number;
  isMixing: boolean;
  isBatchMixing: boolean;
}>();

const emit = defineEmits<{
  selectBgmAudioFile: [];
  selectBgmAudioFolder: [];
  clearBgmAudioFile: [];
  "update:bgmEnabled": [value: boolean];
  "update:originalVolume": [value: number];
  "update:originalVolumeMin": [value: number];
  "update:originalVolumeMax": [value: number];
  "update:originalFadeEnabled": [value: boolean];
  "update:dynamicAdjustEnabled": [value: boolean];
  "update:bgmVolume": [value: number];
  "update:bgmVolumeMin": [value: number];
  "update:bgmVolumeMax": [value: number];
  "update:bgmFadeEnabled": [value: boolean];
  "update:loopPlaybackEnabled": [value: boolean];
  "update:bgmFadeInSeconds": [value: number];
  "update:bgmFadeOutSeconds": [value: number];
  reset: [];
}>();

function updateMainVolumeRange(key: "min" | "max", event: Event) {
  const value = clampNumber(readNumber(event) / 100, 0, 2);
  if (key === "min") emit("update:originalVolumeMin", value);
  else emit("update:originalVolumeMax", value);
  const other = key === "min" ? props.originalVolumeMax : props.originalVolumeMin;
  emit("update:originalVolume", (value + other) / 2);
}

function updateBgmVolumeRange(key: "min" | "max", event: Event) {
  const value = clampNumber(readNumber(event) / 100, 0, 2);
  if (key === "min") emit("update:bgmVolumeMin", value);
  else emit("update:bgmVolumeMax", value);
  const other = key === "min" ? props.bgmVolumeMax : props.bgmVolumeMin;
  emit("update:bgmVolume", (value + other) / 2);
}
</script>

<template>
  <div class="replica-parameter-panel replica-audio-panel">
    <header class="replica-parameter-panel__header">
      <h2>音频设置 - 参数设置</h2>
      <div class="replica-parameter-panel__actions">
        <button class="replica-parameter-button replica-parameter-button--reset" type="button" @click="emit('reset')">重置</button>
        <button class="replica-parameter-button replica-parameter-button--enable" type="button" @click="emit('update:bgmEnabled', true)">
          {{ props.bgmEnabled ? "已启用" : "启用" }}
        </button>
      </div>
    </header>

    <div class="replica-audio-row replica-audio-row--main">
      <strong>主视频音量:</strong>
      <label class="replica-range-field"><input :value="Math.round(props.originalVolumeMin * 100)" type="number" min="0" max="200" @input="updateMainVolumeRange('min', $event)" /></label>
      <span>-</span>
      <label class="replica-range-field"><input :value="Math.round(props.originalVolumeMax * 100)" type="number" min="0" max="200" @input="updateMainVolumeRange('max', $event)" /></label>
      <span>%</span>
      <label class="replica-inline-check"><input :checked="props.originalFadeEnabled" type="checkbox" @change="emit('update:originalFadeEnabled', readChecked($event))" /><span>淡入淡出</span></label>
      <label class="replica-inline-check"><input :checked="props.dynamicAdjustEnabled" type="checkbox" @change="emit('update:dynamicAdjustEnabled', readChecked($event))" /><span>动态调整</span></label>
    </div>

    <div class="replica-audio-row replica-audio-row--file">
      <strong>背景音乐:</strong>
      <span class="replica-file-display" :title="props.bgmAudioFilePath ?? '未选择'">{{ formatFileName(props.bgmAudioFilePath) || "未选择（可选择音频/视频文件）" }}</span>
      <button class="replica-icon-button" type="button" aria-label="选择背景音乐文件" title="选择背景音乐文件" @click="emit('selectBgmAudioFile')"><ToolIcon name="file" /></button>
      <button class="replica-icon-button" type="button" aria-label="选择背景音乐文件夹" title="选择背景音乐文件夹" @click="emit('selectBgmAudioFolder')"><ToolIcon name="folder" /></button>
      <button class="replica-clear-button" type="button" :disabled="!props.bgmAudioFilePath" @click="emit('clearBgmAudioFile')">清除</button>
    </div>

    <div class="replica-audio-row replica-audio-row--bgm">
      <strong>背景音乐音量:</strong>
      <label class="replica-range-field"><input :value="Math.round(props.bgmVolumeMin * 100)" type="number" min="0" max="200" @input="updateBgmVolumeRange('min', $event)" /></label>
      <span>-</span>
      <label class="replica-range-field"><input :value="Math.round(props.bgmVolumeMax * 100)" type="number" min="0" max="200" @input="updateBgmVolumeRange('max', $event)" /></label>
      <span>%</span>
      <label class="replica-inline-check"><input :checked="props.bgmFadeEnabled" type="checkbox" @change="emit('update:bgmFadeEnabled', readChecked($event))" /><span>淡入淡出</span></label>
    </div>

    <label class="replica-inline-check replica-loop-check"><input :checked="props.loopPlaybackEnabled" type="checkbox" @change="emit('update:loopPlaybackEnabled', readChecked($event))" /><span>循环播放</span></label>
    <p class="replica-parameter-help">主视频音量：0%=静音，100%=原音量，200%=2倍音量。背景音量范围设置可让每个视频的背景音乐音量不同。主视频和背景音乐可分别设置淡入淡出，避免突兀的开始和结束。动态调整勾选后系统会自动进行各种微调。背景音乐支持音频和视频格式，选择视频时自动提取音频，可选择单个文件或文件夹（随机选择）。</p>
  </div>
</template>
