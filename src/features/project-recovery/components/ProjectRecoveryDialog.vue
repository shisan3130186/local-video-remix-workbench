<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import type { ProjectSnapshotLoadResult } from "../types";

const props = defineProps<{
  result: ProjectSnapshotLoadResult | null;
  loadError: string | null;
  restoreError: string | null;
  isRestoring: boolean;
}>();

const emit = defineEmits<{
  restore: [];
  discard: [];
}>();

const dialogRef = ref<HTMLDialogElement | null>(null);
const restoreButtonRef = ref<HTMLButtonElement | null>(null);
const shouldOpen = computed(() => Boolean(props.result?.snapshot || props.loadError));
const snapshot = computed(() => props.result?.snapshot ?? null);
const missingPaths = computed(() => [
  ...(props.result?.missingFilePaths ?? []),
  ...(props.result?.missingDirectoryPaths ?? []),
]);

const summaryItems = computed(() => {
  if (!snapshot.value) return [];
  return [
    `${snapshot.value.project.materials.importedVideos.length} 个原始视频`,
    `${snapshot.value.project.materials.splitSegmentPaths.length} 个切片`,
    `${snapshot.value.project.ai.plannedShots.length} 个AI分镜`,
    `${snapshot.value.project.ai.script.trim().length} 字文案`,
  ];
});

watch(
  shouldOpen,
  async (open) => {
    const dialog = dialogRef.value;
    if (!dialog) return;

    if (open && !dialog.open) {
      dialog.showModal();
      await nextTick();
      restoreButtonRef.value?.focus();
    } else if (!open && dialog.open) {
      dialog.close();
    }
  },
  { immediate: true, flush: "post" },
);

function formatSavedAt(value: string) {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString("zh-CN", { hour12: false });
}

function formatFileName(path: string) {
  return path.split(/[\\/]/).pop() ?? path;
}
</script>

<template>
  <dialog ref="dialogRef" class="project-recovery-dialog" @cancel.prevent>
    <section class="project-recovery-card" aria-labelledby="project-recovery-title">
      <header>
        <p class="panel__label">项目恢复</p>
        <h2 id="project-recovery-title">
          {{ loadError ? "上次项目记录无法读取" : "发现上次未完成项目" }}
        </h2>
      </header>

      <div v-if="loadError" class="project-recovery-error" role="alert">
        <strong>无法安全恢复</strong>
        <p>{{ loadError }}</p>
      </div>

      <template v-else-if="snapshot">
        <p class="project-recovery-copy">
          软件在 {{ formatSavedAt(snapshot.savedAt) }} 自动保存了工作进度。恢复不会复制素材，也不会读取或保存API密钥。
        </p>
        <p class="project-recovery-discard-note">
          选择“新建空白项目”后不会自动载入本机素材库；素材库仍会保留，可稍后从左侧手动载入，也不会删除电脑中的原视频。
        </p>

        <div class="project-recovery-summary" aria-label="上次项目摘要">
          <span v-for="item in summaryItems" :key="item">{{ item }}</span>
        </div>

        <div v-if="missingPaths.length > 0" class="project-recovery-warning" role="status">
          <strong>{{ missingPaths.length }} 个路径已经失效</strong>
          <p>恢复时会自动跳过失效素材，并关闭引用失效文件的画中画或BGM。</p>
          <ul>
            <li v-for="path in missingPaths.slice(0, 4)" :key="path">
              {{ formatFileName(path) }}
            </li>
          </ul>
          <small v-if="missingPaths.length > 4">另有 {{ missingPaths.length - 4 }} 项未显示</small>
        </div>
      </template>

      <p v-if="restoreError" class="error-text" role="alert">{{ restoreError }}</p>

      <footer class="project-recovery-actions">
        <button type="button" class="secondary-button" :disabled="isRestoring" @click="emit('discard')">
          {{ loadError ? "清除记录并新建" : "新建空白项目" }}
        </button>
        <button
          v-if="!loadError"
          ref="restoreButtonRef"
          type="button"
          class="primary-button"
          :disabled="isRestoring"
          @click="emit('restore')"
        >
          {{ isRestoring ? "正在恢复..." : missingPaths.length > 0 ? "安全恢复可用内容" : "恢复上次项目" }}
        </button>
      </footer>
    </section>
  </dialog>
</template>
