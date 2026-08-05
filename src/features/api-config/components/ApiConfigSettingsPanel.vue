<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useApiConfig } from "../useApiConfig";
import type { ApiCredentialKind } from "../types";
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
  removeCredential,
  saveConfiguration,
  status,
  successMessage,
} = useApiConfig();

const pendingDelete = ref<ApiCredentialKind | null>(null);

onMounted(() => {
  void loadConfiguration();
});

async function handleSave() {
  if (await saveConfiguration()) {
    emit("changed");
  }
}

async function handleDelete(kind: ApiCredentialKind) {
  if (pendingDelete.value !== kind) {
    pendingDelete.value = kind;
    return;
  }

  if (await removeCredential(kind)) {
    pendingDelete.value = null;
    emit("changed");
  }
}

function keyPlaceholder(kind: ApiCredentialKind) {
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

      <details class="api-config-advanced api-config-advanced--all">
        <summary>高级连接设置</summary>
        <label class="field" for="api-config-ai-model"><span>模型接入点 ID</span><input id="api-config-ai-model" v-model="form.aiModel" type="text" autocomplete="off" spellcheck="false" placeholder="例如：ep-2026xxxxxxxx" :disabled="isSaving" /></label>
        <label class="field" for="api-config-ai-base-url"><span>AI Base URL</span><input id="api-config-ai-base-url" v-model="form.aiBaseUrl" type="url" autocomplete="off" spellcheck="false" :disabled="isSaving" /></label>
        <label class="field" for="api-config-tts-resource"><span>语音资源 ID</span><input id="api-config-tts-resource" v-model="form.ttsResourceId" type="text" spellcheck="false" :disabled="isSaving" /></label>
        <label class="field" for="api-config-tts-speaker"><span>默认音色 ID</span><input id="api-config-tts-speaker" v-model="form.ttsSpeaker" type="text" spellcheck="false" :disabled="isSaving" /></label>
        <div class="api-config-delete-row">
          <button v-if="status?.aiKeyStored" type="button" :class="{ 'api-config-delete--confirm': pendingDelete === 'ai' }" :disabled="isSaving" @click="handleDelete('ai')">{{ pendingDelete === 'ai' ? '再次点击确认' : '删除 Ark 密钥' }}</button>
          <button v-if="status?.ttsKeyStored" type="button" :class="{ 'api-config-delete--confirm': pendingDelete === 'tts' }" :disabled="isSaving" @click="handleDelete('tts')">{{ pendingDelete === 'tts' ? '再次点击确认' : '删除 TTS 密钥' }}</button>
        </div>
      </details>

      <p v-if="error" class="api-config-feedback api-config-feedback--error" role="alert">{{ error }}</p>
      <p v-if="successMessage" class="api-config-feedback api-config-feedback--success" role="status">{{ successMessage }}</p>

      <button class="primary-button api-config-save" type="button" :disabled="isSaving" @click="handleSave">
        {{ isSaving ? "保存中..." : "保存" }}
      </button>
    </template>
  </section>
</template>
