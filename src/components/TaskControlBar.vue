<script setup lang="ts">
defineProps<{
  totalVideos: number;
  completedCount: number;
  failedCount: number;
  outputDirectory: string | null;
  isProcessing: boolean;
}>();

defineEmits<{
  startProcessing: [];
  openDrawer: [drawer: "logs" | "exports" | "batch"];
}>();
</script>

<template>
  <footer class="task-control-bar" aria-label="任务处理区">
    <div class="task-progress">
      <span :style="{ width: `${totalVideos > 0 ? Math.min(100, Math.round((completedCount / totalVideos) * 100)) : 0}%` }"></span>
    </div>
    <div class="task-control-grid">
      <label class="mini-field">
        <span>输出格式</span>
        <select><option>MP4</option></select>
      </label>
      <label class="mini-field">
        <span>分辨率</span>
        <select>
          <option>保持原分辨率</option>
          <option>1080p</option>
          <option>720p</option>
        </select>
      </label>
      <label class="mini-field">
        <span>帧率</span>
        <select>
          <option>原帧率</option>
          <option>30 FPS</option>
          <option>60 FPS</option>
        </select>
      </label>
      <label class="mini-field">
        <span>命名规则</span>
        <select>
          <option>前缀序号</option>
          <option>时间戳</option>
        </select>
      </label>
      <label class="mini-field">
        <span>线程</span>
        <select>
          <option>1 线程</option>
          <option>2 线程</option>
        </select>
      </label>
      <button
        class="primary-button task-start-button"
        type="button"
        :disabled="isProcessing"
        @click="$emit('startProcessing')"
      >
        {{ isProcessing ? "处理中..." : "开始处理" }}
      </button>
    </div>

    <div class="task-control-status">
      <span>总视频数：{{ totalVideos }}</span>
      <span>已完成：{{ completedCount }}</span>
      <span>失败：{{ failedCount }}</span>
      <span class="task-output-path">输出目录：{{ outputDirectory ?? "未选择" }}</span>
      <button class="panel-toggle" type="button" @click="$emit('openDrawer', 'logs')">日志</button>
      <button class="panel-toggle" type="button" @click="$emit('openDrawer', 'exports')">导出结果</button>
      <button class="panel-toggle" type="button" @click="$emit('openDrawer', 'batch')">批量结果</button>
    </div>
  </footer>
</template>
