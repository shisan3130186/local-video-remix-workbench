<script setup lang="ts">
import { ref } from "vue";
import type { DynamicZoomMode, RotationMode } from "../../services/videoMixService";
import type { ToolKey } from "../../types/workbench";
import { readChecked, readNumber, readSelectedValue } from "./inputHelpers";

const props = defineProps<{
  activeTool: ToolKey;
  applyHorizontalMirror: boolean;
  applyVerticalMirror: boolean;
  playbackSpeed: number;
  rotationMode: RotationMode;
  brightness: number;
  contrast: number;
  saturation: number;
  effectScale: number;
  zoomEnabled?: boolean;
  zoomMode?: DynamicZoomMode;
  zoomMinScale?: number;
  zoomMaxScale?: number;
  zoomMinDurationSeconds?: number;
  zoomMaxDurationSeconds?: number;
  isMixing: boolean;
  isBatchMixing: boolean;
}>();

const emit = defineEmits<{
  "update:applyHorizontalMirror": [value: boolean];
  "update:applyVerticalMirror": [value: boolean];
  "update:playbackSpeed": [value: number];
  "update:rotationMode": [value: RotationMode];
  "update:brightness": [value: number];
  "update:contrast": [value: number];
  "update:saturation": [value: number];
  "update:effectScale": [value: number];
  "update:zoomEnabled": [value: boolean];
  "update:zoomMode": [value: DynamicZoomMode];
  "update:zoomMinScale": [value: number];
  "update:zoomMaxScale": [value: number];
  "update:zoomMinDurationSeconds": [value: number];
  "update:zoomMaxDurationSeconds": [value: number];
  reset: [];
}>();

const enabled = ref(false);
const sharpness = ref(5);
const noiseReduction = ref(0);
const temperature = ref(6500);
const visualStyle = ref("random");
const glow = ref(false);
const grain = ref(false);
const vignette = ref(false);
const speedMode = ref<"global" | "segment">("global");
const speedMin = ref(Math.min(0.96, props.playbackSpeed));
const speedMax = ref(Math.max(1.06, props.playbackSpeed));
const segmentMin = ref(2);
const segmentMax = ref(4);
const zoomMode = ref<DynamicZoomMode>(props.zoomMode ?? "push");
const zoomMinScale = ref(props.zoomMinScale ?? 1.02);
const zoomMaxScale = ref(props.zoomMaxScale ?? 1.08);
const zoomMinDuration = ref(props.zoomMinDurationSeconds ?? 5);
const zoomMaxDuration = ref(props.zoomMaxDurationSeconds ?? 10);

function syncSpeed() {
  emit("update:playbackSpeed", speedMin.value === speedMax.value ? speedMin.value : 1);
}
</script>

