<script setup lang="ts">
import type { AiRemixPlannedShot, AiRemixSegment } from "../types";

defineProps<{
  preparedSegments: AiRemixSegment[];
  plannedShots: AiRemixPlannedShot[];
  isPreparing: boolean;
  preparationError: string | null;
  isPlanning: boolean;
  planningProgressText: string | null;
  isGenerating: boolean;
  ttsVideoEnabled: boolean;
  generationProgressText: string | null;
  planError: string | null;
  generateError: string | null;
}>();

defineEmits<{
  plan: [];
  move: [index: number, direction: -1 | 1];
  remove: [index: number];
  replace: [index: number, segmentId: string];
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
        <p class="panel__label">AI 分镜匹配</p>
        <h3 id="ai-remix-title">让每句话匹配合适画面</h3>
      </div>
      <span class="ai-remix-planner__count">可用片段 {{ preparedSegments.length }}</span>
    </div>

    <template v-if="preparedSegments.length === 0">
      <div class="ai-remix-state" role="status">
        <strong>{{ isPreparing ? "正在准备片段信息" : "等待片段" }}</strong>
        <span>{{ isPreparing ? "正在读取时长和预览图，请稍候。" : "请先完成视频切片，再使用 AI 分镜匹配。" }}</span>
      </div>
      <p v-if="preparationError" class="workflow-error" role="alert">{{ preparationError }}</p>
    </template>

    <template v-else>
      <label class="ai-remix-script-field" for="ai-remix-script">
        <span>成片文案（当前仅用于匹配画面）</span>
        <textarea
          id="ai-remix-script"
          v-model="script"
          rows="5"
          maxlength="4000"
          placeholder="例如：这款清洁工具解决了水槽难清理的问题。先展示使用前的污渍，再展示清洁过程，最后展示干净效果。"
          :disabled="isPlanning || isGenerating"
        ></textarea>
        <small>{{ script.length }} / 4000 字</small>
      </label>

      <p class="ai-remix-audio-note" :class="{ 'ai-remix-audio-note--enabled': ttsVideoEnabled }">
        {{ ttsVideoEnabled
          ? "AI配音已开启：将逐句生成语音并静音原片段声音；长配音冻结尾帧，短配音裁短画面。"
          : "当前使用原片段声音。需要文案配音时，请打开右侧“AI配音”并开启逐句配音。" }}
      </p>

      <button
        class="primary-button ai-remix-planner__plan-button"
        type="button"
        :disabled="isPlanning || isGenerating || preparedSegments.length < 2 || script.trim().length === 0"
        @click="$emit('plan')"
      >
        {{ isPlanning ? planningProgressText ?? "AI 正在准备分镜..." : plannedShots.length > 0 ? "重新生成分镜" : "让 AI 生成分镜" }}
      </button>

      <p v-if="isPlanning && planningProgressText" class="ai-remix-progress" role="status">
        {{ planningProgressText }}
      </p>

      <p v-if="preparedSegments.length < 2" class="workflow-error" role="alert">
        AI 智能混剪至少需要 2 个片段，请调整切片秒数后重新切片。
      </p>
      <p v-if="planError" class="workflow-error" role="alert">{{ planError }}</p>

      <div v-if="plannedShots.length > 0" class="ai-remix-plan-result">
        <div class="ai-remix-plan-result__heading">
          <strong>逐句分镜</strong>
          <span>可换备选画面、调整顺序或删除；不会删除磁盘文件。</span>
        </div>

        <ol class="ai-remix-shot-list" aria-label="AI 规划后的逐句分镜">
          <li v-for="(shot, index) in plannedShots" :key="shot.shotId" class="ai-remix-shot">
            <div class="ai-remix-shot__topline">
              <span class="ai-remix-segment__order" aria-hidden="true">{{ index + 1 }}</span>
              <p>{{ shot.text }}</p>
              <span class="ai-remix-segment__actions">
                <button
                  type="button"
                  class="panel-toggle"
                  :disabled="index === 0 || isGenerating"
                  :aria-label="`将第 ${index + 1} 个分镜上移`"
                  @click="$emit('move', index, -1)"
                >上移</button>
                <button
                  type="button"
                  class="panel-toggle"
                  :disabled="index === plannedShots.length - 1 || isGenerating"
                  :aria-label="`将第 ${index + 1} 个分镜下移`"
                  @click="$emit('move', index, 1)"
                >下移</button>
                <button
                  type="button"
                  class="panel-toggle panel-toggle--danger"
                  :disabled="isGenerating"
                  :aria-label="`删除第 ${index + 1} 个分镜`"
                  @click="$emit('remove', index)"
                >删除</button>
              </span>
            </div>

            <div class="ai-remix-shot__primary">
              <img :src="shot.segment.thumbnailUrl" :alt="`${shot.segment.segmentId} 主画面预览图`" />
              <span class="ai-remix-segment__meta">
                <small>当前画面</small>
                <strong>{{ shot.segment.segmentId }}</strong>
                <small>{{ formatSeconds(shot.segment.durationSeconds) }}</small>
              </span>
            </div>

            <div v-if="shot.alternativeSegments.length > 0" class="ai-remix-alternatives">
              <span>备选画面</span>
              <button
                v-for="alternative in shot.alternativeSegments"
                :key="alternative.segmentId"
                type="button"
                class="ai-remix-alternative"
                :disabled="isGenerating"
                :aria-label="`第 ${index + 1} 个分镜改用 ${alternative.segmentId}`"
                @click="$emit('replace', index, alternative.segmentId)"
              >
                <img :src="alternative.thumbnailUrl" :alt="`${alternative.segmentId} 备选画面预览图`" />
                <span>
                  <strong>{{ alternative.segmentId }}</strong>
                  <small>{{ formatSeconds(alternative.durationSeconds) }}</small>
                </span>
              </button>
            </div>
          </li>
        </ol>

        <p v-if="plannedShots.length < 2" class="workflow-error" role="alert">
          至少保留 2 个分镜才能生成视频。
        </p>
        <p v-if="generateError" class="workflow-error" role="alert">{{ generateError }}</p>
        <button
          class="primary-button ai-remix-planner__generate-button"
          type="button"
          :disabled="isGenerating || isPlanning || plannedShots.length < 2"
          @click="$emit('generate')"
        >
          {{ isGenerating
            ? generationProgressText ?? "正在生成 AI 混剪视频..."
            : ttsVideoEnabled ? "生成带AI配音的视频" : "按当前分镜生成视频" }}
        </button>
      </div>
    </template>
  </section>
</template>
