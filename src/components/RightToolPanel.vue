<script setup lang="ts">
import type { FfmpegEnvironmentResult } from "../types/videoProbe";

type ToolKey =
  | "remix"
  | "canvas"
  | "audio"
  | "bgm"
  | "tts"
  | "subtitleStyle"
  | "watermark"
  | "cover"
  | "entrance"
  | "frame"
  | "effects"
  | "transition"
  | "pip"
  | "adjust"
  | "fusion"
  | "rotate"
  | "mirror"
  | "speed"
  | "zoom"
  | "subtitles"
  | "export";

defineProps<{
  isAdvancedMode: boolean;
  environment: FfmpegEnvironmentResult | null;
  statusText: string;
}>();

defineEmits<{
  openTool: [tool: ToolKey];
  openDrawer: [drawer: "logs" | "exports" | "batch"];
  toggleAdvancedMode: [enabled: boolean];
}>();

const mainTools: Array<{ key: ToolKey; title: string; note: string; active?: boolean }> = [
  { key: "remix", title: "混剪设置", note: "切片、抽取、批量生成", active: true },
  { key: "canvas", title: "画布设置", note: "比例和模糊背景" },
  { key: "effects", title: "视频效果", note: "镜像、旋转、变速、画面调整" },
  { key: "transition", title: "平滑混剪", note: "淡入淡出、过滤过短片段" },
  { key: "pip", title: "画中画", note: "叠加视频或图片" },
  { key: "bgm", title: "背景音乐", note: "本地音乐和音量" },
];

const reservedTools: Array<{ key: ToolKey; title: string }> = [
  { key: "cover", title: "视频封面" },
  { key: "frame", title: "视频帧操作" },
  { key: "subtitles", title: "字幕入口" },
  { key: "audio", title: "音频设置" },
  { key: "tts", title: "语音合成" },
  { key: "watermark", title: "去除水印" },
];
</script>

<template>
  <aside class="right-rail tool-rail" aria-label="功能入口">
    <section class="panel engine-panel engine-panel--compact">
      <div>
        <p class="panel__label">本地引擎</p>
        <h2>{{ environment?.available ? "FFmpeg 就绪" : "等待检测" }}</h2>
      </div>
      <span class="engine-status-dot" :class="{ 'engine-status-dot--ok': environment?.available }"></span>
      <p class="engine-message">{{ statusText }}</p>
    </section>

    <section class="panel mode-panel">
      <div class="mode-panel__header">
        <div>
          <p class="panel__label">当前模式</p>
          <h2>{{ isAdvancedMode ? "高级模式" : "新手模式" }}</h2>
        </div>
        <button
          class="mode-switch"
          type="button"
          @click="$emit('toggleAdvancedMode', !isAdvancedMode)"
        >
          {{ isAdvancedMode ? "切回新手" : "进入高级" }}
        </button>
      </div>
      <p>{{ isAdvancedMode ? "显示完整功能入口，适合精细调参。" : "只显示最重要的导入、预览、处理和结果。" }}</p>
    </section>

    <section v-if="!isAdvancedMode" class="panel simple-action-panel">
      <div class="panel__header">
        <div>
          <p class="panel__label">新手流程</p>
          <h2>四步完成</h2>
        </div>
      </div>
      <div class="simple-steps">
        <span>1 导入素材</span>
        <span>2 预览视频</span>
        <span>3 开始处理</span>
        <span>4 查看结果</span>
      </div>
      <div class="simple-entry-list">
        <button class="tool-entry" type="button" @click="$emit('openTool', 'remix')">
          <span><strong>高级设置</strong><small>切片、抽取、批量数量</small></span>
          <em>打开</em>
        </button>
        <button class="tool-entry" type="button" @click="$emit('openDrawer', 'exports')">
          <span><strong>查看结果</strong><small>打开导出文件位置</small></span>
          <em>展开</em>
        </button>
      </div>
    </section>

    <section v-if="isAdvancedMode" class="panel tool-list-panel parameter-panel">
      <div class="parameter-panel__tabs">
        <button class="parameter-tab parameter-tab--active" type="button">设置入口</button>
      </div>
      <div class="tool-list">
        <button
          v-for="tool in mainTools"
          :key="tool.key"
          class="tool-entry parameter-entry"
          :class="{ 'parameter-entry--active': tool.active }"
          type="button"
          @click="$emit('openTool', tool.key)"
        >
          <span class="parameter-entry__icon" aria-hidden="true">◎</span>
          <span>
            <strong>{{ tool.title }}</strong>
            <small>{{ tool.note }}</small>
          </span>
          <em>›</em>
        </button>
      </div>
    </section>

    <section v-if="isAdvancedMode" class="panel reserved-panel">
      <div class="panel__header">
        <div>
          <p class="panel__label">预留入口</p>
          <h2>后续能力</h2>
        </div>
      </div>
      <div class="reserved-chip-grid">
        <button
          v-for="tool in reservedTools"
          :key="tool.key"
          class="reserved-chip"
          type="button"
          @click="$emit('openTool', tool.key)"
        >
          {{ tool.title }}
        </button>
      </div>
    </section>

    <section class="panel tool-list-panel drawer-entry-panel">
      <div class="panel__header">
        <div>
          <p class="panel__label">信息面板</p>
          <h2>日志和结果</h2>
        </div>
      </div>
      <div class="tool-list">
        <button class="tool-entry" type="button" @click="$emit('openDrawer', 'logs')">
          <span><strong>任务日志</strong><small>查看处理过程</small></span>
          <em>展开</em>
        </button>
        <button class="tool-entry" type="button" @click="$emit('openDrawer', 'exports')">
          <span><strong>导出结果</strong><small>打开导出位置</small></span>
          <em>展开</em>
        </button>
        <button class="tool-entry" type="button" @click="$emit('openDrawer', 'batch')">
          <span><strong>批量生成结果</strong><small>查看本轮结果</small></span>
          <em>展开</em>
        </button>
      </div>
    </section>
  </aside>
</template>
