<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch, type CSSProperties } from "vue";
import type { AiRemixPlannedShot, AiRemixSegment } from "../features/ai-remix";
import type { ImportedVideo } from "../types/videoProbe";
import type { CanvasAspectRatio, CanvasBackgroundMode, DynamicZoomMode } from "../services/videoMixService";
import type { WatermarkAssetType, WatermarkKind, WatermarkPosition, WatermarkRemovalRegion, WatermarkTrajectory } from "../features/watermark/types";
import type { DrawerKey, ToolKey } from "../types/workbench";

type PreviewToolKey = Extract<ToolKey, "remix" | "canvas" | "cover" | "effects" | "watermark">;
type ScriptTab = { id: number; title: string; text: string };
type CropHandle = "nw" | "ne" | "sw" | "se";
type PointerInteraction = {
  kind: "crop-move" | "crop-resize" | "text-move" | "text-resize" | "removal-move" | "removal-resize" | "watermark-asset-move" | "watermark-asset-resize";
  regionIndex?: number;
  handle?: CropHandle;
  startX: number;
  startY: number;
  startRect: { x: number; y: number; width: number; height: number };
  startFontSize?: number;
};

const props = defineProps<{
  isAdvancedMode: boolean;
  importedVideoCount: number;
  selectedVideo: ImportedVideo | null;
  previewTitle: string;
  previewUrl: string | null;
  watermarkAssetPreviewUrl: string | null;
  watermarkAssetType: WatermarkAssetType;
  watermarkTrajectory: WatermarkTrajectory;
  watermarkOpacity: number;
  watermarkRemovalEnabled: boolean;
  hslEnabled: boolean;
  hue: number;
  brightness: number;
  saturation: number;
  zoomEnabled: boolean;
  zoomMode: DynamicZoomMode;
  zoomMinScale: number;
  zoomMaxScale: number;
  zoomMinDurationSeconds: number;
  zoomMaxDurationSeconds: number;
  selectedCoverUrl: string | null;
  canvasAspectRatio: CanvasAspectRatio;
  shouldShowBlurBackground: boolean;
  previewCanvasStyle: CSSProperties;
  isSplitting: boolean;
  isMixing: boolean;
  isBatchMixing: boolean;
  isExporting: boolean;
  splitSegmentCount: number | null;
  splitError: string | null;
  randomSelectedCount: number;
  batchMixResultCount: number;
  mixError: string | null;
  batchMixError: string | null;
  aiPreparedSegments: AiRemixSegment[];
  aiPlannedShots: AiRemixPlannedShot[];
  isPreparingAiSegments: boolean;
  aiPreparationError: string | null;
  isPlanningAiRemix: boolean;
  aiPlanningProgressText: string | null;
  isGeneratingAiRemix: boolean;
  ttsVideoEnabled: boolean;
  aiRemixInputMode: "custom" | "script" | "audio";
  asrSourceFileName: string | null;
  asrResultText: string | null;
  generationOutputDirectory: string | null;
  generationOutputError: string | null;
  draftExportFeedback: string | null;
  generationProgressText: string | null;
  generationSummaryText: string | null;
  generationSuccessCount: number;
  generationFailureCount: number;
  aiPlanError: string | null;
  aiGenerateError: string | null;
  isRewritingAiScript: boolean;
  aiRewriteFeedback: string | null;
  formatDuration: (durationSeconds: number | null) => string;
  formatResolution: (video: ImportedVideo) => string;
  formatFrameRate: (frameRate: number | null) => string;
  formatFileSize: (fileSizeBytes: number) => string;
}>();

const emit = defineEmits<{
  splitSelectedVideo: [];
  pickSegmentsRandomly: [];
  concatRandomSegments: [];
  concatCategorizedSegments: [];
  generateBatchMixes: [];
  exportSelectedVideo: [];
  syncPreviewBackground: [];
  openDrawer: [drawer: DrawerKey];
  openTool: [tool: PreviewToolKey];
  planAiRemix: [];
  generateAiRemix: [];
  updateAiRemixInputMode: [mode: "custom" | "script" | "audio"];
  selectAudioSource: [];
  recognizeAudio: [];
  useRecognizedAudio: [];
  openGenerationDirectory: [];
  exportJianyingDraft: [];
  rewriteAiScript: [];
}>();

const previewVideoRef = defineModel<HTMLVideoElement | null>("previewVideoRef");
const previewBackgroundVideoRef = defineModel<HTMLVideoElement | null>("previewBackgroundVideoRef");
const aiScript = defineModel<string>("aiScript", { required: true });
const aiGenerateCount = defineModel<number>("aiGenerateCount", { required: true });
const canvasAspectRatio = defineModel<CanvasAspectRatio>("canvasAspectRatio", { required: true });
const canvasBackgroundMode = defineModel<CanvasBackgroundMode>("canvasBackgroundMode", { required: true });
const effectScale = defineModel<number>("effectScale", { required: true });
const watermarkEnabled = defineModel<boolean>("watermarkEnabled", { required: true });
const watermarkKind = defineModel<WatermarkKind>("watermarkKind", { required: true });
const watermarkText = defineModel<string>("watermarkText", { required: true });
const watermarkPosition = defineModel<WatermarkPosition>("watermarkPosition", { required: true });
const watermarkOpacity = defineModel<number>("watermarkOpacity", { required: true });
const watermarkImageSizeRatio = defineModel<number>("watermarkImageSizeRatio", { required: true });
const watermarkTextFontSize = defineModel<number>("watermarkTextFontSize", { required: true });
const watermarkTextColor = defineModel<string>("watermarkTextColor", { required: true });
const watermarkImagePositionXRatio = defineModel<number>("watermarkImagePositionXRatio", { required: true });
const watermarkImagePositionYRatio = defineModel<number>("watermarkImagePositionYRatio", { required: true });
const watermarkRemovalRegionCount = defineModel<number>("watermarkRemovalRegionCount", { required: true });
const watermarkRemovalManualRegions = defineModel<WatermarkRemovalRegion[]>("watermarkRemovalManualRegions", { required: true });
const ttsSubtitleEnabled = defineModel<boolean>("ttsSubtitleEnabled", { required: true });

