<script setup lang="ts">
import type {
  WatermarkPosition,
  WatermarkRemovalMode,
  WatermarkRemovalSize,
  WatermarkTrackingKeyframe,
} from "../types";
import MovingWatermarkTracker from "./MovingWatermarkTracker.vue";

const props = defineProps<{
  enabled: boolean;
  mode: WatermarkRemovalMode;
  position: WatermarkPosition;
  size: WatermarkRemovalSize;
  margin: number;
  strength: number;
  coverColor: string;
  coverOpacity: number;
  previewUrl: string | null;
  trackingEnabled: boolean;
  trackingRegionWidthRatio: number;
  trackingRegionHeightRatio: number;
  trackingKeyframes: WatermarkTrackingKeyframe[];
  disabled: boolean;
}>();

const emit = defineEmits<{
  "update:enabled": [value: boolean];
  "update:mode": [value: WatermarkRemovalMode];
  "update:position": [value: WatermarkPosition];
  "update:size": [value: WatermarkRemovalSize];
  "update:margin": [value: number];
  "update:strength": [value: number];
  "update:coverColor": [value: string];
  "update:coverOpacity": [value: number];
  "update:trackingEnabled": [value: boolean];
  "update:trackingRegionWidthRatio": [value: number];
  "update:trackingRegionHeightRatio": [value: number];
  "update:trackingKeyframes": [value: WatermarkTrackingKeyframe[]];
  autoDetect: [];
}>();

const modes: Array<{ key: WatermarkRemovalMode; label: string; note: string }> = [
  { key: "crop", label: "边缘裁剪", note: "裁掉顶部或底部" },
  { key: "delogo", label: "融合修复", note: "用周围像素填补" },
  { key: "blur", label: "区域模糊", note: "柔化文字和图标" },
  { key: "mosaic", label: "马赛克", note: "像素化遮挡内容" },
  { key: "cover", label: "色块遮盖", note: "最稳定的完全覆盖" },
];

const positions: Array<{ key: WatermarkPosition; label: string }> = [
  { key: "topLeft", label: "左上" },
  { key: "topRight", label: "右上" },
  { key: "center", label: "居中" },
  { key: "bottomLeft", label: "左下" },
  { key: "bottomRight", label: "右下" },
];

const sizes: Array<{ key: WatermarkRemovalSize; label: string }> = [
  { key: "small", label: "小范围" },
  { key: "medium", label: "中范围" },
  { key: "large", label: "大范围" },
];

function checked(event: Event) {
  return (event.target as HTMLInputElement).checked;
}

function value(event: Event) {
  return (event.target as HTMLInputElement).value;
}

function numberValue(event: Event, minimum: number, maximum: number) {
  const parsed = Number((event.target as HTMLInputElement).value);
  return Math.min(maximum, Math.max(minimum, Number.isFinite(parsed) ? parsed : minimum));
}

function selectMode(mode: WatermarkRemovalMode) {
  if (mode === "crop" && props.position === "center") {
    emit("update:position", "topRight");
  }
  emit("update:mode", mode);
}

function toggleTracking(enabled: boolean) {
  if (enabled && !["blur", "mosaic", "cover"].includes(props.mode)) {
    emit("update:mode", "mosaic");
  }
  emit("update:trackingEnabled", enabled);
}

function modeDisabled(mode: WatermarkRemovalMode) {
  return props.disabled || (props.trackingEnabled && (mode === "crop" || mode === "delogo"));
}

function positionDisabled(position: WatermarkPosition) {
  return props.disabled || (props.mode === "crop" && position === "center");
}
</script>

