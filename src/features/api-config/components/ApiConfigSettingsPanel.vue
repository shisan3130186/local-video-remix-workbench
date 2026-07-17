<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
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

const aiStatusText = computed(() => {
  if (status.value?.aiKeyStored) return "已安全保存";
  if (status.value?.aiEnvironmentFallback) return "使用环境变量";
  return "尚未配置";
});

const ttsStatusText = computed(() => {
  if (status.value?.ttsKeyStored) return "已安全保存";
  if (status.value?.ttsEnvironmentFallback) return "使用环境变量";
  return "尚未配置";
});

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
    <div class="api-config-security-note" role="note">
      <span aria-hidden="true">锁</span>
      <div>
        <strong id="api-config-heading">密钥只保存在这台电脑</strong>
        <small>由 Windows 当前账户加密保护；完整密钥不会回显，也不会写入任务日志。</small>
      </div>
    </div>

    <label class="field api-config-platform-field" for="api-config-platform">
      <span>服务平台</span>
      <select id="api-config-platform" disabled>
        <option value="volcengine">火山引擎（方舟 + 豆包语音）</option>
      </select>
      <small>当前版本只接入火山引擎，后续可以继续扩展其他平台。</small>
    </label>

    <p v-if="isLoading" class="api-config-feedback" role="status">正在读取本机配置...</p>

    <template v-else>
      <section class="api-config-section" aria-labelledby="api-config-ai-title">
        <header>
          <span>
            <strong id="api-config-ai-title">AI 画面理解与分镜</strong>
            <small>用于理解视频片段并根据文案匹配画面</small>
          </span>
          <em :class="{ 'api-config-status--ready': status?.aiConfigured }">{{ aiStatusText }}</em>
        </header>

        <SecretInputField
          id="api-config-ai-key"
          v-model="form.aiApiKey"
          label="方舟 API Key"
          :placeholder="keyPlaceholder('ai')"
          :disabled="isSaving"
        />

        <label class="field" for="api-config-ai-model">
          <span>模型接入点 ID</span>
          <input
            id="api-config-ai-model"
            v-model="form.aiModel"
            type="text"
            autocomplete="off"
            spellcheck="false"
            placeholder="例如：ep-2026xxxxxxxx"
            :disabled="isSaving"
          />
        </label>

        <details class="api-config-advanced">
          <summary>连接地址</summary>
          <label class="field" for="api-config-ai-base-url">
            <span>AI Base URL</span>
            <input
              id="api-config-ai-base-url"
              v-model="form.aiBaseUrl"
              type="url"
              autocomplete="off"
              spellcheck="false"
              :disabled="isSaving"
            />
          </label>
        </details>

        <div v-if="status?.aiKeyStored" class="api-config-delete-row">
          <button
            type="button"
            :class="{ 'api-config-delete--confirm': pendingDelete === 'ai' }"
            :disabled="isSaving"
            @click="handleDelete('ai')"
          >{{ pendingDelete === "ai" ? "再次点击确认删除AI密钥" : "删除本机AI密钥" }}</button>
          <button v-if="pendingDelete === 'ai'" type="button" :disabled="isSaving" @click="pendingDelete = null">取消</button>
        </div>
      </section>

      <section class="api-config-section" aria-labelledby="api-config-tts-title">
        <header>
          <span>
            <strong id="api-config-tts-title">TTS 语音合成</strong>
            <small>用于生成逐句配音和带字幕视频</small>
          </span>
          <em :class="{ 'api-config-status--ready': status?.ttsConfigured }">{{ ttsStatusText }}</em>
        </header>

        <SecretInputField
          id="api-config-tts-key"
          v-model="form.ttsApiKey"
          label="TTS API Key"
          :placeholder="keyPlaceholder('tts')"
          :disabled="isSaving"
        />

        <details class="api-config-advanced">
          <summary>语音参数</summary>
          <label class="field" for="api-config-tts-resource">
            <span>资源 ID</span>
            <input id="api-config-tts-resource" v-model="form.ttsResourceId" type="text" spellcheck="false" :disabled="isSaving" />
          </label>
          <label class="field" for="api-config-tts-speaker">
            <span>默认音色 ID</span>
            <input id="api-config-tts-speaker" v-model="form.ttsSpeaker" type="text" spellcheck="false" :disabled="isSaving" />
          </label>
        </details>

        <div v-if="status?.ttsKeyStored" class="api-config-delete-row">
          <button
            type="button"
            :class="{ 'api-config-delete--confirm': pendingDelete === 'tts' }"
            :disabled="isSaving"
            @click="handleDelete('tts')"
          >{{ pendingDelete === "tts" ? "再次点击确认删除TTS密钥" : "删除本机TTS密钥" }}</button>
          <button v-if="pendingDelete === 'tts'" type="button" :disabled="isSaving" @click="pendingDelete = null">取消</button>
        </div>
      </section>

      <p v-if="error" class="api-config-feedback api-config-feedback--error" role="alert">{{ error }}</p>
      <p v-if="successMessage" class="api-config-feedback api-config-feedback--success" role="status">{{ successMessage }}</p>

      <button class="primary-button primary-button--full" type="button" :disabled="isSaving" @click="handleSave">
        {{ isSaving ? "正在安全保存..." : "保存并立即使用" }}
      </button>
    </template>
  </section>
</template>
