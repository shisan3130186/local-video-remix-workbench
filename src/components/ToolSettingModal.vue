<script setup lang="ts">
import type { CanvasAspectRatio, CanvasBackgroundMode } from "../services/videoMixService";

type ToolKey =
  | "remix"
  | "canvas"
  | "effects"
  | "transition"
  | "pip"
  | "mirror"
  | "speed"
  | "subtitles"
  | "export";

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
  smoothRemixEnabled: boolean;
  playbackSpeed: number;
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
  "update:segmentDurationSeconds": [value: number];
  "update:randomPickCount": [value: number];
  "update:batchGenerateCount": [value: number];
  "update:canvasAspectRatio": [value: CanvasAspectRatio];
  "update:canvasBackgroundMode": [value: CanvasBackgroundMode];
  "update:applyHorizontalMirror": [value: boolean];
  "update:smoothRemixEnabled": [value: boolean];
  "update:playbackSpeed": [value: number];
}>();

const titles: Record<ToolKey, string> = {
  remix: "混剪设置",
  canvas: "画布设置",
  effects: "视频效果",
  transition: "平滑转场",
  pip: "画中画",
  mirror: "镜像旋转",
  speed: "视频变速",
  subtitles: "字幕设置",
  export: "导出设置",
};

function updateNumber(event: Event, name: "segmentDurationSeconds" | "randomPickCount" | "batchGenerateCount" | "playbackSpeed") {
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

  emit("update:playbackSpeed", value);
}

function updateCheckbox(event: Event, name: "applyHorizontalMirror" | "smoothRemixEnabled") {
  const value = (event.target as HTMLInputElement).checked;

  if (name === "applyHorizontalMirror") {
    emit("update:applyHorizontalMirror", value);
    return;
  }

  emit("update:smoothRemixEnabled", value);
}

function updateSelect(event: Event, name: "canvasAspectRatio" | "canvasBackgroundMode") {
  const value = (event.target as HTMLSelectElement).value;

  if (name === "canvasAspectRatio") {
    emit("update:canvasAspectRatio", value as CanvasAspectRatio);
    return;
  }

  emit("update:canvasBackgroundMode", value as CanvasBackgroundMode);
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

        <template v-else-if="activeTool === 'mirror'">
          <label class="option-toggle">
            <input
              :checked="applyHorizontalMirror"
              type="checkbox"
              @change="updateCheckbox($event, 'applyHorizontalMirror')"
            />
            <span>水平镜像</span>
          </label>
          <p class="empty-text">当前版本只开放水平镜像；旋转能力作为后续入口保留。</p>
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
