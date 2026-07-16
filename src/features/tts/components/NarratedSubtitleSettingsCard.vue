<script setup lang="ts">
import type { NarratedSubtitlePosition, NarratedSubtitleSize } from "../types";

defineProps<{
  enabled: boolean;
  position: NarratedSubtitlePosition;
  size: NarratedSubtitleSize;
  disabled: boolean;
}>();

const emit = defineEmits<{
  "update:enabled": [value: boolean];
  "update:position": [value: NarratedSubtitlePosition];
  "update:size": [value: NarratedSubtitleSize];
}>();

function updateEnabled(event: Event) {
  emit("update:enabled", (event.target as HTMLInputElement).checked);
}

function updatePosition(event: Event) {
  emit("update:position", (event.target as HTMLSelectElement).value as NarratedSubtitlePosition);
}

function updateSize(event: Event) {
  emit("update:size", (event.target as HTMLSelectElement).value as NarratedSubtitleSize);
}
</script>

<template>
  <section
    class="tts-setting-card tts-subtitle-card"
    :class="{ 'tts-setting-card--enabled': enabled }"
    aria-labelledby="tts-subtitle-title"
  >
    <div class="tts-setting-card__header">
      <span>
        <strong id="tts-subtitle-title">自动生成字幕</strong>
        <small>文案会按停顿拆成短句，跟随AI配音依次显示</small>
      </span>
      <label class="tts-switch">
        <input
          :checked="enabled"
          type="checkbox"
          :disabled="disabled"
          aria-label="自动生成字幕"
          @change="updateEnabled"
        />
        <span aria-hidden="true"></span>
      </label>
    </div>

    <div class="tts-subtitle-options">
      <label>
        <span>字幕位置</span>
        <select :value="position" :disabled="!enabled || disabled" @change="updatePosition">
          <option value="bottom">底部</option>
          <option value="middle">中间</option>
          <option value="top">顶部</option>
        </select>
      </label>
      <label>
        <span>字幕大小</span>
        <select :value="size" :disabled="!enabled || disabled" @change="updateSize">
          <option value="small">小</option>
          <option value="medium">标准</option>
          <option value="large">大</option>
        </select>
      </label>
    </div>

    <div
      class="tts-subtitle-preview"
      :class="[`tts-subtitle-preview--${position}`, `tts-subtitle-preview--${size}`]"
      aria-label="字幕样式预览"
    >
      <span>显瘦百搭小裙子</span>
    </div>
    <p>长文案会按标点和长度自然分段，避免多行文字同时挤在画面里。</p>
  </section>
</template>
