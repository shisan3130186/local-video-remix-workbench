<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import "../membership.css";

const props = defineProps<{
  visible: boolean;
  missingAuthorization: boolean;
  missingAiService: boolean;
  missingTtsService: boolean;
}>();

const emit = defineEmits<{
  close: [];
  confirm: [];
}>();

const confirmButton = ref<HTMLButtonElement | null>(null);
const missingItems = computed(() => {
  const items: string[] = [];
  if (props.missingAuthorization) items.push("有效卡密授权");
  if (props.missingAiService) items.push("有效的 AI API Key");
  if (props.missingTtsService) items.push("TTS 服务");
  return items;
});
const promptText = computed(() => {
  const requirements = missingItems.value.join("、");
  return `当前工作台可以继续浏览和配置；开始智能分析、混剪或导出前，需要完成${requirements}。`;
});

watch(() => props.visible, async (visible) => {
  if (!visible) return;
  await nextTick();
  confirmButton.value?.focus();
});
</script>

<template>
  <div
    v-if="visible"
    class="access-requirement-backdrop"
    @click.self="emit('close')"
    @keydown.esc="emit('close')"
  >
    <section
      class="access-requirement-dialog"
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="access-requirement-title"
      aria-describedby="access-requirement-description"
    >
      <div class="access-requirement-dialog__icon" aria-hidden="true">!</div>
      <div>
        <h2 id="access-requirement-title">可以先浏览，使用时需要授权</h2>
        <p id="access-requirement-description">{{ promptText }}</p>
      </div>
      <footer class="access-requirement-dialog__actions">
        <button type="button" class="access-requirement-dialog__secondary" @click="emit('close')">继续查看</button>
        <button ref="confirmButton" type="button" @click="emit('confirm')">去开通 / 配置</button>
      </footer>
    </section>
  </div>
</template>
