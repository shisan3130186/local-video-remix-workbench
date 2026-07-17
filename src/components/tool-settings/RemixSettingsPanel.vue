<script setup lang="ts">
import { readNumber } from "./inputHelpers";
import type { SceneSensitivity, SplitMode } from "../../features/materials";

defineProps<{
  segmentDurationSeconds: number;
  splitMode: SplitMode;
  sceneSensitivity: SceneSensitivity;
  minimumSegmentSeconds: number;
  maximumSegmentSeconds: number;
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
  "update:splitMode": [value: SplitMode];
  "update:sceneSensitivity": [value: SceneSensitivity];
  "update:minimumSegmentSeconds": [value: number];
  "update:maximumSegmentSeconds": [value: number];
  "update:randomPickCount": [value: number];
  "update:batchGenerateCount": [value: number];
}>();

const sensitivityOptions = [
  { value: "stable", label: "稳健", hint: "少切" },
  { value: "balanced", label: "均衡", hint: "推荐" },
  { value: "sensitive", label: "灵敏", hint: "多切" },
] as const;
</script>

<template>
  <section class="smart-split-settings" aria-labelledby="split-mode-title">
    <div class="smart-split-settings__heading">
      <div>
        <h3 id="split-mode-title">切片方式</h3>
        <p>智能模式会优先在画面变化处切开，固定模式按相同秒数切开。</p>
      </div>
      <span v-if="splitMode === 'scene'" class="recommended-badge">推荐</span>
    </div>

    <div class="split-mode-options" role="radiogroup" aria-label="切片方式">
      <label :class="['split-mode-option', { 'split-mode-option--active': splitMode === 'scene' }]">
        <input
          type="radio"
          name="split-mode"
          value="scene"
          :checked="splitMode === 'scene'"
          :disabled="isSplitting"
          @change="emit('update:splitMode', 'scene')"
        />
        <span><strong>智能切片</strong><small>识别转场与动作变化</small></span>
      </label>
      <label :class="['split-mode-option', { 'split-mode-option--active': splitMode === 'duration' }]">
        <input
          type="radio"
          name="split-mode"
          value="duration"
          :checked="splitMode === 'duration'"
          :disabled="isSplitting"
          @change="emit('update:splitMode', 'duration')"
        />
        <span><strong>固定时长</strong><small>稳定地按秒数平均切片</small></span>
      </label>
    </div>

    <template v-if="splitMode === 'scene'">
      <fieldset class="sensitivity-fieldset" :disabled="isSplitting">
        <legend>画面变化灵敏度</legend>
        <div class="sensitivity-options">
          <label v-for="option in sensitivityOptions" :key="option.value">
            <input
              type="radio"
              name="scene-sensitivity"
              :value="option.value"
              :checked="sceneSensitivity === option.value"
              @change="emit('update:sceneSensitivity', option.value)"
            />
            <span><strong>{{ option.label }}</strong><small>{{ option.hint }}</small></span>
          </label>
        </div>
      </fieldset>

      <div class="smart-split-duration-grid">
        <label class="field">
          <span>最短片段</span>
          <input
            :value="minimumSegmentSeconds"
            type="number"
            min="0.5"
            max="10"
            step="0.5"
            :disabled="isSplitting"
            @input="emit('update:minimumSegmentSeconds', readNumber($event))"
          />
          <small>过滤闪屏和过短镜头，默认 2 秒。</small>
        </label>
        <label class="field">
          <span>最长片段</span>
          <input
            :value="maximumSegmentSeconds"
            type="number"
            min="2"
            max="60"
            step="1"
            :disabled="isSplitting"
            @input="emit('update:maximumSegmentSeconds', readNumber($event))"
          />
          <small>长镜头没有转场时自动补切，默认 10 秒。</small>
        </label>
      </div>
      <p class="smart-split-note">完成后会为每个片段抽取开头、中间、结尾画面，供 AI 综合理解。</p>
    </template>

    <label v-else class="field">
      <span>每段时长</span>
      <input
        :value="segmentDurationSeconds"
        type="number"
        min="1"
        step="1"
        :disabled="isSplitting"
        @input="emit('update:segmentDurationSeconds', readNumber($event))"
      />
    </label>
  </section>
  <button
    class="primary-button primary-button--full"
    type="button"
    :disabled="isSplitting"
    @click="emit('splitSelectedVideo')"
  >
    {{ isSplitting ? "正在切片..." : splitMode === "scene" ? "开始智能切片" : "开始固定时长切片" }}
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
