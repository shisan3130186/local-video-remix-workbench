<script setup lang="ts">
defineProps<{
  keepOriginalAudio: boolean;
  originalAudioVolume: number;
  disabled: boolean;
}>();

const emit = defineEmits<{
  "update:keepOriginalAudio": [value: boolean];
  "update:originalAudioVolume": [value: number];
}>();

function updateKeepOriginalAudio(event: Event) {
  emit("update:keepOriginalAudio", (event.target as HTMLInputElement).checked);
}

function updateOriginalAudioVolume(event: Event) {
  const percentage = Number((event.target as HTMLInputElement).value);
  emit("update:originalAudioVolume", Math.min(100, Math.max(0, percentage)) / 100);
}
</script>

<template>
  <section
    class="tts-setting-card"
    :class="{ 'tts-setting-card--enabled': keepOriginalAudio }"
    aria-labelledby="tts-original-audio-title"
  >
    <div class="tts-setting-card__header">
      <span>
        <strong id="tts-original-audio-title">保留原视频声音</strong>
        <small>包含原人声、环境声和背景音乐</small>
      </span>
      <label class="tts-switch">
        <input
          :checked="keepOriginalAudio"
          type="checkbox"
          :disabled="disabled"
          aria-label="保留原视频声音"
          @change="updateKeepOriginalAudio"
        />
        <span aria-hidden="true"></span>
      </label>
    </div>

    <label class="tts-volume-control" for="tts-original-audio-volume">
      <span>
        <strong>原声音量</strong>
        <output for="tts-original-audio-volume">
          {{ Math.round(originalAudioVolume * 100) }}%
        </output>
      </span>
      <input
        id="tts-original-audio-volume"
        :value="Math.round(originalAudioVolume * 100)"
        type="range"
        min="0"
        max="100"
        step="5"
        :disabled="!keepOriginalAudio || disabled"
        aria-label="原视频声音音量"
        @input="updateOriginalAudioVolume"
      />
    </label>
    <p>只在生成带AI配音的视频时生效；关闭后仅保留AI配音。</p>
  </section>
</template>
