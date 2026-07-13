<script setup lang="ts">
defineProps<{
  isAdvancedMode: boolean;
  totalVideos: number;
  completedCount: number;
  failedCount: number;
  outputDirectory: string | null;
  isProcessing: boolean;
  primaryActionLabel: string;
  primaryActionDisabled: boolean;
}>();

defineEmits<{
  startProcessing: [];
  openDrawer: [drawer: "logs" | "exports" | "batch"];
}>();
</script>

<template>
  <footer class="task-control-bar" aria-label="任务处理区">
    <div class="task-progress" aria-hidden="true">
      <span :style="{ width: `${totalVideos > 0 ? Math.min(100, Math.round((completedCount / totalVideos) * 100)) : 0}%` }"></span>
    </div>

    <div class="task-control-main">
      <div class="task-control-summary">
        <span class="task-state-dot" :class="{ 'task-state-dot--active': isProcessing }"></span>
        <span>
          <strong>{{ isProcessing ? "正在处理任务" : "准备生成视频" }}</strong>
          <small>{{ outputDirectory ? `输出到：${outputDirectory}` : "请先选择输出目录" }}</small>
        </span>
      </div>

      <div class="task-counts" aria-label="任务统计">
        <span>素材 <strong>{{ totalVideos }}</strong></span>
        <span>完成 <strong>{{ completedCount }}</strong></span>
        <span v-if="failedCount > 0">失败 <strong>{{ failedCount }}</strong></span>
      </div>

      <div class="task-quick-actions">
        <button class="panel-toggle" type="button" @click="$emit('openDrawer', 'logs')">日志</button>
        <button class="panel-toggle" type="button" @click="$emit('openDrawer', 'exports')">结果</button>
        <button v-if="isAdvancedMode" class="panel-toggle" type="button" @click="$emit('openDrawer', 'batch')">批量</button>
      </div>

      <button
        class="primary-button task-primary-action"
        type="button"
        :disabled="primaryActionDisabled"
        @click="$emit('startProcessing')"
      >
        {{ isProcessing ? "处理中..." : primaryActionLabel }}
      </button>
    </div>
  </footer>
</template>
