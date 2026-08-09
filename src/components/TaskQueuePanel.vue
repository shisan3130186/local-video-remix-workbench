<script setup lang="ts">
import { computed } from "vue";
import type { TaskSnapshot, TaskStatus } from "../features/task-center";

defineProps<{
  activeTask: TaskSnapshot | null;
  taskHistory: TaskSnapshot[];
  isTaskRunning: boolean;
  isVisible: boolean;
}>();

const emit = defineEmits<{
  close: [];
  clearHistory: [];
}>();

const statusLabels: Record<TaskStatus, string> = {
  running: "处理中",
  cancelling: "取消中",
  completed: "已完成",
  failed: "失败",
  cancelled: "已取消",
};

const statusTone: Record<TaskStatus, string> = {
  running: "task-queue-panel__chip--running",
  cancelling: "task-queue-panel__chip--cancelling",
  completed: "task-queue-panel__chip--completed",
  failed: "task-queue-panel__chip--failed",
  cancelled: "task-queue-panel__chip--cancelled",
};

const activePercent = computed(() => {
  // placeholder; values are read per render below
  return 0;
});

function activePercentOf(task: TaskSnapshot | null | undefined) {
  if (!task) return 0;
  return Math.max(0, Math.min(100, Math.round(task.progressPercent)));
}

function activeStageOf(task: TaskSnapshot | null | undefined) {
  return task?.stage ?? "准备中";
}

function activeLabelOf(task: TaskSnapshot | null | undefined) {
  return task?.label ?? "暂无任务";
}

function activeMessageOf(task: TaskSnapshot | null | undefined) {
  return task?.message ?? null;
}

function statusLabel(snapshot: TaskSnapshot) {
  return statusLabels[snapshot.status];
}

function statusClass(snapshot: TaskSnapshot) {
  return statusTone[snapshot.status];
}
</script>

<template>
  <Transition name="task-queue-panel-slide">
    <aside
      v-if="isVisible"
      class="task-queue-panel"
      role="complementary"
      aria-label="任务队列面板"
    >
      <header class="task-queue-panel__header">
        <div>
          <h3>任务队列</h3>
          <small>当前进度、最近任务一目了然</small>
        </div>
        <button
          class="task-queue-panel__close"
          type="button"
          aria-label="关闭任务队列面板"
          @click="emit('close')"
        >
          ×
        </button>
      </header>

      <div class="task-queue-panel__body">
        <section
          v-if="activeTask"
          class="task-queue-panel__active"
          :aria-busy="isTaskRunning"
        >
          <div class="task-queue-panel__active-meta">
            <strong class="task-queue-panel__active-label">{{ activeLabelOf(activeTask) }}</strong>
            <span class="task-queue-panel__chip" :class="statusClass(activeTask)">
              {{ statusLabel(activeTask) }}
            </span>
          </div>
          <div
            class="task-queue-panel__progress"
            role="progressbar"
            :aria-valuenow="activePercentOf(activeTask)"
            aria-valuemin="0"
            aria-valuemax="100"
          >
            <span :style="{ width: `${activePercentOf(activeTask)}%` }"></span>
          </div>
          <div class="task-queue-panel__active-stage">
            <span>{{ activeStageOf(activeTask) }}</span>
            <strong>{{ activePercentOf(activeTask) }}%</strong>
          </div>
          <p v-if="activeMessageOf(activeTask)" class="task-queue-panel__active-message" role="status">
            {{ activeMessageOf(activeTask) }}
          </p>
        </section>

        <section v-else class="task-queue-panel__idle">
          <strong>暂无正在进行的任务</strong>
          <span>提交混剪、导出或清理临时文件后，状态会显示在这里。</span>
        </section>

        <section class="task-queue-panel__history">
          <header>
            <h4>最近任务</h4>
            <button
              v-if="taskHistory.length > 0"
              class="task-queue-panel__clear"
              type="button"
              @click="emit('clearHistory')"
            >
              清空
            </button>
          </header>

          <p v-if="taskHistory.length === 0" class="task-queue-panel__history-empty">
            暂无历史任务
          </p>

          <ul v-else class="task-queue-panel__history-list">
            <li
              v-for="snapshot in taskHistory"
              :key="snapshot.taskId"
              class="task-queue-panel__history-item"
            >
              <div class="task-queue-panel__history-row">
                <span class="task-queue-panel__history-label">{{ snapshot.label }}</span>
                <span class="task-queue-panel__chip" :class="statusClass(snapshot)">
                  {{ statusLabel(snapshot) }}
                </span>
              </div>
              <small v-if="snapshot.stage" class="task-queue-panel__history-stage">
                {{ snapshot.stage }}
              </small>
              <p v-if="snapshot.message" class="task-queue-panel__history-message">
                {{ snapshot.message }}
              </p>
            </li>
          </ul>
        </section>
      </div>
    </aside>
  </Transition>
</template>

