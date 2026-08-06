<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";

const props = defineProps<{
  open: boolean;
  batchMixResults: string[];
  generatedResults: string[];
  generationOutputDirectory: string | null;
  formatFileName: (path: string) => string;
}>();

const allResults = computed(() => [...props.batchMixResults, ...props.generatedResults]);
const selectedResultPath = ref<string | null>(null);

watch(
  allResults,
  (results) => {
    if (!selectedResultPath.value || !results.includes(selectedResultPath.value)) {
      selectedResultPath.value = results[0] ?? null;
    }
  },
  { immediate: true },
);

const selectedResultUrl = computed(() =>
  selectedResultPath.value ? convertFileSrc(selectedResultPath.value) : null,
);

defineEmits<{
  close: [];
  openLocation: [];
  exportJianyingDraft: [];
}>();
</script>

<template>
  <aside v-if="open" class="info-drawer workspace-drawer workspace-drawer--results" aria-label="批量生成结果">
    <header class="drawer-header">
      <div>
        <p class="panel__label">批量生成结果</p>
        <h2>本轮结果</h2>
      </div>
      <button class="panel-toggle" type="button" @click="$emit('close')">关闭</button>
    </header>
    <div class="drawer-body">
      <div class="batch-result-drawer__actions">
        <button type="button" :disabled="!props.generationOutputDirectory" @click="$emit('openLocation')">打开本次任务目录</button>
        <button type="button" :disabled="props.generatedResults.length === 0 || !props.generationOutputDirectory" @click="$emit('exportJianyingDraft')">导出剪映草稿</button>
      </div>
      <section v-if="selectedResultPath && selectedResultUrl" class="batch-result-preview" aria-label="当前成片预览">
        <video :src="selectedResultUrl" controls playsinline preload="metadata"></video>
        <strong>{{ props.formatFileName(selectedResultPath) }}</strong>
      </section>
      <ol v-if="allResults.length > 0" class="compact-list batch-result-list" aria-label="成片结果列表">
        <li v-for="resultPath in allResults" :key="resultPath">
          <button
            class="batch-result-item"
            :class="{ 'is-active': resultPath === selectedResultPath }"
            type="button"
            @click="selectedResultPath = resultPath"
          >
            <video :src="convertFileSrc(resultPath)" muted playsinline preload="metadata"></video>
            <span><strong>{{ props.formatFileName(resultPath) }}</strong><small>{{ resultPath }}</small></span>
          </button>
        </li>
      </ol>
      <p v-else class="empty-text">暂无批量生成结果。</p>
    </div>
  </aside>
</template>
