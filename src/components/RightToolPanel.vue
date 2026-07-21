<script setup lang="ts">
import { computed } from "vue";
import type { FfmpegEnvironmentResult } from "../types/videoProbe";
import type { DrawerKey, ToolKey, WorkspaceMode } from "../types/workbench";

const props = defineProps<{
  workspaceMode: Exclude<WorkspaceMode, "tools">;
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
  { key: "apiKeys", title: "API 密钥", note: "一次填写，AI、配音与识别自动使用", symbol: "钥" },
  { key: "remix", title: "智能切片", note: "场景识别、固定切片和批量参数", symbol: "切" },
  { key: "canvas", title: "画布与比例", note: "原画、9:16、模糊背景", symbol: "▣" },
  { key: "effects", title: "画面效果", note: "镜像、旋转、色彩与缩放", symbol: "◐" },
  { key: "transition", title: "平滑混剪", note: "淡入淡出并过滤过短片段", symbol: "≈" },
  { key: "pip", title: "画中画", note: "叠加视频或图片素材", symbol: "▤" },
  { key: "bgm", title: "背景音乐", note: "本地音乐、音量和淡入淡出", symbol: "♫" },
  { key: "tts", title: "AI配音", note: "逐句配音、试听和生成视频", symbol: "声" },
  { key: "asr", title: "语音识别", note: "音频转文字和句子时间轴", symbol: "识" },
  { key: "watermark", title: "水印工具", note: "添加、融合、模糊、马赛克与遮盖", symbol: "印" },
  { key: "export", title: "导出设置", note: "分辨率、帧率、质量和编码方式", symbol: "出" },
];

const advancedTools: Array<{ key: ToolKey; title: string; note: string }> = [
  { key: "cover", title: "视频封面", note: "选择当前视频的封面帧" },
];

const toolsByMode: Record<Exclude<WorkspaceMode, "tools">, ToolKey[]> = {
  ai: ["apiKeys", "remix", "canvas", "bgm", "tts", "asr", "export"],
  batch: ["remix", "canvas", "effects", "transition", "pip", "bgm", "watermark", "export"],
};

const toolsByKey = new Map(primaryTools.map((tool) => [tool.key, tool]));

const groupDefinitions: Record<Exclude<WorkspaceMode, "tools">, Array<{
  title: string;
  note: string;
  keys: ToolKey[];
}>> = {
  ai: [
    { title: "智能准备", note: "先完成服务配置和素材切片", keys: ["apiKeys", "remix"] },
    { title: "声音与文字", note: "配音、识别和背景音乐", keys: ["tts", "asr", "bgm"] },
    { title: "画面与输出", note: "确认比例后设置导出质量", keys: ["canvas", "export"] },
  ],
  batch: [
    { title: "素材与规则", note: "决定如何切片和组合", keys: ["remix"] },
    { title: "画面增强", note: "统一批量视频的视觉效果", keys: ["canvas", "effects", "transition", "pip", "watermark"] },
    { title: "声音与输出", note: "添加音乐并确认导出参数", keys: ["bgm", "export"] },
  ],
};

const toolGroups = computed(() => groupDefinitions[props.workspaceMode].map((group) => ({
  ...group,
  tools: group.keys
    .filter((key) => toolsByMode[props.workspaceMode].includes(key))
    .map((key) => toolsByKey.get(key))
    .filter((tool): tool is NonNullable<typeof tool> => Boolean(tool)),
})));

const panelCopy = computed(() => props.workspaceMode === "ai"
  ? { label: "智能成片", title: "按步骤完成设置" }
  : { label: "批量混剪", title: "按步骤完成设置" });
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
          <p class="panel__label">{{ panelCopy.label }}</p>
          <h2>{{ panelCopy.title }}</h2>
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

      <div class="creation-tool-groups">
        <section v-for="group in toolGroups" :key="group.title" class="creation-tool-group">
          <header>
            <strong>{{ group.title }}</strong>
            <small>{{ group.note }}</small>
          </header>
          <div class="creation-tool-list">
            <button
              v-for="tool in group.tools"
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
        </section>
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
        <button v-if="workspaceMode === 'batch'" type="button" @click="$emit('openDrawer', 'batch')">批量结果</button>
      </div>
    </section>
  </aside>
</template>
