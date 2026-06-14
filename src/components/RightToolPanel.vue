<script setup lang="ts">
import type { FfmpegEnvironmentResult } from "../types/videoProbe";

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

defineProps<{
  environment: FfmpegEnvironmentResult | null;
  statusText: string;
}>();

defineEmits<{
  openTool: [tool: ToolKey];
  openDrawer: [drawer: "logs" | "exports" | "batch"];
}>();

const toolEntries: Array<{ key: ToolKey; title: string; note: string; enabled: boolean }> = [
  { key: "remix", title: "混剪设置", note: "切片、抽取、批量数量", enabled: true },
  { key: "canvas", title: "画布设置", note: "比例、黑边、模糊背景", enabled: true },
  { key: "effects", title: "视频效果", note: "后续效果入口", enabled: false },
  { key: "transition", title: "平滑转场", note: "平滑混剪开关", enabled: true },
  { key: "pip", title: "画中画", note: "后续能力入口", enabled: false },
  { key: "mirror", title: "镜像旋转", note: "水平镜像", enabled: true },
  { key: "speed", title: "视频变速", note: "0.5x 到 2.0x", enabled: true },
  { key: "subtitles", title: "字幕设置", note: "后续能力入口", enabled: false },
  { key: "export", title: "导出设置", note: "输出目录、基础导出", enabled: true },
];
</script>

<template>
  <aside class="right-rail tool-rail" aria-label="功能入口">
    <section class="panel engine-panel">
      <div class="panel__header">
        <div>
          <p class="panel__label">环境</p>
          <h2>本地引擎</h2>
        </div>
      </div>
      <p class="engine-message">{{ statusText }}</p>
      <div class="engine-list">
        <div>
          <span>ffmpeg</span>
          <strong>{{ environment?.ffmpeg.available ? "已检测到" : "未检测到" }}</strong>
        </div>
        <div>
          <span>ffprobe</span>
          <strong>{{ environment?.ffprobe.available ? "已检测到" : "未检测到" }}</strong>
        </div>
      </div>
    </section>

    <section class="panel tool-list-panel">
      <div class="panel__header">
        <div>
          <p class="panel__label">工具</p>
          <h2>功能入口</h2>
        </div>
      </div>
      <div class="tool-list">
        <button
          v-for="tool in toolEntries"
          :key="tool.key"
          class="tool-entry"
          type="button"
          @click="$emit('openTool', tool.key)"
        >
          <span>
            <strong>{{ tool.title }}</strong>
            <small>{{ tool.note }}</small>
          </span>
          <em>{{ tool.enabled ? "设置" : "预留" }}</em>
        </button>
      </div>
    </section>

    <section class="panel tool-list-panel">
      <div class="panel__header">
        <div>
          <p class="panel__label">面板</p>
          <h2>底部信息</h2>
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
