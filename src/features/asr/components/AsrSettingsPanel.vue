<script setup lang="ts">
import { ref, watch } from "vue";
import type { AsrRecognitionResult } from "../types";
import "../asr.css";

const props = defineProps<{
  configured: boolean;
  resourceId: string;
  sourceFilePath: string | null;
  sourceFileName: string | null;
  isLoadingConfig: boolean;
  isRecognizing: boolean;
  error: string | null;
  result: AsrRecognitionResult | null;
  isSavingToLibrary: boolean;
  libraryFeedback: string | null;
}>();

const emit = defineEmits<{
  selectSource: [];
  recognize: [];
  clear: [];
  saveToLibrary: [];
  saveAndUse: [];
}>();

const copyFeedback = ref<string | null>(null);

watch(
  () => props.result,
  () => {
    copyFeedback.value = null;
  },
);

async function copyFullText() {
  if (!props.result?.text) return;
  try {
    await navigator.clipboard.writeText(props.result.text);
    copyFeedback.value = "完整文字已复制";
  } catch {
    copyFeedback.value = "复制失败，请在文字框中手动选择复制";
  }
}

function formatTimestamp(milliseconds: number) {
  const safeMilliseconds = Math.max(0, Math.round(milliseconds));
  const totalSeconds = Math.floor(safeMilliseconds / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  const remainder = safeMilliseconds % 1000;
  return `${minutes.toString().padStart(2, "0")}:${seconds
    .toString()
    .padStart(2, "0")}.${remainder.toString().padStart(3, "0")}`;
}

function formatDuration(value: number | null) {
  if (value === null) return "时长未知";
  const totalSeconds = Math.max(0, Math.round(value));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  return hours > 0
    ? `${hours}小时${minutes}分${seconds}秒`
    : `${minutes}分${seconds}秒`;
}

function formatBytes(value: number) {
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(0)} KB`;
  return `${(value / 1024 / 1024).toFixed(1)} MB`;
}
</script>

<template>
  <section class="asr-panel" aria-labelledby="asr-panel-heading">
    <div class="asr-config-summary" role="status">
      <span class="asr-config-dot" :class="{ 'asr-config-dot--ready': configured }" aria-hidden="true"></span>
      <span>
        <strong id="asr-panel-heading">{{ configured ? "火山语音识别已配置" : "火山语音识别尚未配置" }}</strong>
        <small>识别资源：{{ resourceId }}</small>
      </span>
    </div>

    <p v-if="isLoadingConfig" class="empty-text">正在读取语音识别配置...</p>
    <p v-else-if="!configured" class="asr-config-help">
      请先在“API 密钥”中保存TTS/ASR语音密钥，同一个密钥可同时用于配音和语音识别。
    </p>

    <section class="asr-source-card" aria-label="待识别文件">
      <div>
        <strong>{{ sourceFileName ?? "选择音频或带声音的视频" }}</strong>
        <small v-if="sourceFilePath" class="asr-source-path">{{ sourceFilePath }}</small>
        <small v-else>支持 MP3、WAV、M4A、MP4、MOV、MKV 等常用格式，单个文件不超过2小时。</small>
      </div>
      <div class="asr-source-actions">
        <button class="ghost-button" type="button" :disabled="isRecognizing" @click="emit('selectSource')">
          {{ sourceFilePath ? "更换文件" : "选择文件" }}
        </button>
        <button v-if="sourceFilePath" class="asr-clear-button" type="button" :disabled="isRecognizing" @click="emit('clear')">
          清空
        </button>
      </div>
    </section>

    <p class="asr-cost-note" role="note">
      相同文件未发生修改时会直接读取本地缓存，避免重复调用云端接口和重复消耗额度。
    </p>

    <button
      class="primary-button primary-button--full"
      type="button"
      :disabled="isRecognizing || isLoadingConfig || !configured || !sourceFilePath"
      @click="emit('recognize')"
    >
      {{ isRecognizing ? "正在识别，请稍候..." : result ? "重新检查并识别" : "开始语音识别" }}
    </button>

    <p v-if="error" class="error-text" role="alert">{{ error }}</p>

    <section v-if="result" class="asr-result" aria-label="语音识别结果">
      <header class="asr-result__header">
        <span>
          <strong>识别完成</strong>
          <small>
            {{ formatDuration(result.durationSeconds) }} · {{ result.utterances.length }} 条句子 · 上传音频 {{ formatBytes(result.normalizedAudioBytes) }}
          </small>
        </span>
        <em :class="{ 'asr-cache-badge--hit': result.cacheHit }">
          {{ result.cacheHit ? "本地缓存" : "云端新识别" }}
        </em>
      </header>

      <div class="asr-text-heading">
        <strong>完整文字</strong>
        <button type="button" @click="copyFullText">复制文字</button>
      </div>
      <textarea class="asr-full-text" :value="result.text" rows="7" readonly aria-label="完整识别文字"></textarea>
      <p v-if="copyFeedback" class="asr-copy-feedback" role="status">{{ copyFeedback }}</p>

      <div class="asr-library-actions">
        <button
          class="ghost-button"
          type="button"
          :disabled="isSavingToLibrary"
          @click="emit('saveToLibrary')"
        >{{ isSavingToLibrary ? "正在保存..." : "保存到文案库" }}</button>
        <button
          class="primary-button"
          type="button"
          :disabled="isSavingToLibrary"
          @click="emit('saveAndUse')"
        >保存并使用到当前文案</button>
      </div>
      <p v-if="libraryFeedback" class="asr-library-feedback" role="status">{{ libraryFeedback }}</p>

      <div class="asr-timeline-heading">
        <strong>句子时间轴</strong>
        <small>开始时间 - 结束时间</small>
      </div>
      <ol v-if="result.utterances.length > 0" class="asr-timeline">
        <li v-for="(utterance, index) in result.utterances" :key="`${utterance.startTimeMs}-${index}`">
          <time>{{ formatTimestamp(utterance.startTimeMs) }} - {{ formatTimestamp(utterance.endTimeMs) }}</time>
          <p>{{ utterance.text }}</p>
        </li>
      </ol>
      <p v-else class="empty-text">云端返回了完整文字，但没有提供可用的句子时间轴。</p>
    </section>
  </section>
</template>
