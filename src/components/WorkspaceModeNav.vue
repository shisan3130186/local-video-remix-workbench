<script setup lang="ts">
import type { WorkspaceMode } from "../types/workbench";

defineProps<{
  activeMode: WorkspaceMode;
}>();

defineEmits<{
  change: [mode: WorkspaceMode];
}>();

const modes: Array<{ key: WorkspaceMode; label: string }> = [
  { key: "ai", label: "AI 成片" },
  { key: "batch", label: "批量混剪" },
  { key: "tools", label: "效率工具" },
];
</script>

<template>
  <nav class="workspace-mode-nav" aria-label="切换工作区">
    <button
      v-for="mode in modes"
      :key="mode.key"
      type="button"
      :class="{ 'workspace-mode-nav__item--active': activeMode === mode.key }"
      :aria-current="activeMode === mode.key ? 'page' : undefined"
      @click="$emit('change', mode.key)"
    >
      <span>{{ mode.label }}</span>
    </button>
  </nav>
</template>
