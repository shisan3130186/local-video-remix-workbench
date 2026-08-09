<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { ImportedVideo } from "../types/videoProbe";
import type { FfmpegEnvironmentResult } from "../types/videoProbe";
import type { DrawerKey, ToolKey, WorkspaceMode } from "../types/workbench";
import type { CanvasAspectRatio, CanvasBackgroundMode, DynamicZoomMode } from "../services/videoMixService";
import type { WatermarkAssetType, WatermarkTrajectory } from "../features/watermark/types";
import type { FixedMaterialKind, MaterialFolder, MaterialFolderSettings, MaterialFolderVideoMode } from "../features/materials/types";
import MaterialFolderSettingsPanel from "../features/materials/components/MaterialFolderSettingsPanel.vue";
import BatchPreviewCanvas from "./BatchPreviewCanvas.vue";
import RightToolPanel from "./RightToolPanel.vue";
import ToolIcon from "./ToolIcon.vue";

type BatchToolKey = Extract<ToolKey, "remix" | "canvas" | "effects" | "transition" | "pip" | "bgm" | "watermark" | "export" | "tts" | "subtitleStyle">;

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
  materialFolders: MaterialFolder[];
  productionWorkspaceMode: Exclude<WorkspaceMode, "tools">;
  isAdvancedMode: boolean;
  environment: FfmpegEnvironmentResult | null;
  statusText: string;
  bgmAudioFilePath: string | null;
  isProcessing: boolean;
  formatDuration: (durationSeconds: number | null) => string;
  formatResolution: (video: ImportedVideo) => string;
  formatFileName: (path: string) => string;
}>();

const emit = defineEmits<{
  importVideoFolder: [];
  selectVideo: [video: ImportedVideo];
  selectSegment: [segmentPath: string];
  splitSelectedVideo: [];
  pickSegmentsRandomly: [];
  concatRandomSegments: [];
  concatCategorizedSegments: [];
  generateBatchMixes: [];
  selectAudioSource: [];
  recognizeAudio: [];
  useRecognizedAudio: [];
  openTool: [tool: BatchToolKey];
  openDrawer: [drawer: DrawerKey];
  toggleAdvancedMode: [enabled: boolean];
  selectBgmAudioFile: [];
  selectWatermarkAsset: [];
  updateMaterialFolderSettings: [folderId: string, patch: Partial<MaterialFolderSettings>];
  selectFixedMaterial: [folderId: string, kind: FixedMaterialKind];
  applyFolderSettingsToAll: [settings: MaterialFolderSettings];
  "update:batchGenerateCount": [count: number];
}>();

const script = defineModel<string>("script", { required: true });
const originalVolume = defineModel<number>("originalVolume", { required: true });
const bgmEnabled = defineModel<boolean>("bgmEnabled", { required: true });
const bgmVolume = defineModel<number>("bgmVolume", { required: true });
const ttsEnabled = defineModel<boolean>("ttsEnabled", { required: true });
const ttsSpeaker = defineModel<string>("ttsSpeaker", { required: true });
const speechVolume = defineModel<number>("speechVolume", { required: true });
const speechSpeed = defineModel<number>("speechSpeed", { required: true });
const subtitleEnabled = defineModel<boolean>("subtitleEnabled", { required: true });
const subtitlePosition = defineModel<string>("subtitlePosition", { required: true });
const subtitleSize = defineModel<string>("subtitleSize", { required: true });
const watermarkRemovalEnabled = defineModel<boolean>("watermarkRemovalEnabled", { required: true });
const watermarkRemovalRegionCount = defineModel<number>("watermarkRemovalRegionCount", { required: true });
const watermarkEnabled = defineModel<boolean>("watermarkEnabled", { required: true });
const watermarkAssetType = defineModel<WatermarkAssetType>("watermarkAssetType", { required: true });
const watermarkImageFilePath = defineModel<string | null>("watermarkImageFilePath", { required: true });
const watermarkOpacity = defineModel<number>("watermarkOpacity", { required: true });
const watermarkImageSizeRatio = defineModel<number>("watermarkImageSizeRatio", { required: true });
const watermarkTrajectory = defineModel<WatermarkTrajectory>("watermarkTrajectory", { required: true });
const hslEnabled = defineModel<boolean>("hslEnabled", { required: true });
const hue = defineModel<number>("hue", { required: true });
const brightness = defineModel<number>("brightness", { required: true });
const saturation = defineModel<number>("saturation", { required: true });
const zoomEnabled = defineModel<boolean>("zoomEnabled", { required: true });
const zoomMode = defineModel<DynamicZoomMode>("zoomMode", { required: true });
const zoomMinScale = defineModel<number>("zoomMinScale", { required: true });
const zoomMaxScale = defineModel<number>("zoomMaxScale", { required: true });
const zoomMinDurationSeconds = defineModel<number>("zoomMinDurationSeconds", { required: true });
const zoomMaxDurationSeconds = defineModel<number>("zoomMaxDurationSeconds", { required: true });
const canvasAspectRatio = defineModel<CanvasAspectRatio>("canvasAspectRatio", { required: true });
const canvasBackgroundMode = defineModel<CanvasBackgroundMode>("canvasBackgroundMode", { required: true });

