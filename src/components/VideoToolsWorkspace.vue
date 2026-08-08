<script setup lang="ts">
import { computed, ref } from "vue";
import type { AiRemixSegment } from "../features/ai-remix";
import type { ImportedVideo } from "../types/videoProbe";
import type { DrawerKey, ToolKey } from "../types/workbench";
import type { VideoProcessingState } from "../types/workbench";
import type { OutputFrameRate, OutputQuality, OutputResolution, VideoEncoder } from "../types/outputSettings";
import ToolIcon from "./ToolIcon.vue";
import type { ToolIconName } from "./ToolIcon.vue";

const props = defineProps<{
  view: "effects" | "extract";
  importedVideos: ImportedVideo[];
  selectedVideo: ImportedVideo | null;
  previewUrl: string | null;
  importedVideoCount: number;
  videoCoverUrls: Record<string, string>;
  splitSegmentPaths: string[];
  selectedSegmentPath: string | null;
  segmentThumbnailUrls: Record<string, string>;
  preparedSegments: AiRemixSegment[];
  outputDirectory: string | null;
  isProcessing: boolean;
  progressText: string | null;
  error: string | null;
  videoProcessingStates: Record<string, VideoProcessingState>;
  activeProcessingVideoId: string | null;
  processingProgress: number;
  formatDuration: (durationSeconds: number | null) => string;
  formatFileName: (path: string) => string;
}>();

const emit = defineEmits<{
  importVideos: [];
  importVideoFolder: [];
  clearVideos: [];
  selectVideo: [video: ImportedVideo];
  selectSegment: [segmentPath: string];
  selectOutputDirectory: [];
  splitSelectedVideo: [];
  analyzeContent: [];
  generateCategorizedSegments: [];
  startProcessing: [];
  openTool: [tool: ToolKey];
  openDrawer: [drawer: DrawerKey];
}>();

const splitMode = ref<"disabled" | "scene" | "fixed">("scene");
const segmentSeconds = ref(5);
const removeIntroSeconds = ref(0);
const removeOutroSeconds = ref(0);
const confidenceScore = ref(0.7);
const filterShortSeconds = ref(1.5);
const outputGrouping = ref<"file" | "folder" | "none">("file");
const variantCount = ref(10);
const settingsTab = ref<"effects" | "stickers">("effects");
const configurationMode = ref<"manual" | "smart">("smart");
const enabledEffectKeys = ref<ToolKey[]>([]);
const previewOrientationClass = computed(() => {
  const width = props.selectedVideo?.width ?? 0;
  const height = props.selectedVideo?.height ?? 0;
  return width > height ? "is-landscape" : "is-portrait";
});
const outputResolution = defineModel<OutputResolution>("outputResolution", { required: true });
const outputFrameRate = defineModel<OutputFrameRate>("outputFrameRate", { required: true });
const outputQuality = defineModel<OutputQuality>("outputQuality", { required: true });
const outputEncoder = defineModel<VideoEncoder>("outputEncoder", { required: true });
const guideOpen = ref(typeof window === "undefined" || window.localStorage.getItem("video-tools-guide-dismissed") !== "1");
const guideDontRemind = ref(false);
const outputFormat = ref<"mp4" | "mov" | "webm">("mp4");
const keepOriginal = ref<"keep" | "remove">("keep");
const namingMode = ref<"serial" | "source">("serial");
const threadMode = ref<"single" | "auto" | "multi">("single");
const guideSteps = [
  { title: "导入素材", text: "拖拽视频文件到软件，或点击左侧「导入视频」/「导入文件夹」按钮。选择文件夹后，软件会自动扫描其中的视频文件。" },
  { title: "配置效果 或 智能配置", text: "新手可直接点击右侧底部的「智能配置」，一般无需手动调整。也可以点击右侧效果卡片手动配置参数，右键效果卡片可快速开关。" },
  { title: "AI 智能分割和自定义间隔", text: "AI 智能分割会根据镜头切换自动拆分片段；自定义间隔可以按指定秒数分割为多个片段。" },
  { title: "添加贴画与水印", text: "切换到右侧「贴画与水印」标签，可添加贴画、文本水印、图片 / 视频水印，或手动框选 / AI 自动检测去除水印。" },
  { title: "选择输出目录 → 开始处理", text: "左侧底部选择输出目录，底部面板调整分辨率、帧率、格式等参数，点击「开始处理」执行批量处理。" },
];
function closeGuide() {
  if (guideDontRemind.value) window.localStorage.setItem("video-tools-guide-dismissed", "1");
  guideOpen.value = false;
}
const preparedSegmentByPath = computed(
  () => new Map(props.preparedSegments.map((segment) => [segment.path, segment])),
);
const analyzedSegmentCount = computed(
  () =>
    props.splitSegmentPaths.filter(
      (path) => preparedSegmentByPath.value.get(path)?.contentAnalysis,
    ).length,
);
const allSegmentsAnalyzed = computed(
  () =>
    props.splitSegmentPaths.length >= 2 &&
    analyzedSegmentCount.value === props.splitSegmentPaths.length,
);

