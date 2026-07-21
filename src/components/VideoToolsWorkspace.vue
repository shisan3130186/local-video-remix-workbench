<script setup lang="ts">
import type { ImportedVideo } from "../types/videoProbe";
import type { DrawerKey, ToolKey } from "../types/workbench";

defineProps<{
  selectedVideo: ImportedVideo | null;
  previewUrl: string | null;
  importedVideoCount: number;
}>();

defineEmits<{
  openTool: [tool: ToolKey];
  openDrawer: [drawer: DrawerKey];
}>();

const toolGroups: Array<{
  title: string;
  note: string;
  tools: Array<{ key: ToolKey; title: string; note: string; symbol: string; requiresVideo?: boolean }>;
}> = [
  {
    title: "画面处理",
    note: "处理比例、颜色、封面和水印",
    tools: [
      { key: "watermark", title: "水印工具", note: "添加水印、固定区域处理和移动水印跟踪", symbol: "印", requiresVideo: true },
      { key: "effects", title: "画面调整", note: "镜像、旋转、变速、亮度与色彩", symbol: "调", requiresVideo: true },
      { key: "canvas", title: "画布与比例", note: "原画、9:16 和模糊背景填充", symbol: "框", requiresVideo: true },
      { key: "cover", title: "视频封面", note: "从当前视频截取一帧作为封面", symbol: "封", requiresVideo: true },
      { key: "pip", title: "画中画", note: "叠加另一个视频或图片素材", symbol: "叠", requiresVideo: true },
    ],
  },
  {
    title: "声音与文字",
    note: "独立完成配音、识别和背景音乐",
    tools: [
      { key: "asr", title: "语音识别", note: "音频或视频转文字，并保存到文案库", symbol: "识" },
      { key: "tts", title: "AI 配音", note: "选择可视化音色，试听并生成配音", symbol: "声" },
      { key: "bgm", title: "背景音乐", note: "调整原声、音乐音量和淡入淡出", symbol: "乐", requiresVideo: true },
    ],
  },
  {
    title: "设置与输出",
    note: "管理服务密钥和最终导出参数",
    tools: [
      { key: "apiKeys", title: "AI / TTS 密钥", note: "填写一次后加密保存在当前 Windows 用户中", symbol: "钥" },
      { key: "export", title: "导出与格式转换", note: "设置分辨率、帧率、质量和编码方式", symbol: "出", requiresVideo: true },
    ],
  },
];
</script>

<template>
  <section class="video-tools-workspace" aria-label="视频工具中心">
    <header class="workspace-page-heading">
      <div>
        <p class="panel__label">单项处理</p>
        <h2>视频工具中心</h2>
        <small>只选一个需要的工具，不进入完整成片流程。</small>
      </div>
      <div class="video-tools-workspace__header-actions">
        <button class="ghost-button" type="button" @click="$emit('openDrawer', 'scripts')">文案库</button>
        <button class="panel-toggle" type="button" @click="$emit('openDrawer', 'exports')">导出结果</button>
      </div>
    </header>

    <div class="video-tools-workspace__layout">
      <section class="panel tool-source-preview">
        <div class="panel__header">
          <div>
            <p class="panel__label">当前处理对象</p>
            <h3>{{ selectedVideo?.fileName ?? "尚未选择视频" }}</h3>
          </div>
          <span class="count-badge">素材 {{ importedVideoCount }}</span>
        </div>
        <div v-if="previewUrl" class="tool-source-preview__video">
          <span class="preview-fit-badge">完整画面</span>
          <video :src="previewUrl" controls playsinline preload="metadata" aria-label="当前工具素材完整画面预览"></video>
        </div>
        <div v-else class="tool-source-preview__empty">
          <strong>需要画面处理时，先从左侧导入视频</strong>
          <small>语音识别、AI 配音和密钥设置也可以直接打开。</small>
        </div>
        <div class="tool-source-preview__tips">
          <strong>这里不会自动执行完整混剪</strong>
          <small>选中工具、调整参数、确认输出即可。</small>
        </div>
      </section>

      <div class="tool-group-list">
        <section v-for="group in toolGroups" :key="group.title" class="tool-group-section">
          <header>
            <div><h3>{{ group.title }}</h3><small>{{ group.note }}</small></div>
          </header>
          <div class="tool-center-grid">
            <button
              v-for="tool in group.tools"
              :key="tool.key"
              type="button"
              :disabled="tool.requiresVideo && !selectedVideo"
              @click="$emit('openTool', tool.key)"
            >
              <span aria-hidden="true">{{ tool.symbol }}</span>
              <strong>{{ tool.title }}</strong>
              <small>{{ tool.requiresVideo && !selectedVideo ? "请先导入视频" : tool.note }}</small>
              <em aria-hidden="true">进入</em>
            </button>
          </div>
        </section>
      </div>
    </div>
  </section>
</template>
