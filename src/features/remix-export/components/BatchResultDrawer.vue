<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";

defineProps<{
  open: boolean;
  batchMixResults: string[];
  generatedResults: string[];
  generationOutputDirectory: string | null;
  formatFileName: (path: string) => string;
}>();

defineEmits<{
  close: [];
  openLocation: [];
  exportJianyingDraft: [];
}>();
</script>

<template>
  <aside v-if="open" class="info-drawer" aria-label="批量生成结果">
    <header class="drawer-header">
      <div>
        <p class="panel__label">批量生成结果</p>
        <h2>本轮结果</h2>
      </div>
      <button class="panel-toggle" type="button" @click="$emit('close')">关闭</button>
    </header>
    <div class="drawer-body">
      <div class="batch-result-drawer__actions">
        <button type="button" :disabled="!generationOutputDirectory" @click="$emit('openLocation')">打开本次任务目录</button>
        <button type="button" :disabled="generatedResults.length === 0 || !generationOutputDirectory" @click="$emit('exportJianyingDraft')">导出剪映草稿</button>
      </div>
      <ol v-if="batchMixResults.length + generatedResults.length > 0" class="compact-list">
        <li v-for="resultPath in [...batchMixResults, ...generatedResults]" :key="resultPath" class="batch-result-item">
          <video :src="convertFileSrc(resultPath)" muted playsinline preload="metadata"></video>
          <span><strong>{{ formatFileName(resultPath) }}</strong><small>{{ resultPath }}</small></span>
        </li>
      </ol>
      <p v-else class="empty-text">暂无批量生成结果。</p>
    </div>
  </aside>
</template>