const activeTab = ref<"basic" | "visual">("basic");
const expandedFolderIds = ref<string[]>([]);
const settingsFolderId = ref<string | null>(null);
const guideOpen = ref(true);
const taskOpen = ref(false);
const scriptCount = ref(1);
const categoryMode = ref<MaterialFolderVideoMode>("custom");

const folderVideos = (folder: MaterialFolder) => props.importedVideos.filter((video) => folder.videoPaths.includes(video.filePath));
const selectedResolution = computed(() => props.selectedVideo ? props.formatResolution(props.selectedVideo) : "未选择素材");
const taskReady = computed(() => props.materialFolders.length > 0 && Boolean(props.selectedVideo));
const categoryModeLabel = computed(() => ({ custom: "自定义", script: "文案模式", audio: "音频模式" })[categoryMode.value]);
const categorySequenceLabel = computed(() => props.materialFolders.map((folder) => folder.folderName).join(" → ") || "未导入分类文件夹");

watch(() => props.kind, () => {
  guideOpen.value = true;
  taskOpen.value = false;
  settingsFolderId.value = null;
  categoryMode.value = "custom";
});

function toggleFolder(folderId: string) {
  expandedFolderIds.value = expandedFolderIds.value.includes(folderId)
    ? expandedFolderIds.value.filter((id) => id !== folderId)
    : [...expandedFolderIds.value, folderId];
}

function openFolderSettings(folderId: string) {
  settingsFolderId.value = settingsFolderId.value === folderId ? null : folderId;
  if (!expandedFolderIds.value.includes(folderId)) expandedFolderIds.value.push(folderId);
}

function submitTask() {
  taskOpen.value = false;
  if (props.kind === "category") {
    emit("concatCategorizedSegments");
    return;
  }
  emit("generateBatchMixes");
}

function setCategoryMode(mode: MaterialFolderVideoMode) {
  categoryMode.value = mode;
  props.materialFolders.forEach((folder) => {
    emit("updateMaterialFolderSettings", folder.id, { videoMode: mode });
  });
}
</script>

