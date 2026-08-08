<script setup lang="ts">
import { formatFileName, readNumber } from "./inputHelpers";
import ToolIcon from "../ToolIcon.vue";

defineProps<{
  selectedCoverUrl: string | null;
  selectedCoverPath: string | null;
  coverFrameSeconds: number;
  isGeneratingCover: boolean;
  coverError: string | null;
}>();

const emit = defineEmits<{
  selectCoverImageFile: [];
  selectCoverImageFolder: [];
  generateCoverFrame: [];
  "update:coverFrameSeconds": [value: number];
  reset: [];
}>();
</script>

<template>
  <div class="replica-parameter-panel replica-cover-panel">
    <header class="replica-parameter-panel__header">
      <h2>视频封面 - 参数设置</h2>
      <div class="replica-parameter-panel__actions">
        <button class="replica-parameter-button replica-parameter-button--reset" type="button" @click="emit('reset')">重置</button>
        <button class="replica-parameter-button replica-parameter-button--enable" type="button">启用</button>
      </div>
    </header>

    <div class="replica-cover-row">
      <strong>封面图片:</strong>
      <span class="replica-file-display" :title="selectedCoverPath ?? '未选择封面图片'">{{ selectedCoverPath ? formatFileName(selectedCoverPath) : "未选择封面图片" }}</span>
      <button class="replica-icon-button" type="button" aria-label="选择封面图片文件" title="选择封面图片文件" @click="emit('selectCoverImageFile')"><ToolIcon name="file" /></button>
      <button class="replica-icon-button" type="button" aria-label="选择封面图片文件夹" title="选择封面图片文件夹" @click="emit('selectCoverImageFolder')"><ToolIcon name="folder" /></button>
    </div>

    <div class="replica-cover-row replica-cover-row--frame">
      <strong>显示帧数:</strong>
      <label class="replica-range-field"><input :value="coverFrameSeconds" type="number" min="0" step="0.1" :disabled="isGeneratingCover" @input="emit('update:coverFrameSeconds', readNumber($event))" /></label>
      <span>帧</span>
      <small>（可控制封面显示时长）</small>
    </div>

    <button class="replica-cover-frame-button" type="button" :disabled="isGeneratingCover" @click="emit('generateCoverFrame')">
      {{ isGeneratingCover ? "正在生成封面帧..." : "使用当前视频帧作为封面" }}
    </button>
    <p v-if="coverError" class="replica-parameter-error">{{ coverError }}</p>
    <p class="replica-parameter-help">选择单个文件为固定使用该封面，选择文件夹可在批量处理时随机抽取一张作为封面。也可以通过上方的显示帧数控制封面在视频开头的显示时长。</p>
  </div>
</template>
