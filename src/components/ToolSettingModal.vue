<script setup lang="ts">
import type {
  CanvasAspectRatio,
  CanvasBackgroundMode,
  PipPosition,
  RotationMode,
} from "../services/videoMixService";
import type { ToolKey } from "../types/workbench";

const props = defineProps<{
  activeTool: ToolKey | null;
  isSplitting: boolean;
  isMixing: boolean;
  isBatchMixing: boolean;
  isExporting: boolean;
  splitError: string | null;
  splitOutputDirectory: string | null;
  splitSegmentCount: number | null;
  randomPickError: string | null;
  outputDirectory: string | null;
  outputDirectoryError: string | null;
  segmentDurationSeconds: number;
  randomPickCount: number;
  batchGenerateCount: number;
  canvasAspectRatio: CanvasAspectRatio;
  canvasBackgroundMode: CanvasBackgroundMode;
  applyHorizontalMirror: boolean;
  applyVerticalMirror: boolean;
  smoothRemixEnabled: boolean;
  playbackSpeed: number;
  rotationMode: RotationMode;
  brightness: number;
  contrast: number;
  saturation: number;
  effectScale: number;
  pipEnabled: boolean;
  pipOverlayFilePath: string | null;
  pipPosition: PipPosition;
  pipSizeRatio: number;
  pipOpacity: number;
  pipMargin: number;
  bgmEnabled: boolean;
  bgmAudioFilePath: string | null;
  originalVolume: number;
  bgmVolume: number;
  bgmFadeInSeconds: number;
  bgmFadeOutSeconds: number;
  selectedCoverUrl: string | null;
  coverFrameSeconds: number;
  isGeneratingCover: boolean;
  coverError: string | null;
}>();

const emit = defineEmits<{
  close: [];
  reset: [tool: ToolKey];
  splitSelectedVideo: [];
  pickSegmentsRandomly: [];
  concatRandomSegments: [];
  generateBatchMixes: [];
  selectOutputDirectory: [];
  openOutputDirectory: [];
  exportSelectedVideo: [];
  selectPipOverlayFile: [];
  selectBgmAudioFile: [];
  generateCoverFrame: [];
  "update:segmentDurationSeconds": [value: number];
  "update:randomPickCount": [value: number];
  "update:batchGenerateCount": [value: number];
  "update:canvasAspectRatio": [value: CanvasAspectRatio];
  "update:canvasBackgroundMode": [value: CanvasBackgroundMode];
  "update:applyHorizontalMirror": [value: boolean];
  "update:applyVerticalMirror": [value: boolean];
  "update:smoothRemixEnabled": [value: boolean];
  "update:playbackSpeed": [value: number];
  "update:rotationMode": [value: RotationMode];
  "update:brightness": [value: number];
  "update:contrast": [value: number];
  "update:saturation": [value: number];
  "update:effectScale": [value: number];
  "update:pipEnabled": [value: boolean];
  "update:pipPosition": [value: PipPosition];
  "update:pipSizeRatio": [value: number];
  "update:pipOpacity": [value: number];
  "update:pipMargin": [value: number];
  "update:bgmEnabled": [value: boolean];
  "update:originalVolume": [value: number];
  "update:bgmVolume": [value: number];
  "update:bgmFadeInSeconds": [value: number];
  "update:bgmFadeOutSeconds": [value: number];
  "update:coverFrameSeconds": [value: number];
}>();

const titles: Record<ToolKey, string> = {
  remix: "混剪设置",
  canvas: "画布设置",
  audio: "音频设置",
  bgm: "背景音乐",
  tts: "语音合成",
  subtitleStyle: "字幕样式",
  watermark: "去除水印",
  cover: "视频封面",
  entrance: "入场效果",
  frame: "视频帧操作",
  effects: "视频效果",
  transition: "平滑转场",
  pip: "画中画",
  adjust: "画面调整",
  fusion: "镜像融合",
  rotate: "旋转镜像",
  mirror: "镜像旋转",
  speed: "视频变速",
  zoom: "动态缩放",
  subtitles: "字幕设置",
  export: "导出设置",
};

