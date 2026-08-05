<script setup lang="ts">
import { computed, nextTick, ref } from "vue";
import {
  findTtsVoice,
  TTS_GENDER_LABELS,
  TTS_LANGUAGE_LABELS,
  TTS_SCENE_LABELS,
  TTS_VOICE_CATALOG,
} from "../voiceCatalog";
import VoiceLibraryDialog from "./VoiceLibraryDialog.vue";
import "../voice-library.css";

const props = defineProps<{ speaker: string; disabled: boolean }>();
const emit = defineEmits<{ "update:speaker": [speakerId: string] }>();
const isLibraryOpen = ref(false);
const selectButton = ref<HTMLButtonElement | null>(null);
const selectedVoice = computed(() => findTtsVoice(props.speaker));
const quickVoices = TTS_VOICE_CATALOG.filter(
  (voice) => voice.recommended && voice.languages.includes("zh"),
).slice(0, 4);

const AVATAR_PALETTES = [
  ["#3b6b58", "#65c5a3"],
  ["#315e78", "#6ab3da"],
  ["#7c6031", "#e0b264"],
  ["#7c4857", "#e07a99"],
  ["#5f507f", "#9c8bce"],
  ["#4c5a56", "#9eafaa"],
];

function avatarGradient(id: string): string {
  let hash = 0;
  for (let i = 0; i < id.length; i++) hash = (hash * 31 + id.charCodeAt(i)) >>> 0;
  const [a, b] = AVATAR_PALETTES[hash % AVATAR_PALETTES.length];
  return `linear-gradient(135deg, ${a}, ${b})`;
}

function avatarLabel(name: string): string {
  if (!name) return "?";
  const first = name.charAt(0);
  if (/[\u4e00-\u9fff]/.test(first)) return first;
  return name.substring(0, 2).toUpperCase();
}

const avatarFailures = ref<Set<string>>(new Set());

function onAvatarError(id: string) {
  avatarFailures.value.add(id);
  avatarFailures.value = new Set(avatarFailures.value);
}

function updateCustomSpeaker(event: Event) {
  emit("update:speaker", (event.target as HTMLInputElement).value);
}

function selectSpeaker(speakerId: string) {
  emit("update:speaker", speakerId);
  closeLibrary();
}

function selectQuickSpeaker(speakerId: string) {
  emit("update:speaker", speakerId);
}

function closeLibrary() {
  isLibraryOpen.value = false;
  void nextTick(() => selectButton.value?.focus());
}
</script>

<template>
  <section class="voice-selector-card" aria-label="当前AI音色">
    <div class="voice-selector-summary">
      <span
        class="voice-selector-avatar"
        :style="selectedVoice ? { background: avatarGradient(selectedVoice.id) } : { background: '#5f507f' }"
        aria-hidden="true"
      >
        <img
          v-if="selectedVoice && !avatarFailures.has(selectedVoice.id)"
          :src="selectedVoice.avatarUrl"
          :alt="selectedVoice.name"
          loading="lazy"
          @error="onAvatarError(selectedVoice.id)"
        />
        <span v-else>{{ avatarLabel(selectedVoice?.name ?? '自定') }}</span>
      </span>
      <div>
        <small>当前音色</small>
        <strong>{{ selectedVoice?.name ?? "自定义音色" }}</strong>
        <em v-if="selectedVoice">
          {{ TTS_GENDER_LABELS[selectedVoice.gender] }} ·
          {{ selectedVoice.languages.map((item) => TTS_LANGUAGE_LABELS[item]).join(" / ") }}
        </em>
        <em v-else>{{ speaker }}</em>
      </div>
      <button
        ref="selectButton"
        class="ghost-button"
        type="button"
        :disabled="disabled"
        @click="isLibraryOpen = true"
      >
        选择音色
      </button>
    </div>

    <div class="voice-quick-heading">
      <span>常用音色</span>
      <small>点击卡片直接选择</small>
    </div>
    <div class="voice-quick-grid" aria-label="常用AI音色">
      <button
        v-for="voice in quickVoices"
        :key="voice.id"
        class="voice-quick-card"
        :class="{ 'voice-quick-card--selected': voice.id === speaker }"
        type="button"
        :disabled="disabled"
        :aria-pressed="voice.id === speaker"
        @click="selectQuickSpeaker(voice.id)"
      >
        <span
          class="voice-selector-avatar voice-selector-avatar--sm"
          :style="{ background: avatarGradient(voice.id) }"
          aria-hidden="true"
        >
          <img
            v-if="!avatarFailures.has(voice.id)"
            :src="voice.avatarUrl"
            :alt="voice.name"
            loading="lazy"
            @error="onAvatarError(voice.id)"
          />
          <span v-else>{{ avatarLabel(voice.name) }}</span>
        </span>
        <div>
          <strong>{{ voice.name }}</strong>
          <small>{{ TTS_GENDER_LABELS[voice.gender] }} · {{ TTS_SCENE_LABELS[voice.scenes[0]] }}</small>
        </div>
      </button>
    </div>

    <p>{{ selectedVoice?.description ?? "正在使用手动填写的火山音色ID。" }}</p>

    <details class="voice-selector-custom">
      <summary>高级：手动填写音色ID</summary>
      <label class="field">
        <span>音色ID</span>
        <input
          :value="speaker"
          type="text"
          autocomplete="off"
          spellcheck="false"
          placeholder="例如：zh_female_vv_uranus_bigtts"
          :disabled="disabled"
          @input="updateCustomSpeaker"
        />
      </label>
    </details>
  </section>

  <VoiceLibraryDialog
    v-if="isLibraryOpen"
    :current-speaker="speaker"
    @close="closeLibrary"
    @select="selectSpeaker"
  />
</template>