<template>
  <section class="replica-parameter-panel replica-effect-panel">
    <header class="replica-parameter-panel__header">
      <h2>{{ activeTool === 'zoom' ? '动态缩放' : activeTool === 'speed' ? '视频变速' : activeTool === 'adjust' || activeTool === 'effects' ? '画面调整' : '旋转换像' }} - 参数设置</h2>
      <div class="replica-parameter-panel__actions">
        <button class="replica-parameter-button" type="button" @click="emit('reset')">重置</button>
        <button class="replica-parameter-button replica-parameter-button--enable" :class="{ 'is-enabled': enabled }" type="button" @click="enabled = !enabled">{{ enabled ? '已启用' : '启用' }}</button>
      </div>
    </header>

    <template v-if="activeTool === 'mirror' || activeTool === 'rotate'">
      <div class="replica-effect-section-title">视频镜像</div>
      <label class="replica-effect-check"><input :checked="props.applyHorizontalMirror" type="checkbox" @change="emit('update:applyHorizontalMirror', readChecked($event))" /><span>启用镜像效果</span></label>
      <label class="replica-effect-check"><input type="checkbox" /><span>启用黑边裁剪</span></label>
      <div class="replica-effect-section-title">旋转翻转</div>
      <label class="replica-effect-check"><input :checked="props.applyVerticalMirror" type="checkbox" @change="emit('update:applyVerticalMirror', readChecked($event))" /><span>垂直翻转</span></label>
      <div class="replica-effect-segmented">
        <button type="button" :class="{ 'is-active': props.rotationMode === 'counterclockwise90' }" @click="emit('update:rotationMode', 'counterclockwise90')">左旋90°</button>
        <button type="button" :class="{ 'is-active': props.rotationMode === 'clockwise90' }" @click="emit('update:rotationMode', 'clockwise90')">右旋90°</button>
      </div>
      <div class="replica-effect-row"><strong>随机偏移角度</strong><label class="replica-range-field"><input type="number" value="-2" /></label><em>—</em><label class="replica-range-field"><input type="number" value="2" /></label><span>°</span></div>
    </template>

    <template v-else-if="activeTool === 'adjust' || activeTool === 'effects'">
      <div class="replica-effect-grid">
        <label>对比度<input :value="props.contrast" type="number" min="0.5" max="2" step="0.01" @input="emit('update:contrast', readNumber($event))" /></label>
        <label>饱和度<input :value="props.saturation" type="number" min="0.5" max="2" step="0.01" @input="emit('update:saturation', readNumber($event))" /></label>
        <label>锐化强度<input v-model.number="sharpness" type="number" min="0" max="100" /></label>
        <label>亮度调整<input :value="props.brightness" type="number" min="-100" max="100" @input="emit('update:brightness', readNumber($event))" /></label>
        <label>降噪强度<input v-model.number="noiseReduction" type="number" min="0" max="100" /></label>
        <label>色温调整<input v-model.number="temperature" type="number" min="1000" max="12000" /></label>
      </div>
      <label class="replica-effect-select">画面风格<select v-model="visualStyle"><option value="random">随机选择</option><option value="bw">黑白风格</option><option value="invert">反向风格</option><option value="retro">复古风格</option><option value="cross">交叉风格</option><option value="cartoon">卡通风格</option><option value="emboss">浮雕风格</option><option value="pixel">像素画风格</option><option value="outline">轮廓风格</option></select></label>
      <div class="replica-effect-section-title">可组合效果</div>
      <div class="replica-effect-checks"><label class="replica-effect-check"><input v-model="glow" type="checkbox" /><span>光晕</span></label><label class="replica-effect-check"><input v-model="grain" type="checkbox" /><span>颗粒</span></label><label class="replica-effect-check"><input v-model="vignette" type="checkbox" /><span>暗角</span></label></div>
    </template>

    <template v-else-if="activeTool === 'speed'">
      <div class="replica-effect-section-title">变速类型</div>
      <div class="replica-effect-segmented"><button type="button" :class="{ 'is-active': speedMode === 'global' }" @click="speedMode = 'global'">全局变速</button><button type="button" :class="{ 'is-active': speedMode === 'segment' }" @click="speedMode = 'segment'">分段变速</button></div>
      <div class="replica-effect-row"><strong>变速倍率</strong><label class="replica-range-field"><input v-model.number="speedMin" type="number" min="0.25" max="4" step="0.01" @change="syncSpeed" /></label><em>—</em><label class="replica-range-field"><input v-model.number="speedMax" type="number" min="0.25" max="4" step="0.01" @change="syncSpeed" /></label><small>相同则固定倍率</small></div>
      <div v-if="speedMode === 'segment'" class="replica-effect-row"><strong>分段时长</strong><label class="replica-range-field"><input v-model.number="segmentMin" type="number" min="1" max="120" /></label><em>—</em><label class="replica-range-field"><input v-model.number="segmentMax" type="number" min="1" max="120" /></label><span>秒</span><small>相同则固定时间</small></div>
      <p class="replica-parameter-help">全局变速会对视频进行整体变速，分段变速会在你设置时间范围内随机分段，并对每个分段使用随机变速倍率，最后完整导出。</p>
    </template>

    <template v-else-if="activeTool === 'zoom'">
      <div class="replica-effect-section-title">缩放模式</div>
      <div class="replica-effect-segmented"><button type="button" :class="{ 'is-active': zoomMode === 'push' }" @click="zoomMode = 'push'; emit('update:zoomMode', zoomMode)">画面推进</button><button type="button" :class="{ 'is-active': zoomMode === 'pull' }" @click="zoomMode = 'pull'; emit('update:zoomMode', zoomMode)">画面拉出</button><button type="button" :class="{ 'is-active': zoomMode === 'random' }" @click="zoomMode = 'random'; emit('update:zoomMode', zoomMode)">随机轨迹</button></div>
      <div class="replica-effect-row"><strong>缩放倍率</strong><label class="replica-range-field"><input v-model.number="zoomMinScale" type="number" min="1" max="1.5" step="0.01" @change="emit('update:zoomMinScale', zoomMinScale)" /></label><em>—</em><label class="replica-range-field"><input v-model.number="zoomMaxScale" type="number" min="1" max="1.5" step="0.01" @change="emit('update:zoomMaxScale', zoomMaxScale)" /></label><small>相同则固定倍率</small></div>
      <div class="replica-effect-row"><strong>持续时间</strong><label class="replica-range-field"><input v-model.number="zoomMinDuration" type="number" min="1" max="120" @change="emit('update:zoomMinDurationSeconds', zoomMinDuration)" /></label><em>—</em><label class="replica-range-field"><input v-model.number="zoomMaxDuration" type="number" min="1" max="120" @change="emit('update:zoomMaxDurationSeconds', zoomMaxDuration)" /></label><span>秒</span><small>相同则固定时间</small></div>
      <p class="replica-parameter-help">动态缩放可实现类似运镜的效果。推进 = 镜头推进，拉出 = 镜头拉远，随机轨迹 = 画面随机移动。</p>
    </template>
  </section>
</template>
