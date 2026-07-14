<script setup lang="ts">
defineProps<{
  outputDirectory: string | null;
  outputDirectoryError: string | null;
  isExporting: boolean;
}>();

const emit = defineEmits<{
  selectOutputDirectory: [];
  openOutputDirectory: [];
  exportSelectedVideo: [];
}>();
</script>

<template>
  <p v-if="outputDirectory" class="output-path">{{ outputDirectory }}</p>
  <p v-else class="empty-text">请选择导出结果保存位置。</p>
  <button class="ghost-button ghost-button--full" type="button" @click="emit('selectOutputDirectory')">
    选择输出目录
  </button>
  <button
    class="ghost-button ghost-button--full"
    type="button"
    :disabled="!outputDirectory"
    @click="emit('openOutputDirectory')"
  >
    打开输出目录
  </button>
  <button
    class="primary-button primary-button--full"
    type="button"
    :disabled="isExporting"
    @click="emit('exportSelectedVideo')"
  >
    {{ isExporting ? "正在导出..." : "导出当前视频" }}
  </button>
  <p v-if="outputDirectoryError" class="error-text">{{ outputDirectoryError }}</p>
</template>
