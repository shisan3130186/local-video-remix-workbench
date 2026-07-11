<script setup lang="ts">
import type { AiRemixSegment } from "../services/aiRemixService";

defineProps<{
  preparedSegments: AiRemixSegment[];
  orderedSegments: AiRemixSegment[];
  isPreparing: boolean;
  preparationError: string | null;
  isPlanning: boolean;
  isGenerating: boolean;
  planError: string | null;
  generateError: string | null;
}>();

defineEmits<{
  plan: [];
  move: [index: number, direction: -1 | 1];
  remove: [index: number];
  generate: [];
}>();

const script = defineModel<string>("script", { required: true });

function formatSeconds(value: number) {
  return `${value.toFixed(2)} 秒`;
}
</script>

<template>
  <section class="ai-remix-planner" aria-labelledby="ai-remix-title">
    <div class="ai-remix-planner__header">
      <div>
        <p class="panel__label">AI 智能排序</p>
        <h3 id="ai-remix-title">按文案规划片段顺序</h3>
      </div>
      <span class="ai-remix-planner__count">可用片段 {{ preparedSegments.length }}</span>
    </div>

    <template v-if="preparedSegments.length === 0">
      <div class="ai-remix-state" role="status">
        <strong>{{ isPreparing ? "正在准备片段信息" : "等待片段" }}</strong>
        <span>{{ isPreparing ? "正在读取时长和预览图，请稍候。" : "请先完成视频切片，再使用 AI 排序。" }}</span>
      </div>
      <p v-if="preparationError" class="workflow-error" role="alert">{{ preparationError }}</p>
    </template>

    <template v-else>
      <label class="ai-remix-script-field" for="ai-remix-script">
        <span>成片文案</span>
        <textarea
          id="ai-remix-script"
          v-model="script"
          rows="4"
          maxlength="4000"
          placeholder="例如：先用效果对比吸引注意，再展示产品细节和使用过程，最后给出购买引导。"
          :disabled="isPlanning || isGenerating"
        ></textarea>
        <small>{{ script.length }} / 4000 字</small>
      </label>

      <button
        class="primary-button ai-remix-planner__plan-button"
        type="button"
        :disabled="isPlanning || isGenerating || preparedSegments.length < 2 || script.trim().length === 0"
        @click="$emit('plan')"
      >
        {{ isPlanning ? "AI 正在分析片段..." : orderedSegments.length > 0 ? "重新生成排序" : "让 AI 规划顺序" }}
      </button>

      <p v-if="preparedSegments.length < 2" class="workflow-error" role="alert">
        AI 智能混剪至少需要 2 个片段，请调整切片秒数后重新切片。
      </p>

      <p v-if="planError" class="workflow-error" role="alert">{{ planError }}</p>

      <div v-if="orderedSegments.length > 0" class="ai-remix-plan-result">
        <div class="ai-remix-plan-result__heading">
          <strong>成片顺序</strong>
          <span>可上移、下移或删除，原始切片文件不会被删除。</span>
        </div>

        <ol class="ai-remix-segment-list" aria-label="AI 规划后的片段顺序">
          <li v-for="(segment, index) in orderedSegments" :key="segment.segmentId" class="ai-remix-segment">
            <span class="ai-remix-segment__order" aria-hidden="true">{{ index + 1 }}</span>
            <img :src="segment.thumbnailUrl" :alt="`${segment.segmentId} 预览图`" />
            <span class="ai-remix-segment__meta">
              <strong>{{ segment.segmentId }}</strong>
              <small>{{ formatSeconds(segment.durationSeconds) }}</small>
            </span>
            <span class="ai-remix-segment__actions">
              <button
                type="button"
                class="panel-toggle"
                :disabled="index === 0 || isGenerating"
                :aria-label="`将 ${segment.segmentId} 上移`"
                @click="$emit('move', index, -1)"
              >上移</button>
              <button
                type="button"
                class="panel-toggle"
                :disabled="index === orderedSegments.length - 1 || isGenerating"
                :aria-label="`将 ${segment.segmentId} 下移`"
                @click="$emit('move', index, 1)"
              >下移</button>
              <button
                type="button"
                class="panel-toggle panel-toggle--danger"
                :disabled="isGenerating"
                :aria-label="`从成片计划中删除 ${segment.segmentId}`"
                @click="$emit('remove', index)"
              >删除</button>
            </span>
          </li>
        </ol>

        <p v-if="orderedSegments.length < 2" class="workflow-error" role="alert">
          至少保留 2 个片段才能生成视频。
        </p>
        <p v-if="generateError" class="workflow-error" role="alert">{{ generateError }}</p>
        <button
          class="primary-button ai-remix-planner__generate-button"
          type="button"
          :disabled="isGenerating || isPlanning || orderedSegments.length < 2"
          @click="$emit('generate')"
        >
          {{ isGenerating ? "正在生成 AI 混剪视频..." : "生成 AI 混剪视频" }}
        </button>
      </div>
    </template>
  </section>
</template>
