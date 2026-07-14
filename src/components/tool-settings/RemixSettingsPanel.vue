<script setup lang="ts">
import { readNumber } from "./inputHelpers";

defineProps<{
  segmentDurationSeconds: number;
  randomPickCount: number;
  batchGenerateCount: number;
  isSplitting: boolean;
  isBatchMixing: boolean;
  splitError: string | null;
  splitOutputDirectory: string | null;
  splitSegmentCount: number | null;
  randomPickError: string | null;
}>();

const emit = defineEmits<{
  splitSelectedVideo: [];
  pickSegmentsRandomly: [];
  generateBatchMixes: [];
  "update:segmentDurationSeconds": [value: number];
  "update:randomPickCount": [value: number];
  "update:batchGenerateCount": [value: number];
}>();
</script>

<template>
  <label class="field">
    <span>切片秒数</span>
    <input
      :value="segmentDurationSeconds"
      type="number"
      min="1"
      step="1"
      :disabled="isSplitting"
      @input="emit('update:segmentDurationSeconds', readNumber($event))"
    />
  </label>
  <button
    class="primary-button primary-button--full"
    type="button"
    :disabled="isSplitting"
    @click="emit('splitSelectedVideo')"
  >
    {{ isSplitting ? "正在切片..." : "切片当前视频" }}
  </button>
  <p v-if="splitError" class="error-text">{{ splitError }}</p>
  <p v-else-if="splitOutputDirectory" class="success-text">已生成 {{ splitSegmentCount }} 个片段。</p>

  <div class="divider"></div>
  <label class="field">
    <span>抽取数量</span>
    <input
      :value="randomPickCount"
      type="number"
      min="1"
      step="1"
      @input="emit('update:randomPickCount', readNumber($event))"
    />
  </label>
  <button class="ghost-button ghost-button--full" type="button" @click="emit('pickSegmentsRandomly')">
    随机抽取
  </button>
  <p v-if="randomPickError" class="error-text">{{ randomPickError }}</p>

  <div class="divider"></div>
  <label class="field">
    <span>批量生成数量</span>
    <input
      :value="batchGenerateCount"
      type="number"
      min="1"
      step="1"
      :disabled="isBatchMixing"
      @input="emit('update:batchGenerateCount', readNumber($event))"
    />
  </label>
  <button
    class="primary-button primary-button--full"
    type="button"
    :disabled="isBatchMixing"
    @click="emit('generateBatchMixes')"
  >
    {{ isBatchMixing ? "正在批量生成..." : "批量生成混剪" }}
  </button>
</template>
