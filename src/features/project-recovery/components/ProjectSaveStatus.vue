<script setup lang="ts">
import type { ProjectAutoSaveStatus } from "../types";

defineProps<{
  status: ProjectAutoSaveStatus;
  text: string;
  error: string | null;
}>();
</script>

<template>
  <span
    class="project-save-status"
    :class="`project-save-status--${status}`"
    :title="error ?? text"
    role="status"
    aria-live="polite"
  >
    <i aria-hidden="true"></i>
    {{ text }}
  </span>
</template>
