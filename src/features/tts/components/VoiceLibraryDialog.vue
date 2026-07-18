<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from "vue";
import { TTS_VOICE_CATALOG } from "../voiceCatalog";
import type { TtsVoiceLanguage, TtsVoiceScene } from "../voiceCatalog";
import "../voice-library.css";
import VoiceCatalogCard from "./VoiceCatalogCard.vue";
import VoiceLibraryFilters from "./VoiceLibraryFilters.vue";

const props = defineProps<{ currentSpeaker: string }>();
const emit = defineEmits<{ close: []; select: [speakerId: string] }>();

const dialogElement = ref<HTMLElement | null>(null);
const searchText = ref("");
const language = ref<TtsVoiceLanguage | "all" | "multilingual">("all");
const scene = ref<TtsVoiceScene | "all">("all");
const pendingSpeaker = ref(props.currentSpeaker);
const playingSpeaker = ref<string | null>(null);
const previewError = ref<string | null>(null);
let previewAudio: HTMLAudioElement | null = null;

const filteredVoices = computed(() => {
  const keyword = searchText.value.trim().toLocaleLowerCase();
  return TTS_VOICE_CATALOG.filter((voice) => {
    const matchesKeyword =
      !keyword ||
      voice.name.toLocaleLowerCase().includes(keyword) ||
      voice.description.toLocaleLowerCase().includes(keyword) ||
      voice.id.toLocaleLowerCase().includes(keyword);
    const matchesLanguage =
      language.value === "all" ||
      (language.value === "multilingual"
        ? voice.languages.length > 1
        : voice.languages.includes(language.value));
    const matchesScene = scene.value === "all" || voice.scenes.includes(scene.value);
    return matchesKeyword && matchesLanguage && matchesScene;
  });
});

const pendingVoice = computed(
  () => TTS_VOICE_CATALOG.find((voice) => voice.id === pendingSpeaker.value) ?? null,
);

onMounted(() => {
  window.addEventListener("keydown", handleKeydown);
  void nextTick(() => dialogElement.value?.querySelector<HTMLInputElement>("[data-voice-search]")?.focus());
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
  stopPreview();
});

function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    emit("close");
    return;
  }
  if (event.key !== "Tab" || !dialogElement.value) return;

  const focusableElements = Array.from(
    dialogElement.value.querySelectorAll<HTMLElement>(
      'button:not([disabled]), input:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ),
  );
  if (focusableElements.length === 0) return;

  const firstElement = focusableElements[0];
  const lastElement = focusableElements[focusableElements.length - 1];
  if (event.shiftKey && document.activeElement === firstElement) {
    event.preventDefault();
    lastElement.focus();
  } else if (!event.shiftKey && document.activeElement === lastElement) {
    event.preventDefault();
    firstElement.focus();
  }
}

async function togglePreview(speakerId: string, previewUrl: string) {
  previewError.value = null;
  if (playingSpeaker.value === speakerId) {
    stopPreview();
    return;
  }
  stopPreview();
  previewAudio = new Audio(previewUrl);
  playingSpeaker.value = speakerId;
  previewAudio.addEventListener("ended", stopPreview, { once: true });
  previewAudio.addEventListener(
    "error",
    () => {
      previewError.value = "官方样音暂时无法播放，请检查网络后重试。";
      stopPreview();
    },
    { once: true },
  );
  try {
    await previewAudio.play();
  } catch {
    previewError.value = "浏览器阻止了样音播放，请再次点击试听。";
    stopPreview();
  }
}

function stopPreview() {
  previewAudio?.pause();
  previewAudio = null;
  playingSpeaker.value = null;
}

function applySelection() {
  if (!pendingVoice.value) return;
  emit("select", pendingVoice.value.id);
}
</script>

<template>
  <div class="voice-library-backdrop" @click.self="emit('close')">
    <section
      ref="dialogElement"
      class="voice-library-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="voice-library-title"
      aria-describedby="voice-library-description"
    >
      <header class="voice-library-header">
        <div>
          <p class="panel__label">火山TTS 2.0</p>
          <h3 id="voice-library-title">选择AI音色</h3>
          <small id="voice-library-description">官方样音试听不消耗你的TTS额度</small>
        </div>
        <button class="panel-toggle" type="button" @click="emit('close')">关闭</button>
      </header>

      <VoiceLibraryFilters
        :search-text="searchText"
        :language="language"
        :scene="scene"
        @update:search-text="searchText = $event"
        @update:language="language = $event"
        @update:scene="scene = $event"
      />

      <div class="voice-library-result-bar" aria-live="polite">
        <span>找到 {{ filteredVoices.length }} 款音色</span>
        <small>实际可用范围以你的火山账号权限为准</small>
      </div>

      <div class="voice-library-grid" role="list">
        <VoiceCatalogCard
          v-for="voice in filteredVoices"
          :key="voice.id"
          :voice="voice"
          :selected="pendingSpeaker === voice.id"
          :playing="playingSpeaker === voice.id"
          @select="pendingSpeaker = $event"
          @toggle-preview="togglePreview"
        />
      </div>

      <div v-if="filteredVoices.length === 0" class="voice-library-empty" role="status">
        <strong>没有找到匹配音色</strong>
        <span>可以清空搜索词，或切换到“全部”重新选择。</span>
      </div>

      <p v-if="previewError" class="voice-library-error" role="alert">{{ previewError }}</p>

      <footer class="voice-library-footer">
        <span>
          <small>当前选择</small>
          <strong>{{ pendingVoice?.name ?? "请选择一款音色" }}</strong>
        </span>
        <button class="primary-button" type="button" :disabled="!pendingVoice" @click="applySelection">
          使用所选音色
        </button>
      </footer>
    </section>
  </div>
</template>
