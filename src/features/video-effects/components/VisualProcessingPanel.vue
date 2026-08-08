<script setup lang="ts">
import type { DynamicZoomMode } from "../../../services/videoMixService";
import type { WatermarkAssetType, WatermarkTrajectory } from "../../watermark/types";

const props = defineProps<{
  watermarkRemovalEnabled: boolean;
  watermarkRemovalRegionCount: number;
  watermarkEnabled: boolean;
  watermarkAssetType: WatermarkAssetType;
  watermarkImageFilePath: string | null;
  watermarkOpacity: number;
  watermarkImageSizeRatio: number;
  watermarkTrajectory: WatermarkTrajectory;
  hslEnabled: boolean;
  hue: number;
  saturation: number;
  brightness: number;
  zoomEnabled: boolean;
  zoomMode: DynamicZoomMode;
  zoomMinScale: number;
  zoomMaxScale: number;
  zoomMinDurationSeconds: number;
  zoomMaxDurationSeconds: number;
  disabled: boolean;
}>();

const emit = defineEmits<{
  "update:watermarkRemovalEnabled": [value: boolean];
  "update:watermarkRemovalRegionCount": [value: number];
  "update:watermarkEnabled": [value: boolean];
  "update:watermarkAssetType": [value: WatermarkAssetType];
  "update:watermarkOpacity": [value: number];
  "update:watermarkImageSizeRatio": [value: number];
  "update:watermarkTrajectory": [value: WatermarkTrajectory];
  "update:hslEnabled": [value: boolean];
  "update:hue": [value: number];
  "update:saturation": [value: number];
  "update:brightness": [value: number];
  "update:zoomEnabled": [value: boolean];
  "update:zoomMode": [value: DynamicZoomMode];
  "update:zoomMinScale": [value: number];
  "update:zoomMaxScale": [value: number];
  "update:zoomMinDurationSeconds": [value: number];
  "update:zoomMaxDurationSeconds": [value: number];
  selectWatermarkAsset: [];
}>();

const trajectoryOptions: Array<{ key: WatermarkTrajectory; label: string }> = [
  { key: "static", label: "静止" },
  { key: "horizontal", label: "水平" },
  { key: "vertical", label: "垂直" },
  { key: "diagonal", label: "对角" },
  { key: "random", label: "随机" },
];

const zoomOptions: Array<{ key: DynamicZoomMode; label: string }> = [
  { key: "push", label: "推近" },
  { key: "pull", label: "拉远" },
  { key: "random", label: "随机" },
];

function fileName(path: string | null) {
  if (!path) return "未选择素材";
  return path.split(/[\\/]/).pop() ?? path;
}

function numberValue(event: Event, min: number, max: number) {
  const value = Number((event.target as HTMLInputElement).value);
  return Math.min(max, Math.max(min, Number.isFinite(value) ? value : min));
}

function toggleValue(event: Event) {
  return (event.target as HTMLInputElement).checked;
}
</script>