const isCropEditorOpen = ref(false);
const isTextEditorOpen = ref(false);
const previewCanvasRef = ref<HTMLElement | null>(null);
const watermarkAssetLayerRef = ref<HTMLElement | null>(null);
const textEditorRef = ref<HTMLElement | null>(null);
const previewCanvasSize = ref({ width: 0, height: 0 });
const cropRect = ref({ x: 18, y: 10, width: 64, height: 78 });
const textPosition = ref({ x: 50, y: 50 });
const pointerInteraction = ref<PointerInteraction | null>(null);
const isPreviewPlaying = ref(false);
let previewResizeObserver: ResizeObserver | null = null;
const isBatchImportOpen = ref(false);
const batchImportText = ref("");
const nextScriptId = ref(2);
const activeScriptId = ref(1);
const scriptTabs = ref<ScriptTab[]>([{ id: 1, title: "文案 1", text: aiScript.value }]);
let syncingScript = false;

const aspectOptions: Array<{ value: CanvasAspectRatio; label: string }> = [
  { value: "original", label: "原比例" },
  { value: "portrait916", label: "9:16" },
  { value: "landscape169", label: "16:9" },
  { value: "square11", label: "1:1" },
];
const fillOptions: Array<{ value: CanvasBackgroundMode; label: string }> = [
  { value: "black", label: "适应填充" },
  { value: "blur", label: "模糊填充" },
];
const positionOptions: Array<{ value: WatermarkPosition; label: string }> = [
  { value: "topLeft", label: "左上" },
  { value: "topRight", label: "右上" },
  { value: "center", label: "居中" },
  { value: "bottomLeft", label: "左下" },
  { value: "bottomRight", label: "右下" },
];

const isPortraitPreview = computed(() => {
  if (canvasAspectRatio.value === "portrait916") return true;
  if (canvasAspectRatio.value !== "original") return false;
  const width = props.selectedVideo?.width;
  const height = props.selectedVideo?.height;
  return Boolean(width && height && height > width);
});

const targetAspectRatio = computed(() => {
  if (canvasAspectRatio.value === "portrait916") return 9 / 16;
  if (canvasAspectRatio.value === "landscape169") return 16 / 9;
  if (canvasAspectRatio.value === "square11") return 1;
  const width = props.selectedVideo?.width;
  const height = props.selectedVideo?.height;
  return width && height ? width / height : 16 / 9;
});

const sourceAspectRatio = computed(() => {
  const width = props.selectedVideo?.width;
  const height = props.selectedVideo?.height;
  return width && height ? width / height : targetAspectRatio.value;
});

const mediaLayerStyle = computed<CSSProperties>(() => {
  const { width, height } = previewCanvasSize.value;
  if (!width || !height) return { inset: "0" };
  const canvasRatio = width / height;
  if (canvasRatio >= sourceAspectRatio.value) {
    const layerWidth = (sourceAspectRatio.value / canvasRatio) * 100;
    return { top: "0", left: `${(100 - layerWidth) / 2}%`, width: `${layerWidth}%`, height: "100%" };
  }
  const layerHeight = (canvasRatio / sourceAspectRatio.value) * 100;
  return { top: `${(100 - layerHeight) / 2}%`, left: "0", width: "100%", height: `${layerHeight}%` };
});

const aspectGuideStyle = computed<CSSProperties>(() => {
  const { width, height } = previewCanvasSize.value;
  if (!width || !height) return { width: "100%", height: "100%" };
  const availableWidth = Math.max(1, width);
  const availableHeight = Math.max(1, height);
  const containerRatio = availableWidth / availableHeight;
  if (containerRatio >= targetAspectRatio.value) {
    return { width: `${availableHeight * targetAspectRatio.value}px`, height: `${availableHeight}px` };
  }
  return { width: `${availableWidth}px`, height: `${availableWidth / targetAspectRatio.value}px` };
});

const cropBoxStyle = computed<CSSProperties>(() => ({
  left: `${cropRect.value.x}%`,
  top: `${cropRect.value.y}%`,
  width: `${cropRect.value.width}%`,
  height: `${cropRect.value.height}%`,
}));

const textBoxStyle = computed<CSSProperties>(() => ({
  left: `${textPosition.value.x}%`,
  top: `${textPosition.value.y}%`,
  color: watermarkTextColor.value,
  opacity: watermarkOpacity.value,
  fontSize: `${Math.max(18, Math.min(96, watermarkTextFontSize.value))}px`,
}));

const foregroundVideoStyle = computed<CSSProperties>(() => ({
  // 裁剪和文字编辑都只操作画布覆盖层，不改变原生 video 的几何尺寸。
  // 这样添加文本时不会把素材画面、时间戳或控制条一起放大。
  transform: isCropEditorOpen.value || isTextEditorOpen.value ? undefined : `scale(${Math.max(1, Math.min(1.2, effectScale.value))})`,
  filter: props.hslEnabled
    ? `hue-rotate(${props.hue}deg) saturate(${Math.max(0, props.saturation)}) brightness(${Math.max(0, props.brightness + 1)})`
    : undefined,
}));

