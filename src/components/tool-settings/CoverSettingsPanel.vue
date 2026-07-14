<script setup lang="ts">
import { readNumber } from "./inputHelpers";

defineProps<{
  selectedCoverUrl: string | null;
  coverFrameSeconds: number;
  isGeneratingCover: boolean;
  coverError: string | null;
}>();

const emit = defineEmits<{
  generateCoverFrame: [];
  "update:coverFrameSeconds": [value: number];
}>();
</script>

<template>
  <div class="cover-preview-box">
    <img v-if="selectedCoverUrl" :src="selectedCoverUrl" alt="" />
    <span v-else>暂未生成封面帧</span>
  </div>
  <label class="field">
    <span>抽帧时间（秒）</span>
    <input
      :value="coverFrameSeconds"
      type="number"
      min="0"
      step="0.1"
      :disabled="isGeneratingCover"
      @input="emit('update:coverFrameSeconds', readNumber($event))"
    />
  </label>
  <button
    class="primary-button primary-button--full"
    type="button"
    :disabled="isGeneratingCover"
    @click="emit('generateCoverFrame')"
  >
    {{ isGeneratingCover ? "正在生成..." : "设为封面帧" }}
  </button>
  <p v-if="coverError" class="error-text">{{ coverError }}</p>
  <p v-else class="empty-text">本阶段只生成封面帧和片段预览图，不做复杂关键帧动画。</p>
</template>
