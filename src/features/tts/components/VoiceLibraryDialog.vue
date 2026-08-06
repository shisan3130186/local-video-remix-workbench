<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
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
const generatingPreviewFor = ref<string | null>(null);

// 试听缓存：speaker_id -> base64 mp3（生成过的试听音频）
const previewCache = new Map<string, string>();
const onlyFavored = ref<boolean>(false);
const pendingSpeaker = ref(props.currentSpeaker);
const playingSpeaker = ref<string | null>(null);
const previewError = ref<string | null>(null);
const randomTargetId = ref<string | null>(null);
const previewRequestVersion = ref(0);

const FAVORITES_KEY = "smartcut.voice-library.favorites.v1";
const favoriteIds = ref<Set<string>>(new Set());

function loadFavorites() {
  if (typeof localStorage === "undefined") return;
  try {
    const raw = localStorage.getItem(FAVORITES_KEY);
    if (!raw) return;
    const parsed = JSON.parse(raw);
    if (Array.isArray(parsed)) {
      favoriteIds.value = new Set(parsed.filter((item) => typeof item === "string"));
    }
  } catch {
    /* ignore corrupted localStorage */
  }
}

function persistFavorites() {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(FAVORITES_KEY, JSON.stringify([...favoriteIds.value]));
  } catch {
    /* localStorage may be unavailable; silently ignore */
  }
}

function toggleFavorite(speakerId: string) {
  if (favoriteIds.value.has(speakerId)) {
    favoriteIds.value.delete(speakerId);
  } else {
    favoriteIds.value.add(speakerId);
  }
  // Force reactivity for Set mutations
  favoriteIds.value = new Set(favoriteIds.value);
  persistFavorites();
}

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
    const matchesFavored = !onlyFavored.value || favoriteIds.value.has(voice.id);
    return matchesKeyword && matchesLanguage && matchesScene && matchesFavored;
  });
});

const pendingVoice = computed(
  () => TTS_VOICE_CATALOG.find((voice) => voice.id === pendingSpeaker.value) ?? null,
);

const totalCount = TTS_VOICE_CATALOG.length;
const favoredCount = computed(() => favoriteIds.value.size);

// 试听音频池：每个 speaker 保留最新实例，便于 WebView2 第二次点击重试
const audioPool = new Map<string, HTMLAudioElement>();
const audioPoolRetried = new Set<string>();

onMounted(() => {
  loadFavorites();
  window.addEventListener("keydown", handleKeydown);
  void nextTick(() => dialogElement.value?.querySelector<HTMLInputElement>("[data-voice-search]")?.focus());
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
  stopAllPreviews();
});

watch(
  () => props.currentSpeaker,
  (next) => {
    pendingSpeaker.value = next;
  },
);

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

function stopAllPreviews() {
  previewRequestVersion.value += 1;
  for (const audio of audioPool.values()) {
    try {
      audio.pause();
      audio.currentTime = 0;
    } catch {
      /* ignore */
    }
  }
  audioPool.clear();
  audioPoolRetried.clear();
  playingSpeaker.value = null;
}

function stopPreviewExcept(targetId: string) {
  for (const [id, audio] of audioPool.entries()) {
    if (id === targetId) continue;
    try {
      audio.pause();
      audio.currentTime = 0;
    } catch {
      /* ignore */
    }
    audioPool.delete(id);
    audioPoolRetried.delete(id);
  }
}

async function togglePreview(speakerId: string, previewUrl: string) {
  previewError.value = null;
  const requestVersion = previewRequestVersion.value + 1;
  previewRequestVersion.value = requestVersion;

  // 同一音色当前在播放：停止
  if (playingSpeaker.value === speakerId) {
    const current = audioPool.get(speakerId);
    if (current) {
      current.pause();
      current.currentTime = 0;
    }
    audioPoolRetried.delete(speakerId);
    playingSpeaker.value = null;
    return;
  }

  // 切换到新音色：先清掉其它实例
  stopPreviewExcept(speakerId);

  let audio = audioPool.get(speakerId);
  let isRetry = false;
  if (!audio) {
    audio = new Audio();
    audio.preload = "none";
    audio.src = previewUrl;
    audio.addEventListener("ended", () => {
      if (playingSpeaker.value === speakerId) playingSpeaker.value = null;
    });
    audio.addEventListener("error", () => {
      if (playingSpeaker.value === speakerId) playingSpeaker.value = null;
      audioPool.delete(speakerId);
      audioPoolRetried.delete(speakerId);
      void generateAndPlayPreview(speakerId, requestVersion);
    });
    audioPool.set(speakerId, audio);
  } else if (audioPoolRetried.has(speakerId)) {
    // 之前 autoplay 被阻止过，直接重试 play（user gesture 仍然有效）
    isRetry = true;
    audioPoolRetried.delete(speakerId);
  }

  playingSpeaker.value = speakerId;
  try {
    await audio.play();
  } catch {
    // 区分错误类型：先看 src 是否已经失败（说明是网络问题不是 autoplay 阻止）
    const mediaErr = audio.error;
    if (mediaErr?.code === MediaError.MEDIA_ERR_SRC_NOT_SUPPORTED) {
      // 官方没发布 mp3：自动 fallback 到 TTS 实时合成试听
      audioPool.delete(speakerId);
      audioPoolRetried.delete(speakerId);
      playingSpeaker.value = null;
      void generateAndPlayPreview(speakerId, requestVersion);
      return;
    }
    // 网络失败也自动改走 TTS，确保每个可用音色都有试听路径。
    if (mediaErr?.code === MediaError.MEDIA_ERR_NETWORK) {
      playingSpeaker.value = null;
      void generateAndPlayPreview(speakerId, requestVersion);
      return;
    }
    // 真正的 autoplay 阻止：标记为可重试
    audioPoolRetried.add(speakerId);
    previewError.value = isRetry
      ? "试听仍被阻止，请关闭后重试，或联系管理员。"
      : "首次播放被浏览器阻止，请再次点击试听按钮。";
    playingSpeaker.value = null;
  }
}