const dynamicZoomStyle = computed<CSSProperties>(() => {
  if (!props.zoomEnabled) return {};
  const min = Math.max(1, Math.min(1.5, props.zoomMinScale));
  const max = Math.max(min, Math.min(1.5, props.zoomMaxScale));
  return {
    "--zoom-min": String(min),
    "--zoom-max": String(max),
    "--zoom-duration": `${Math.max(1, props.zoomMaxDurationSeconds)}s`,
  } as CSSProperties;
});

const dynamicZoomClass = computed(() => {
  if (isCropEditorOpen.value || isTextEditorOpen.value || !props.zoomEnabled || !isPreviewPlaying.value) return "";
  if (props.zoomMode === "pull") return "video-frame__foreground--zoom-pull";
  if (props.zoomMode === "random") return "video-frame__foreground--zoom-random";
  return "video-frame__foreground--zoom-push";
});

const watermarkAssetStyle = computed<CSSProperties>(() => ({
  opacity: props.watermarkOpacity,
  width: `${Math.max(8, Math.min(50, watermarkImageSizeRatio.value * 100))}%`,
  left: `${clamp(watermarkImagePositionXRatio.value, Math.max(0.04, watermarkImageSizeRatio.value / 2), Math.min(0.96, 1 - watermarkImageSizeRatio.value / 2)) * 100}%`,
  top: `${clamp(watermarkImagePositionYRatio.value, 0.08, 0.92) * 100}%`,
}));

const watermarkAssetClass = computed(() => {
  if (props.watermarkTrajectory === "horizontal") return "replica-preview-asset-watermark--horizontal";
  if (props.watermarkTrajectory === "vertical") return "replica-preview-asset-watermark--vertical";
  if (props.watermarkTrajectory === "diagonal") return "replica-preview-asset-watermark--diagonal";
  if (props.watermarkTrajectory === "random") return "replica-preview-asset-watermark--random";
  return "";
});

function removalRegionStyle(region: WatermarkRemovalRegion): CSSProperties {
  return {
    left: `${region.xRatio * 100}%`,
    top: `${region.yRatio * 100}%`,
    width: `${region.widthRatio * 100}%`,
    height: `${region.heightRatio * 100}%`,
  };
}

const watermarkStyle = computed<CSSProperties>(() => {
  const edge = "6%";
  const style: CSSProperties = {
    color: watermarkTextColor.value,
    opacity: watermarkOpacity.value,
    fontSize: `${Math.max(12, Math.min(72, watermarkTextFontSize.value))}px`,
  };
  if (watermarkPosition.value.includes("Left")) style.left = edge;
  if (watermarkPosition.value.includes("Right")) style.right = edge;
  if (watermarkPosition.value === "center") {
    style.left = "50%";
    style.top = "50%";
    style.transform = "translate(-50%, -50%)";
  } else if (watermarkPosition.value.includes("top")) {
    style.top = edge;
  } else {
    style.bottom = edge;
  }
  return style;
});

const activeScript = computed(() => scriptTabs.value.find((tab) => tab.id === activeScriptId.value) ?? scriptTabs.value[0]);

function updatePreviewCanvasSize() {
  const rect = previewCanvasRef.value?.getBoundingClientRect();
  if (!rect) return;
  previewCanvasSize.value = { width: rect.width, height: rect.height };
}

function observePreviewCanvas(element: HTMLElement | null) {
  previewResizeObserver?.disconnect();
  previewResizeObserver = null;
  if (!element) return;
  previewResizeObserver = new ResizeObserver(updatePreviewCanvasSize);
  previewResizeObserver.observe(element);
  updatePreviewCanvasSize();
}

watch(previewCanvasRef, observePreviewCanvas);

onMounted(() => updatePreviewCanvasSize());
onUnmounted(() => {
  previewResizeObserver?.disconnect();
  window.removeEventListener("pointermove", handlePointerMove);
  window.removeEventListener("pointerup", stopPointerInteraction);
});

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

function startCropDrag(event: PointerEvent) {
  if (!previewCanvasRef.value) return;
  event.preventDefault();
  pointerInteraction.value = {
    kind: "crop-move",
    startX: event.clientX,
    startY: event.clientY,
    startRect: { ...cropRect.value },
  };
  window.addEventListener("pointermove", handlePointerMove);
  window.addEventListener("pointerup", stopPointerInteraction, { once: true });
}

function startCropResize(event: PointerEvent, handle: CropHandle) {
  if (!previewCanvasRef.value) return;
  event.preventDefault();
  pointerInteraction.value = {
    kind: "crop-resize",
    handle,
    startX: event.clientX,
    startY: event.clientY,
    startRect: { ...cropRect.value },
  };
  window.addEventListener("pointermove", handlePointerMove);
  window.addEventListener("pointerup", stopPointerInteraction, { once: true });
}

function startRemovalDrag(event: PointerEvent, regionIndex: number) {
  if (!previewCanvasRef.value) return;
  const region = watermarkRemovalManualRegions.value[regionIndex];
  if (!region) return;
  event.preventDefault();
  pointerInteraction.value = {
    kind: "removal-move",
    regionIndex,
    startX: event.clientX,
    startY: event.clientY,
    startRect: { x: region.xRatio * 100, y: region.yRatio * 100, width: region.widthRatio * 100, height: region.heightRatio * 100 },
  };
  window.addEventListener("pointermove", handlePointerMove);
  window.addEventListener("pointerup", stopPointerInteraction, { once: true });
}

function startRemovalResize(event: PointerEvent, regionIndex: number, handle: CropHandle) {
  if (!previewCanvasRef.value) return;
  const region = watermarkRemovalManualRegions.value[regionIndex];
  if (!region) return;
  event.preventDefault();
  pointerInteraction.value = {
    kind: "removal-resize",
    regionIndex,
    handle,
    startX: event.clientX,
    startY: event.clientY,
    startRect: { x: region.xRatio * 100, y: region.yRatio * 100, width: region.widthRatio * 100, height: region.heightRatio * 100 },
  };
  window.addEventListener("pointermove", handlePointerMove);
  window.addEventListener("pointerup", stopPointerInteraction, { once: true });
}