const effectGroups: Array<{ title: string; key: ToolKey; note: string; icon: ToolIconName }> = [
  { title: "音频设置", key: "bgm", note: "音频/背景音乐相关", icon: "audio" },
  { title: "视频封面", key: "cover", note: "设置视频封面首帧", icon: "cover" },
  { title: "入场效果", key: "entrance", note: "视频入场效果设置", icon: "entrance" },
  { title: "视频帧操作", key: "frame", note: "抽帧/插帧操作", icon: "frame" },
  { title: "画中画", key: "pip", note: "视频小窗 + 背景虚化", icon: "pip" },
  { title: "画面调整", key: "effects", note: "视频画面相关调整", icon: "adjust" },
  { title: "像素融合", key: "fusion", note: "空间域像素融合", icon: "fusion" },
  { title: "旋转换像", key: "rotate", note: "旋转、翻转、去黑边", icon: "rotate" },
  { title: "视频变速", key: "speed", note: "全局变速和分段变速", icon: "speed" },
  { title: "动态缩放", key: "zoom", note: "实现类似远镜的效果", icon: "zoom" },
];

const watermarkGroups: Array<{ title: string; key: ToolKey; note: string; icon: ToolIconName }> = [
  { title: "贴画", key: "pip", note: "添加贴画或叠加素材", icon: "pip" },
  { title: "文本水印", key: "watermark", note: "在画面中添加文字水印", icon: "file" },
  { title: "图片 / 视频水印", key: "watermark", note: "添加图片或视频水印", icon: "cover" },
  { title: "去除水印", key: "watermark", note: "手动框选或自动检测", icon: "adjust" },
];

const enabledEffectKeySet = computed(() => new Set(enabledEffectKeys.value));

function isEffectEnabled(key: ToolKey) {
  return enabledEffectKeySet.value.has(key);
}

function configureSmartly() {
  enabledEffectKeys.value = [...new Set(effectGroups.map((group) => group.key))];
  configurationMode.value = "smart";
}

function openEffect(group: { key: ToolKey }) {
  if (!enabledEffectKeys.value.includes(group.key)) {
    enabledEffectKeys.value = [...enabledEffectKeys.value, group.key];
  }
  emit("openTool", group.key);
}

function processingState(video: ImportedVideo): VideoProcessingState {
  return props.videoProcessingStates[video.id] ?? { status: "pending", progress: 0, message: "待处理" };
}

function processingLabel(video: ImportedVideo) {
  const state = processingState(video);
  if (state.status === "processing") {
    const index = props.importedVideos.findIndex((item) => item.id === video.id);
    const unit = props.importedVideos.length > 0 ? 100 / props.importedVideos.length : 100;
    const start = index * unit;
    const progress = props.activeProcessingVideoId === video.id
      ? Math.round(Math.max(0, Math.min(100, (props.processingProgress - start) / unit * 100)))
      : state.progress;
    return `处理中 ${progress}%`;
  }
  if (state.status === "completed") return "已完成";
  if (state.status === "failed") return "处理失败";
  if (state.status === "cancelled") return "已取消";
  return "待处理";
}
</script>