const PREVIEW_DEFAULT_TEXT = "你好，这是智剪的音色试听样本。";

async function generateAndPlayPreview(speakerId: string, requestVersion: number) {
  if (requestVersion !== previewRequestVersion.value) return;
  // 命中缓存直接播
  const cached = previewCache.get(speakerId);
  if (cached) {
    playBase64Preview(speakerId, cached, requestVersion);
    return;
  }
  if (generatingPreviewFor.value === speakerId) return;
  generatingPreviewFor.value = speakerId;
  previewError.value = "官方样音不可用，正在用豆包实时合成试听…";
  try {
    const base64 = await invoke<string>("synthesize_preview_audio", {
      text: PREVIEW_DEFAULT_TEXT,
      speaker: speakerId,
    });
    previewCache.set(speakerId, base64);
    if (requestVersion === previewRequestVersion.value) {
      playBase64Preview(speakerId, base64, requestVersion);
    }
  } catch (error) {
    const message = typeof error === "string" ? error : (error as Error)?.message ?? String(error);
    if (/not\s*found|couldn't\s+find|no\s+such\s+command/i.test(message)) {
      previewError.value = "生成试听功能尚未注册，请关闭应用后重新打开（首次会编译 Rust 后端，约 30-60 秒）。";
    } else if (/apikey|api[_\s-]?key|未配置|insufficient|quota|余额/i.test(message)) {
      previewError.value = "生成试听失败：TTS API Key 未配置或余额不足，请到「个人中心 → TTS 服务」检查。";
    } else {
      previewError.value = `生成试听失败：${message}`;
    }
  } finally {
    generatingPreviewFor.value = null;
  }
}

function playBase64Preview(speakerId: string, base64: string, requestVersion: number) {
  if (requestVersion !== previewRequestVersion.value) return;
  stopPreviewExcept(speakerId);
  const audio = new Audio(`data:audio/mp3;base64,${base64}`);
  audio.preload = "auto";
  audio.addEventListener("ended", () => {
    if (playingSpeaker.value === speakerId) playingSpeaker.value = null;
  });
  audio.addEventListener("error", () => {
    previewError.value = "试听音频播放失败。";
    if (playingSpeaker.value === speakerId) playingSpeaker.value = null;
  });
  audioPool.set(speakerId, audio);
  playingSpeaker.value = speakerId;
  previewError.value = null;
  void audio.play().catch(() => {
    previewError.value = "试听播放被阻止，请重试。";
    playingSpeaker.value = null;
  });
}

function pickRandomVoice() {
  const list = filteredVoices.value;
  if (list.length === 0) {
    randomTargetId.value = null;
    return;
  }
  const index = Math.floor(Math.random() * list.length);
  const target = list[index];
  randomTargetId.value = target.id;
  pendingSpeaker.value = target.id;
  void nextTick(() => {
    const card = dialogElement.value?.querySelector<HTMLElement>(
      `[data-voice-id="${target.id}"]`,
    );
    card?.scrollIntoView({ behavior: "smooth", block: "center" });
  });
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
        <div class="voice-library-header__title">
          <p class="panel__label">火山TTS 2.0</p>
          <h3 id="voice-library-title">选择AI音色</h3>
          <small id="voice-library-description">官方样音优先，加载失败时自动切换豆包实时试听</small>
        </div>
        <div class="voice-library-header__actions">
          <button
            class="voice-library-random"
            type="button"
            title="随机音色"
            :disabled="filteredVoices.length === 0"
            @click="pickRandomVoice"
          >🎲</button>
          <button class="panel-toggle" type="button" @click="emit('close')">关闭</button>
        </div>
      </header>

      <VoiceLibraryFilters
        :search-text="searchText"
        :language="language"
        :scene="scene"
        :only-favored="onlyFavored"
        :favored-count="favoredCount"
        @update:search-text="searchText = $event"
        @update:language="language = $event"
        @update:scene="scene = $event"
        @update:only-favored="onlyFavored = $event"
      />

      <div class="voice-library-result-bar" aria-live="polite">
        <span>共 <strong>{{ filteredVoices.length }}</strong> / {{ totalCount }} 款音色</span>
        <small>实际可用范围以你的火山账号权限为准</small>
      </div>

      <div
        v-if="filteredVoices.length > 0"
        class="voice-library-grid"
        role="list"
      >
        <VoiceCatalogCard
          v-for="voice in filteredVoices"
          :key="voice.id"
          :voice="voice"
          :selected="pendingSpeaker === voice.id"
          :playing="playingSpeaker === voice.id"
          :favored="favoriteIds.has(voice.id)"
          :data-voice-id="voice.id"
          @select="pendingSpeaker = $event"
          @toggle-preview="togglePreview"
          @toggle-favorite="toggleFavorite"
        />
      </div>

      <div v-else class="voice-library-empty" role="status">
        <strong>没有找到匹配音色</strong>
        <span>可以清空搜索词，或切换到"全部"重新选择。</span>
      </div>

      <p v-if="previewError" class="voice-library-error" role="alert">{{ previewError }}</p>

      <footer class="voice-library-footer">
        <span class="voice-library-footer__current">
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