function startTextDrag(event: PointerEvent) {
  if (!watermarkAssetLayerRef.value || (event.target as HTMLElement).isContentEditable) return;
  event.preventDefault();
  pointerInteraction.value = {
    kind: "text-move",
    startX: event.clientX,
    startY: event.clientY,
    startRect: { x: textPosition.value.x, y: textPosition.value.y, width: 0, height: 0 },
  };
  window.addEventListener("pointermove", handlePointerMove);
  window.addEventListener("pointerup", stopPointerInteraction, { once: true });
}

function startTextResize(event: PointerEvent, handle: CropHandle) {
  if (!watermarkAssetLayerRef.value) return;
  event.preventDefault();
  pointerInteraction.value = {
    kind: "text-resize",
    handle,
    startX: event.clientX,
    startY: event.clientY,
    startRect: { x: textPosition.value.x, y: textPosition.value.y, width: 0, height: 0 },
    startFontSize: watermarkTextFontSize.value,
  };
  window.addEventListener("pointermove", handlePointerMove);
  window.addEventListener("pointerup", stopPointerInteraction, { once: true });
}

function startAssetWatermarkDrag(event: PointerEvent) {
  const layerRect = watermarkAssetLayerRef.value?.getBoundingClientRect();
  if (!layerRect) return;
  event.preventDefault();
  pointerInteraction.value = {
    kind: "watermark-asset-move",
    startX: event.clientX,
    startY: event.clientY,
    startRect: {
      x: watermarkImagePositionXRatio.value * 100,
      y: watermarkImagePositionYRatio.value * 100,
      width: watermarkImageSizeRatio.value * 100,
      height: 0,
    },
  };
  window.addEventListener("pointermove", handlePointerMove);
  window.addEventListener("pointerup", stopPointerInteraction, { once: true });
}

function startAssetWatermarkResize(event: PointerEvent, handle: CropHandle) {
  const layerRect = watermarkAssetLayerRef.value?.getBoundingClientRect();
  if (!layerRect) return;
  event.preventDefault();
  pointerInteraction.value = {
    kind: "watermark-asset-resize",
    handle,
    startX: event.clientX,
    startY: event.clientY,
    startRect: {
      x: watermarkImagePositionXRatio.value * 100,
      y: watermarkImagePositionYRatio.value * 100,
      width: watermarkImageSizeRatio.value * 100,
      height: 0,
    },
  };
  window.addEventListener("pointermove", handlePointerMove);
  window.addEventListener("pointerup", stopPointerInteraction, { once: true });
}

function handlePointerMove(event: PointerEvent) {
  const interaction = pointerInteraction.value;
  const canvasRect = previewCanvasRef.value?.getBoundingClientRect();
  const assetLayerRect = watermarkAssetLayerRef.value?.getBoundingClientRect();
  if (!interaction || !canvasRect) return;
  const interactionRect = interaction.kind === "watermark-asset-move"
    || interaction.kind === "watermark-asset-resize"
    || interaction.kind === "text-move"
    || interaction.kind === "text-resize"
    ? assetLayerRect
    : canvasRect;
  if (!interactionRect) return;
  const deltaX = ((event.clientX - interaction.startX) / interactionRect.width) * 100;
  const deltaY = ((event.clientY - interaction.startY) / interactionRect.height) * 100;

  if (interaction.kind === "watermark-asset-move") {
    const halfWidth = interaction.startRect.width / 2;
    watermarkImagePositionXRatio.value = clamp((interaction.startRect.x + deltaX) / 100, halfWidth / 100, 1 - halfWidth / 100);
    watermarkImagePositionYRatio.value = clamp((interaction.startRect.y + deltaY) / 100, 0.06, 0.94);
    return;
  }

  if (interaction.kind === "watermark-asset-resize") {
    const handle = interaction.handle;
    if (!handle) return;
    const horizontalDelta = handle.includes("w") ? -deltaX : deltaX;
    const verticalDelta = handle.includes("n") ? -deltaY : deltaY;
    const sizeDelta = Math.abs(verticalDelta) > Math.abs(horizontalDelta) ? verticalDelta : horizontalDelta;
    watermarkImageSizeRatio.value = clamp((interaction.startRect.width + sizeDelta) / 100, 0.08, 0.5);
    return;
  }

  if (interaction.kind === "text-move") {
    textPosition.value = {
      x: clamp(interaction.startRect.x + deltaX, 8, 92),
      y: clamp(interaction.startRect.y + deltaY, 10, 90),
    };
    return;
  }

  if (interaction.kind === "text-resize") {
    const direction = interaction.handle?.includes("n") ? -1 : 1;
    watermarkTextFontSize.value = clamp((interaction.startFontSize ?? watermarkTextFontSize.value) + direction * deltaY * .45, 18, 96);
    return;
  }

  if (interaction.kind === "removal-move" || interaction.kind === "removal-resize") {
    const index = interaction.regionIndex ?? -1;
    const region = watermarkRemovalManualRegions.value[index];
    if (!region) return;
    let next = { ...interaction.startRect };
    if (interaction.kind === "removal-move") {
      next.x = clamp(interaction.startRect.x + deltaX, 0, 100 - interaction.startRect.width);
      next.y = clamp(interaction.startRect.y + deltaY, 0, 100 - interaction.startRect.height);
    } else {
      const handle = interaction.handle;
      if (!handle) return;
      let left = interaction.startRect.x;
      let top = interaction.startRect.y;
      let right = interaction.startRect.x + interaction.startRect.width;
      let bottom = interaction.startRect.y + interaction.startRect.height;
      if (handle.includes("w")) left = clamp(interaction.startRect.x + deltaX, 0, right - 4);
      if (handle.includes("e")) right = clamp(interaction.startRect.x + interaction.startRect.width + deltaX, left + 4, 100);
      if (handle.includes("n")) top = clamp(interaction.startRect.y + deltaY, 0, bottom - 4);
      if (handle.includes("s")) bottom = clamp(interaction.startRect.y + interaction.startRect.height + deltaY, top + 4, 100);
      next = { x: left, y: top, width: right - left, height: bottom - top };
    }
    watermarkRemovalManualRegions.value = watermarkRemovalManualRegions.value.map((item, regionIndex) => regionIndex === index ? {
      xRatio: next.x / 100,
      yRatio: next.y / 100,
      widthRatio: next.width / 100,
      heightRatio: next.height / 100,
    } : { ...item });
    return;
  }

  if (interaction.kind === "crop-move") {
    cropRect.value = {
      ...cropRect.value,
      x: clamp(interaction.startRect.x + deltaX, 0, 100 - interaction.startRect.width),
      y: clamp(interaction.startRect.y + deltaY, 0, 100 - interaction.startRect.height),
    };
    return;
  }

  const start = interaction.startRect;
  const handle = interaction.handle;
  if (!handle) return;
  let left = start.x;
  let top = start.y;
  let right = start.x + start.width;
  let bottom = start.y + start.height;
  if (handle.includes("w")) left = clamp(start.x + deltaX, 0, right - 12);
  if (handle.includes("e")) right = clamp(start.x + start.width + deltaX, left + 12, 100);
  if (handle.includes("n")) top = clamp(start.y + deltaY, 0, bottom - 12);
  if (handle.includes("s")) bottom = clamp(start.y + start.height + deltaY, top + 12, 100);
  cropRect.value = { x: left, y: top, width: right - left, height: bottom - top };
  effectScale.value = clamp(86 / cropRect.value.width, 1, 1.2);
}

