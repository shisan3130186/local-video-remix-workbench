<script setup lang="ts">
import { computed, ref } from "vue";
import { TTS_GENDER_LABELS, TTS_LANGUAGE_LABELS, TTS_SCENE_LABELS } from "../voiceCatalog";
import type { TtsVoiceCatalogItem } from "../voiceCatalog";

const props = defineProps<{
  voice: TtsVoiceCatalogItem;
  selected: boolean;
  playing: boolean;
  favored: boolean;
}>();

const emit = defineEmits<{
  select: [speakerId: string];
  "toggle-preview": [speakerId: string, previewUrl: string];
  "toggle-favorite": [speakerId: string];
}>();

const avatarFailed = ref(false);
const displayAvatarUrl = ref(props.voice.avatarUrl);

const AVATAR_PALETTES = [
  ["#3b6b58", "#65c5a3"],
  ["#315e78", "#6ab3da"],
  ["#7c6031", "#e0b264"],
  ["#7c4857", "#e07a99"],
  ["#5f507f", "#9c8bce"],
  ["#4c5a56", "#9eafaa"],
];

const avatarGradient = computed(() => {
  let hash = 0;
  for (let i = 0; i < props.voice.id.length; i++) {
    hash = (hash * 31 + props.voice.id.charCodeAt(i)) >>> 0;
  }
  return AVATAR_PALETTES[hash % AVATAR_PALETTES.length];
});

const avatarLabel = computed(() => {
  const name = props.voice.name;
  if (!name) return "?";
  // 取第一个汉字，如果是英文取第一个字母
  const firstChar = name.charAt(0);
  if (/[\u4e00-\u9fff]/.test(firstChar)) return firstChar;
  return name.substring(0, 2).toUpperCase();
});

const languageText = computed(() => {
  if (props.voice.languages.length > 1) return "多语种";
  return TTS_LANGUAGE_LABELS[props.voice.languages[0]] ?? props.voice.languages[0];
});

const sceneText = computed(
  () => TTS_SCENE_LABELS[props.voice.scenes[0]] ?? props.voice.scenes[0],
);

function onAvatarError() {
  avatarFailed.value = true;
}
</script>

<template>
  <div
    class="voice-card"
    :class="{ 'voice-card--selected': selected }"
    :data-voice-id="voice.id"
    role="listitem"
    @click="emit('select', voice.id)"
  >
    <span v-if="selected" class="voice-card__selected-tag">✓</span>

    <div class="voice-card__avatar-wrap">
      <img
        v-if="!avatarFailed"
        class="voice-card__avatar"
        :src="displayAvatarUrl"
        :alt="voice.name"
        loading="lazy"
        @error="onAvatarError"
      />
      <span
        v-else
        class="voice-card__avatar--fallback"
        :style="{ background: `linear-gradient(135deg, ${avatarGradient[0]}, ${avatarGradient[1]})` }"
      >{{ avatarLabel }}</span>
      <button
        class="voice-card__fav"
        :class="{ 'voice-card__fav--active': favored }"
        :aria-label="favored ? '取消收藏' : '收藏'"
        type="button"
        @click.stop="emit('toggle-favorite', voice.id)"
      >{{ favored ? '★' : '☆' }}</button>
    </div>

    <span class="voice-card__name">{{ voice.name }}</span>

    <div class="voice-card__badges">
      <span class="voice-card__badge voice-card__badge--lang">{{ languageText }}</span>
      <span class="voice-card__badge voice-card__badge--scene">{{ sceneText }}</span>
    </div>

    <button
      class="voice-card__play"
      :class="{ 'voice-card__play--playing': playing }"
      :aria-label="playing ? '停止试听' : '试听'"
      type="button"
      @click.stop="emit('toggle-preview', voice.id, voice.previewUrl)"
    >{{ playing ? '■' : '▶' }}</button>
  </div>
</template>