<template>
  <section v-if="kind === 'remix'" class="replica-batch-workspace replica-batch-workspace--remix" aria-label="视频混剪工作台">
    <aside class="replica-batch-source replica-batch-source--folders">
       <div class="replica-source-toolbar"><button class="is-primary" type="button" @click="emit('importVideoFolder')"><ToolIcon name="folder" />导入文件夹</button></div>
       <div v-if="materialFolders.length === 0" class="replica-source-empty"><span><ToolIcon name="folder" /></span><strong>暂无素材文件夹</strong><small>拖拽文件夹到窗口，或点击“导入文件夹”添加素材</small></div>
      <div v-else class="replica-source-list replica-source-list--folders">
        <article v-for="folder in materialFolders" :key="folder.id" class="replica-material-folder">
          <header class="replica-material-folder__header">
            <button class="replica-material-folder__toggle" type="button" @click="toggleFolder(folder.id)">
               <span class="replica-material-folder__caret" :class="{ 'is-open': expandedFolderIds.includes(folder.id) }">⌄</span><span class="replica-material-folder__icon"><ToolIcon name="folder" /></span><strong :title="folder.folderPath">{{ folder.folderName }}</strong><small>{{ folderVideos(folder).length }}</small>
            </button>
             <button class="replica-material-folder__settings" type="button" :class="{ 'is-active': settingsFolderId === folder.id }" aria-label="文件夹混剪设置" @click.stop="openFolderSettings(folder.id)"><ToolIcon name="settings" /></button>
          </header>
          <div v-if="expandedFolderIds.includes(folder.id) && settingsFolderId !== folder.id" class="replica-material-folder__body">
            <button v-for="video in folderVideos(folder)" :key="video.id" type="button" class="replica-material-video" :class="{ 'is-active': selectedVideo?.id === video.id }" @click="emit('selectVideo', video)">
              <img v-if="videoCoverUrls[video.id]" :src="videoCoverUrls[video.id]" alt="" /><span v-else>▶</span><b>{{ video.fileName }}</b><small>{{ formatDuration(video.durationSeconds) }}</small>
            </button>
          </div>
          <MaterialFolderSettingsPanel v-if="settingsFolderId === folder.id" mode="batch" :settings="folder.settings" :apply-to-all-disabled="materialFolders.length < 2" @update="emit('updateMaterialFolderSettings', folder.id, $event)" @select-fixed-material="emit('selectFixedMaterial', folder.id, $event)" @apply-to-all="emit('applyFolderSettingsToAll', folder.settings)" />
        </article>
      </div>
      <footer class="replica-batch-source-footer replica-batch-source-footer--folders"><strong>已导入 {{ materialFolders.length }} 个素材文件夹</strong><small>文件夹是独立素材库，可分别配置混剪规则</small></footer>
    </aside>

    <main class="replica-batch-center replica-batch-center--remix">
      <BatchPreviewCanvas v-model:canvas-aspect-ratio="canvasAspectRatio" v-model:canvas-background-mode="canvasBackgroundMode" :preview-url="sourcePreviewUrl" :selected-video="selectedVideo" @open-guide="guideOpen = true" />
      <section class="replica-batch-content replica-batch-content--script">
        <header><div><strong>视频文案</strong><small>文案模式下，每条文案生成一个视频</small></div><span>{{ scriptCount }} 条文案</span></header>
        <div class="replica-script-tabs"><button type="button" class="is-active">文案 1</button><button type="button" @click="scriptCount += 1">＋ 添加文案</button><button type="button" :disabled="!script.trim()" @click="script = `${script}\n\n`">批量粘贴</button></div>
        <textarea v-model="script" maxlength="4000" placeholder="请粘贴视频文案；每条文案将按文件夹设置生成对应视频"></textarea>
        <div class="replica-batch-script-footer"><span>支持多条文案 · 可在右侧设置语音与字幕</span><button type="button" :disabled="!script.trim()" @click="script = script.trim()">AI 改写</button></div>
      </section>
    </main>

    <aside class="replica-batch-right-column">
      <RightToolPanel
        :workspace-mode="productionWorkspaceMode"
        :is-advanced-mode="isAdvancedMode"
        :environment="environment"
        :status-text="statusText"
        :bgm-audio-file-path="bgmAudioFilePath"
        :is-processing="isProcessing"
        v-model:original-volume="originalVolume"
        v-model:bgm-enabled="bgmEnabled"
        v-model:bgm-volume="bgmVolume"
        v-model:tts-enabled="ttsEnabled"
        v-model:tts-speaker="ttsSpeaker"
        v-model:speech-volume="speechVolume"
        v-model:speech-speed="speechSpeed"
        v-model:subtitle-enabled="subtitleEnabled"
        v-model:subtitle-position="subtitlePosition"
        v-model:subtitle-size="subtitleSize"
        v-model:watermark-removal-enabled="watermarkRemovalEnabled"
        v-model:watermark-removal-region-count="watermarkRemovalRegionCount"
        v-model:watermark-enabled="watermarkEnabled"
        v-model:watermark-asset-type="watermarkAssetType"
        v-model:watermark-image-file-path="watermarkImageFilePath"
        v-model:watermark-opacity="watermarkOpacity"
        v-model:watermark-image-size-ratio="watermarkImageSizeRatio"
        v-model:watermark-trajectory="watermarkTrajectory"
        v-model:hsl-enabled="hslEnabled"
        v-model:hue="hue"
        v-model:brightness="brightness"
        v-model:saturation="saturation"
        v-model:zoom-enabled="zoomEnabled"
        v-model:zoom-mode="zoomMode"
        v-model:zoom-min-scale="zoomMinScale"
        v-model:zoom-max-scale="zoomMaxScale"
        v-model:zoom-min-duration-seconds="zoomMinDurationSeconds"
        v-model:zoom-max-duration-seconds="zoomMaxDurationSeconds"
        @open-tool="emit('openTool', $event as BatchToolKey)"
        @open-drawer="emit('openDrawer', $event)"
        @toggle-advanced-mode="emit('toggleAdvancedMode', $event)"
        @select-bgm-audio-file="emit('selectBgmAudioFile')"
        @select-watermark-asset="emit('selectWatermarkAsset')"
      />
      <footer class="replica-batch-create replica-batch-create--task"><button type="button" :disabled="!taskReady || isBatchMixing" @click="taskOpen = true">{{ isBatchMixing ? '正在提交...' : '创建任务' }}</button><div><button type="button" @click="emit('openDrawer', 'logs')">任务日志</button><button type="button" @click="emit('openDrawer', 'batch')">结果 {{ batchMixResultCount }}</button></div></footer>
    </aside>

    <div v-if="taskOpen" class="replica-batch-task-backdrop" @click.self="taskOpen = false">
      <section class="replica-batch-task-dialog" role="dialog" aria-modal="true" aria-label="确认提交混剪任务"><header><strong>创建混剪任务</strong><button type="button" @click="taskOpen = false">×</button></header><div class="replica-batch-task-summary"><p><span>素材文件夹</span><b>{{ materialFolders.length }} 个</b></p><p><span>当前文案</span><b>{{ script.trim() ? '已填写' : '未填写（按自定义模式执行）' }}</b></p><p><span>视频预览</span><b>{{ selectedResolution }}</b></p></div><footer><button type="button" @click="taskOpen = false">返回调整</button><button class="is-primary" type="button" @click="submitTask">提交任务</button></footer></section>
    </div>

    <div v-if="guideOpen" class="replica-batch-guide-backdrop" @click.self="guideOpen = false">
      <section class="replica-batch-guide" role="dialog" aria-modal="true" aria-label="视频混剪使用指引"><header><div><strong>视频混剪 · 使用指引</strong><small>按照以下步骤完成一次混剪任务</small></div><button type="button" @click="guideOpen = false">×</button></header><ol><li><strong>确认素材类型（重要）</strong><span>一镜到底的实拍素材可以直接混剪。如果素材是剪辑过的成片（有多种镜头切换），强烈建议先用「视频效果处理」模块做 AI 智能分割，可显著提升混剪质量。</span></li><li><strong>导入素材文件夹</strong><span>拖拽文件夹到窗口，或点击左侧「导入文件夹」按钮，单击选中目标文件夹，再点弹窗右下角的「选择文件夹」。</span></li><li><strong>配置混剪参数</strong><span>已添加的文件夹会显示齿轮图标，点击可设置混剪模式和素材使用规则。自定义模式手动指定素材数量和导出数量；文案模式粘贴文案后由 AI 配音；音频模式添加已有配音文件并自动识别字幕。</span></li><li><strong>设置配音与字幕</strong><span>在右侧面板选择音色、调整语速音量，开启字幕并调整样式，也可添加背景音乐、图片 / 视频水印、HSL 调色等全局效果。</span></li><li><strong>创建任务 → 提交</strong><span>点击右侧「创建任务」按钮，确认任务列表后点击「提交任务」。软件会自动完成素材抽取、拼接和视频合成。</span></li></ol><footer><label><input type="checkbox" /> 下次不再自动显示</label><button class="is-primary" type="button" @click="guideOpen = false">开始使用</button></footer></section>
    </div>
  </section>

  <section v-else class="replica-batch-workspace replica-batch-workspace--category" aria-label="分类混剪工作台">
    <aside class="replica-batch-source replica-batch-source--folders">
       <div class="replica-source-toolbar"><button class="is-primary" type="button" @click="emit('importVideoFolder')"><ToolIcon name="folder" />导入文件夹</button></div>
       <div v-if="materialFolders.length === 0" class="replica-source-empty"><span><ToolIcon name="folder" /></span><strong>暂无分类文件夹</strong><small>拖拽文件夹到窗口，或点击“导入文件夹”添加分类素材</small></div>
      <div v-else class="replica-source-list replica-source-list--folders">
        <article v-for="(folder, index) in materialFolders" :key="folder.id" class="replica-material-folder">
          <header class="replica-material-folder__header">
            <button class="replica-material-folder__toggle" type="button" @click="toggleFolder(folder.id)">
               <span class="replica-material-folder__caret" :class="{ 'is-open': expandedFolderIds.includes(folder.id) }"><ToolIcon name="chevron" /></span><span class="replica-material-folder__icon"><ToolIcon name="folder" /></span><strong :title="folder.folderPath">{{ index + 1 }}. {{ folder.folderName }}</strong><small>{{ folderVideos(folder).length }}</small>
            </button>
            <button class="replica-material-folder__settings" type="button" :class="{ 'is-active': settingsFolderId === folder.id }" aria-label="分类文件夹设置" @click.stop="openFolderSettings(folder.id)"><ToolIcon name="settings" /></button>
          </header>
          <div v-if="expandedFolderIds.includes(folder.id) && settingsFolderId !== folder.id" class="replica-material-folder__body">
            <button v-for="video in folderVideos(folder)" :key="video.id" type="button" class="replica-material-video" :class="{ 'is-active': selectedVideo?.id === video.id }" @click="emit('selectVideo', video)">
              <img v-if="videoCoverUrls[video.id]" :src="videoCoverUrls[video.id]" alt="" /><span v-else><ToolIcon name="video" /></span><b>{{ video.fileName }}</b><small>{{ formatDuration(video.durationSeconds) }}</small>
            </button>
          </div>
          <MaterialFolderSettingsPanel v-if="settingsFolderId === folder.id" mode="category" :settings="folder.settings" :apply-to-all-disabled="materialFolders.length < 2" @update="emit('updateMaterialFolderSettings', folder.id, $event)" @select-fixed-material="emit('selectFixedMaterial', folder.id, $event)" @apply-to-all="emit('applyFolderSettingsToAll', folder.settings)" />
        </article>
      </div>
      <footer class="replica-batch-source-footer replica-batch-source-footer--folders replica-category-source-footer">
        <div class="replica-category-mode-heading"><strong>混剪模式</strong><small>{{ categoryModeLabel }}</small></div>
        <div class="replica-mode-switch" role="group" aria-label="全局混剪模式">
          <button type="button" :class="{ 'is-active': categoryMode === 'custom' }" :aria-pressed="categoryMode === 'custom'" @click="setCategoryMode('custom')">自定义</button>
          <button type="button" :class="{ 'is-active': categoryMode === 'script' }" :aria-pressed="categoryMode === 'script'" @click="setCategoryMode('script')">文案模式</button>
          <button type="button" :class="{ 'is-active': categoryMode === 'audio' }" :aria-pressed="categoryMode === 'audio'" @click="setCategoryMode('audio')">音频模式</button>
        </div>
        <label class="replica-category-count"><span>混剪导出数量</span><span class="replica-number-field"><input :value="batchGenerateCount" type="number" min="1" max="999" @input="emit('update:batchGenerateCount', Math.max(1, Math.floor(Number(($event.target as HTMLInputElement).value) || 1)))" /><em>条</em></span></label>
        <small>已导入 {{ materialFolders.length }} 个分类文件夹，顺序决定成片结构</small>
      </footer>
    </aside>

    <main class="replica-batch-center replica-batch-center--category">
      <BatchPreviewCanvas v-model:canvas-aspect-ratio="canvasAspectRatio" v-model:canvas-background-mode="canvasBackgroundMode" :preview-url="sourcePreviewUrl" :selected-video="selectedVideo" @open-guide="guideOpen = true" />
      <section class="replica-batch-content replica-batch-content--category">
        <header><div><strong>分类素材</strong><small>按左侧文件夹顺序抽取、拼接并生成视频</small></div><span>{{ materialFolders.length }} 个分类</span></header>
        <div v-if="materialFolders.length === 0" class="replica-category-empty"><strong>请先导入分类文件夹</strong><small>每个文件夹代表一个视频分类，文件夹顺序就是最终拼接顺序</small><button type="button" @click="emit('importVideoFolder')">导入文件夹</button></div>
        <template v-else>
          <div class="replica-category-sequence" aria-label="分类顺序">
            <button v-for="(folder, index) in materialFolders" :key="folder.id" type="button" @click="folderVideos(folder)[0] && emit('selectVideo', folderVideos(folder)[0])">
              <span>{{ index + 1 }}</span><b>{{ folder.folderName }}</b><small>{{ folderVideos(folder).length }} 个素材</small>
            </button>
          </div>
          <div v-if="categoryMode === 'custom'" class="replica-category-rules">
            <div class="replica-category-rules__heading"><strong>分类参数概览</strong><small>点击左侧齿轮可逐个设置截取范围、数量与抽取方式</small></div>
            <div v-for="folder in materialFolders" :key="folder.id" class="replica-category-rule-row"><span>{{ folder.folderName }}</span><small>{{ folder.settings.clipMinSeconds || 0 }}-{{ folder.settings.clipMaxSeconds || '不限' }} 秒 · {{ folder.settings.materialCount }} 个 · {{ folder.settings.extractionOrder === 'ordered' ? '顺序抽取' : '随机抽取' }}</small></div>
          </div>
          <div v-else class="replica-category-script-area">
            <div class="replica-script-tabs"><button type="button" class="is-active">{{ categoryMode === 'audio' ? '配音音频' : '视频文案' }}</button><button v-if="categoryMode === 'script'" type="button" @click="scriptCount += 1">＋ 添加文案</button><button v-if="categoryMode === 'script'" type="button" :disabled="!script.trim()" @click="script = `${script}\n\n`">批量粘贴</button></div>
            <textarea v-model="script" maxlength="4000" :placeholder="categoryMode === 'audio' ? '请添加已有配音音频，系统会自动识别语音并生成字幕' : '请粘贴视频文案；每条文案将按分类顺序生成对应视频'"></textarea>
            <div class="replica-batch-script-footer"><span>{{ categoryMode === 'audio' ? '音频模式将自动识别语音并生成字幕' : `支持多条文案 · 当前 ${scriptCount} 条` }}</span><div><button v-if="categoryMode === 'audio'" type="button" @click="emit('selectAudioSource')">选择音频</button><button v-if="categoryMode === 'audio'" type="button" @click="emit('recognizeAudio')">识别字幕</button><button v-if="categoryMode === 'audio'" type="button" :disabled="!script.trim()" @click="emit('useRecognizedAudio')">使用识别结果</button><button v-else type="button" :disabled="!script.trim()" @click="script = script.trim()">AI 改写</button></div></div>
          </div>
        </template>
        <p v-if="splitError || mixError || batchMixError" class="workflow-error" role="alert">{{ splitError || mixError || batchMixError }}</p>
      </section>
    </main>

    <aside class="replica-batch-right-column">
      <RightToolPanel
        :workspace-mode="productionWorkspaceMode"
        :is-advanced-mode="isAdvancedMode"
        :environment="environment"
        :status-text="statusText"
        :bgm-audio-file-path="bgmAudioFilePath"
        :is-processing="isProcessing"
        v-model:original-volume="originalVolume"
        v-model:bgm-enabled="bgmEnabled"
        v-model:bgm-volume="bgmVolume"
        v-model:tts-enabled="ttsEnabled"
        v-model:tts-speaker="ttsSpeaker"
        v-model:speech-volume="speechVolume"
        v-model:speech-speed="speechSpeed"
        v-model:subtitle-enabled="subtitleEnabled"
        v-model:subtitle-position="subtitlePosition"
        v-model:subtitle-size="subtitleSize"
        v-model:watermark-removal-enabled="watermarkRemovalEnabled"
        v-model:watermark-removal-region-count="watermarkRemovalRegionCount"
        v-model:watermark-enabled="watermarkEnabled"
        v-model:watermark-asset-type="watermarkAssetType"
        v-model:watermark-image-file-path="watermarkImageFilePath"
        v-model:watermark-opacity="watermarkOpacity"
        v-model:watermark-image-size-ratio="watermarkImageSizeRatio"
        v-model:watermark-trajectory="watermarkTrajectory"
        v-model:hsl-enabled="hslEnabled"
        v-model:hue="hue"
        v-model:brightness="brightness"
        v-model:saturation="saturation"
        v-model:zoom-enabled="zoomEnabled"
        v-model:zoom-mode="zoomMode"
        v-model:zoom-min-scale="zoomMinScale"
        v-model:zoom-max-scale="zoomMaxScale"
        v-model:zoom-min-duration-seconds="zoomMinDurationSeconds"
        v-model:zoom-max-duration-seconds="zoomMaxDurationSeconds"
        @open-tool="emit('openTool', $event as BatchToolKey)"
        @open-drawer="emit('openDrawer', $event)"
        @toggle-advanced-mode="emit('toggleAdvancedMode', $event)"
        @select-bgm-audio-file="emit('selectBgmAudioFile')"
        @select-watermark-asset="emit('selectWatermarkAsset')"
      />
      <footer class="replica-batch-create replica-batch-create--task"><button type="button" :disabled="!taskReady || isBatchMixing" @click="taskOpen = true">{{ isBatchMixing ? '正在提交...' : '创建任务' }}</button><div><button type="button" @click="emit('openDrawer', 'logs')">任务日志</button><button type="button" @click="emit('openDrawer', 'batch')">结果 {{ batchMixResultCount }}</button></div></footer>
    </aside>

    <div v-if="taskOpen" class="replica-batch-task-backdrop" @click.self="taskOpen = false">
      <section class="replica-batch-task-dialog" role="dialog" aria-modal="true" aria-label="确认提交分类混剪任务"><header><strong>创建分类混剪任务</strong><button type="button" aria-label="关闭任务确认" @click="taskOpen = false">×</button></header><div class="replica-batch-task-summary"><p><span>分类文件夹</span><b>{{ materialFolders.length }} 个</b></p><p><span>混剪模式</span><b>{{ categoryModeLabel }}</b></p><p><span>分类顺序</span><b :title="categorySequenceLabel">{{ categorySequenceLabel }}</b></p><p><span>当前预览</span><b>{{ selectedResolution }}</b></p></div><footer><button type="button" @click="taskOpen = false">返回调整</button><button class="is-primary" type="button" @click="submitTask">提交任务</button></footer></section>
    </div>

    <div v-if="guideOpen" class="replica-batch-guide-backdrop" @click.self="guideOpen = false">
      <section class="replica-batch-guide" role="dialog" aria-modal="true" aria-label="分类混剪使用指引"><header><div><strong>分类混剪 · 使用指引</strong><small>按照以下步骤完成一次分类混剪任务</small></div><button type="button" aria-label="关闭使用指引" @click="guideOpen = false">×</button></header><ol><li><strong>导入分类文件夹</strong><span>点击「导入文件夹」或拖拽添加，每个文件夹代表一个分类，例如开头、产品外观展示、核心卖点演示、使用场景代入和结尾引导。</span></li><li><strong>选择混剪模式 + 调整分类参数</strong><span>左侧底部切换全局模式：自定义、文案或音频。每个文件夹右侧的齿轮可设置截取范围、抽取数量和抽取方式。</span></li><li><strong>设置配音与字幕</strong><span>在右侧面板选择音色、调整语速音量，开启字幕并调整样式，也可添加背景音乐、图片 / 视频水印等全局效果。</span></li><li><strong>创建任务 → 提交</strong><span>点击右侧「创建任务」按钮，确认任务列表后点击「提交任务」。软件会按分类顺序逐类抽取素材，自动拼接合成。</span></li></ol><footer><label><input type="checkbox" /> 下次不再自动显示</label><button class="is-primary" type="button" @click="guideOpen = false">开始使用</button></footer></section>
    </div>
  </section>
</template>