<template>
  <section class="replica-tools-workspace" :class="`is-${view}`" :aria-label="view === 'effects' ? '视频效果处理工作台' : '视频内容提炼工作台'">
    <aside class="replica-tools-source">
      <div class="replica-source-toolbar"><button class="is-primary" type="button" @click="emit('importVideos')"><ToolIcon name="file" />导入视频</button><button type="button" @click="emit('importVideoFolder')"><ToolIcon name="folder" />导入文件夹</button><button type="button" :disabled="!importedVideoCount" aria-label="清空视频列表" @click="emit('clearVideos')"><ToolIcon name="trash" /></button></div>
      <div v-if="!importedVideoCount" class="replica-source-empty"><span><ToolIcon name="video" /></span><strong>先导入视频素材</strong><small>支持单个、多选或整个文件夹</small></div>
      <div v-else class="replica-tools-file-list">
        <button v-for="video in importedVideos" :key="video.id" type="button" :class="{ 'is-active': selectedVideo?.id === video.id }" @click="emit('selectVideo', video)">
          <img v-if="videoCoverUrls[video.id]" :src="videoCoverUrls[video.id]" alt="" /><span v-else><ToolIcon name="video" /></span>
          <b>{{ video.fileName }}</b><small>{{ formatDuration(video.durationSeconds) }}</small>
          <small class="replica-video-processing-state" :class="`is-${processingState(video).status}`">{{ processingLabel(video) }}</small>
          <i class="replica-video-processing-track" aria-hidden="true"><em :style="{ width: `${processingState(video).status === 'processing' && activeProcessingVideoId === video.id ? Math.max(0, Math.min(100, (processingProgress - importedVideos.findIndex((item) => item.id === video.id) * (100 / Math.max(1, importedVideos.length))) / (100 / Math.max(1, importedVideos.length)) * 100)) : processingState(video).progress}%` }"></em></i>
        </button>
      </div>
      <button class="replica-output-path" type="button" @click="emit('selectOutputDirectory')"><span>输出路径</span><strong>{{ outputDirectory ?? '请选择输出文件夹' }}</strong><b><ToolIcon name="folder-open" /></b></button>
    </aside>

    <main class="replica-tools-center">
      <section class="replica-tools-preview">
        <header class="replica-tools-preview__header">
          <button class="replica-guide-trigger" type="button" @click="guideOpen = true"><ToolIcon name="info" />使用指引</button>
          <span>{{ selectedVideo?.fileName ?? '' }}</span>
          <div class="replica-preview-actions">
            <button type="button" @click="emit('openTool', 'canvas')">视频裁剪</button>
            <button type="button" @click="emit('openTool', 'watermark')">添加文本</button>
          </div>
        </header>
        <div v-if="previewUrl" class="replica-tools-screen">
          <video
            :src="previewUrl"
            :class="previewOrientationClass"
            :width="selectedVideo?.width ?? undefined"
            :height="selectedVideo?.height ?? undefined"
            controls
            playsinline
            preload="metadata"
          ></video>
        </div>
        <div v-else class="replica-tools-screen replica-tools-screen--empty"><span><ToolIcon name="video" /></span><strong>导入视频后在此预览</strong><small>选择左侧素材即可查看画面</small></div>
      </section>

      <section v-if="view === 'effects'" class="replica-split-settings">
        <header><strong>视频分割：</strong><div class="replica-split-modes"><button type="button" :class="{ 'is-active': splitMode === 'disabled' }" @click="splitMode = 'disabled'">不启用</button><button type="button" :class="{ 'is-active': splitMode === 'scene' }" @click="splitMode = 'scene'">AI智能分割</button><button type="button" :class="{ 'is-active': splitMode === 'fixed' }" @click="splitMode = 'fixed'">自定义间隔</button></div><span class="replica-split-output-label">分段输出：</span><div class="replica-split-output"><button type="button" :class="{ 'is-active': outputGrouping === 'file' }" @click="outputGrouping = 'file'">文件名分类</button><button type="button" :class="{ 'is-active': outputGrouping === 'folder' }" @click="outputGrouping = 'folder'">文件夹分类</button><button type="button" :class="{ 'is-active': outputGrouping === 'none' }" @click="outputGrouping = 'none'">不分类</button></div></header>
        <div class="replica-split-parameters">
          <label>去除片头：<input v-model.number="removeIntroSeconds" type="number" min="0" max="60" step="0.1" /> 秒</label>
          <label>去除片尾：<input v-model.number="removeOutroSeconds" type="number" min="0" max="60" step="0.1" /> 秒</label>
          <label>模型置信度：<input v-model.number="confidenceScore" type="number" min="0" max="1" step="0.05" /> 分</label>
          <label>过滤小于：<input v-model.number="filterShortSeconds" type="number" min="0" max="30" step="0.5" /> 秒的片段</label>
          <label v-if="splitMode === 'fixed'">间隔：<input v-model.number="segmentSeconds" type="number" min="1" max="120" /> 秒</label>
          <button class="is-primary" type="button" :disabled="!selectedVideo || isProcessing || splitMode === 'disabled'" @click="emit('splitSelectedVideo')">{{ isProcessing ? '处理中...' : '开始分割' }}</button>
        </div>
      </section>

      <section v-else class="replica-extract-results">
        <header><strong>提炼片段</strong><span>{{ splitSegmentPaths.length }} 个</span></header>
        <div v-if="splitSegmentPaths.length === 0" class="replica-category-empty"><strong>还没有内容片段</strong><small>先切片，再用本地或云端模型分析片段内容</small><button type="button" :disabled="!selectedVideo || isProcessing" @click="emit('splitSelectedVideo')">生成内容切片</button></div>
        <div v-else class="replica-extract-grid"><button v-for="segmentPath in splitSegmentPaths" :key="segmentPath" type="button" :class="{ 'is-active': selectedSegmentPath === segmentPath }" @click="emit('selectSegment', segmentPath)"><img v-if="segmentThumbnailUrls[segmentPath]" :src="segmentThumbnailUrls[segmentPath]" alt="" /><span v-else>▶</span><b>{{ formatFileName(segmentPath) }}</b><small v-if="preparedSegmentByPath.get(segmentPath)?.contentAnalysis">{{ preparedSegmentByPath.get(segmentPath)?.contentAnalysis?.theme }}</small><small v-else>等待内容分析</small></button></div>
      </section>
    </main>

    <aside class="replica-tools-settings">
      <template v-if="view === 'effects'">
        <div class="replica-tools-settings-tabs"><button type="button" :class="{ 'is-active': settingsTab === 'effects' }" @click="settingsTab = 'effects'">视频效果</button><button type="button" :class="{ 'is-active': settingsTab === 'stickers' }" @click="settingsTab = 'stickers'">贴画与水印</button></div>
        <div class="replica-tools-settings-scroll">
          <div class="replica-effect-list"><button v-for="group in (settingsTab === 'effects' ? effectGroups : watermarkGroups)" :key="`${settingsTab}-${group.title}`" type="button" :class="{ 'is-enabled': settingsTab === 'effects' && isEffectEnabled(group.key) }" @click="openEffect(group)"><span class="replica-effect-icon"><ToolIcon :name="group.icon" /></span><span><strong>{{ group.title }}</strong><small>{{ group.note }}</small></span><b><ToolIcon :name="settingsTab === 'effects' && isEffectEnabled(group.key) ? 'check' : 'chevron'" /></b></button></div>
        </div>
        <div class="replica-configuration-switch"><button type="button" :class="{ 'is-active': configurationMode === 'manual' }" @click="configurationMode = 'manual'">手动配置</button><button type="button" :class="{ 'is-active': configurationMode === 'smart', 'is-configured': enabledEffectKeys.length > 0 }" @click="configureSmartly">智能配置</button></div>
      </template>
      <template v-else>
        <header><strong>内容提炼</strong><small>识别主题、动作、卖点和镜头类型</small></header>
        <section class="replica-setting-block"><strong>分析模式</strong><label class="replica-radio-row"><input checked type="radio" name="extract-mode" />本地快速模型</label><label class="replica-radio-row"><input type="radio" name="extract-mode" />云端精确模型</label></section>
        <section class="replica-setting-block"><strong>提炼维度</strong><label class="replica-checkbox-row"><input checked type="checkbox" />镜头类型</label><label class="replica-checkbox-row"><input checked type="checkbox" />人物动作</label><label class="replica-checkbox-row"><input checked type="checkbox" />产品卖点</label><label class="replica-checkbox-row"><input checked type="checkbox" />可用文案</label></section>
        <p class="replica-progress-text">已完成 {{ analyzedSegmentCount }}/{{ splitSegmentPaths.length }} 个片段的内容提炼</p>
        <button class="replica-analyze-button" type="button" :disabled="splitSegmentPaths.length === 0 || isProcessing" @click="emit('analyzeContent')">{{ isProcessing ? '正在分析...' : '开始内容提炼' }}</button>
        <button class="replica-analyze-button is-primary" type="button" :disabled="!allSegmentsAnalyzed || isProcessing" @click="emit('generateCategorizedSegments')">按提炼结果生成精华成片</button>
      </template>
      <p v-if="progressText" class="replica-progress-text">{{ progressText }}</p><p v-if="error" class="workflow-error" role="alert">{{ error }}</p>
    </aside>

    <footer v-if="view === 'effects'" class="replica-tools-output-bar">
      <label>分辨率<select v-model="outputResolution"><option value="followCanvas">跟随画布</option><option value="hd720">720P</option><option value="fullHd1080">1080P</option></select></label>
      <label>帧率<select v-model="outputFrameRate"><option value="source">原帧率</option><option value="fps24">24fps</option><option value="fps25">25fps</option><option value="fps30">30fps</option><option value="fps50">50fps</option><option value="fps60">60fps</option></select></label>
      <label>格式<select v-model="outputFormat"><option value="mp4">MP4</option><option value="mov">MOV</option><option value="webm">WEBM</option></select></label>
      <label>原文件<select v-model="keepOriginal"><option value="keep">保留</option><option value="remove">不保留</option></select></label>
      <label>命名<select v-model="namingMode"><option value="serial">前缀序号</option><option value="source">原文件名_处理</option></select></label>
      <label>线程<select v-model="threadMode"><option value="single">1线程</option><option value="auto">自动</option><option value="multi">多线程</option></select></label>
      <label>裂变<input v-model.number="variantCount" type="number" min="1" max="100" /></label>
      <div class="replica-tools-output-summary"><span>素材 {{ importedVideos.length }}</span><span>完成 {{ Object.values(videoProcessingStates).filter((state) => state.status === 'completed').length }}</span><span v-if="isProcessing">总进度 {{ Math.round(processingProgress) }}%</span></div><button type="button" :disabled="!importedVideoCount || !outputDirectory || isProcessing || enabledEffectKeys.length === 0" @click="emit('startProcessing')">{{ isProcessing ? '处理中...' : '开始处理' }}</button>
    </footer>

    <div v-if="guideOpen" class="replica-tools-guide-backdrop" role="presentation" @click.self="closeGuide">
      <section class="replica-tools-guide" role="dialog" aria-modal="true" aria-labelledby="video-tools-guide-title">
        <header>
          <div><h2 id="video-tools-guide-title">视频效果处理 - 使用指引</h2><small>操作步骤</small></div>
          <button type="button" aria-label="关闭使用指引" @click="closeGuide">×</button>
        </header>
        <main>
          <article v-for="(step, index) in guideSteps" :key="step.title">
            <b>{{ index + 1 }}</b><div><strong>{{ step.title }}</strong><p>{{ step.text }}</p></div>
          </article>
          <div class="replica-tools-guide__tip"><strong>提示</strong><p>裂变数量大于 1 时，每个视频会输出多个版本，各版本的效果参数完全随机，用于差异化处理。启用分割功能后裂变功能自动关闭。</p></div>
        </main>
        <footer>
          <label><input v-model="guideDontRemind" type="checkbox" /> 下次不再提醒</label>
          <span>有疑问？花几分钟看一遍操作演示，上手更快</span>
          <button type="button" @click="closeGuide">开始使用</button>
        </footer>
      </section>
    </div>
  </section>
</template>
