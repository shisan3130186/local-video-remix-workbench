<script setup lang="ts">
import type { FfmpegEnvironmentResult } from "../types/videoProbe";
import type { DrawerKey, ToolKey } from "../types/workbench";

defineProps<{
  isAdvancedMode: boolean;
  environment: FfmpegEnvironmentResult | null;
  statusText: string;
}>();

defineEmits<{
  openTool: [tool: ToolKey];
  openDrawer: [drawer: DrawerKey];
  toggleAdvancedMode: [enabled: boolean];
}>();

const primaryTools: Array<{ key: ToolKey; title: string; note: string; symbol: string }> = [
  { key: "apiKeys", title: "API 密钥", note: "一次填写，AI与配音自动使用", symbol: "钥" },
  { key: "canvas", title: "画布与比例", note: "原画、9:16、模糊背景", symbol: "▣" },
  { key: "effects", title: "画面效果", note: "镜像、旋转、色彩与缩放", symbol: "◐" },
  { key: "transition", title: "平滑混剪", note: "淡入淡出并过滤过短片段", symbol: "≈" },
  { key: "pip", title: "画中画", note: "叠加视频或图片素材", symbol: "▤" },
  { key: "bgm", title: "背景音乐", note: "本地音乐、音量和淡入淡出", symbol: "♫" },
  { key: "tts", title: "AI配音", note: "逐句配音、试听和生成视频", symbol: "声" },
];

const advancedTools: Array<{ key: ToolKey; title: string; note: string }> = [
  { key: "remix", title: "切片与批量参数", note: "切片时长、抽取数量、生成数量" },
  { key: "cover", title: "视频封面", note: "选择当前视频的封面帧" },
];
</script>

<template>
  <aside class="right-rail tool-rail" aria-label="成片设置">
    <section class="panel workspace-status-panel">
      <span class="engine-status-dot" :class="{ 'engine-status-dot--ok': environment?.available }"></span>
      <span>
        <strong>{{ environment?.available ? "本地引擎已就绪" : "正在检测本地引擎" }}</strong>
        <small>{{ statusText }}</small>
      </span>
    </section>

    <section class="panel creation-settings-panel">
      <div class="creation-settings-panel__header">
        <div>
          <p class="panel__label">成片设置</p>
          <h2>常用调整</h2>
        </div>
        <button
          class="mode-switch"
          type="button"
          :aria-pressed="isAdvancedMode"
          @click="$emit('toggleAdvancedMode', !isAdvancedMode)"
        >
          {{ isAdvancedMode ? "精简显示" : "显示高级" }}
        </button>
      </div>

      <div class="creation-tool-list">
        <button
          v-for="tool in primaryTools"
          :key="tool.key"
          class="creation-tool"
          type="button"
          @click="$emit('openTool', tool.key)"
        >
          <span class="creation-tool__symbol" aria-hidden="true">{{ tool.symbol }}</span>
          <span><strong>{{ tool.title }}</strong><small>{{ tool.note }}</small></span>
          <em aria-hidden="true">›</em>
        </button>
      </div>

      <details v-if="isAdvancedMode" class="advanced-tool-group">
        <summary>更多高级设置</summary>
        <button
          v-for="tool in advancedTools"
          :key="tool.key"
          type="button"
          @click="$emit('openTool', tool.key)"
        >
          <span><strong>{{ tool.title }}</strong><small>{{ tool.note }}</small></span>
          <em aria-hidden="true">›</em>
        </button>
      </details>
    </section>

    <section class="panel result-shortcuts-panel">
      <div>
        <p class="panel__label">任务信息</p>
        <h2>日志与结果</h2>
      </div>
      <div class="result-shortcuts">
        <button type="button" @click="$emit('openDrawer', 'logs')">任务日志</button>
        <button type="button" @click="$emit('openDrawer', 'exports')">导出结果</button>
        <button type="button" @click="$emit('openDrawer', 'batch')">批量结果</button>
      </div>
    </section>
  </aside>
</template>
