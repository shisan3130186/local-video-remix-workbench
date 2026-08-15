<script setup lang="ts">
import type { PipPosition } from "../../services/videoMixService";
import { formatFileName, readChecked, readNumber, readSelectedValue } from "./inputHelpers";
import ToolIcon from "../ToolIcon.vue";

const props = defineProps<{
  pipEnabled: boolean;
  pipOverlayFilePath: string | null;
  pipMode: "main" | "external";
  pipMainSizeMin: number;
  pipMainSizeMax: number;
  pipBlurMin: number;
  pipBlurMax: number;
  pipOffsetXMin: number;
  pipOffsetXMax: number;
  pipOffsetYMin: number;
  pipOffsetYMax: number;
  pipPosition: PipPosition;
  pipSizeRatio: number;
  pipOpacity: number;
  pipMargin: number;
}>();

const emit = defineEmits<{
  selectPipOverlayFile: [];
  "update:pipEnabled": [value: boolean];
  "update:pipMode": [value: "main" | "external"];
  "update:pipMainSizeMin": [value: number];
  "update:pipMainSizeMax": [value: number];
  "update:pipBlurMin": [value: number];
  "update:pipBlurMax": [value: number];
  "update:pipOffsetXMin": [value: number];
  "update:pipOffsetXMax": [value: number];
  "update:pipOffsetYMin": [value: number];
  "update:pipOffsetYMax": [value: number];
  "update:pipPosition": [value: PipPosition];
  "update:pipSizeRatio": [value: number];
  "update:pipOpacity": [value: number];
  "update:pipMargin": [value: number];
  reset: [];
}>();

</script>

<template>
  <section class="replica-parameter-panel replica-effect-panel replica-effect-panel--pip">
    <header class="replica-parameter-panel__header">
      <h2>画中画 - 参数设置</h2>
      <div class="replica-parameter-panel__actions">
        <button class="replica-parameter-button" type="button" @click="emit('reset')">重置</button>
        <button class="replica-parameter-button replica-parameter-button--enable" :class="{ 'is-enabled': props.pipEnabled }" type="button" @click="emit('update:pipEnabled', !props.pipEnabled)">{{ props.pipEnabled ? "已启用" : "启用" }}</button>
      </div>
    </header>

    <div class="replica-effect-file-row">
      <strong>背景素材</strong>
      <span class="replica-file-display" :title="props.pipOverlayFilePath ?? '未选择素材'">{{ props.pipOverlayFilePath ? formatFileName(props.pipOverlayFilePath) : "本地图片 / 视频文件" }}</span>
      <button class="replica-icon-button" type="button" aria-label="选择画中画背景素材" title="选择画中画背景素材" @click="emit('selectPipOverlayFile')"><ToolIcon name="file" /></button>
      <button class="replica-clear-button" type="button" :disabled="!props.pipOverlayFilePath" @click="emit('reset')">清除</button>
    </div>

    <div class="replica-effect-mode-row">
      <strong>画中画模式</strong>
      <label class="replica-effect-radio"><input :checked="props.pipMode === 'main'" value="main" type="radio" @change="emit('update:pipMode', 'main')" /><span>主视频模式</span></label>
      <label class="replica-effect-radio"><input :checked="props.pipMode === 'external'" value="external" type="radio" @change="emit('update:pipMode', 'external')" /><span>外部素材模式</span></label>
    </div>

    <div class="replica-effect-row">
      <strong>主视频大小</strong>
      <label class="replica-range-field"><input :value="props.pipMainSizeMin" type="number" min="0.1" max="1" step="0.01" @input="emit('update:pipMainSizeMin', readNumber($event))" /></label><em>—</em><label class="replica-range-field"><input :value="props.pipMainSizeMax" type="number" min="0.1" max="1" step="0.01" @input="emit('update:pipMainSizeMax', readNumber($event))" /></label>
    </div>
    <div class="replica-effect-row">
      <strong>虚化强度</strong>
      <label class="replica-range-field"><input :value="props.pipBlurMin" type="number" min="0" max="100" @input="emit('update:pipBlurMin', readNumber($event))" /></label><em>—</em><label class="replica-range-field"><input :value="props.pipBlurMax" type="number" min="0" max="100" @input="emit('update:pipBlurMax', readNumber($event))" /></label>
    </div>
    <div class="replica-effect-row">
      <strong>左右偏移 X</strong>
      <label class="replica-range-field"><input :value="props.pipOffsetXMin" type="number" min="0" max="1" step="0.01" @input="emit('update:pipOffsetXMin', readNumber($event))" /></label><em>—</em><label class="replica-range-field"><input :value="props.pipOffsetXMax" type="number" min="0" max="1" step="0.01" @input="emit('update:pipOffsetXMax', readNumber($event))" /></label>
    </div>
    <div class="replica-effect-row">
      <strong>上下偏移 Y</strong>
      <label class="replica-range-field"><input :value="props.pipOffsetYMin" type="number" min="0" max="1" step="0.01" @input="emit('update:pipOffsetYMin', readNumber($event))" /></label><em>—</em><label class="replica-range-field"><input :value="props.pipOffsetYMax" type="number" min="0" max="1" step="0.01" @input="emit('update:pipOffsetYMax', readNumber($event))" /></label>
    </div>

    <div class="replica-effect-row replica-effect-row--legacy">
      <strong>预览位置</strong>
      <select :value="props.pipPosition" @change="emit('update:pipPosition', readSelectedValue($event) as PipPosition)">
        <option value="topLeft">左上</option><option value="topRight">右上</option><option value="bottomLeft">左下</option><option value="bottomRight">右下</option><option value="center">居中</option>
      </select>
      <label class="replica-range-field"><input :value="props.pipSizeRatio" type="number" min="0.2" max="0.5" step="0.05" @input="emit('update:pipSizeRatio', readNumber($event))" /></label>
      <label class="replica-range-field"><input :value="props.pipOpacity" type="number" min="0" max="1" step="0.05" @input="emit('update:pipOpacity', readNumber($event))" /></label>
      <input :value="props.pipMargin" type="hidden" @input="emit('update:pipMargin', readNumber($event))" />
    </div>

    <p class="replica-parameter-help">左右 / 上下偏移为中心点 0 - 1（0.5 = 居中；小于 0.5 向左 / 上，大于 0.5 向右 / 下）；最小值 = 最大值为固定参数，否则会在范围内随机参数。</p>
  </section>
</template>