<template>
  <button class="watermark-auto-detect" type="button" :disabled="disabled || !previewUrl" @click="emit('autoDetect')">
    自动检测常见水印区域
  </button>
  <p class="watermark-removal-note">分析三帧画面的静态标记，生成可继续拖动调整的处理区域。</p>
  <section class="watermark-removal-panel" aria-label="处理原水印设置">
    <label class="option-toggle">
      <input :checked="enabled" type="checkbox" :disabled="disabled" @change="emit('update:enabled', checked($event))" />
      <span>处理画面中的原水印</span>
    </label>

    <p class="watermark-removal-note">
      这是传统区域处理，适合固定在角落或中间的水印；不会自动识别，也不是AI无痕复原。
    </p>

    <div class="watermark-removal-modes" aria-label="处理方式">
      <button
        v-for="item in modes"
        :key="item.key"
        type="button"
        :aria-pressed="mode === item.key"
        :class="{ 'watermark-removal-mode--active': mode === item.key }"
        :disabled="modeDisabled(item.key)"
        @click="selectMode(item.key)"
      >
        <strong>{{ item.label }}</strong>
        <small>{{ item.note }}</small>
      </button>
    </div>

    <label class="option-toggle watermark-tracking-toggle">
      <input
        :checked="trackingEnabled"
        type="checkbox"
        :disabled="disabled"
        @change="toggleTracking(checked($event))"
      />
      <span>移动水印关键帧跟踪</span>
    </label>

    <p v-if="trackingEnabled" class="watermark-removal-note">
      第一版用于处理当前完整视频，画布比例请使用“原画”，支持模糊、马赛克和色块遮盖。请先导出清理后的视频，再导入混剪。
    </p>

    <MovingWatermarkTracker
      v-if="trackingEnabled"
      :preview-url="previewUrl"
      :keyframes="trackingKeyframes"
      :region-width-ratio="trackingRegionWidthRatio"
      :region-height-ratio="trackingRegionHeightRatio"
      :disabled="disabled"
      @update:keyframes="emit('update:trackingKeyframes', $event)"
      @update:region-width-ratio="emit('update:trackingRegionWidthRatio', $event)"
      @update:region-height-ratio="emit('update:trackingRegionHeightRatio', $event)"
    />

    <div v-else class="watermark-removal-preview" aria-label="处理区域预览">
      <span
        class="watermark-removal-preview__region"
        :class="[
          `watermark-removal-preview__region--${position}`,
          `watermark-removal-preview__region--${size}`,
          `watermark-removal-preview__region--${mode}`,
        ]"
      >{{ mode === "crop" ? "裁剪区域" : "处理区域" }}</span>
    </div>

    <p v-if="!trackingEnabled && mode === 'crop'" class="watermark-help">
      裁剪会移除水印所在的整条顶部或底部区域，再保持画面比例自动放大铺满；适合贴近上下边缘的固定水印。
    </p>

    <fieldset v-if="!trackingEnabled" class="watermark-position" :disabled="disabled">
      <legend>原水印位置</legend>
      <button
        v-for="item in positions"
        :key="item.key"
        type="button"
        :aria-pressed="position === item.key"
        :class="{ 'watermark-position__button--active': position === item.key }"
        :disabled="positionDisabled(item.key)"
        @click="emit('update:position', item.key)"
      >{{ item.label }}</button>
    </fieldset>

    <div v-if="!trackingEnabled" class="watermark-removal-sizes" aria-label="处理范围">
      <button
        v-for="item in sizes"
        :key="item.key"
        type="button"
        :aria-pressed="size === item.key"
        :class="{ 'watermark-removal-size--active': size === item.key }"
        :disabled="disabled"
        @click="emit('update:size', item.key)"
      >{{ item.label }}</button>
    </div>

    <div v-if="!trackingEnabled || mode === 'blur' || mode === 'mosaic'" class="watermark-fields">
      <label v-if="!trackingEnabled" class="field">
        <span>画面边距</span>
        <input
          :value="margin"
          type="number"
          min="0"
          max="160"
          step="4"
          :disabled="disabled"
          @input="emit('update:margin', numberValue($event, 0, 160))"
        />
      </label>
      <label v-if="mode === 'blur' || mode === 'mosaic'" class="field">
        <span>处理强度</span>
        <input
          :value="strength"
          type="number"
          min="4"
          max="24"
          step="2"
          :disabled="disabled"
          @input="emit('update:strength', numberValue($event, 4, 24))"
        />
      </label>
    </div>

    <div v-if="mode === 'cover'" class="watermark-fields">
      <label class="field">
        <span>遮盖颜色</span>
        <input :value="coverColor" type="color" :disabled="disabled" @input="emit('update:coverColor', value($event))" />
      </label>
      <label class="field">
        <span>遮盖透明度</span>
        <input
          :value="Math.round(coverOpacity * 100)"
          type="number"
          min="10"
          max="100"
          step="5"
          :disabled="disabled"
          @input="emit('update:coverOpacity', numberValue($event, 10, 100) / 100)"
        />
      </label>
    </div>
  </section>
</template>
