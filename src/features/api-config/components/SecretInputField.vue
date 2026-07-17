<script setup lang="ts">
import { ref } from "vue";

defineProps<{
  id: string;
  label: string;
  placeholder: string;
  disabled: boolean;
}>();

const model = defineModel<string>({ required: true });
const isVisible = ref(false);
</script>

<template>
  <label class="field" :for="id">
    <span>{{ label }}</span>
    <span class="api-secret-input">
      <input
        :id="id"
        v-model="model"
        :type="isVisible ? 'text' : 'password'"
        autocomplete="new-password"
        spellcheck="false"
        :placeholder="placeholder"
        :disabled="disabled"
      />
      <button
        type="button"
        :aria-label="isVisible ? `隐藏${label}` : `显示${label}`"
        :disabled="disabled"
        @click="isVisible = !isVisible"
      >{{ isVisible ? "隐藏" : "显示" }}</button>
    </span>
  </label>
</template>
