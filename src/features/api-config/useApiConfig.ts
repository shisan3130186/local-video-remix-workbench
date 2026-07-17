import { reactive, ref } from "vue";
import {
  deleteApiCredential,
  getApiConfigStatus,
  saveApiConfig,
} from "./services/apiConfigService";
import type { ApiConfigStatus, ApiCredentialKind } from "./types";

const DEFAULT_AI_BASE_URL = "https://ark.cn-beijing.volces.com/api/v3";
const DEFAULT_TTS_RESOURCE_ID = "seed-tts-2.0";
const DEFAULT_TTS_SPEAKER = "zh_female_vv_uranus_bigtts";

export function useApiConfig() {
  const status = ref<ApiConfigStatus | null>(null);
  const isLoading = ref(false);
  const isSaving = ref(false);
  const error = ref<string | null>(null);
  const successMessage = ref<string | null>(null);
  const form = reactive({
    aiApiKey: "",
    aiBaseUrl: DEFAULT_AI_BASE_URL,
    aiModel: "",
    ttsApiKey: "",
    ttsResourceId: DEFAULT_TTS_RESOURCE_ID,
    ttsSpeaker: DEFAULT_TTS_SPEAKER,
  });

  async function loadConfiguration() {
    isLoading.value = true;
    error.value = null;

    try {
      const nextStatus = await getApiConfigStatus();
      applyStatus(nextStatus);
      return nextStatus;
    } catch (loadError) {
      error.value = formatError(loadError, "读取API密钥配置失败。");
      return null;
    } finally {
      isLoading.value = false;
    }
  }

  async function saveConfiguration() {
    isSaving.value = true;
    error.value = null;
    successMessage.value = null;

    try {
      const nextStatus = await saveApiConfig({ ...form });
      form.aiApiKey = "";
      form.ttsApiKey = "";
      applyStatus(nextStatus);
      successMessage.value = nextStatus.aiConfigured || nextStatus.ttsConfigured
        ? "配置已安全保存，后续启动软件会自动使用。"
        : "连接参数已保存，但尚未填写AI或TTS密钥。";
      return nextStatus;
    } catch (saveError) {
      error.value = formatError(saveError, "保存API密钥配置失败。");
      return null;
    } finally {
      isSaving.value = false;
    }
  }

  async function removeCredential(kind: ApiCredentialKind) {
    isSaving.value = true;
    error.value = null;
    successMessage.value = null;

    try {
      const nextStatus = await deleteApiCredential(kind);
      applyStatus(nextStatus);
      successMessage.value = kind === "ai" ? "已删除本机保存的AI密钥。" : "已删除本机保存的TTS密钥。";
      return nextStatus;
    } catch (deleteError) {
      error.value = formatError(deleteError, "删除API密钥失败。");
      return null;
    } finally {
      isSaving.value = false;
    }
  }

  function applyStatus(nextStatus: ApiConfigStatus) {
    status.value = nextStatus;
    form.aiBaseUrl = nextStatus.aiBaseUrl || DEFAULT_AI_BASE_URL;
    form.aiModel = nextStatus.aiModel;
    form.ttsResourceId = nextStatus.ttsResourceId || DEFAULT_TTS_RESOURCE_ID;
    form.ttsSpeaker = nextStatus.ttsSpeaker || DEFAULT_TTS_SPEAKER;
  }

  return {
    error,
    form,
    isLoading,
    isSaving,
    loadConfiguration,
    removeCredential,
    saveConfiguration,
    status,
    successMessage,
  };
}

function formatError(error: unknown, fallback: string) {
  if (error instanceof Error) {
    return error.message;
  }

  const value = String(error ?? "").trim();
  return value || fallback;
}
