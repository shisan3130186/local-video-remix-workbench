<script setup lang="ts">
import type { ExportResultItem } from "../types/workbench";

defineProps<{
  open: boolean;
  exportResultItems: ExportResultItem[];
  fileManagerError: string | null;
  formatFileName: (path: string) => string;
}>();

defineEmits<{
  close: [];
  openResultLocation: [path: string];
}>();
</script>

<template>
  <aside v-if="open" class="info-drawer" aria-label="导出结果">
    <header class="drawer-header">
      <div>
        <p class="panel__label">导出结果</p>
        <h2>结果列表</h2>
      </div>
      <button class="panel-toggle" type="button" @click="$emit('close')">关闭</button>
    </header>
    <div class="drawer-body">
      <ol v-if="exportResultItems.length > 0" class="compact-list result-list">
        <li v-for="item in exportResultItems" :key="item.id">
          <span>{{ item.type }} · {{ formatFileName(item.path) }}</span>
          <button class="ghost-button result-action" type="button" @click="$emit('openResultLocation', item.path)">
            打开位置
          </button>
          <small>{{ item.time }} · {{ item.path }}</small>
        </li>
      </ol>
      <p v-if="fileManagerError" class="error-text">{{ fileManagerError }}</p>
      <p v-if="exportResultItems.length === 0" class="empty-text">暂无导出结果。</p>
    </div>
  </aside>
</template>
