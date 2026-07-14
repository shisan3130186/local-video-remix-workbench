<script setup lang="ts">
import { readChecked } from "./inputHelpers";

defineProps<{
  smoothRemixEnabled: boolean;
}>();

const emit = defineEmits<{
  "update:smoothRemixEnabled": [value: boolean];
}>();
</script>

<template>
  <label class="option-toggle">
    <input
      :checked="smoothRemixEnabled"
      type="checkbox"
      @change="emit('update:smoothRemixEnabled', readChecked($event))"
    />
    <span>平滑混剪</span>
  </label>
  <p class="empty-text">开启后，拼接和批量生成会应用 0.2 秒淡入淡出，并过滤小于 1.5 秒的片段。</p>
</template>