function stopPointerInteraction() {
  pointerInteraction.value = null;
  window.removeEventListener("pointermove", handlePointerMove);
}

async function enableTextEditing() {
  isTextEditorOpen.value = !isTextEditorOpen.value;
  if (isTextEditorOpen.value) {
    isCropEditorOpen.value = false;
    watermarkKind.value = "text";
    watermarkEnabled.value = true;
    if (!watermarkText.value.trim()) {
      watermarkText.value = "这里是文字";
      watermarkTextFontSize.value = 40;
    }
    await nextTick();
    if (!textEditorRef.value) return;
    textEditorRef.value.textContent = watermarkText.value;
    textEditorRef.value.focus();
    const selection = window.getSelection();
    const range = document.createRange();
    range.selectNodeContents(textEditorRef.value);
    range.collapse(false);
    selection?.removeAllRanges();
    selection?.addRange(range);
  }
}

function updateWatermarkText(event: Event) {
  watermarkText.value = (event.target as HTMLElement).textContent ?? "";
  watermarkEnabled.value = true;
}

watch(watermarkText, (value) => {
  if (!isTextEditorOpen.value || document.activeElement === textEditorRef.value) return;
  if (textEditorRef.value && textEditorRef.value.textContent !== value) {
    textEditorRef.value.textContent = value;
  }
});

watch(aiScript, (value) => {
  if (syncingScript || !activeScript.value) return;
  activeScript.value.text = value;
});

watch(
  () => props.selectedVideo?.filePath,
  () => {
    scriptTabs.value = [{ id: 1, title: "文案 1", text: aiScript.value }];
    activeScriptId.value = 1;
    nextScriptId.value = 2;
  },
);

function selectScriptTab(id: number) {
  const target = scriptTabs.value.find((tab) => tab.id === id);
  if (!target) return;
  activeScriptId.value = id;
  syncingScript = true;
  aiScript.value = target.text;
  syncingScript = false;
}

function updateActiveScript(value: string) {
  if (!activeScript.value) return;
  activeScript.value.text = value;
  syncingScript = true;
  aiScript.value = value;
  syncingScript = false;
}

function addScriptTab() {
  const id = nextScriptId.value++;
  scriptTabs.value.push({ id, title: `文案 ${id}`, text: "" });
  selectScriptTab(id);
  aiGenerateCount.value = Math.max(aiGenerateCount.value, scriptTabs.value.length);
}

function removeScriptTab(id: number) {
  if (scriptTabs.value.length === 1) {
    updateActiveScript("");
    return;
  }
  const index = scriptTabs.value.findIndex((tab) => tab.id === id);
  scriptTabs.value = scriptTabs.value.filter((tab) => tab.id !== id);
  if (activeScriptId.value === id) {
    selectScriptTab(scriptTabs.value[Math.max(0, index - 1)].id);
  }
}

function importScripts() {
  const entries = batchImportText.value.split(/\n\s*\n/).map((text) => text.trim()).filter(Boolean);
  if (entries.length === 0) return;
  scriptTabs.value = entries.map((text, index) => ({ id: index + 1, title: `文案 ${index + 1}`, text }));
  nextScriptId.value = entries.length + 1;
  activeScriptId.value = 1;
  aiScript.value = entries[0];
  aiGenerateCount.value = Math.max(aiGenerateCount.value, entries.length);
  batchImportText.value = "";
  isBatchImportOpen.value = false;
}

function clearScripts() {
  scriptTabs.value = [{ id: 1, title: "文案 1", text: "" }];
  activeScriptId.value = 1;
  aiScript.value = "";
  nextScriptId.value = 2;
}

