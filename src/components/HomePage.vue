<script setup lang="ts">
import { ref } from "vue";
import type { FeatureKey, ToolKey, WorkspaceMode } from "../types/workbench";

type HubSection = "creation" | "utilities";

defineProps<{
  hasRecentProject: boolean;
  recentProjectTitle: string;
  recentProjectSummary: string;
  environmentAvailable: boolean | null;
  aiConfigured: boolean | null;
  speechConfigured: boolean | null;
  diagnosticsAvailable: boolean;
}>();

const emit = defineEmits<{
  openWorkspace: [mode: WorkspaceMode, batchKind?: "remix" | "category"];
  openTool: [tool: ToolKey];
  openFeature: [feature: FeatureKey];
  continueProject: [];
  openWelcome: [];
}>();

const activeSection = ref<HubSection>("creation");

const creationModules: Array<{
  title: string;
  tags: string[];
  description: string;
  action?: () => void;
  available: boolean;
}> = [
  {
    title: "AI 智能混剪",
    tags: ["AI智能混剪", "语音合成", "批量成片", "视频理解"],
    description: "添加文案或音频后，由AI理解素材、匹配镜头并自动生成配音，一次输出多条结构完整的视频。",
    action: () => emit("openWorkspace", "ai"),
    available: true,
  },
  {
    title: "视频效果处理",
    tags: ["批量处理", "画面去重", "智能分割", "差异化处理"],
    description: "集中完成水印、镜像、变速、画布、画中画和色彩调整，并支持智能或固定间隔切片。",
    action: () => emit("openTool", "effects"),
    available: true,
  },
  {
    title: "视频混剪",
    tags: ["视频混剪", "语音合成", "批量混剪", "随机重组"],
    description: "从多条原视频中生成片段并随机组合，可叠加配音、字幕和背景音乐，批量生成不同版本。",
    action: () => emit("openWorkspace", "batch", "remix"),
    available: true,
  },
  {
    title: "分类混剪",
    tags: ["分类混剪", "语音合成", "批量混剪", "多分类"],
    description: "按开头、产品、细节、效果等镜头分类组合，支持规则抽取和结构化批量成片。",
    action: () => emit("openWorkspace", "batch", "category"),
    available: true,
  },
  {
    title: "文案改写",
    tags: ["AI改写", "文案优化", "批量改写"],
    description: "粘贴一条或多条文案，按指定语气、长度和提示词生成差异化版本。",
    action: () => emit("openFeature", "copyRewrite"),
    available: true,
  },
  {
    title: "视频内容提炼",
    tags: ["视频切片", "精华提取", "内容分析", "AI分类"],
    description: "批量分析视频内容，提取有价值片段并标记主题、动作、卖点和镜头类型。",
    action: () => emit("openWorkspace", "tools"),
    available: true,
  },
];

const utilityModules: Array<{
  title: string;
  tags: string[];
  description: string;
  action?: () => void;
  available: boolean;
}> = [
  {
    title: "文件批量改名",
    tags: ["文件管理", "批量改名", "序号命名", "查找替换"],
    description: "支持统一命名、查找替换、添加前后缀和日期时间，并实时预览改名结果。",
    action: () => emit("openFeature", "fileRenamer"),
    available: true,
  },
  {
    title: "字幕识别",
    tags: ["音频识别", "编辑修正", "SRT导出", "文案导出"],
    description: "批量导入视频或音频，自动识别语音并生成字幕，可继续编辑或保存到文案库。",
    action: () => emit("openFeature", "subtitleEditor"),
    available: true,
  },
  {
    title: "大字报设计",
    tags: ["文字设计", "营销海报", "样式模板", "一键排版"],
    description: "粘贴文案后快速生成适合视频贴画和混剪使用的文字海报素材。",
    action: () => emit("openFeature", "posterMaker"),
    available: true,
  },
  {
    title: "图片转视频",
    tags: ["批量处理", "硬件加速", "图片转视频", "素材生产"],
    description: "把图片批量转换为固定时长的视频片段，生成后可直接加入混剪素材库。",
    action: () => emit("openFeature", "imageToVideo"),
    available: true,
  },
];
</script>

<template>
  <main class="replica-home">
    <section class="replica-home__content">
      <header class="replica-home__heading">
        <div>
          <h2>智剪 SmartCut</h2>
          <p>把重复劳动交给机器，把创造力留给自己。</p>
        </div>
        <nav class="replica-home__tabs" aria-label="首页功能分类">
          <button
            type="button"
            :class="{ 'is-active': activeSection === 'creation' }"
            :aria-current="activeSection === 'creation' ? 'page' : undefined"
            @click="activeSection = 'creation'"
          >
            创作中心 <span>6</span>
          </button>
          <button
            type="button"
            :class="{ 'is-active': activeSection === 'utilities' }"
            :aria-current="activeSection === 'utilities' ? 'page' : undefined"
            @click="activeSection = 'utilities'"
          >
            效率工具 <span>4</span>
          </button>
        </nav>
      </header>

      <section class="replica-module-grid" :class="{ 'replica-module-grid--utilities': activeSection === 'utilities' }">
        <button
          v-for="module in activeSection === 'creation' ? creationModules : utilityModules"
          :key="module.title"
          type="button"
          :disabled="!module.available"
          class="replica-module-card"
          @click="module.action?.()"
        >
          <span class="replica-module-card__title-row">
            <strong>{{ module.title }}</strong>
            <em :class="{ 'is-pending': !module.available }">{{ module.available ? "功能可用" : "即将接入" }}</em>
          </span>
          <span class="replica-module-card__tags">
            <small v-for="tag in module.tags" :key="tag">{{ tag }}</small>
          </span>
          <span class="replica-module-card__description">{{ module.description }}</span>
        </button>
      </section>

      <footer class="replica-home__footer">
        <button v-if="hasRecentProject" type="button" @click="emit('continueProject')">继续上次项目</button>
        <span>v0.5.0</span>
      </footer>
    </section>
  </main>
</template>