<template>
  <div class="visual-processing-panel" aria-label="画面处理参数">
    <section class="visual-processing-card">
      <header class="visual-processing-card__header">
        <strong>去除水印</strong>
        <label class="replica-switch">
          <input :checked="props.watermarkRemovalEnabled" type="checkbox" :disabled="props.disabled" @change="emit('update:watermarkRemovalEnabled', toggleValue($event))" />
          <span></span>
        </label>
      </header>
      <div class="visual-processing-field-label">位置：</div>
      <div class="visual-processing-segmented visual-processing-segmented--two">
        <button type="button" class="is-active" disabled>手动框选</button>
        <button type="button" disabled>自动检测</button>
      </div>
      <label class="visual-processing-number-row">
        <span>区域数</span>
        <input :value="props.watermarkRemovalRegionCount" type="number" min="1" max="8" step="1" :disabled="props.disabled || !props.watermarkRemovalEnabled" @input="emit('update:watermarkRemovalRegionCount', numberValue($event, 1, 8))" />
      </label>
      <small class="visual-processing-help">在视频画面中直接拖动红色框，框的数量与这里一致。</small>
    </section>

    <section class="visual-processing-card">
      <header class="visual-processing-card__header">
        <strong>图片/视频水印</strong>
        <label class="replica-switch">
          <input :checked="props.watermarkEnabled" type="checkbox" :disabled="props.disabled" @change="emit('update:watermarkEnabled', toggleValue($event))" />
          <span></span>
        </label>
      </header>
      <div class="visual-processing-file-row">
        <span :title="props.watermarkImageFilePath ?? '未选择素材'">{{ fileName(props.watermarkImageFilePath) }}</span>
        <button type="button" :disabled="props.disabled" aria-label="选择图片或视频水印" @click="emit('selectWatermarkAsset')">选择</button>
      </div>
      <div class="visual-processing-segmented visual-processing-segmented--two">
        <button type="button" :class="{ 'is-active': props.watermarkAssetType === 'image' }" :disabled="props.disabled" @click="emit('update:watermarkAssetType', 'image')">图片</button>
        <button type="button" :class="{ 'is-active': props.watermarkAssetType === 'video' }" :disabled="props.disabled" @click="emit('update:watermarkAssetType', 'video')">视频</button>
      </div>
      <label class="visual-processing-range-row">
        <span>透明度</span>
        <input :value="Math.round(props.watermarkOpacity * 100)" type="range" min="10" max="100" step="1" :disabled="props.disabled || !props.watermarkEnabled" @input="emit('update:watermarkOpacity', numberValue($event, 10, 100) / 100)" />
        <output>{{ Math.round(props.watermarkOpacity * 100) }}%</output>
      </label>
      <label class="visual-processing-range-row">
        <span>大小</span>
        <input :value="Math.round(props.watermarkImageSizeRatio * 100)" type="range" min="8" max="50" step="1" :disabled="props.disabled || !props.watermarkEnabled" @input="emit('update:watermarkImageSizeRatio', numberValue($event, 8, 50) / 100)" />
        <output>{{ Math.round(props.watermarkImageSizeRatio * 100) }}%</output>
      </label>
      <div class="visual-processing-field-label">轨迹</div>
      <div class="visual-processing-trajectory-row">
        <button v-for="option in trajectoryOptions" :key="option.key" type="button" :class="{ 'is-active': props.watermarkTrajectory === option.key }" :disabled="props.disabled || !props.watermarkEnabled" @click="emit('update:watermarkTrajectory', option.key)">{{ option.label }}</button>
      </div>
    </section>

    <section class="visual-processing-card">
      <header class="visual-processing-card__header">
        <strong>HSL 调色</strong>
        <label class="replica-switch">
          <input :checked="props.hslEnabled" type="checkbox" :disabled="props.disabled" @change="emit('update:hslEnabled', toggleValue($event))" />
          <span></span>
        </label>
      </header>
      <label class="visual-processing-range-row">
        <span>色相</span>
        <input :value="props.hue" type="range" min="-180" max="180" step="1" :disabled="props.disabled || !props.hslEnabled" @input="emit('update:hue', numberValue($event, -180, 180))" />
        <output>{{ props.hue > 0 ? '+' : '' }}{{ Math.round(props.hue) }}°</output>
      </label>
      <label class="visual-processing-range-row">
        <span>饱和度</span>
        <input :value="Math.round(props.saturation * 100)" type="range" min="0" max="200" step="1" :disabled="props.disabled || !props.hslEnabled" @input="emit('update:saturation', numberValue($event, 0, 200) / 100)" />
        <output>{{ Math.round(props.saturation * 100) }}%</output>
      </label>
      <label class="visual-processing-range-row">
        <span>亮度</span>
        <input :value="Math.round((props.brightness + 1) * 100)" type="range" min="0" max="200" step="1" :disabled="props.disabled || !props.hslEnabled" @input="emit('update:brightness', numberValue($event, 0, 200) / 100 - 1)" />
        <output>{{ Math.round((props.brightness + 1) * 100) }}%</output>
      </label>
    </section>

    <section class="visual-processing-card">
      <header class="visual-processing-card__header">
        <strong>动态缩放</strong>
        <label class="replica-switch">
          <input :checked="props.zoomEnabled" type="checkbox" :disabled="props.disabled" @change="emit('update:zoomEnabled', toggleValue($event))" />
          <span></span>
        </label>
      </header>
      <div class="visual-processing-field-label">模式</div>
      <div class="visual-processing-segmented visual-processing-segmented--three">
        <button v-for="option in zoomOptions" :key="option.key" type="button" :class="{ 'is-active': props.zoomMode === option.key }" :disabled="props.disabled || !props.zoomEnabled" @click="emit('update:zoomMode', option.key)">{{ option.label }}</button>
      </div>
      <div class="visual-processing-grid-fields">
        <label><span>最小缩放</span><input :value="props.zoomMinScale" type="number" min="1" max="1.5" step="0.01" :disabled="props.disabled || !props.zoomEnabled" @input="emit('update:zoomMinScale', numberValue($event, 1, 1.5))" /></label>
        <label><span>最大缩放</span><input :value="props.zoomMaxScale" type="number" min="1" max="1.5" step="0.01" :disabled="props.disabled || !props.zoomEnabled" @input="emit('update:zoomMaxScale', numberValue($event, 1, 1.5))" /></label>
        <label><span>最小时长(s)</span><input :value="props.zoomMinDurationSeconds" type="number" min="1" max="120" step="1" :disabled="props.disabled || !props.zoomEnabled" @input="emit('update:zoomMinDurationSeconds', numberValue($event, 1, 120))" /></label>
        <label><span>最大时长(s)</span><input :value="props.zoomMaxDurationSeconds" type="number" min="1" max="120" step="1" :disabled="props.disabled || !props.zoomEnabled" @input="emit('update:zoomMaxDurationSeconds', numberValue($event, 1, 120))" /></label>
      </div>
    </section>
  </div>
</template>
