<script setup lang="ts">
import { ref } from "vue";
import { clampNumber, formatFileName } from "./inputHelpers";
import ToolIcon from "../ToolIcon.vue";

const props = defineProps<{
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
  reset: [];
}>();

const mainVolumeMin = ref(95);
const mainVolumeMax = ref(105);
const mainFadeEnabled = ref(false);
const dynamicAdjustEnabled = ref(true);
const bgmVolumeMin = ref(25);
const bgmVolumeMax = ref(35);
const bgmFadeEnabled = ref(false);
const loopPlaybackEnabled = ref(true);

function syncMainVolume() {
  emit("update:originalVolume", clampNumber((mainVolumeMin.value + mainVolumeMax.value) / 200, 0, 2));
}

function syncBgmVolume() {
  emit("update:bgmVolume", clampNumber((bgmVolumeMin.value + bgmVolumeMax.value) / 200, 0, 2));
}

function resetSettings() {
  mainVolumeMin.value = 95;
  mainVolumeMax.value = 105;
  mainFadeEnabled.value = false;
  dynamicAdjustEnabled.value = true;
  bgmVolumeMin.value = 25;
  bgmVolumeMax.value = 35;
  bgmFadeEnabled.value = false;
  loopPlaybackEnabled.value = true;
  emit("update:bgmEnabled", false);
  emit("update:originalVolume", 1);
  emit("update:bgmVolume", 0.3);
  emit("update:bgmFadeInSeconds", 0);
  emit("update:bgmFadeOutSeconds", 0);
  emit("reset");
}
</script>

<template>
  <div class="replica-parameter-panel replica-audio-panel">
    <header class="replica-parameter-panel__header">
      <h2>音频设置 - 参数设置</h2>
      <div class="replica-parameter-panel__actions">
        <button class="replica-parameter-button replica-parameter-button--reset" type="button" @click="resetSettings">重置</button>
        <button class="replica-parameter-button replica-parameter-button--enable" type="button" @click="emit('update:bgmEnabled', true)">
          {{ props.bgmEnabled ? "已启用" : "启用" }}
        </button>
      </div>
    </header>

    <div class="replica-audio-row replica-audio-row--main">
      <strong>主视频音量:</strong>
      <label class="replica-range-field"><input v-model.number="mainVolumeMin" type="number" min="0" max="200" @change="syncMainVolume" /></label>
      <span>-</span>
      <label class="replica-range-field"><input v-model.number="mainVolumeMax" type="number" min="0" max="200" @change="syncMainVolume" /></label>
      <span>%</span>
      <label class="replica-inline-check"><input v-model="mainFadeEnabled" type="checkbox" /><span>淡入淡出</span></label>
      <label class="replica-inline-check"><input v-model="dynamicAdjustEnabled" type="checkbox" /><span>动态调整</span></label>
    </div>

    <div class="replica-audio-row replica-audio-row--file">
      <strong>背景音乐:</strong>
      <span class="replica-file-display" :title="props.bgmAudioFilePath ?? '未选择'">{{ formatFileName(props.bgmAudioFilePath) || "未选择（可选择音频/视频文件）" }}</span>
      <button class="replica-icon-button" type="button" aria-label="选择背景音乐文件" title="选择背景音乐文件" @click="emit('selectBgmAudioFile')"><ToolIcon name="file" /></button>
      <button class="replica-icon-button" type="button" aria-label="选择背景音乐文件夹" title="选择背景音乐文件夹" @click="emit('selectBgmAudioFile')"><ToolIcon name="folder" /></button>
      <button class="replica-clear-button" type="button" :disabled="!props.bgmAudioFilePath" @click="emit('update:bgmEnabled', false)">清除</button>
    </div>

    <div class="replica-audio-row replica-audio-row--bgm">
      <strong>背景音乐音量:</strong>
      <label class="replica-range-field"><input v-model.number="bgmVolumeMin" type="number" min="0" max="200" @change="syncBgmVolume" /></label>
      <span>-</span>
      <label class="replica-range-field"><input v-model.number="bgmVolumeMax" type="number" min="0" max="200" @change="syncBgmVolume" /></label>
      <span>%</span>
      <label class="replica-inline-check"><input v-model="bgmFadeEnabled" type="checkbox" /><span>淡入淡出</span></label>
    </div>

    <label class="replica-inline-check replica-loop-check"><input v-model="loopPlaybackEnabled" type="checkbox" /><span>循环播放</span></label>
    <p class="replica-parameter-help">主视频音量：0%=静音，100%=原音量，200%=2倍音量。背景音量范围设置可让每个视频的背景音乐音量不同。主视频和背景音乐可分别设置淡入淡出，避免突兀的开始和结束。动态调整勾选后系统会自动进行各种微调。背景音乐支持音频和视频格式，选择视频时自动提取音频，可选择单个文件或文件夹（随机选择）。</p>
  </div>
</template>