function updateNumber(
  event: Event,
  name:
    | "segmentDurationSeconds"
    | "randomPickCount"
    | "batchGenerateCount"
    | "playbackSpeed"
    | "brightness"
    | "contrast"
    | "saturation"
    | "effectScale"
    | "pipSizeRatio"
    | "pipOpacity"
    | "pipMargin"
    | "originalVolume"
    | "bgmVolume"
    | "bgmFadeInSeconds"
    | "bgmFadeOutSeconds"
    | "coverFrameSeconds",
) {
  const value = Number((event.target as HTMLInputElement).value);
  if (name === "segmentDurationSeconds") {
    emit("update:segmentDurationSeconds", value);
    return;
  }

  if (name === "randomPickCount") {
    emit("update:randomPickCount", value);
    return;
  }

  if (name === "batchGenerateCount") {
    emit("update:batchGenerateCount", value);
    return;
  }

  if (name === "playbackSpeed") {
    emit("update:playbackSpeed", value);
    return;
  }

  if (name === "brightness") {
    emit("update:brightness", value);
    return;
  }

  if (name === "contrast") {
    emit("update:contrast", value);
    return;
  }

  if (name === "saturation") {
    emit("update:saturation", value);
    return;
  }

  if (name === "effectScale") {
    emit("update:effectScale", value);
    return;
  }

  if (name === "pipSizeRatio") {
    emit("update:pipSizeRatio", clampNumber(value, 0.2, 0.5));
    return;
  }

  if (name === "pipOpacity") {
    emit("update:pipOpacity", clampNumber(value, 0, 1));
    return;
  }

  if (name === "originalVolume") {
    emit("update:originalVolume", clampNumber(value, 0, 2));
    return;
  }

  if (name === "bgmVolume") {
    emit("update:bgmVolume", clampNumber(value, 0, 2));
    return;
  }

  if (name === "bgmFadeInSeconds") {
    emit("update:bgmFadeInSeconds", clampNumber(value, 0, 10));
    return;
  }

  if (name === "bgmFadeOutSeconds") {
    emit("update:bgmFadeOutSeconds", clampNumber(value, 0, 10));
    return;
  }

  if (name === "coverFrameSeconds") {
    emit("update:coverFrameSeconds", value);
    return;
  }

  emit("update:pipMargin", clampNumber(value, 0, 240));
}

function clampNumber(value: number, min: number, max: number) {
  if (!Number.isFinite(value)) {
    return min;
  }

  return Math.min(max, Math.max(min, value));
}

function updateCheckbox(
  event: Event,
  name:
    | "applyHorizontalMirror"
    | "applyVerticalMirror"
    | "smoothRemixEnabled"
    | "pipEnabled"
    | "bgmEnabled",
) {
  const value = (event.target as HTMLInputElement).checked;

  if (name === "applyHorizontalMirror") {
    emit("update:applyHorizontalMirror", value);
    return;
  }

  if (name === "applyVerticalMirror") {
    emit("update:applyVerticalMirror", value);
    return;
  }

  if (name === "smoothRemixEnabled") {
    emit("update:smoothRemixEnabled", value);
    return;
  }

  if (name === "pipEnabled") {
    emit("update:pipEnabled", value);
    return;
  }

  emit("update:bgmEnabled", value);
}

function updateSelect(
  event: Event,
  name: "canvasAspectRatio" | "canvasBackgroundMode" | "rotationMode" | "pipPosition",
) {
  const value = (event.target as HTMLSelectElement).value;

  if (name === "canvasAspectRatio") {
    emit("update:canvasAspectRatio", value as CanvasAspectRatio);
    return;
  }

  if (name === "canvasBackgroundMode") {
    emit("update:canvasBackgroundMode", value as CanvasBackgroundMode);
    return;
  }

  if (name === "rotationMode") {
    emit("update:rotationMode", value as RotationMode);
    return;
  }

  emit("update:pipPosition", value as PipPosition);
}