function toggleCropEditor() {
  isCropEditorOpen.value = !isCropEditorOpen.value;
  if (isCropEditorOpen.value) isTextEditorOpen.value = false;
}

function createRemovalRegion(index: number): WatermarkRemovalRegion {
  const column = index % 3;
  const row = Math.floor(index / 3);
  return {
    xRatio: 0.12 + column * 0.28,
    yRatio: 0.12 + row * 0.24,
    widthRatio: 0.18,
    heightRatio: 0.12,
  };
}

watch(watermarkRemovalRegionCount, (count) => {
  const normalized = Math.max(1, Math.min(8, Math.round(count || 1)));
  const regions = watermarkRemovalManualRegions.value.slice(0, normalized);
  while (regions.length < normalized) regions.push(createRemovalRegion(regions.length));
  watermarkRemovalManualRegions.value = regions;
}, { immediate: true });

function previewPlay() {
  isPreviewPlaying.value = true;
  emit("syncPreviewBackground");
}

function previewPause() {
  isPreviewPlaying.value = false;
  emit("syncPreviewBackground");
}

</script>

<template>
  <section class="center-stage replica-ai-stage" aria-label="AI 成片工作区">
    <section class="panel preview-panel preview-panel--focused replica-video-preview">
      <div class="replica-pane-heading replica-pane-heading--canvas">
        <strong>视频画面</strong>
        <div class="replica-canvas-controls" aria-label="视频画布直接设置">
          <div class="replica-control-group">
            <span>视频比例</span>
            <button v-for="option in aspectOptions" :key="option.value" type="button" :class="{ 'is-active': canvasAspectRatio === option.value }" @click="canvasAspectRatio = option.value">{{ option.label }}</button>
          </div>
          <div class="replica-control-group">
            <span>填充方式</span>
            <button v-for="option in fillOptions" :key="option.value" type="button" :class="{ 'is-active': canvasBackgroundMode === option.value }" :disabled="canvasAspectRatio === 'original'" @click="canvasBackgroundMode = option.value">{{ option.label }}</button>
          </div>
          <button class="replica-inline-tool" type="button" :class="{ 'is-active': isCropEditorOpen }" @click="toggleCropEditor">视频裁剪</button>
          <button class="replica-inline-tool" type="button" :class="{ 'is-active': isTextEditorOpen }" @click="enableTextEditing">添加文本</button>
        </div>
      </div>

      <div v-if="isCropEditorOpen" class="replica-inline-editor replica-inline-editor--crop">
        <label for="preview-crop-scale">画面缩放</label>
        <input id="preview-crop-scale" v-model.number="effectScale" type="range" min="1" max="1.2" step="0.01" />
        <output>{{ effectScale.toFixed(2) }}×</output>
        <button type="button" @click="effectScale = 1">重置</button>
      </div>
      <div v-if="previewUrl" class="video-frame replica-stable-preview-frame" :class="{ 'video-frame--portrait': isPortraitPreview, 'is-text-editing': isTextEditorOpen, 'is-crop-editing': isCropEditorOpen }">
        <div ref="previewCanvasRef" class="video-frame__canvas" :class="{ 'video-frame__canvas--fit': canvasAspectRatio !== 'original', 'video-frame__canvas--blur': shouldShowBlurBackground }" :style="previewCanvasStyle">
          <video v-if="shouldShowBlurBackground" ref="previewBackgroundVideoRef" class="video-frame__background" :src="previewUrl" muted autoplay loop playsinline preload="metadata" tabindex="-1" aria-hidden="true" @loadeddata="emit('syncPreviewBackground')"></video>
          <video :key="previewUrl" ref="previewVideoRef" class="video-frame__foreground" :class="dynamicZoomClass" :src="previewUrl" controls preload="metadata" :style="{ ...foregroundVideoStyle, ...dynamicZoomStyle }" @play="previewPlay" @pause="previewPause" @seeked="emit('syncPreviewBackground')" @timeupdate="emit('syncPreviewBackground')" @ratechange="emit('syncPreviewBackground')"></video>
          <div v-if="canvasAspectRatio !== 'original' && !isCropEditorOpen" class="replica-aspect-guide" aria-hidden="true">
            <span class="replica-aspect-guide__frame" :style="aspectGuideStyle"></span>
          </div>
          <div v-if="isCropEditorOpen" class="replica-crop-box" :style="cropBoxStyle" aria-label="可调整的视频裁剪区域" @pointerdown="startCropDrag">
            <span class="replica-crop-grid replica-crop-grid--vertical" aria-hidden="true"></span>
            <span class="replica-crop-grid replica-crop-grid--horizontal" aria-hidden="true"></span>
            <button v-for="handle in (['nw', 'ne', 'sw', 'se'] as CropHandle[])" :key="handle" class="replica-crop-handle" :class="`replica-crop-handle--${handle}`" type="button" :aria-label="`调整裁剪框${handle}`" @pointerdown.stop="startCropResize($event, handle)"></button>
            <span class="replica-crop-size">{{ Math.round(cropRect.width) }}% × {{ Math.round(cropRect.height) }}%</span>
          </div>
          <template v-if="props.watermarkRemovalEnabled">
            <div
              v-for="(region, regionIndex) in watermarkRemovalManualRegions"
              :key="`removal-region-${regionIndex}`"
              class="replica-removal-region"
              :style="removalRegionStyle(region)"
              :aria-label="`去除水印区域 ${regionIndex + 1}`"
              @pointerdown="startRemovalDrag($event, regionIndex)"
            >
              <span class="replica-removal-region__label">{{ regionIndex + 1 }}</span>
              <button v-for="handle in (['nw', 'ne', 'sw', 'se'] as CropHandle[])" :key="handle" class="replica-removal-region__handle" :class="`replica-removal-region__handle--${handle}`" type="button" :aria-label="`调整水印区域 ${regionIndex + 1} ${handle}`" @pointerdown.stop="startRemovalResize($event, regionIndex, handle)"></button>
            </div>
          </template>
          <div ref="watermarkAssetLayerRef" class="video-frame__media-layer" :style="mediaLayerStyle">
            <div v-if="isTextEditorOpen" class="replica-text-box" :style="textBoxStyle" aria-label="画面文字编辑框" @pointerdown="startTextDrag">
              <span ref="textEditorRef" class="replica-text-box__content" contenteditable="true" role="textbox" aria-multiline="true" spellcheck="false" @input="updateWatermarkText" @pointerdown.stop></span>
              <button v-for="handle in (['nw', 'ne', 'sw', 'se'] as CropHandle[])" :key="handle" class="replica-text-handle" :class="`replica-text-handle--${handle}`" type="button" :aria-label="`调整文字${handle}`" @pointerdown.stop="startTextResize($event, handle)"></button>
              <button class="replica-text-box__close" type="button" aria-label="关闭文字编辑" @click.stop="isTextEditorOpen = false">×</button>
            </div>
            <span v-if="!isTextEditorOpen && watermarkEnabled && watermarkKind === 'text' && watermarkText.trim()" class="replica-preview-watermark" :style="watermarkStyle">{{ watermarkText }}</span>
            <div v-if="watermarkEnabled && watermarkKind === 'image' && props.watermarkAssetPreviewUrl" class="replica-preview-asset-watermark" :class="watermarkAssetClass" :style="watermarkAssetStyle" aria-label="可调整的图片/视频水印" @pointerdown="startAssetWatermarkDrag">
              <video v-if="props.watermarkAssetType === 'video'" :src="props.watermarkAssetPreviewUrl" muted autoplay loop playsinline aria-hidden="true"></video>
              <img v-else :src="props.watermarkAssetPreviewUrl" alt="图片/视频水印预览" />
              <span class="replica-preview-asset-watermark__hint">拖动调整位置</span>
              <button v-for="handle in (['nw', 'ne', 'sw', 'se'] as CropHandle[])" :key="handle" class="replica-preview-asset-watermark__handle" :class="`replica-preview-asset-watermark__handle--${handle}`" type="button" :aria-label="`调整图片水印${handle}`" @pointerdown.stop="startAssetWatermarkResize($event, handle)"></button>
            </div>
          </div>
        </div>
      </div>
      <div v-else class="video-placeholder preview-empty-state"><span aria-hidden="true">▶</span><strong>选择素材后在此预览</strong></div>
    </section>

    <section class="panel ai-workflow-panel replica-script-workspace">
      <div class="replica-script-heading replica-script-heading--copy">
        <strong>视频文案</strong>
        <div class="replica-subtitle-picker" aria-label="视频字幕设置">
          <span>视频字幕</span>
          <button type="button" :class="{ 'is-active': ttsSubtitleEnabled }" @click="ttsSubtitleEnabled = true">自动识别</button>
          <button type="button" :class="{ 'is-active': !ttsSubtitleEnabled }" @click="ttsSubtitleEnabled = false">不启用</button>
        </div>
      </div>

      <div v-if="selectedVideo" class="replica-script-tabs" role="tablist" aria-label="多条视频文案">
        <button v-for="tab in scriptTabs" :key="tab.id" type="button" role="tab" :aria-selected="activeScriptId === tab.id" :class="{ 'is-active': activeScriptId === tab.id }" @click="selectScriptTab(tab.id)">
          <span>{{ tab.title }}</span><b v-if="scriptTabs.length > 1" aria-label="删除这条文案" @click.stop="removeScriptTab(tab.id)">×</b>
        </button>
        <button class="replica-script-add" type="button" aria-label="新增文案" @click="addScriptTab">＋</button>
        <button class="replica-rewrite-button" type="button" :disabled="isRewritingAiScript || !aiScript.trim()" @click="emit('rewriteAiScript')">{{ isRewritingAiScript ? "改写中…" : "AI 改写" }}</button>
      </div>

      <div v-if="selectedVideo" class="replica-script-editor">
        <textarea :value="aiScript" rows="5" maxlength="4000" placeholder="在这里输入视频文案，可新增多条文案批量生成视频" aria-label="当前视频文案" @input="updateActiveScript(($event.target as HTMLTextAreaElement).value)"></textarea>
        <div class="replica-script-editor__footer"><div class="replica-script-editor__actions"><button type="button" @click="isBatchImportOpen = !isBatchImportOpen">批量导入</button><button type="button" @click="clearScripts">清空全部</button><span v-if="aiRewriteFeedback" class="replica-inline-feedback" role="status">{{ aiRewriteFeedback }}</span></div><span>文案字数：{{ aiScript.length }}</span></div>
      </div>
      <div v-if="isBatchImportOpen" class="replica-batch-import"><textarea v-model="batchImportText" rows="3" placeholder="每条文案之间空一行，导入后会自动生成多个文案标签"></textarea><div><button type="button" @click="isBatchImportOpen = false">取消</button><button class="is-primary" type="button" :disabled="!batchImportText.trim()" @click="importScripts">导入文案</button></div></div>

      <div v-if="!selectedVideo" class="replica-script-empty"><span aria-hidden="true">▶</span><strong>请在左侧选择一个视频素材</strong><small>选中素材后可直接填写文案并设置字幕</small></div>

      <details v-if="selectedVideo" class="replica-advanced-workflows">
        <summary>后台匹配与生成</summary>
        <div v-if="aiRemixInputMode === 'audio'" class="ai-remix-audio-source">
          <div><strong>{{ asrSourceFileName || "尚未选择音频" }}</strong><small>{{ asrResultText ? "已识别，可直接用于匹配画面" : "选择音频后识别文案" }}</small></div>
          <div class="ai-remix-audio-source__actions"><button type="button" @click="emit('selectAudioSource')">选择音频</button><button type="button" :disabled="!asrSourceFileName" @click="emit('recognizeAudio')">识别文案</button><button class="primary-button" type="button" :disabled="!asrResultText" @click="emit('useRecognizedAudio')">使用识别结果</button></div>
        </div>
        <div class="replica-prepare-row"><span><strong>{{ isSplitting || isPreparingAiSegments ? "正在准备素材" : "素材准备" }}</strong><small>{{ isSplitting || isPreparingAiSegments ? "正在生成可匹配的画面片段" : `已导入 ${importedVideoCount} 个视频，准备后即可后台匹配` }}</small><small v-if="splitError || aiPreparationError" class="workflow-inline-error" role="alert">{{ splitError || aiPreparationError }}</small></span><button class="primary-button" type="button" :disabled="isSplitting || isPreparingAiSegments" @click="emit('splitSelectedVideo')">{{ isSplitting || isPreparingAiSegments ? "准备中…" : "生成素材片段" }}</button></div>
        <section class="replica-ai-status" aria-label="后台画面匹配状态"><header class="replica-ai-status__heading"><div><span>AI 画面匹配</span><strong>画面在后台完成，前端只保留文案</strong></div><b>{{ aiPlannedShots.length > 0 ? "已完成" : isPlanningAiRemix ? "匹配中" : "待处理" }}</b></header><div class="replica-ai-status__summary"><span class="replica-ai-status__icon" aria-hidden="true">{{ aiPlannedShots.length > 0 ? "✓" : isPlanningAiRemix ? "…" : "○" }}</span><div><strong>{{ isPlanningAiRemix ? (aiPlanningProgressText || "正在为文案匹配画面") : aiPlannedShots.length > 0 ? `已为 ${aiPlannedShots.length} 句文案完成后台匹配` : aiPreparedSegments.length >= 2 ? `已准备 ${aiPreparedSegments.length} 个素材片段` : "先生成素材片段，再开始后台匹配" }}</strong><small>{{ aiPlannedShots.length > 0 ? "匹配结果将直接用于成片" : "前端不展示画面卡片，避免文案区被挤占" }}</small></div></div><div class="replica-ai-status__actions"><label for="ai-remix-generate-count">生成数量 <input id="ai-remix-generate-count" v-model.number="aiGenerateCount" type="number" min="1" max="10" step="1" :disabled="isGeneratingAiRemix || isPlanningAiRemix" /></label><button class="replica-status-button" type="button" :disabled="isPlanningAiRemix || isGeneratingAiRemix || aiPreparedSegments.length < 2 || !aiScript.trim()" @click="emit('planAiRemix')">{{ isPlanningAiRemix ? "正在匹配…" : aiPlannedShots.length > 0 ? "重新匹配画面" : "开始后台匹配" }}</button><button v-if="aiPlannedShots.length > 0" class="replica-status-button replica-status-button--primary" type="button" :disabled="isGeneratingAiRemix || isPlanningAiRemix" @click="emit('generateAiRemix')">{{ isGeneratingAiRemix ? (generationProgressText || "正在生成…") : `生成 ${aiGenerateCount} 条视频` }}</button></div><p v-if="generationProgressText" class="replica-ai-status__progress" role="status">{{ generationProgressText }}</p><p v-if="generationSummaryText" class="replica-ai-status__result" role="status">{{ generationSummaryText }}</p><p v-if="aiPreparationError || aiPlanError || aiGenerateError" class="workflow-error" role="alert">{{ aiPreparationError || aiPlanError || aiGenerateError }}</p></section>
      </details>

      <details v-if="isAdvancedMode && selectedVideo" class="secondary-workflows"><summary>其他混剪方式与高级操作</summary><div class="secondary-workflows__actions"><button class="ghost-button" type="button" :disabled="isSplitting" @click="emit('splitSelectedVideo')">重新切片</button><button class="ghost-button" type="button" @click="emit('pickSegmentsRandomly')">随机抽取</button><button class="ghost-button" type="button" :disabled="isMixing" @click="emit('concatRandomSegments')">拼接抽中片段</button><button class="ghost-button" type="button" :disabled="isMixing" @click="emit('concatCategorizedSegments')">分类混剪</button><button class="ghost-button" type="button" :disabled="isBatchMixing" @click="emit('generateBatchMixes')">随机批量生成</button><button class="ghost-button" type="button" :disabled="isExporting" @click="emit('exportSelectedVideo')">导出当前视频</button><button class="ghost-button" type="button" @click="emit('openTool', 'remix')">混剪参数</button></div></details>
      <p v-if="mixError || batchMixError" class="workflow-error" role="alert">{{ mixError || batchMixError }}</p><p v-if="generationOutputError" class="workflow-error" role="alert">{{ generationOutputError }}</p>
      <div v-if="generationOutputDirectory || draftExportFeedback" class="ai-remix-output-bar"><span><strong>本次任务</strong><small>{{ generationOutputDirectory || "等待创建任务目录" }}</small></span><button type="button" :disabled="!generationOutputDirectory" @click="emit('openGenerationDirectory')">打开目录</button><button type="button" :disabled="!generationOutputDirectory || generationSuccessCount === 0" @click="emit('exportJianyingDraft')">导出剪映草稿</button></div><p v-if="draftExportFeedback" class="workflow-success" role="status">{{ draftExportFeedback }}</p>
    </section>
  </section>
</template>