<style scoped>
.task-queue-panel {
  position: fixed;
  top: 64px;
  right: 0;
  bottom: 0;
  width: 320px;
  max-width: calc(100vw - 32px);
  background: var(--theme-panel, #1a1d23);
  color: var(--theme-text, #e6e9ef);
  border-left: 1px solid var(--theme-border, #2d3033);
  box-shadow: -12px 0 32px var(--theme-shadow, rgba(0, 0, 0, 0.35));
  display: flex;
  flex-direction: column;
  z-index: 40;
  overflow: hidden;
}

.task-queue-panel__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px 10px;
  border-bottom: 1px solid var(--theme-border, #2d3033);
  background: var(--theme-bar, #191b1d);
}

.task-queue-panel__header h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--theme-text, #e6e9ef);
}

.task-queue-panel__header small {
  display: block;
  color: var(--theme-muted, #8d9296);
  font-size: 11px;
  margin-top: 2px;
}

.task-queue-panel__close {
  background: transparent;
  border: 0;
  color: var(--theme-muted, #8d9296);
  font-size: 22px;
  line-height: 1;
  cursor: pointer;
  padding: 0 4px;
}

.task-queue-panel__close:hover {
  color: var(--theme-text, #e6e9ef);
}

.task-queue-panel__body {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  padding: 14px;
  gap: 14px;
  overflow-y: auto;
  background: var(--theme-panel, #1a1d23);
}

.task-queue-panel__active {
  background: var(--theme-panel-soft, #252729);
  border: 1px solid var(--theme-border, #2d3033);
  border-radius: 10px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.task-queue-panel__active-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.task-queue-panel__active-label {
  font-size: 13px;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--theme-text, #e6e9ef);
}

.task-queue-panel__chip {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 999px;
  background: var(--theme-panel-muted, #1b1d1f);
  color: var(--theme-text, #e6e9ef);
  flex-shrink: 0;
}

.task-queue-panel__chip--running {
  background: var(--theme-accent-soft, rgba(255, 255, 255, 0.12));
  color: var(--theme-accent, #d1d1d6);
}

.task-queue-panel__chip--cancelling {
  background: rgba(255, 193, 7, 0.18);
  color: #ffc107;
}

.task-queue-panel__chip--completed {
  background: rgba(72, 199, 116, 0.18);
  color: #48c774;
}

.task-queue-panel__chip--failed {
  background: rgba(241, 70, 104, 0.18);
  color: #f14668;
}

.task-queue-panel__chip--cancelled {
  background: rgba(149, 149, 149, 0.18);
  color: #b5b5b5;
}

.task-queue-panel__progress {
  height: 6px;
  background: var(--theme-panel-muted, #1b1d1f);
  border-radius: 999px;
  overflow: hidden;
}

.task-queue-panel__progress span {
  display: block;
  height: 100%;
  background: linear-gradient(90deg, #8e8e93, #e5e5ea);
  transition: none;
}

.task-queue-panel__active-stage {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 11px;
  color: var(--theme-muted, #8d9296);
}

.task-queue-panel__active-stage strong {
  font-size: 13px;
  color: var(--theme-text, #e6e9ef);
}

.task-queue-panel__active-message {
  margin: 0;
  font-size: 11px;
  color: var(--theme-muted, #8d9296);
  line-height: 1.4;
  max-height: 36px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.task-queue-panel__idle {
  background: var(--theme-panel-soft, #252729);
  border: 1px solid var(--theme-border, #2d3033);
  border-radius: 10px;
  padding: 14px 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.task-queue-panel__idle strong {
  font-size: 13px;
  font-weight: 600;
  color: var(--theme-text, #e6e9ef);
}

.task-queue-panel__idle span {
  font-size: 11px;
  color: var(--theme-muted, #8d9296);
  line-height: 1.5;
}

.task-queue-panel__history header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.task-queue-panel__history h4 {
  margin: 0;
  font-size: 12px;
  font-weight: 600;
  color: var(--theme-text-soft, #b9bdc0);
}

.task-queue-panel__clear {
  background: transparent;
  border: 0;
  color: var(--theme-muted, #8d9296);
  font-size: 11px;
  cursor: pointer;
  padding: 0;
}

.task-queue-panel__clear:hover {
  color: var(--theme-text, #e6e9ef);
}

.task-queue-panel__history-empty {
  margin: 0;
  font-size: 11px;
  color: var(--theme-faint, #62676a);
  text-align: center;
  padding: 12px 0;
}

.task-queue-panel__history-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.task-queue-panel__history-item {
  background: var(--theme-panel-soft, #252729);
  border: 1px solid var(--theme-border, #2d3033);
  border-radius: 8px;
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.task-queue-panel__history-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.task-queue-panel__history-label {
  font-size: 12px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  color: var(--theme-text, #e6e9ef);
}

.task-queue-panel__history-stage {
  font-size: 10px;
  color: var(--theme-muted, #8d9296);
}

.task-queue-panel__history-message {
  margin: 0;
  font-size: 10px;
  color: var(--theme-muted, #8d9296);
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.task-queue-panel-slide-enter-active,
.task-queue-panel-slide-leave-active {
  transition: transform 0.25s ease, opacity 0.25s ease;
}

.task-queue-panel-slide-enter-from,
.task-queue-panel-slide-leave-to {
  transform: translateX(360px);
  opacity: 0;
}
</style>
