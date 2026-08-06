<script setup lang="ts">
import { onMounted } from "vue";
import { useApiConfig } from "../useApiConfig";
import SecretInputField from "./SecretInputField.vue";
import "../api-config.css";

const emit = defineEmits<{
  changed: [];
}>();

const {
  error,
  form,
  isLoading,
  isSaving,
  loadConfiguration,
  saveConfiguration,
  status,
  successMessage,
} = useApiConfig();

onMounted(() => {
  void loadConfiguration();
});

async function handleSave() {
  if (await saveConfiguration()) {
    emit("changed");
  }
}

function keyPlaceholder(kind: "ai" | "tts") {
  const keyStored = kind === "ai" ? status.value?.aiKeyStored : status.value?.ttsKeyStored;
  const environmentFallback = kind === "ai"
    ? status.value?.aiEnvironmentFallback
    : status.value?.ttsEnvironmentFallback;

  if (keyStored) return "已安全保存；留空表示不更换";
  if (environmentFallback) return "正在使用环境变量；输入后改为本机保存";
  return kind === "ai" ? "请输入方舟 API Key" : "请输入火山语音 API Key";
}
</script>

<template>
  <section class="api-config-panel" aria-labelledby="api-config-heading">
    <label class="field api-config-platform-field" for="api-config-platform">
      <strong id="api-config-heading">平台选择</strong>
      <select id="api-config-platform" disabled>
        <option value="volcengine">火山引擎</option>
      </select>
    </label>

    <p v-if="isLoading" class="api-config-feedback" role="status">正在读取本机配置...</p>

    <template v-else>
      <SecretInputField id="api-config-ai-key" v-model="form.aiApiKey" label="Ark API Key" :placeholder="keyPlaceholder('ai')" :disabled="isSaving" />
      <SecretInputField id="api-config-tts-key" v-model="form.ttsApiKey" label="TTS API Key" :placeholder="keyPlaceholder('tts')" :disabled="isSaving" />

      <p class="api-config-note">平台已固定为火山引擎。模型、语音资源和默认音色由软件统一配置，你只需要填写下面两个 Key。</p>

      <p v-if="error" class="api-config-feedback api-config-feedback--error" role="alert">{{ error }}</p>
      <p v-if="successMessage" class="api-config-feedback api-config-feedback--success" role="status">{{ successMessage }}</p>

      <button class="primary-button api-config-save" type="button" :disabled="isSaving" @click="handleSave">
        {{ isSaving ? "保存中..." : "保存" }}
      </button>
    </template>
  </section>
</template>
