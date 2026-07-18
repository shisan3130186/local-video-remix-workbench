<script setup lang="ts">
import { TTS_GENDER_LABELS, TTS_LANGUAGE_LABELS, TTS_SCENE_LABELS } from "../voiceCatalog";
import type { TtsVoiceCatalogItem } from "../voiceCatalog";

defineProps<{
  voice: TtsVoiceCatalogItem;
  selected: boolean;
  playing: boolean;
}>();

const emit = defineEmits<{
  select: [speakerId: string];
  "toggle-preview": [speakerId: string, previewUrl: string];
}>();
</script>

<template>
  <article
    class="voice-card"
    :class="[`voice-card--${voice.tone}`, { 'voice-card--selected': selected }]"
    role="listitem"
  >
    <button
      class="voice-card__select"
      type="button"
      :aria-pressed="selected"
      :aria-label="`选择音色：${voice.name}`"
      @click="emit('select', voice.id)"
    >
      <span class="voice-card__avatar" aria-hidden="true">{{ voice.name.slice(0, 2) }}</span>
      <span class="voice-card__content">
        <span class="voice-card__title">
          <strong>{{ voice.name }}</strong>
          <em v-if="voice.recommended">推荐</em>
        </span>
        <small>
          {{ TTS_GENDER_LABELS[voice.gender] }} ·
          {{ voice.languages.map((item) => TTS_LANGUAGE_LABELS[item]).join(" / ") }}
        </small>
        <span>{{ voice.description }}</span>
      </span>
      <b v-if="selected" aria-label="已选择">✓</b>
    </button>
    <div class="voice-card__footer">
      <span>{{ voice.scenes.map((item) => TTS_SCENE_LABELS[item]).join(" · ") }}</span>
      <button
        type="button"
        :aria-label="`${playing ? '停止试听' : '试听'}：${voice.name}`"
        @click="emit('toggle-preview', voice.id, voice.previewUrl)"
      >
        {{ playing ? "停止" : "试听" }}
      </button>
    </div>
  </article>
</template>
