<script setup lang="ts">
import { computed } from "vue";
import type { TaskSnapshot } from "../features/task-center";

const props = defineProps<{
  isAdvancedMode: boolean;
  totalVideos: number;
  completedCount: number;
  failedCount: number;
  outputDirectory: string | null;
  isProcessing: boolean;
  primaryActionLabel: string;
  primaryActionDisabled: boolean;
  activeTask: TaskSnapshot | null;
  canRetryFailures: boolean;
  isCleaningTempFiles: boolean;
  tempCleanupFeedback: string | null;
}>();

const taskTitle = computed(() => {
  if (!props.activeTask) return "准备生成视频";
  if (props.isProcessing) return props.activeTask.label;
  if (props.activeTask.status === "completed") return `${props.activeTask.label}已完成`;
  if (props.activeTask.status === "failed") return `${props.activeTask.label}失败`;
  if (props.activeTask.status === "cancelled") return `${props.activeTask.label}已取消`;
  return "准备生成视频";
});

const taskDetail = computed(() => {
  if (props.activeTask && !props.isProcessing) {
    return props.activeTask.message || `任务进度 ${Math.round(props.activeTask.progressPercent)}%`;
  }
  return props.outputDirectory ? `输出到：${props.outputDirectory}` : "请先选择输出目录";
});

defineEmits<{
  startProcessing: [];
  cancelTask: [];
  retryFailures: [];
  cleanupTempFiles: [];
  openDrawer: [drawer: "logs" | "exports" | "batch"];
}>();
</script>

<template>
  <footer class="task-control-bar" aria-label="任务处理区">
    <div class="task-progress" aria-hidden="true">
      <span :style="{ width: `${props.activeTask ? Math.min(100, Math.max(0, props.activeTask.progressPercent)) : 0}%` }"></span>
    </div>

    <div class="task-control-main">
      <div class="task-control-summary">
        <span class="task-state-dot" :class="{ 'task-state-dot--active': props.isProcessing, 'task-state-dot--done': props.activeTask?.status === 'completed', 'task-state-dot--error': props.activeTask?.status === 'failed' }"></span>
        <span>
          <strong>{{ taskTitle }}</strong>
          <small v-if="props.activeTask && props.isProcessing">
            {{ props.activeTask.stage }} · {{ Math.round(props.activeTask.progressPercent) }}%
          </small>
          <small v-else-if="props.tempCleanupFeedback" class="task-cleanup-feedback" role="status">
            {{ props.tempCleanupFeedback }}
          </small>
          <small v-else>{{ taskDetail }}</small>
        </span>
      </div>

      <div class="task-counts" aria-label="任务统计">
        <span>素材 <strong>{{ props.totalVideos }}</strong></span>
        <span>完成 <strong>{{ props.completedCount }}</strong></span>
        <span v-if="props.failedCount > 0">失败 <strong>{{ props.failedCount }}</strong></span>
      </div>

      <div class="task-quick-actions">
        <button class="panel-toggle" type="button" @click="$emit('openDrawer', 'logs')">日志</button>
        <button class="panel-toggle" type="button" @click="$emit('openDrawer', 'exports')">结果</button>
        <button v-if="props.isAdvancedMode" class="panel-toggle" type="button" @click="$emit('openDrawer', 'batch')">批量</button>
        <button class="panel-toggle" type="button" :disabled="props.isProcessing || props.isCleaningTempFiles" @click="$emit('cleanupTempFiles')">
          {{ props.isCleaningTempFiles ? "清理中..." : "清理临时文件" }}
        </button>
      </div>

      <button
        v-if="props.isProcessing && props.activeTask"
        class="ghost-button task-cancel-action"
        type="button"
        :disabled="props.activeTask?.status === 'cancelling'"
        @click="$emit('cancelTask')"
      >
        {{ props.activeTask?.status === "cancelling" ? "正在取消..." : "取消任务" }}
      </button>

      <button
        v-else-if="props.canRetryFailures"
        class="ghost-button task-retry-action"
        type="button"
        @click="$emit('retryFailures')"
      >
        重试失败项
      </button>

      <button
        class="primary-button task-primary-action"
        type="button"
        :disabled="props.primaryActionDisabled"
        @click="$emit('startProcessing')"
      >
        {{ props.isProcessing ? "处理中..." : props.primaryActionLabel }}
      </button>
    </div>
  </footer>
</template>
