<script setup lang="ts">
import type { WatermarkKind, WatermarkPosition } from "../types";
import "../watermark.css";

defineProps<{
  enabled: boolean;
  kind: WatermarkKind;
  text: string;
  imageFilePath: string | null;
  position: WatermarkPosition;
  opacity: number;
  margin: number;
  textFontSize: number;
  textColor: string;
  imageSizeRatio: number;
  disabled: boolean;
}>();

const emit = defineEmits<{
  selectImage: [];
  "update:enabled": [value: boolean];
  "update:kind": [value: WatermarkKind];
  "update:text": [value: string];
  "update:position": [value: WatermarkPosition];
  "update:opacity": [value: number];
  "update:margin": [value: number];
  "update:textFontSize": [value: number];
  "update:textColor": [value: string];
  "update:imageSizeRatio": [value: number];
}>();

const positions: Array<{ key: WatermarkPosition; label: string }> = [
  { key: "topLeft", label: "左上" },
  { key: "topRight", label: "右上" },
  { key: "center", label: "居中" },
  { key: "bottomLeft", label: "左下" },
  { key: "bottomRight", label: "右下" },
];

function fileName(path: string | null) {
  if (!path) return "尚未选择图片";
  return path.split(/[\\/]/).pop() ?? path;
}

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
</script>

<template>
  <section class="watermark-panel" aria-label="添加水印设置">
    <label class="option-toggle">
      <input :checked="enabled" type="checkbox" :disabled="disabled" @change="emit('update:enabled', checked($event))" />
      <span>为导出视频添加水印</span>
    </label>

    <div class="watermark-kind" aria-label="水印类型">
      <button
        type="button"
        :class="{ 'watermark-kind__button--active': kind === 'text' }"
        :aria-pressed="kind === 'text'"
        :disabled="disabled"
        @click="emit('update:kind', 'text')"
      >文字水印</button>
      <button
        type="button"
        :class="{ 'watermark-kind__button--active': kind === 'image' }"
        :aria-pressed="kind === 'image'"
        :disabled="disabled"
        @click="emit('update:kind', 'image')"
      >图片水印</button>
    </div>

    <div class="watermark-preview" aria-label="水印位置预览">
      <span
        class="watermark-preview__mark"
        :class="`watermark-preview__mark--${position}`"
      >{{ kind === "text" ? text.trim() || "文字水印" : "图片水印" }}</span>
    </div>

    <label v-if="kind === 'text'" class="field">
      <span>水印文字</span>
      <input
        :value="text"
        type="text"
        maxlength="80"
        placeholder="例如：品牌名称 / 账号名称"
        :disabled="disabled"
        @input="emit('update:text', value($event))"
      />
      <small>{{ text.length }}/80</small>
    </label>

    <template v-else>
      <p class="output-path">{{ fileName(imageFilePath) }}</p>
      <button class="ghost-button ghost-button--full" type="button" :disabled="disabled" @click="emit('selectImage')">
        选择图片水印
      </button>
    </template>

    <fieldset class="watermark-position" :disabled="disabled">
      <legend>显示位置</legend>
      <button
        v-for="item in positions"
        :key="item.key"
        type="button"
        :aria-pressed="position === item.key"
        :class="{ 'watermark-position__button--active': position === item.key }"
        @click="emit('update:position', item.key)"
      >{{ item.label }}</button>
    </fieldset>

    <div class="watermark-fields">
      <label class="field">
        <span>透明度</span>
        <input
          :value="Math.round(opacity * 100)"
          type="number"
          min="10"
          max="100"
          step="5"
          :disabled="disabled"
          @input="emit('update:opacity', numberValue($event, 10, 100) / 100)"
        />
        <small>10%～100%</small>
      </label>
      <label class="field">
        <span>边距</span>
        <input
          :value="margin"
          type="number"
          min="0"
          max="240"
          step="4"
          :disabled="disabled"
          @input="emit('update:margin', numberValue($event, 0, 240))"
        />
        <small>距离画面边缘</small>
      </label>
    </div>

    <div v-if="kind === 'text'" class="watermark-fields">
      <label class="field">
        <span>字号</span>
        <input
          :value="textFontSize"
          type="number"
          min="16"
          max="120"
          step="2"
          :disabled="disabled"
          @input="emit('update:textFontSize', numberValue($event, 16, 120))"
        />
      </label>
      <label class="field">
        <span>文字颜色</span>
        <input :value="textColor" type="color" :disabled="disabled" @input="emit('update:textColor', value($event))" />
      </label>
    </div>

    <label v-else class="field">
      <span>图片宽度</span>
      <input
        :value="Math.round(imageSizeRatio * 100)"
        type="number"
        min="8"
        max="50"
        step="1"
        :disabled="disabled"
        @input="emit('update:imageSizeRatio', numberValue($event, 8, 50) / 100)"
      />
      <small>占成片宽度的8%～50%</small>
    </label>

    <p class="watermark-help">水印会应用到基础导出、普通混剪、批量视频和AI配音成片。</p>
  </section>
</template>
