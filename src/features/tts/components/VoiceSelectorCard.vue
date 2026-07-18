<script setup lang="ts">
import { computed, nextTick, ref } from "vue";
import { findTtsVoice, TTS_GENDER_LABELS, TTS_LANGUAGE_LABELS } from "../voiceCatalog";
import VoiceLibraryDialog from "./VoiceLibraryDialog.vue";
import "../voice-library.css";

const props = defineProps<{ speaker: string; disabled: boolean }>();
const emit = defineEmits<{ "update:speaker": [speakerId: string] }>();
const isLibraryOpen = ref(false);
const selectButton = ref<HTMLButtonElement | null>(null);
const selectedVoice = computed(() => findTtsVoice(props.speaker));

function updateCustomSpeaker(event: Event) {
  emit("update:speaker", (event.target as HTMLInputElement).value);
}

function selectSpeaker(speakerId: string) {
  emit("update:speaker", speakerId);
  closeLibrary();
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
        :class="selectedVoice ? `voice-selector-avatar--${selectedVoice.tone}` : 'voice-selector-avatar--custom'"
        aria-hidden="true"
      >{{ selectedVoice?.name.slice(0, 2) ?? "自定" }}</span>
      <span>
        <small>当前音色</small>
        <strong>{{ selectedVoice?.name ?? "自定义音色" }}</strong>
        <em v-if="selectedVoice">
          {{ TTS_GENDER_LABELS[selectedVoice.gender] }} ·
          {{ selectedVoice.languages.map((item) => TTS_LANGUAGE_LABELS[item]).join(" / ") }}
        </em>
        <em v-else>{{ speaker }}</em>
      </span>
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
