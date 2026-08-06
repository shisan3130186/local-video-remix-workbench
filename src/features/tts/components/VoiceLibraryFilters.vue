<script setup lang="ts">
import { TTS_LANGUAGE_LABELS, TTS_SCENE_LABELS } from "../voiceCatalog";
import type { TtsVoiceLanguage, TtsVoiceScene } from "../voiceCatalog";

defineProps<{
  searchText: string;
  language: TtsVoiceLanguage | "all" | "multilingual";
  scene: TtsVoiceScene | "all";
  onlyFavored: boolean;
  favoredCount: number;
}>();

const emit = defineEmits<{
  "update:searchText": [value: string];
  "update:language": [value: TtsVoiceLanguage | "all" | "multilingual"];
  "update:scene": [value: TtsVoiceScene | "all"];
  "update:onlyFavored": [value: boolean];
}>();

const LANGUAGES: { value: TtsVoiceLanguage | "all" | "multilingual"; label: string }[] = [
  { value: "all", label: "全部语言" },
  { value: "multilingual", label: "多语种" },
  { value: "zh", label: TTS_LANGUAGE_LABELS.zh },
  { value: "en", label: TTS_LANGUAGE_LABELS.en },
  { value: "ja", label: TTS_LANGUAGE_LABELS.ja },
];

const SCENES: { value: TtsVoiceScene | "all"; label: string }[] = [
  { value: "all", label: "全部场景" },
  { value: "general", label: TTS_SCENE_LABELS.general },
  { value: "narration", label: TTS_SCENE_LABELS.narration },
  { value: "commercial", label: TTS_SCENE_LABELS.commercial },
  { value: "character", label: TTS_SCENE_LABELS.character },
  { value: "youth", label: TTS_SCENE_LABELS.youth },
  { value: "customer-service", label: TTS_SCENE_LABELS["customer-service"] },
];
</script>

<template>
  <div class="voice-library-filters">
    <div class="voice-library-filters__search">
      <input
        data-voice-search
        type="search"
        placeholder="搜索音色名称、语言、场景…"
        :value="searchText"
        @input="emit('update:searchText', ($event.target as HTMLInputElement).value)"
      />
    </div>

    <div class="voice-library-filters__row">
      <span class="voice-library-filters__label">场景</span>
      <div class="voice-library-filters__chips">
        <button
          v-for="s in SCENES"
          :key="s.value"
          class="voice-library-filter-chip"
          :class="{ 'voice-library-filter-chip--active': scene === s.value }"
          type="button"
          @click="emit('update:scene', s.value)"
        >{{ s.label }}</button>
      </div>
    </div>

    <div class="voice-library-filters__row">
      <span class="voice-library-filters__label">语言</span>
      <div class="voice-library-filters__chips">
        <button
          v-for="lang in LANGUAGES"
          :key="lang.value"
          class="voice-library-filter-chip"
          :class="{ 'voice-library-filter-chip--active': language === lang.value }"
          type="button"
          @click="emit('update:language', lang.value)"
        >{{ lang.label }}</button>
      </div>
    </div>

    <div class="voice-library-filters__legacy-fav">
      <button
        class="voice-library-filters__fav"
        :class="{ 'voice-library-filters__fav--active': onlyFavored }"
        type="button"
        @click="emit('update:onlyFavored', !onlyFavored)"
      >★ 我的收藏 <span class="voice-library-filters__fav-count">{{ favoredCount }}</span></button>
    </div>
  </div>
</template>
