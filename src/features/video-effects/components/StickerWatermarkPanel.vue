<script setup lang="ts">
import type { PipPosition, SubtitlePosition, SubtitleSize } from "../../../services/videoMixService";
import type {
  WatermarkAssetType,
  WatermarkRemovalSettings,
  WatermarkSettings,
  WatermarkTrajectory,
} from "../../watermark/types";
import ToolIcon from "../../../components/ToolIcon.vue";

const props = defineProps<{
  pipEnabled: boolean;
  pipOverlayFilePath: string | null;
  pipPosition: PipPosition;
  pipSizeRatio: number;
  pipOpacity: number;
  watermarkSettings: WatermarkSettings;
  watermarkEnabled: boolean;
  watermarkAssetType: WatermarkAssetType;
  watermarkImageFilePath: string | null;
  watermarkOpacity: number;
  watermarkImageSizeRatio: number;
  watermarkTrajectory: WatermarkTrajectory;
  watermarkRemovalSettings: WatermarkRemovalSettings;
  subtitleEnabled: boolean;
  subtitlePosition: SubtitlePosition;
  subtitleSize: SubtitleSize;
  disabled: boolean;
}>();

const emit = defineEmits<{
  selectPipOverlayFile: [];
  selectWatermarkAsset: [];
  "update:pipEnabled": [value: boolean];
  "update:pipPosition": [value: PipPosition];
  "update:pipSizeRatio": [value: number];
  "update:pipOpacity": [value: number];
  "update:watermarkTextEnabled": [value: boolean];
  "update:watermarkAssetEnabled": [value: boolean];
  "update:watermarkKind": [value: WatermarkSettings["kind"]];
  "update:watermarkText": [value: string];
  "update:watermarkPosition": [value: WatermarkSettings["position"]];
  "update:watermarkOpacity": [value: number];
  "update:watermarkTextFontSize": [value: number];
  "update:watermarkTextColor": [value: string];
  "update:watermarkRemovalEnabled": [value: boolean];
  "update:watermarkRemovalRegionCount": [value: number];
  "update:subtitleEnabled": [value: boolean];
  "update:subtitlePosition": [value: SubtitlePosition];
  "update:subtitleSize": [value: SubtitleSize];
  "update:watermarkAssetType": [value: WatermarkAssetType];
  "update:watermarkOpacityAsset": [value: number];
  "update:watermarkImageSizeRatio": [value: number];
  "update:watermarkTrajectory": [value: WatermarkTrajectory];
  autoDetectWatermark: [];
}>();

const watermarkColors = ["#ffffff", "#f4d22f", "#50d7b0", "#f08c57", "#76a9ff", "#ef6f91", "#111317", "#2cc7b1"];
const positions: Array<{ value: WatermarkSettings["position"]; label: string }> = [
  { value: "topLeft", label: "左上" },
  { value: "topRight", label: "右上" },
  { value: "center", label: "居中" },
  { value: "bottomLeft", label: "左下" },
  { value: "bottomRight", label: "右下" },
];
const trajectories: Array<{ value: WatermarkTrajectory; label: string }> = [
  { value: "static", label: "静止" },
  { value: "horizontal", label: "水平" },
  { value: "vertical", label: "垂直" },
  { value: "diagonal", label: "对角" },
  { value: "random", label: "随机" },
];

function fileName(path: string | null) {
  if (!path) return "未选择素材";
  return path.split(/[\\/]/).pop() ?? path;
}

function numberValue(event: Event, minimum: number, maximum: number) {
  const parsed = Number((event.target as HTMLInputElement).value);
  return Math.min(maximum, Math.max(minimum, Number.isFinite(parsed) ? parsed : minimum));
}

function checked(event: Event) {
  return (event.target as HTMLInputElement).checked;
}
</script>

