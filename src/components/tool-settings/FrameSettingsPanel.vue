<script setup lang="ts">
import { computed } from "vue";
import { formatFileName } from "./inputHelpers";
import type { FrameOperationSettings } from "../../services/videoMixService";
import ToolIcon from "../ToolIcon.vue";

const props = defineProps<{
  materialFilePath: string | null;
  settings: FrameOperationSettings;
  isMixing: boolean;
  isBatchMixing: boolean;
}>();

const emit = defineEmits<{
  selectMaterialFile: [];
  clearMaterialFile: [];
  reset: [];
  "update:settings": [value: FrameOperationSettings];
}>();

const settings = computed({ get: () => props.settings, set: (value: FrameOperationSettings) => emit("update:settings", value) });
const isBusy = computed(() => props.isMixing || props.isBatchMixing);
const needsMaterial = computed(() => settings.value.mode !== "extract");
</script>

<template>
  <section class="replica-parameter-panel replica-effect-panel replica-effect-panel--frame">
    <header class="replica-parameter-panel__header">
      <h2>视频帧操作 - 参数设置</h2>
      <div class="replica-parameter-panel__actions">
        <button class="replica-parameter-button" type="button" @click="emit('reset')">重置</button>
        <button
          class="replica-parameter-button replica-parameter-button--enable"
          :class="{ 'is-enabled': settings.enabled }"
          type="button"
          :disabled="isBusy"
          @click="settings = { ...settings, enabled: !settings.enabled }"
        >{{ settings.enabled ? "已启用" : "启用" }}</button>
      </div>
    </header>

    <div class="replica-effect-mode-row">
      <strong>操作模式</strong>
      <label v-for="option in [['extract', '仅抽帧'], ['insert', '仅插帧'], ['mixed', '混合模式'] ]" :key="option[0]" class="replica-effect-radio">
        <input :checked="settings.mode === option[0]" :value="option[0]" type="radio" @change="settings = { ...settings, mode: option[0] as FrameOperationSettings['mode'] }" />
        <span>{{ option[1] }}</span>
      </label>
    </div>

    <div class="replica-effect-row">
      <strong>随机间隔</strong>
      <label class="replica-range-field"><input :value="settings.intervalMin" type="number" min="1" @input="settings = { ...settings, intervalMin: Number(($event.target as HTMLInputElement).value) || 1 }" /><span>帧</span></label>
      <em>—</em>
      <label class="replica-range-field"><input :value="settings.intervalMax" type="number" min="1" @input="settings = { ...settings, intervalMax: Number(($event.target as HTMLInputElement).value) || 1 }" /><span>帧</span></label>
      <small>每隔随机帧数执行一次操作</small>
    </div>

    <div class="replica-effect-row">
      <strong>操作帧数</strong>
      <label class="replica-range-field"><input :value="settings.frameMin" type="number" min="1" @input="settings = { ...settings, frameMin: Number(($event.target as HTMLInputElement).value) || 1 }" /><span>帧</span></label>
      <em>—</em>
      <label class="replica-range-field"><input :value="settings.frameMax" type="number" min="1" @input="settings = { ...settings, frameMax: Number(($event.target as HTMLInputElement).value) || 1 }" /><span>帧</span></label>
    </div>

    <div class="replica-effect-file-row">
      <strong>素材文件</strong>
      <span class="replica-file-display" :title="props.materialFilePath ?? '未选择素材'">{{ props.materialFilePath ? formatFileName(props.materialFilePath) : "未选择素材" }}</span>
      <button class="replica-icon-button" type="button" aria-label="选择视频帧操作素材" title="选择视频帧操作素材" @click="emit('selectMaterialFile')"><ToolIcon name="file" /></button>
      <button class="replica-clear-button" type="button" :disabled="!props.materialFilePath" @click="emit('clearMaterialFile')">清除</button>
    </div>

    <div class="replica-effect-row">
      <strong>素材透明度</strong>
      <label class="replica-range-field"><input :value="settings.opacity" type="number" min="0" max="100" step="1" @input="settings = { ...settings, opacity: Number(($event.target as HTMLInputElement).value) || 0 }" /></label>
      <span>%</span>
    </div>

    <p v-if="needsMaterial && !props.materialFilePath" class="replica-parameter-warning">在启用插帧模式时，至少选择一个素材，否则无法进行处理，文件夹模式下系统将随机抽取文件夹中的素材。</p>
    <p v-else class="replica-parameter-help">仅抽帧会在随机间隔内删除画面帧；插帧会使用上传素材插入画面，混合模式会同时执行抽帧和插帧。</p>
  </section>
</template>
