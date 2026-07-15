<script setup lang="ts">
import { ref, watch } from "vue";
import type { TtsSynthesisResult } from "../types";
import "../tts.css";

const props = defineProps<{
  text: string;
  speaker: string;
  resourceId: string;
  configured: boolean;
  isLoadingConfig: boolean;
  isGenerating: boolean;
  configError: string | null;
  error: string | null;
  result: TtsSynthesisResult | null;
  audioUrl: string | null;
}>();

const emit = defineEmits<{
  generate: [];
  "update:text": [value: string];
  "update:speaker": [value: string];
}>();

const audioDurationSeconds = ref<number | null>(null);

watch(
  () => props.audioUrl,
  () => {
    audioDurationSeconds.value = null;
  },
);

function updateText(event: Event) {
  emit("update:text", (event.target as HTMLTextAreaElement).value);
}

function updateSpeaker(event: Event) {
  emit("update:speaker", (event.target as HTMLInputElement).value);
}

function readAudioDuration(event: Event) {
  const duration = (event.target as HTMLAudioElement).duration;
  audioDurationSeconds.value = Number.isFinite(duration) ? duration : null;
}

function formatDuration(value: number | null) {
  if (value === null) {
    return "读取中";
  }

  const totalSeconds = Math.round(value);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = (totalSeconds % 60)
    .toString()
    .padStart(2, "0");
  return `${minutes}:${seconds}`;
}
</script>

<template>
  <div class="tts-config-summary" role="status">
    <span class="tts-config-dot" :class="{ 'tts-config-dot--ready': configured }" aria-hidden="true"></span>
    <span>
      <strong>{{ configured ? "火山TTS已配置" : "火山TTS尚未配置" }}</strong>
      <small>模型资源：{{ resourceId }}</small>
    </span>
  </div>

  <p v-if="isLoadingConfig" class="empty-text">正在读取TTS配置...</p>
  <p v-else-if="configError" class="error-text">{{ configError }}</p>
  <p v-else-if="!configured" class="tts-config-help">
    请在启动软件的PowerShell窗口配置 <code>TTS_API_KEY</code>，Key不会进入前端或日志。
  </p>

  <label class="field tts-script-field">
    <span>配音文案</span>
    <textarea
      :value="text"
      rows="7"
      placeholder="输入需要朗读的文案；这里与AI混剪文案保持同步。"
      @input="updateText"
    ></textarea>
    <small>{{ text.trim().length }} 个字符</small>
  </label>

  <label class="field">
    <span>音色ID</span>
    <input
      :value="speaker"
      type="text"
      autocomplete="off"
      spellcheck="false"
      placeholder="例如：zh_female_vv_uranus_bigtts"
      @input="updateSpeaker"
    />
  </label>

  <button
    class="primary-button primary-button--full"
    type="button"
    :disabled="isGenerating || isLoadingConfig || !configured"
    @click="emit('generate')"
  >
    {{ isGenerating ? "正在生成配音..." : result ? "重新生成配音" : "生成配音MP3" }}
  </button>

  <p v-if="error" class="error-text">{{ error }}</p>

  <section v-if="result && audioUrl" class="tts-result" aria-label="配音生成结果">
    <div class="tts-result__heading">
      <span>
        <strong>配音已生成</strong>
        <small>{{ result.textLength }} 字 · {{ formatDuration(audioDurationSeconds) }}</small>
      </span>
    </div>
    <audio
      class="tts-audio-player"
      :src="audioUrl"
      controls
      preload="metadata"
      @loadedmetadata="readAudioDuration"
    ></audio>
    <p class="output-path">{{ result.outputPath }}</p>
    <p class="empty-text">第一阶段只生成和试听配音文件，暂时不会自动加入视频。</p>
  </section>
</template>
