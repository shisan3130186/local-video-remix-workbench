<script setup lang="ts">
import type { TaskSnapshot } from "../features/task-center";

defineProps<{
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
      <span :style="{ width: `${activeTask ? Math.min(100, Math.max(0, activeTask.progressPercent)) : 0}%` }"></span>
    </div>

    <div class="task-control-main">
      <div class="task-control-summary">
        <span class="task-state-dot" :class="{ 'task-state-dot--active': isProcessing }"></span>
        <span>
          <strong>{{ activeTask && isProcessing ? activeTask.label : activeTask?.status === "cancelled" ? "任务已取消" : activeTask?.status === "failed" ? "任务失败" : "准备生成视频" }}</strong>
          <small v-if="activeTask && isProcessing">
            {{ activeTask.stage }} · {{ Math.round(activeTask.progressPercent) }}%
          </small>
          <small v-else-if="tempCleanupFeedback" class="task-cleanup-feedback" role="status">
            {{ tempCleanupFeedback }}
          </small>
          <small v-else>{{ outputDirectory ? `输出到：${outputDirectory}` : "请先选择输出目录" }}</small>
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
        <button class="panel-toggle" type="button" :disabled="isProcessing || isCleaningTempFiles" @click="$emit('cleanupTempFiles')">
          {{ isCleaningTempFiles ? "清理中..." : "清理临时文件" }}
        </button>
      </div>

      <button
        v-if="isProcessing && activeTask"
        class="ghost-button task-cancel-action"
        type="button"
        :disabled="activeTask?.status === 'cancelling'"
        @click="$emit('cancelTask')"
      >
        {{ activeTask?.status === "cancelling" ? "正在取消..." : "取消任务" }}
      </button>

      <button
        v-else-if="canRetryFailures"
        class="ghost-button task-retry-action"
        type="button"
        @click="$emit('retryFailures')"
      >
        重试失败项
      </button>

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
