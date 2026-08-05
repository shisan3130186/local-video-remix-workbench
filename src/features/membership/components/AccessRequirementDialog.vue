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
const promptText = computed(() => `暂无使用权限，请先在右上角个人中心完成${missingItems.value.join("、")}配置。`);

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
        <h2 id="access-requirement-title">没有权限</h2>
        <p id="access-requirement-description">{{ promptText }}</p>
      </div>
      <button ref="confirmButton" type="button" @click="emit('confirm')">确定</button>
    </section>
  </div>
</template>
