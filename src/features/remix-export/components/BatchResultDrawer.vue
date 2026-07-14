<script setup lang="ts">
defineProps<{
  open: boolean;
  batchMixResults: string[];
  formatFileName: (path: string) => string;
}>();

defineEmits<{
  close: [];
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
      <ol v-if="batchMixResults.length > 0" class="compact-list">
        <li v-for="resultPath in batchMixResults" :key="resultPath">
          <span>{{ formatFileName(resultPath) }}</span>
          <small>{{ resultPath }}</small>
        </li>
      </ol>
      <p v-else class="empty-text">暂无批量生成结果。</p>
    </div>
  </aside>
</template>
