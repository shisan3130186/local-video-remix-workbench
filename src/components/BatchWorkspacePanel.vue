<script setup lang="ts">
import { computed, ref } from "vue";
import type { ImportedVideo } from "../types/videoProbe";
import type { DrawerKey, ToolKey } from "../types/workbench";

type BatchToolKey = Extract<ToolKey, "remix" | "canvas" | "effects" | "transition" | "pip" | "bgm" | "watermark" | "export">;

const props = defineProps<{
  kind: "remix" | "category";
  importedVideos: ImportedVideo[];
  selectedVideo: ImportedVideo | null;
  sourcePreviewUrl: string | null;
  previewUrl: string | null;
  videoCoverUrls: Record<string, string>;
  splitSegmentPaths: string[];
  selectedSegmentPath: string | null;
  segmentThumbnailUrls: Record<string, string>;
  randomSelectedCount: number;
  batchGenerateCount: number;
  batchMixResultCount: number;
  isSplitting: boolean;
  isMixing: boolean;
  isBatchMixing: boolean;
  splitError: string | null;
  mixError: string | null;
  batchMixError: string | null;
  formatDuration: (durationSeconds: number | null) => string;
  formatResolution: (video: ImportedVideo) => string;
  formatFileName: (path: string) => string;
}>();

const emit = defineEmits<{
  importVideos: [];
  importVideoFolder: [];
  selectVideo: [video: ImportedVideo];
  selectSegment: [segmentPath: string];
  splitSelectedVideo: [];
  pickSegmentsRandomly: [];
  concatRandomSegments: [];
  concatCategorizedSegments: [];
  generateBatchMixes: [];
  openAiWorkspace: [];
  openTool: [tool: BatchToolKey];
  openDrawer: [drawer: DrawerKey];
  "update:batchGenerateCount": [count: number];
}>();

const script = defineModel<string>("script", { required: true });
const originalVolume = defineModel<number>("originalVolume", { required: true });
const bgmEnabled = defineModel<boolean>("bgmEnabled", { required: true });
const activeTab = ref<"basic" | "visual">("basic");
const narrationMode = ref<"custom" | "copy" | "audio">("custom");
const categoryNames = ["开头", "产品", "细节", "效果", "场景", "结尾"];
const selectedSegmentTitle = computed(() => props.selectedSegmentPath ? props.formatFileName(props.selectedSegmentPath) : "等待生成切片");
const previewOrientationClass = computed(() => {
  const width = props.selectedVideo?.width ?? 0;
  const height = props.selectedVideo?.height ?? 0;
  return width > height ? "is-landscape" : "is-portrait";
});

function startBatch() {
  if (props.kind === "category") emit("concatCategorizedSegments");
  else emit("generateBatchMixes");
}
</script>

