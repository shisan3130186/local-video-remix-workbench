<script setup lang="ts">
import { computed } from "vue";
import { formatFileName } from "./inputHelpers";
import ToolIcon from "../ToolIcon.vue";
import type { FusionSettings } from "../../services/videoMixService";

const props = defineProps<{ materialFilePath: string | null; settings: FusionSettings }>();
const emit = defineEmits<{ selectMaterialFile: []; clearMaterialFile: []; reset: []; "update:settings": [value: FusionSettings] }>();
const settings = computed({ get: () => props.settings, set: (value: FusionSettings) => emit("update:settings", value) });
</script>

<template>
  <section class="replica-parameter-panel replica-effect-panel replica-effect-panel--fusion">
    <header class="replica-parameter-panel__header">
      <h2>像素融合 - 参数设置</h2>
      <div class="replica-parameter-panel__actions">
        <button class="replica-parameter-button" type="button" @click="emit('reset')">重置</button>
        <button class="replica-parameter-button replica-parameter-button--enable" :class="{ 'is-enabled': settings.enabled }" type="button" @click="settings = { ...settings, enabled: !settings.enabled }">{{ settings.enabled ? "已启用" : "启用" }}</button>
      </div>
    </header>

    <div class="replica-effect-file-row">
      <strong>融合素材</strong>
      <span class="replica-file-display" :title="props.materialFilePath ?? '未选择素材'">{{ props.materialFilePath ? formatFileName(props.materialFilePath) : "未选择素材" }}</span>
      <button class="replica-icon-button" type="button" aria-label="选择像素融合素材" title="选择像素融合素材" @click="emit('selectMaterialFile')"><ToolIcon name="file" /></button>
      <button class="replica-clear-button" type="button" :disabled="!props.materialFilePath" @click="emit('clearMaterialFile')">清除</button>
    </div>

    <div class="replica-effect-row">
      <strong>融合间隔</strong>
      <label class="replica-range-field"><input :value="settings.intervalMin" type="number" min="1" @input="settings = { ...settings, intervalMin: Number(($event.target as HTMLInputElement).value) || 1 }" /><span>帧</span></label>
      <em>—</em>
      <label class="replica-range-field"><input :value="settings.intervalMax" type="number" min="1" @input="settings = { ...settings, intervalMax: Number(($event.target as HTMLInputElement).value) || 1 }" /><span>帧</span></label>
    </div>
    <div class="replica-effect-row">
      <strong>融合强度</strong>
      <label class="replica-range-field"><input :value="settings.strength" type="number" min="0.01" max="1" step="0.01" @input="settings = { ...settings, strength: Number(($event.target as HTMLInputElement).value) || 0.01 }" /></label>
      <small>范围 0.01 - 1.0</small>
    </div>
    <p class="replica-parameter-help">每间隔随机帧数做一次像素融合，设置 1 - 1 可实现全局融合。可根据需求自行调整融合强度。</p>
  </section>
</template>
