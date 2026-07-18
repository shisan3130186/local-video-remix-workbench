<script setup lang="ts">
import { TTS_LANGUAGE_LABELS, TTS_SCENE_LABELS } from "../voiceCatalog";
import type { TtsVoiceLanguage, TtsVoiceScene } from "../voiceCatalog";

type LanguageFilter = TtsVoiceLanguage | "all" | "multilingual";
type SceneFilter = TtsVoiceScene | "all";

defineProps<{
  searchText: string;
  language: LanguageFilter;
  scene: SceneFilter;
}>();

const emit = defineEmits<{
  "update:searchText": [value: string];
  "update:language": [value: LanguageFilter];
  "update:scene": [value: SceneFilter];
}>();

const languageOptions: Array<{ key: LanguageFilter; label: string }> = [
  { key: "all", label: "全部" },
  { key: "multilingual", label: "多语种" },
  ...Object.entries(TTS_LANGUAGE_LABELS).map(([key, label]) => ({
    key: key as TtsVoiceLanguage,
    label,
  })),
];

const sceneOptions: Array<{ key: SceneFilter; label: string }> = [
  { key: "all", label: "全部" },
  ...Object.entries(TTS_SCENE_LABELS).map(([key, label]) => ({
    key: key as TtsVoiceScene,
    label,
  })),
];

function updateSearch(event: Event) {
  emit("update:searchText", (event.target as HTMLInputElement).value);
}
</script>

<template>
  <div class="voice-library-controls">
    <label class="voice-library-search">
      <span class="sr-only">搜索音色</span>
      <input
        :value="searchText"
        type="search"
        placeholder="搜索音色名称、风格或音色ID"
        data-voice-search
        @input="updateSearch"
      />
    </label>

    <div class="voice-filter-row" aria-label="语言筛选">
      <strong>语言</strong>
      <button
        v-for="option in languageOptions"
        :key="option.key"
        type="button"
        :aria-pressed="language === option.key"
        :class="{ 'voice-filter-chip--active': language === option.key }"
        @click="emit('update:language', option.key)"
      >{{ option.label }}</button>
    </div>

    <div class="voice-filter-row" aria-label="使用场景筛选">
      <strong>场景</strong>
      <button
        v-for="option in sceneOptions"
        :key="option.key"
        type="button"
        :aria-pressed="scene === option.key"
        :class="{ 'voice-filter-chip--active': scene === option.key }"
        @click="emit('update:scene', option.key)"
      >{{ option.label }}</button>
    </div>
  </div>
</template>