<template>
  <section class="replica-batch-workspace" :class="`is-${kind}`" :aria-label="kind === 'category' ? '分类混剪工作台' : '视频混剪工作台'">
    <aside class="replica-batch-source">
      <div class="replica-source-toolbar">
        <button class="is-primary" type="button" @click="emit('importVideoFolder')">▣ 导入文件夹</button>
        <button type="button" @click="emit('importVideos')">导入视频</button>
      </div>
      <div v-if="importedVideos.length === 0" class="replica-source-empty"><span>□</span><strong>暂无视频素材</strong><small>点击导入按钮或拖拽添加素材</small></div>
      <div v-else class="replica-batch-source-list">
        <button v-for="video in importedVideos" :key="video.id" type="button" :class="{ 'is-active': selectedVideo?.id === video.id }" @click="emit('selectVideo', video)">
          <img v-if="videoCoverUrls[video.id]" :src="videoCoverUrls[video.id]" alt="" /><span v-else>▶</span>
          <b>{{ video.fileName }}</b><small>{{ formatDuration(video.durationSeconds) }}</small>
        </button>
      </div>
      <footer class="replica-batch-source-footer">
        <template v-if="kind === 'category'">
          <label><span>导出数量</span><input :value="batchGenerateCount" type="number" min="1" max="20" @input="emit('update:batchGenerateCount', Number(($event.target as HTMLInputElement).value))" /></label>
          <div class="replica-mode-switch"><button v-for="mode in (['custom','copy','audio'] as const)" :key="mode" type="button" :class="{ 'is-active': narrationMode === mode }" @click="narrationMode = mode">{{ mode === 'custom' ? '自定义' : mode === 'copy' ? '文案' : '音频' }}</button></div>
        </template>
        <template v-else><strong>已导入 {{ importedVideos.length }} 个视频</strong><small>切片后将随机重组生成差异版本</small></template>
      </footer>
    </aside>

    <main class="replica-batch-center">
      <section class="replica-batch-preview">
        <header><strong>ⓘ 使用指引</strong><span>{{ selectedVideo ? formatResolution(selectedVideo) : '' }}</span></header>
        <div v-if="sourcePreviewUrl" class="replica-batch-screen">
          <video
            :src="sourcePreviewUrl"
            :class="previewOrientationClass"
            :width="selectedVideo?.width ?? undefined"
            :height="selectedVideo?.height ?? undefined"
            controls
            playsinline
            preload="metadata"
          ></video>
        </div>
        <div v-else class="replica-batch-screen replica-batch-screen--empty"><span>□</span><strong>选择素材后在此预览</strong></div>
      </section>

      <section class="replica-batch-content">
        <header><strong>{{ kind === 'category' ? '切片分类' : '视频文案' }}</strong><span>{{ splitSegmentPaths.length }} 个片段</span></header>
        <template v-if="kind === 'remix'">
          <textarea v-model="script" maxlength="4000" placeholder="输入配音文案；不需要配音可留空"></textarea>
          <div class="replica-batch-inline-actions">
            <button type="button" :disabled="!selectedVideo || isSplitting" @click="emit('splitSelectedVideo')">{{ isSplitting ? '正在切片...' : '生成视频切片' }}</button>
            <button type="button" :disabled="splitSegmentPaths.length < 2" @click="emit('pickSegmentsRandomly')">随机抽取</button>
            <button type="button" :disabled="randomSelectedCount === 0 || isMixing" @click="emit('concatRandomSegments')">生成样片</button>
          </div>
        </template>
        <template v-else>
          <div v-if="splitSegmentPaths.length === 0" class="replica-category-empty"><strong>请先生成切片</strong><small>切片完成后可按镜头用途分类组合</small><button type="button" :disabled="!selectedVideo || isSplitting" @click="emit('splitSelectedVideo')">{{ isSplitting ? '正在切片...' : '开始智能切片' }}</button></div>
          <div v-else class="replica-category-grid">
            <button v-for="(segmentPath, index) in splitSegmentPaths" :key="segmentPath" type="button" :class="{ 'is-active': selectedSegmentPath === segmentPath }" @click="emit('selectSegment', segmentPath)">
              <img v-if="segmentThumbnailUrls[segmentPath]" :src="segmentThumbnailUrls[segmentPath]" alt="" /><span v-else>▶</span><b>{{ categoryNames[index % categoryNames.length] }}</b><small>{{ formatFileName(segmentPath) }}</small>
            </button>
          </div>
          <div class="replica-current-segment"><strong>{{ selectedSegmentTitle }}</strong><video v-if="selectedSegmentPath && previewUrl" :src="previewUrl" controls playsinline preload="metadata"></video></div>
        </template>
        <p v-if="splitError || mixError || batchMixError" class="workflow-error" role="alert">{{ splitError || mixError || batchMixError }}</p>
      </section>
    </main>

    <aside class="replica-batch-settings">
      <div class="replica-settings-tabs"><button type="button" :class="{ 'is-active': activeTab === 'basic' }" @click="activeTab = 'basic'">基础设置</button><button type="button" :class="{ 'is-active': activeTab === 'visual' }" @click="activeTab = 'visual'">画面处理</button></div>
      <div v-if="activeTab === 'basic'" class="replica-settings-scroll">
        <section class="replica-setting-block replica-volume-row"><label>原视频音量</label><input v-model.number="originalVolume" type="range" min="0" max="2" step="0.05" /><output>{{ Math.round(originalVolume * 100) }}%</output></section>
        <section class="replica-setting-block"><header><strong>背景音乐</strong><label class="replica-switch"><input v-model="bgmEnabled" type="checkbox" /><span></span></label></header><button class="replica-file-field" type="button" @click="emit('openTool','bgm')"><span>选择本地音乐</span><b>▢</b></button></section>
        <section class="replica-setting-block"><header><strong>混剪方式</strong></header><button class="replica-setting-link" type="button" @click="emit('openTool','remix')">智能切片与组合规则 <b>›</b></button><button class="replica-setting-link" type="button" @click="emit('openTool','export')">导出数量与输出规格 <b>›</b></button></section>
      </div>
      <div v-else class="replica-settings-scroll replica-visual-tools">
        <button v-for="item in ([['canvas','画布比例'],['effects','画面调整'],['transition','转场衔接'],['pip','画中画'],['watermark','水印处理']] as const)" :key="item[0]" type="button" @click="emit('openTool', item[0])"><span>{{ item[1] }}</span><small>打开完整参数设置</small><b>›</b></button>
      </div>
      <footer class="replica-batch-create"><button type="button" :disabled="splitSegmentPaths.length < 2 || isBatchMixing || isMixing" @click="startBatch">{{ isBatchMixing || isMixing ? '正在生成...' : kind === 'category' ? '开始分类混剪' : '开始批量混剪' }}</button><div><button type="button" @click="emit('openDrawer','logs')">任务日志</button><button type="button" @click="emit('openDrawer','batch')">结果 {{ batchMixResultCount }}</button></div></footer>
    </aside>
  </section>
</template>
