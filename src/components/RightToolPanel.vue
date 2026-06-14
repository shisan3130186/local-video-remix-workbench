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
  environment: FfmpegEnvironmentResult | null;
  statusText: string;
}>();

defineEmits<{
  openTool: [tool: ToolKey];
  openDrawer: [drawer: "logs" | "exports" | "batch"];
}>();

const toolEntries: Array<{ key: ToolKey; title: string; note: string; enabled: boolean }> = [
  { key: "audio", title: "音频设置", note: "音量、原声、音轨", enabled: false },
  { key: "bgm", title: "背景音乐", note: "BGM 和混音", enabled: false },
  { key: "tts", title: "语音合成", note: "后续 TTS 入口", enabled: false },
  { key: "subtitleStyle", title: "字幕样式", note: "字体、颜色、描边", enabled: false },
  { key: "watermark", title: "去除水印", note: "后续能力入口", enabled: false },
  { key: "cover", title: "视频封面", note: "封面帧选择", enabled: false },
  { key: "entrance", title: "入场效果", note: "片段入场方式", enabled: false },
  { key: "frame", title: "视频帧操作", note: "关键帧、预览图", enabled: false },
  { key: "effects", title: "视频效果", note: "亮度、对比度、饱和度", enabled: true },
  { key: "transition", title: "平滑转场", note: "平滑混剪开关", enabled: true },
  { key: "pip", title: "画中画", note: "后续能力入口", enabled: false },
  { key: "adjust", title: "画面调整", note: "亮度、对比度、饱和度", enabled: true },
  { key: "fusion", title: "镜像融合", note: "空间镜像融合", enabled: false },
  { key: "rotate", title: "旋转镜像", note: "90° / 180° 旋转", enabled: true },
  { key: "mirror", title: "镜像旋转", note: "水平镜像", enabled: true },
  { key: "speed", title: "视频变速", note: "0.5x 到 2.0x", enabled: true },
  { key: "zoom", title: "动态缩放", note: "轻微缩放", enabled: true },
  { key: "export", title: "导出设置", note: "输出目录、基础导出", enabled: true },
];

const quickEntries: Array<{ key: ToolKey; title: string; note: string }> = [
  { key: "remix", title: "混剪设置", note: "固定切片 / 随机抽取 / 批量生成" },
  { key: "canvas", title: "画布比例", note: "原画 / 9:16 / 1:1 / 16:9 / 模糊背景" },
  { key: "transition", title: "平滑混剪", note: "淡入淡出 / 过滤短片段" },
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

    <section class="panel tool-list-panel quick-tool-panel">
      <div class="panel__header">
        <div>
          <p class="panel__label">当前模块</p>
          <h2>已有功能入口</h2>
        </div>
      </div>
      <div class="quick-tool-grid">
        <button
          v-for="tool in quickEntries"
          :key="tool.key"
          class="quick-tool-card"
          type="button"
          @click="$emit('openTool', tool.key)"
        >
          <strong>{{ tool.title }}</strong>
          <small>{{ tool.note }}</small>
        </button>
      </div>
    </section>

    <section class="panel tool-list-panel">
      <div class="panel__header">
        <div>
          <p class="panel__label">工具</p>
          <h2>功能入口</h2>
        </div>
      </div>
      <div class="tool-card-grid">
        <button
          v-for="tool in toolEntries"
          :key="tool.key"
          class="tool-entry tool-entry--card"
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
