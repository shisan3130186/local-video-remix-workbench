<script setup lang="ts">
import type { GroupedTaskLogEntry } from "../types/workbench";

defineProps<{
  open: boolean;
  taskLogs: GroupedTaskLogEntry[];
}>();

defineEmits<{
  close: [];
}>();
</script>

<template>
  <aside v-if="open" class="info-drawer workspace-drawer info-drawer--wide" aria-label="任务日志">
    <header class="drawer-header">
      <div>
        <p class="panel__label">任务日志</p>
        <h2>当前处理记录</h2>
      </div>
      <button class="panel-toggle" type="button" @click="$emit('close')">关闭</button>
    </header>
    <div class="drawer-body">
      <p v-if="taskLogs.length === 0" class="empty-text">
        开始导出、切片、拼接或批量生成后，这里会显示任务过程。
      </p>
      <ol v-else class="log-list">
        <li
          v-for="log in taskLogs"
          :key="`${log.group}-${log.id}`"
          class="log-item"
          :class="`log-item--${log.level}`"
        >
          <span class="log-item__time">{{ log.time }}</span>
          <span class="log-item__group">{{ log.group }}</span>
          <span class="log-item__message">{{ log.message }}</span>
        </li>
      </ol>
    </div>
  </aside>
</template>