function formatFileName(filePath: string | null) {
  return filePath?.split(/[\\/]/).pop() ?? "未选择文件";
}
</script>

<template>
  <div v-if="activeTool" class="modal-backdrop" @click.self="$emit('close')">
    <section class="tool-modal" role="dialog" aria-modal="true">
      <header class="tool-modal__header">
        <div>
          <p class="panel__label">二级设置</p>
          <h2>{{ titles[activeTool] }}</h2>
        </div>
        <button class="panel-toggle" type="button" @click="$emit('close')">关闭</button>
      </header>

      <div class="tool-modal__body">
        <template v-if="activeTool === 'remix'">
          <label class="field">
            <span>切片秒数</span>
            <input
              :value="segmentDurationSeconds"
              type="number"
              min="1"
              step="1"
              :disabled="isSplitting"
              @input="updateNumber($event, 'segmentDurationSeconds')"
            />
          </label>
          <button class="primary-button primary-button--full" type="button" :disabled="isSplitting" @click="$emit('splitSelectedVideo')">
            {{ isSplitting ? "正在切片..." : "切片当前视频" }}
          </button>
          <p v-if="splitError" class="error-text">{{ splitError }}</p>
          <p v-else-if="splitOutputDirectory" class="success-text">已生成 {{ splitSegmentCount }} 个片段。</p>

          <div class="divider"></div>
          <label class="field">
            <span>抽取数量</span>
            <input :value="randomPickCount" type="number" min="1" step="1" @input="updateNumber($event, 'randomPickCount')" />
          </label>
          <button class="ghost-button ghost-button--full" type="button" @click="$emit('pickSegmentsRandomly')">随机抽取</button>
          <p v-if="randomPickError" class="error-text">{{ randomPickError }}</p>

          <div class="divider"></div>
          <label class="field">
            <span>批量生成数量</span>
            <input
              :value="batchGenerateCount"
              type="number"
              min="1"
              step="1"
              :disabled="isBatchMixing"
              @input="updateNumber($event, 'batchGenerateCount')"
            />
          </label>
          <button class="primary-button primary-button--full" type="button" :disabled="isBatchMixing" @click="$emit('generateBatchMixes')">
            {{ isBatchMixing ? "正在批量生成..." : "批量生成混剪" }}
          </button>
        </template>

        <template v-else-if="activeTool === 'canvas'">
          <label class="field">
            <span>视频比例</span>
            <select :value="canvasAspectRatio" @change="updateSelect($event, 'canvasAspectRatio')">
              <option value="original">原画</option>
              <option value="portrait916">9:16 竖屏</option>
              <option value="square11">1:1 方屏</option>
              <option value="landscape169">16:9 横屏</option>
            </select>
          </label>
          <label class="field">
            <span>背景方式</span>
            <select
              :value="canvasBackgroundMode"
              :disabled="canvasAspectRatio === 'original'"
              @change="updateSelect($event, 'canvasBackgroundMode')"
            >
              <option value="black">黑边</option>
              <option value="blur">模糊背景</option>
            </select>
          </label>
        </template>

        <template v-else-if="activeTool === 'transition'">
          <label class="option-toggle">
            <input
              :checked="smoothRemixEnabled"
              type="checkbox"
              @change="updateCheckbox($event, 'smoothRemixEnabled')"
            />
            <span>平滑混剪</span>
          </label>
          <p class="empty-text">开启后，拼接和批量生成会应用 0.2 秒淡入淡出，并过滤小于 1.5 秒的片段。</p>
        </template>

        <template v-else-if="activeTool === 'pip'">
          <label class="option-toggle">
            <input
              :checked="pipEnabled"
              type="checkbox"
              @change="updateCheckbox($event, 'pipEnabled')"
            />
            <span>启用画中画</span>
          </label>
          <p class="output-path">{{ formatFileName(pipOverlayFilePath) }}</p>
          <button class="ghost-button ghost-button--full" type="button" @click="$emit('selectPipOverlayFile')">
            选择叠加视频 / 图片
          </button>
          <label class="field">
            <span>位置</span>
            <select :value="pipPosition" @change="updateSelect($event, 'pipPosition')">
              <option value="topLeft">左上</option>
              <option value="topRight">右上</option>
              <option value="bottomLeft">左下</option>
              <option value="bottomRight">右下</option>
              <option value="center">居中</option>
            </select>
          </label>
          <label class="field">
            <span>大小比例</span>
            <input
              :value="pipSizeRatio"
              type="number"
              min="0.2"
              max="0.5"
              step="0.05"
              @input="updateNumber($event, 'pipSizeRatio')"
            />
          </label>
          <label class="field">
            <span>透明度</span>
            <input
              :value="pipOpacity"
              type="number"
              min="0"
              max="1"
              step="0.05"
              @input="updateNumber($event, 'pipOpacity')"
            />
          </label>
          <label class="field">
            <span>边距</span>
            <input
              :value="pipMargin"
              type="number"
              min="0"
              max="240"
              step="4"
              @input="updateNumber($event, 'pipMargin')"
            />
          </label>
        </template>

        <template v-else-if="activeTool === 'bgm' || activeTool === 'audio'">
          <label class="option-toggle">
            <input
              :checked="bgmEnabled"
              type="checkbox"
              @change="updateCheckbox($event, 'bgmEnabled')"
            />
            <span>启用 BGM</span>
          </label>
          <p class="output-path">{{ formatFileName(bgmAudioFilePath) }}</p>
          <button class="ghost-button ghost-button--full" type="button" @click="$emit('selectBgmAudioFile')">
            选择本地音乐
          </button>
          <label class="field">
            <span>原视频音量</span>
            <input
              :value="originalVolume"
              type="number"
              min="0"
              max="2"
              step="0.05"
              :disabled="isMixing || isBatchMixing"
              @input="updateNumber($event, 'originalVolume')"
            />
          </label>
          <label class="field">
            <span>BGM 音量</span>
            <input
              :value="bgmVolume"
              type="number"
              min="0"
              max="2"
              step="0.05"
              :disabled="isMixing || isBatchMixing"
              @input="updateNumber($event, 'bgmVolume')"
            />
          </label>
          <label class="field">
            <span>BGM 淡入秒数</span>
            <input
              :value="bgmFadeInSeconds"
              type="number"
              min="0"
              max="10"
              step="0.1"
              :disabled="isMixing || isBatchMixing"
              @input="updateNumber($event, 'bgmFadeInSeconds')"
            />
          </label>
          <label class="field">
            <span>BGM 淡出秒数</span>
            <input
              :value="bgmFadeOutSeconds"
              type="number"
              min="0"
              max="10"
              step="0.1"
              :disabled="isMixing || isBatchMixing"
              @input="updateNumber($event, 'bgmFadeOutSeconds')"
            />
          </label>
          <p class="empty-text">BGM 会自动适配导出视频长度：短了会循环，长了会截断；淡入淡出只影响背景音乐。</p>
        </template>

        <template v-else-if="activeTool === 'cover' || activeTool === 'frame'">
          <div class="cover-preview-box">
            <img v-if="selectedCoverUrl" :src="selectedCoverUrl" alt="" />
            <span v-else>暂未生成封面帧</span>
          </div>
          <label class="field">
            <span>抽帧时间（秒）</span>
            <input
              :value="coverFrameSeconds"
              type="number"
              min="0"
              step="0.1"
              :disabled="isGeneratingCover"
              @input="updateNumber($event, 'coverFrameSeconds')"
            />
          </label>
          <button
            class="primary-button primary-button--full"
            type="button"
            :disabled="isGeneratingCover"
            @click="$emit('generateCoverFrame')"
          >
            {{ isGeneratingCover ? "正在生成..." : "设为封面帧" }}
          </button>
          <p v-if="coverError" class="error-text">{{ coverError }}</p>
          <p v-else class="empty-text">本阶段只生成封面帧和片段预览图，不做复杂关键帧动画。</p>
        </template>

        <template v-else-if="activeTool === 'mirror'">
          <label class="option-toggle">
            <input
              :checked="applyHorizontalMirror"
              type="checkbox"
              @change="updateCheckbox($event, 'applyHorizontalMirror')"
            />
            <span>水平镜像</span>
          </label>
          <label class="option-toggle">
            <input
              :checked="applyVerticalMirror"
              type="checkbox"
              @change="updateCheckbox($event, 'applyVerticalMirror')"
            />
            <span>垂直镜像</span>
          </label>
          <p class="empty-text">镜像效果会应用到拼接抽中片段和批量生成混剪。</p>
        </template>

        <template v-else-if="activeTool === 'rotate'">
          <label class="field">
            <span>旋转方式</span>
            <select :value="rotationMode" @change="updateSelect($event, 'rotationMode')">
              <option value="none">不旋转</option>
              <option value="clockwise90">顺时针 90°</option>
              <option value="counterclockwise90">逆时针 90°</option>
              <option value="rotate180">旋转 180°</option>
            </select>
          </label>
        </template>

        <template v-else-if="activeTool === 'speed'">
          <label class="field">
            <span>变速倍数</span>
            <input
              :value="playbackSpeed"
              type="number"
              min="0.5"
              max="2"
              step="0.1"
              :disabled="isMixing || isBatchMixing"
              @input="updateNumber($event, 'playbackSpeed')"
            />
          </label>
        </template>

        <template v-else-if="activeTool === 'effects' || activeTool === 'adjust'">
          <label class="field">
            <span>亮度</span>
            <input
              :value="brightness"
              type="number"
              min="-1"
              max="1"
              step="0.05"
              :disabled="isMixing || isBatchMixing"
              @input="updateNumber($event, 'brightness')"
            />
          </label>
          <label class="field">
            <span>对比度</span>
            <input
              :value="contrast"
              type="number"
              min="0"
              max="3"
              step="0.05"
              :disabled="isMixing || isBatchMixing"
              @input="updateNumber($event, 'contrast')"
            />
          </label>
          <label class="field">
            <span>饱和度</span>
            <input
              :value="saturation"
              type="number"
              min="0"
              max="3"
              step="0.05"
              :disabled="isMixing || isBatchMixing"
              @input="updateNumber($event, 'saturation')"
            />
          </label>
        </template>

        <template v-else-if="activeTool === 'zoom'">
          <label class="field">
            <span>轻微缩放</span>
            <input
              :value="effectScale"
              type="number"
              min="1"
              max="1.2"
              step="0.01"
              :disabled="isMixing || isBatchMixing"
              @input="updateNumber($event, 'effectScale')"
            />
          </label>
          <p class="empty-text">建议范围 1.00 到 1.20，用于轻微放大画面。</p>
        </template>

        <template v-else-if="activeTool === 'export'">
          <p v-if="outputDirectory" class="output-path">{{ outputDirectory }}</p>
          <p v-else class="empty-text">请选择导出结果保存位置。</p>
          <button class="ghost-button ghost-button--full" type="button" @click="$emit('selectOutputDirectory')">选择输出目录</button>
          <button class="ghost-button ghost-button--full" type="button" :disabled="!outputDirectory" @click="$emit('openOutputDirectory')">
            打开输出目录
          </button>
          <button class="primary-button primary-button--full" type="button" :disabled="isExporting" @click="$emit('exportSelectedVideo')">
            {{ isExporting ? "正在导出..." : "导出当前视频" }}
          </button>
          <p v-if="outputDirectoryError" class="error-text">{{ outputDirectoryError }}</p>
        </template>

        <template v-else>
          <p class="empty-text">这个入口先定版 UI 位置，本次不开发真实功能。</p>
        </template>
      </div>

      <footer class="tool-modal__footer">
        <button v-if="props.activeTool" class="ghost-button" type="button" @click="$emit('reset', props.activeTool)">重置</button>
        <button class="primary-button" type="button" @click="$emit('close')">应用</button>
      </footer>
    </section>
  </div>
</template>