<template>
  <button class="watermark-auto-detect" type="button" :disabled="props.disabled" @click="emit('autoDetectWatermark')">
    自动检测常见水印区域
  </button>
  <div class="replica-sticker-watermark-panel" aria-label="贴画与水印设置">
    <section class="replica-inline-effect-card">
      <header class="replica-inline-effect-card__header">
        <strong>添加贴画</strong>
        <label class="replica-switch">
          <input :checked="props.pipEnabled" type="checkbox" :disabled="props.disabled" @change="emit('update:pipEnabled', checked($event))" />
          <span></span>
        </label>
      </header>
      <div class="replica-inline-file-row">
        <span :title="props.pipOverlayFilePath ?? '未选择素材'">{{ fileName(props.pipOverlayFilePath) }}</span>
        <button type="button" :disabled="props.disabled" aria-label="选择贴画素材" @click="emit('selectPipOverlayFile')"><ToolIcon name="file" /></button>
      </div>
      <label class="replica-inline-range-row"><span>透明度</span><input :value="Math.round(props.pipOpacity * 100)" type="range" min="0" max="100" :disabled="props.disabled || !props.pipEnabled" @input="emit('update:pipOpacity', numberValue($event, 0, 100) / 100)" /><output>{{ Math.round(props.pipOpacity * 100) }}%</output></label>
      <label class="replica-inline-range-row"><span>大小</span><input :value="Math.round(props.pipSizeRatio * 100)" type="range" min="20" max="50" :disabled="props.disabled || !props.pipEnabled" @input="emit('update:pipSizeRatio', numberValue($event, 20, 50) / 100)" /><output>{{ Math.round(props.pipSizeRatio * 100) }}%</output></label>
      <label class="replica-inline-select-row"><span>位置</span><select :value="props.pipPosition" :disabled="props.disabled || !props.pipEnabled" @change="emit('update:pipPosition', ($event.target as HTMLSelectElement).value as PipPosition)"><option value="topLeft">左上</option><option value="topRight">右上</option><option value="bottomLeft">左下</option><option value="bottomRight">右下</option><option value="center">居中</option></select></label>
    </section>

    <section class="replica-inline-effect-card">
      <header class="replica-inline-effect-card__header">
        <strong>水印和花字</strong>
        <label class="replica-switch">
          <input :checked="props.watermarkSettings.enabled" type="checkbox" :disabled="props.disabled" @change="emit('update:watermarkTextEnabled', checked($event))" />
          <span></span>
        </label>
      </header>
      <div class="replica-inline-segmented replica-inline-segmented--two">
        <button type="button" :class="{ 'is-active': props.watermarkSettings.kind === 'text' }" :disabled="props.disabled" @click="emit('update:watermarkKind', 'text')">文字水印</button>
        <button type="button" :class="{ 'is-active': props.watermarkSettings.kind === 'text' }" :disabled="props.disabled" @click="emit('update:watermarkKind', 'text')">花字样式</button>
      </div>
      <label class="replica-inline-field"><span>水印文案</span><input :value="props.watermarkSettings.text" type="text" maxlength="80" placeholder="输入水印或花字文案" :disabled="props.disabled || props.watermarkSettings.kind !== 'text'" @input="emit('update:watermarkText', ($event.target as HTMLInputElement).value)" /></label>
      <div class="replica-inline-style-grid">
        <label><span>字号</span><input :value="props.watermarkSettings.textFontSize" type="number" min="16" max="120" step="2" :disabled="props.disabled || props.watermarkSettings.kind !== 'text'" @input="emit('update:watermarkTextFontSize', numberValue($event, 16, 120))" /></label>
        <label><span>透明度</span><input :value="Math.round(props.watermarkSettings.opacity * 100)" type="number" min="10" max="100" step="5" :disabled="props.disabled" @input="emit('update:watermarkOpacity', numberValue($event, 10, 100) / 100)" /></label>
      </div>
      <div class="replica-inline-color-row" aria-label="花字颜色">
        <button v-for="color in watermarkColors" :key="color" type="button" :class="{ 'is-active': props.watermarkSettings.textColor === color }" :style="{ color }" :disabled="props.disabled || props.watermarkSettings.kind !== 'text'" :aria-label="`选择颜色 ${color}`" @click="emit('update:watermarkTextColor', color)">A</button>
      </div>
      <label class="replica-inline-select-row"><span>位置</span><select :value="props.watermarkSettings.position" :disabled="props.disabled" @change="emit('update:watermarkPosition', ($event.target as HTMLSelectElement).value as WatermarkSettings['position'])"><option v-for="item in positions" :key="item.value" :value="item.value">{{ item.label }}</option></select></label>
    </section>

    <section class="replica-inline-effect-card">
      <header class="replica-inline-effect-card__header">
        <strong>图片/视频水印</strong>
        <label class="replica-switch">
          <input :checked="props.watermarkEnabled" type="checkbox" :disabled="props.disabled" @change="emit('update:watermarkAssetEnabled', checked($event))" />
          <span></span>
        </label>
      </header>
      <div class="replica-inline-file-row">
        <span :title="props.watermarkImageFilePath ?? '未选择素材'">{{ fileName(props.watermarkImageFilePath) }}</span>
        <button type="button" :disabled="props.disabled" aria-label="选择图片或视频水印" @click="emit('selectWatermarkAsset')"><ToolIcon name="file" /></button>
      </div>
      <div class="replica-inline-segmented replica-inline-segmented--two">
        <button type="button" :class="{ 'is-active': props.watermarkAssetType === 'image' }" :disabled="props.disabled" @click="emit('update:watermarkAssetType', 'image')">图片</button>
        <button type="button" :class="{ 'is-active': props.watermarkAssetType === 'video' }" :disabled="props.disabled" @click="emit('update:watermarkAssetType', 'video')">视频</button>
      </div>
      <label class="replica-inline-range-row"><span>透明度</span><input :value="Math.round(props.watermarkOpacity * 100)" type="range" min="10" max="100" :disabled="props.disabled || !props.watermarkEnabled" @input="emit('update:watermarkOpacityAsset', numberValue($event, 10, 100) / 100)" /><output>{{ Math.round(props.watermarkOpacity * 100) }}%</output></label>
      <label class="replica-inline-range-row"><span>大小</span><input :value="Math.round(props.watermarkImageSizeRatio * 100)" type="range" min="8" max="50" :disabled="props.disabled || !props.watermarkEnabled" @input="emit('update:watermarkImageSizeRatio', numberValue($event, 8, 50) / 100)" /><output>{{ Math.round(props.watermarkImageSizeRatio * 100) }}%</output></label>
      <div class="replica-inline-label">轨迹</div>
      <div class="replica-inline-trajectory-row"><button v-for="item in trajectories" :key="item.value" type="button" :class="{ 'is-active': props.watermarkTrajectory === item.value }" :disabled="props.disabled || !props.watermarkEnabled" @click="emit('update:watermarkTrajectory', item.value)">{{ item.label }}</button></div>
    </section>

    <section class="replica-inline-effect-card">
      <header class="replica-inline-effect-card__header">
        <strong>去除水印 / 字幕</strong>
        <label class="replica-switch">
          <input :checked="props.watermarkRemovalSettings.enabled" type="checkbox" :disabled="props.disabled" @change="emit('update:watermarkRemovalEnabled', checked($event))" />
          <span></span>
        </label>
      </header>
      <div class="replica-inline-label">区域来源：自动检测后可在预览画布中手动微调</div>
      <label class="replica-inline-number-row"><span>区域数</span><input :value="props.watermarkRemovalSettings.regionCount" type="number" min="1" max="8" :disabled="props.disabled || !props.watermarkRemovalSettings.enabled" @input="emit('update:watermarkRemovalRegionCount', numberValue($event, 1, 8))" /></label>
      <small class="replica-inline-help">启用后在预览画面中拖动红色框，区域数量与这里保持一致。</small>
      <div class="replica-inline-divider"></div>
      <label class="replica-inline-check"><input :checked="props.subtitleEnabled" type="checkbox" :disabled="props.disabled" @change="emit('update:subtitleEnabled', checked($event))" /><span>自动生成字幕</span></label>
      <div class="replica-inline-style-grid">
        <label><span>字幕位置</span><select :value="props.subtitlePosition" :disabled="props.disabled || !props.subtitleEnabled" @change="emit('update:subtitlePosition', ($event.target as HTMLSelectElement).value as SubtitlePosition)"><option value="top">顶部</option><option value="middle">居中</option><option value="bottom">底部</option></select></label>
        <label><span>字幕大小</span><select :value="props.subtitleSize" :disabled="props.disabled || !props.subtitleEnabled" @change="emit('update:subtitleSize', ($event.target as HTMLSelectElement).value as SubtitleSize)"><option value="small">小号</option><option value="medium">中号</option><option value="large">大号</option></select></label>
      </div>
    </section>
  </div>
</template>
