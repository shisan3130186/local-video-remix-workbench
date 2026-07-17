<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type {
  SegmentCategory,
  SegmentCategoryOption,
} from "../../../services/videoMixService";
import type { AiRemixContentAnalysis, AiRemixSegment } from "../../ai-remix/types";

const props = defineProps<{
  selectedSegment: AiRemixSegment | null;
  selectedCategory: SegmentCategory | "";
  categoryOptions: SegmentCategoryOption[];
  analyzedCount: number;
  totalCount: number;
  isAnalyzing: boolean;
  progressText: string | null;
  error: string | null;
}>();

const emit = defineEmits<{
  analyzeAll: [];
  updateAnalysis: [segmentPath: string, analysis: AiRemixContentAnalysis];
  updateCategory: [segmentPath: string, category: SegmentCategory | ""];
}>();

const theme = ref("");
const sellingPointsText = ref("");
const action = ref("");
const tagsText = ref("");

watch(
  () => props.selectedSegment,
  (segment) => {
    theme.value = segment?.contentAnalysis?.theme ?? "";
    sellingPointsText.value = segment?.contentAnalysis?.sellingPoints.join("\n") ?? "";
    action.value = segment?.contentAnalysis?.action ?? "";
    tagsText.value = segment?.contentAnalysis?.tags.join("、") ?? "";
  },
  { immediate: true, deep: true },
);

const canSave = computed(
  () =>
    Boolean(props.selectedSegment?.contentAnalysis) &&
    Boolean(theme.value.trim()) &&
    Boolean(action.value.trim()) &&
    splitTags(tagsText.value).length > 0,
);

function saveAnalysis() {
  if (!props.selectedSegment || !canSave.value) return;
  emit("updateAnalysis", props.selectedSegment.path, {
    theme: theme.value.trim(),
    sellingPoints: splitLines(sellingPointsText.value).slice(0, 3),
    action: action.value.trim(),
    tags: splitTags(tagsText.value).slice(0, 5),
  });
}

function updateCategory(event: Event) {
  if (!props.selectedSegment) return;
  emit(
    "updateCategory",
    props.selectedSegment.path,
    (event.target as HTMLSelectElement).value as SegmentCategory | "",
  );
}

function splitLines(value: string) {
  return unique(value.split(/\r?\n|[，,；;]/));
}

function splitTags(value: string) {
  return unique(value.split(/[\s，,、；;]+/));
}

function unique(values: string[]) {
  return Array.from(new Set(values.map((value) => value.trim()).filter(Boolean)));
}
</script>

<template>
  <section class="segment-analysis" aria-labelledby="segment-analysis-title">
    <div class="segment-analysis__heading">
      <div>
        <strong id="segment-analysis-title">AI 内容提炼</strong>
        <small>已完成 {{ analyzedCount }}/{{ totalCount }}</small>
      </div>
      <button
        class="primary-button segment-analysis__run"
        type="button"
        :disabled="totalCount === 0 || isAnalyzing || analyzedCount === totalCount"
        @click="$emit('analyzeAll')"
      >
        {{ isAnalyzing ? "正在提炼..." : analyzedCount === totalCount && totalCount > 0 ? "结果已缓存" : "AI 提炼全部片段" }}
      </button>
    </div>

    <p v-if="progressText" class="segment-analysis__status" role="status">{{ progressText }}</p>
    <p v-if="error" class="error-text" role="alert">{{ error }}</p>

    <div v-if="!selectedSegment" class="segment-analysis__empty">
      选择一个视频片段后，可查看并修改它的提炼结果。
    </div>
    <div v-else-if="!selectedSegment.contentAnalysis" class="segment-analysis__empty">
      <strong>当前片段尚未提炼</strong>
      <p v-if="selectedSegment.description">多帧画面理解已缓存，运行提炼时不会重复上传图片。</p>
      <p v-else>点击上方按钮后，软件会先理解多帧画面，再提炼结构化内容。</p>
    </div>
    <form v-else class="segment-analysis__form" @submit.prevent="saveAnalysis">
      <label>
        <span>镜头类型</span>
        <select :value="selectedCategory" @change="updateCategory">
          <option value="">未分类</option>
          <option v-for="option in categoryOptions" :key="option.key" :value="option.key">
            {{ option.label }}
          </option>
        </select>
      </label>
      <label>
        <span>内容主题</span>
        <input v-model="theme" maxlength="80" autocomplete="off" />
      </label>
      <label>
        <span>可见卖点 <small>每行一个，最多 3 个</small></span>
        <textarea v-model="sellingPointsText" rows="2" maxlength="200"></textarea>
      </label>
      <label>
        <span>主体动作</span>
        <input v-model="action" maxlength="100" autocomplete="off" />
      </label>
      <label>
        <span>内容标签 <small>用逗号分隔，最多 5 个</small></span>
        <input v-model="tagsText" maxlength="140" autocomplete="off" />
      </label>
      <button class="ghost-button" type="submit" :disabled="!canSave">保存人工修改</button>
    </form>
  </section>
</template>
