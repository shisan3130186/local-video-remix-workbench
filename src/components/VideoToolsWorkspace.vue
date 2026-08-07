<script setup lang="ts">
import { computed, ref } from "vue";
import type { AiRemixSegment } from "../features/ai-remix";
import type { ImportedVideo } from "../types/videoProbe";
import type { DrawerKey, ToolKey } from "../types/workbench";
import type { OutputFrameRate, OutputQuality, OutputResolution, VideoEncoder } from "../types/outputSettings";

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

const splitMode = ref<"scene" | "fixed">("scene");
const segmentSeconds = ref(5);
const previewOrientationClass = computed(() => {
  const width = props.selectedVideo?.width ?? 0;
  const height = props.selectedVideo?.height ?? 0;
  return width > height ? "is-landscape" : "is-portrait";
});
const outputResolution = defineModel<OutputResolution>("outputResolution", { required: true });
const outputFrameRate = defineModel<OutputFrameRate>("outputFrameRate", { required: true });
const outputQuality = defineModel<OutputQuality>("outputQuality", { required: true });
const outputEncoder = defineModel<VideoEncoder>("outputEncoder", { required: true });
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

const effectGroups: Array<{ title: string; key: ToolKey; note: string }> = [
  { title: "基础画面", key: "effects", note: "镜像、旋转、变速、缩放与色彩" },
  { title: "去除水印", key: "watermark", note: "固定区域、移动跟踪、模糊与遮盖" },
  { title: "画布背景", key: "canvas", note: "比例适配、模糊背景与裁切方式" },
  { title: "画中画", key: "pip", note: "融合图片或视频叠加素材" },
  { title: "背景音乐", key: "bgm", note: "原声、音乐音量与淡入淡出" },
  { title: "视频封面", key: "cover", note: "截取画面并设置导出封面" },
];
</script>

<template>
  <section class="replica-tools-workspace" :class="`is-${view}`" :aria-label="view === 'effects' ? '视频效果处理工作台' : '视频内容提炼工作台'">
    <aside class="replica-tools-source">
      <div class="replica-source-toolbar"><button class="is-primary" type="button" @click="emit('importVideos')">导入视频</button><button type="button" @click="emit('importVideoFolder')">文件夹</button><button type="button" :disabled="!importedVideoCount" @click="emit('clearVideos')">清空</button></div>
      <div v-if="!importedVideoCount" class="replica-source-empty"><span>□</span><strong>暂无视频文件</strong><small>支持导入单个、多选或整个文件夹</small></div>
      <div v-else class="replica-tools-file-list"><button v-for="video in importedVideos" :key="video.id" type="button" :class="{ 'is-active': selectedVideo?.id === video.id }" @click="emit('selectVideo', video)"><img v-if="videoCoverUrls[video.id]" :src="videoCoverUrls[video.id]" alt="" /><span v-else>▶</span><b>{{ video.fileName }}</b><small>{{ formatDuration(video.durationSeconds) }}</small></button></div>
      <button class="replica-output-path" type="button" @click="emit('selectOutputDirectory')"><span>输出路径</span><strong>{{ outputDirectory ?? '请选择输出文件夹' }}</strong><b>▢</b></button>
    </aside>

    <main class="replica-tools-center">
      <section class="replica-tools-preview">
        <header><strong>{{ view === 'effects' ? '视频预览' : '原视频预览' }}</strong><span>{{ selectedVideo?.fileName ?? '' }}</span></header>
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
        <div v-else class="replica-tools-screen replica-tools-screen--empty"><span>□</span><strong>选择视频后在此预览</strong></div>
      </section>

      <section v-if="view === 'effects'" class="replica-split-settings">
        <header><strong>视频分割</strong><small>长视频可以先拆成多个片段再统一处理</small></header>
        <div class="replica-split-options"><button type="button" :class="{ 'is-active': splitMode === 'scene' }" @click="splitMode = 'scene'">智能场景分割</button><button type="button" :class="{ 'is-active': splitMode === 'fixed' }" @click="splitMode = 'fixed'">固定时长分割</button><label v-if="splitMode === 'fixed'">每段 <input v-model.number="segmentSeconds" type="number" min="1" max="120" /> 秒</label><button class="is-primary" type="button" :disabled="!selectedVideo || isProcessing" @click="emit('splitSelectedVideo')">{{ isProcessing ? '处理中...' : '开始分割' }}</button></div>
      </section>

      <section v-else class="replica-extract-results">
        <header><strong>提炼片段</strong><span>{{ splitSegmentPaths.length }} 个</span></header>
        <div v-if="splitSegmentPaths.length === 0" class="replica-category-empty"><strong>还没有内容片段</strong><small>先切片，再用本地或云端模型分析片段内容</small><button type="button" :disabled="!selectedVideo || isProcessing" @click="emit('splitSelectedVideo')">生成内容切片</button></div>
        <div v-else class="replica-extract-grid"><button v-for="segmentPath in splitSegmentPaths" :key="segmentPath" type="button" :class="{ 'is-active': selectedSegmentPath === segmentPath }" @click="emit('selectSegment', segmentPath)"><img v-if="segmentThumbnailUrls[segmentPath]" :src="segmentThumbnailUrls[segmentPath]" alt="" /><span v-else>▶</span><b>{{ formatFileName(segmentPath) }}</b><small v-if="preparedSegmentByPath.get(segmentPath)?.contentAnalysis">{{ preparedSegmentByPath.get(segmentPath)?.contentAnalysis?.theme }}</small><small v-else>等待内容分析</small></button></div>
      </section>
    </main>

    <aside class="replica-tools-settings">
      <template v-if="view === 'effects'">
        <header><strong>效果设置</strong><small>点击卡片打开详细参数</small></header>
        <div class="replica-effect-list"><button v-for="group in effectGroups" :key="group.key" type="button" @click="emit('openTool', group.key)"><span><strong>{{ group.title }}</strong><small>{{ group.note }}</small></span><b>›</b></button></div>
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
      <label>质量<select v-model="outputQuality"><option value="compact">节省空间</option><option value="standard">标准</option><option value="high">高质量</option></select></label>
      <label>编码<select v-model="outputEncoder"><option value="auto">自动</option><option value="cpu">CPU</option><option value="nvidia">NVIDIA</option><option value="intel">Intel</option><option value="amd">AMD</option></select></label>
      <span>格式 MP4</span><span>保留原文件</span><span>命名 原文件名_处理</span><span>线程 自动</span>
      <button type="button" :disabled="!selectedVideo || !outputDirectory || isProcessing" @click="emit('startProcessing')">{{ isProcessing ? '处理中...' : '开始处理' }}</button>
    </footer>
  </section>
</template>
