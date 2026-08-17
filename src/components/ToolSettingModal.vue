<script setup lang="ts">
import AudioSettingsPanel from "./tool-settings/AudioSettingsPanel.vue";
import { ApiConfigSettingsPanel } from "../features/api-config";
import { AsrSettingsPanel } from "../features/asr";
import CanvasSettingsPanel from "./tool-settings/CanvasSettingsPanel.vue";
import CoverSettingsPanel from "./tool-settings/CoverSettingsPanel.vue";
import EntranceSettingsPanel from "./tool-settings/EntranceSettingsPanel.vue";
import ExportSettingsPanel from "./tool-settings/ExportSettingsPanel.vue";
import FrameSettingsPanel from "./tool-settings/FrameSettingsPanel.vue";
import FusionSettingsPanel from "./tool-settings/FusionSettingsPanel.vue";
import { TtsSettingsPanel } from "../features/tts";
import PictureInPictureSettingsPanel from "./tool-settings/PictureInPictureSettingsPanel.vue";
import RemixSettingsPanel from "./tool-settings/RemixSettingsPanel.vue";
import TransitionSettingsPanel from "./tool-settings/TransitionSettingsPanel.vue";
import type { ToolSettingModalEmits, ToolSettingModalProps } from "./tool-settings/types";
import VideoEffectsSettingsPanel from "./tool-settings/VideoEffectsSettingsPanel.vue";
import type { ToolKey } from "../types/workbench";
import { WatermarkToolPanel } from "../features/watermark";

defineProps<ToolSettingModalProps>();
const emit = defineEmits<ToolSettingModalEmits>();

const titles: Record<ToolKey, string> = {
  apiKeys: "API 密钥",
  remix: "智能切片与批量",
  canvas: "画布设置",
  audio: "音频设置",
  bgm: "音频设置",
  tts: "语音合成",
  asr: "语音识别",
  subtitleStyle: "字幕样式",
  watermark: "水印工具",
  cover: "视频封面",
  entrance: "入场效果",
  frame: "视频帧操作",
  effects: "视频效果",
  transition: "平滑转场",
  pip: "画中画",
  adjust: "画面调整",
  fusion: "镜像融合",
  rotate: "旋转镜像",
  mirror: "镜像旋转",
  speed: "视频变速",
  zoom: "动态缩放",
  subtitles: "字幕设置",
  export: "导出设置",
};

const videoEffectTools: ToolKey[] = ["mirror", "rotate", "speed", "effects", "adjust", "zoom"];
const replicaParameterTools: ToolKey[] = ["audio", "bgm", "cover", "frame", "entrance", "pip", "adjust", "fusion", "rotate", "speed", "zoom", "mirror", "effects"];
</script>

<template>
  <div v-if="activeTool" class="modal-backdrop" @click.self="emit('close')">
    <section
      class="tool-modal"
      :class="[
        `tool-modal--${activeTool}`,
        { 'tool-modal--asr': activeTool === 'asr' },
        { 'tool-modal--replica-parameter': replicaParameterTools.includes(activeTool) },
      ]"
      role="dialog"
      aria-modal="true"
      aria-labelledby="tool-setting-title"
    >
      <header v-if="!replicaParameterTools.includes(activeTool)" class="tool-modal__header">
        <div>
          <p class="panel__label">二级设置</p>
          <h2 id="tool-setting-title">{{ titles[activeTool] }}</h2>
        </div>
        <button class="panel-toggle" type="button" @click="emit('close')">关闭</button>
      </header>

      <div class="tool-modal__body" :class="{ 'tool-modal__body--replica-parameter': replicaParameterTools.includes(activeTool) }">
        <ApiConfigSettingsPanel
          v-if="activeTool === 'apiKeys'"
          @changed="emit('apiConfigChanged')"
        />

        <RemixSettingsPanel
          v-else-if="activeTool === 'remix'"
          :segment-duration-seconds="segmentDurationSeconds"
          :split-mode="splitMode"
          :scene-sensitivity="sceneSensitivity"
          :minimum-segment-seconds="minimumSegmentSeconds"
          :maximum-segment-seconds="maximumSegmentSeconds"
          :random-pick-count="randomPickCount"
          :batch-generate-count="batchGenerateCount"
          :is-splitting="isSplitting"
          :is-batch-mixing="isBatchMixing"
          :split-error="splitError"
          :split-output-directory="splitOutputDirectory"
          :split-segment-count="splitSegmentCount"
          :random-pick-error="randomPickError"
          @split-selected-video="emit('splitSelectedVideo')"
          @pick-segments-randomly="emit('pickSegmentsRandomly')"
          @generate-batch-mixes="emit('generateBatchMixes')"
          @update:segment-duration-seconds="emit('update:segmentDurationSeconds', $event)"
          @update:split-mode="emit('update:splitMode', $event)"
          @update:scene-sensitivity="emit('update:sceneSensitivity', $event)"
          @update:minimum-segment-seconds="emit('update:minimumSegmentSeconds', $event)"
          @update:maximum-segment-seconds="emit('update:maximumSegmentSeconds', $event)"
          @update:random-pick-count="emit('update:randomPickCount', $event)"
          @update:batch-generate-count="emit('update:batchGenerateCount', $event)"
        />

        <CanvasSettingsPanel
          v-else-if="activeTool === 'canvas'"
          :canvas-aspect-ratio="canvasAspectRatio"
          :canvas-background-mode="canvasBackgroundMode"
          @update:canvas-aspect-ratio="emit('update:canvasAspectRatio', $event)"
          @update:canvas-background-mode="emit('update:canvasBackgroundMode', $event)"
        />

        <TransitionSettingsPanel
          v-else-if="activeTool === 'transition'"
          :smooth-remix-enabled="smoothRemixEnabled"
          @update:smooth-remix-enabled="emit('update:smoothRemixEnabled', $event)"
        />

        <PictureInPictureSettingsPanel
          v-else-if="activeTool === 'pip'"
          :pip-enabled="pipEnabled"
          :pip-overlay-file-path="pipOverlayFilePath"
          :pip-mode="pipMode"
          :pip-main-size-min="pipMainSizeMin"
          :pip-main-size-max="pipMainSizeMax"
          :pip-blur-min="pipBlurMin"
          :pip-blur-max="pipBlurMax"
          :pip-offset-x-min="pipOffsetXMin"
          :pip-offset-x-max="pipOffsetXMax"
          :pip-offset-y-min="pipOffsetYMin"
          :pip-offset-y-max="pipOffsetYMax"
          :pip-position="pipPosition"
          :pip-size-ratio="pipSizeRatio"
          :pip-opacity="pipOpacity"
          :pip-margin="pipMargin"
          @select-pip-overlay-file="emit('selectPipOverlayFile')"
          @update:pip-enabled="emit('update:pipEnabled', $event)"
          @update:pip-mode="emit('update:pipMode', $event)"
          @update:pip-main-size-min="emit('update:pipMainSizeMin', $event)"
          @update:pip-main-size-max="emit('update:pipMainSizeMax', $event)"
          @update:pip-blur-min="emit('update:pipBlurMin', $event)"
          @update:pip-blur-max="emit('update:pipBlurMax', $event)"
          @update:pip-offset-x-min="emit('update:pipOffsetXMin', $event)"
          @update:pip-offset-x-max="emit('update:pipOffsetXMax', $event)"
          @update:pip-offset-y-min="emit('update:pipOffsetYMin', $event)"
          @update:pip-offset-y-max="emit('update:pipOffsetYMax', $event)"
          @update:pip-position="emit('update:pipPosition', $event)"
          @update:pip-size-ratio="emit('update:pipSizeRatio', $event)"
          @update:pip-opacity="emit('update:pipOpacity', $event)"
          @update:pip-margin="emit('update:pipMargin', $event)"
          @reset="emit('reset', activeTool)"
        />

        <AudioSettingsPanel
          v-else-if="activeTool === 'bgm' || activeTool === 'audio'"
          :bgm-enabled="bgmEnabled"
          :bgm-audio-file-path="bgmAudioFilePath"
          :original-volume="originalVolume"
          :original-volume-min="originalVolumeMin"
          :original-volume-max="originalVolumeMax"
          :original-fade-enabled="originalFadeEnabled"
          :dynamic-adjust-enabled="dynamicAudioAdjustEnabled"
          :bgm-volume="bgmVolume"
          :bgm-volume-min="bgmVolumeMin"
          :bgm-volume-max="bgmVolumeMax"
          :bgm-fade-enabled="bgmFadeEnabled"
          :loop-playback-enabled="bgmLoopPlaybackEnabled"
          :bgm-fade-in-seconds="bgmFadeInSeconds"
          :bgm-fade-out-seconds="bgmFadeOutSeconds"
          :is-mixing="isMixing"
          :is-batch-mixing="isBatchMixing"
          @select-bgm-audio-file="emit('selectBgmAudioFile')"
          @select-bgm-audio-folder="emit('selectBgmAudioFolder')"
          @clear-bgm-audio-file="emit('clearBgmAudioFile')"
          @update:bgm-enabled="emit('update:bgmEnabled', $event)"
          @update:original-volume="emit('update:originalVolume', $event)"
          @update:original-volume-min="emit('update:originalVolumeMin', $event)"
          @update:original-volume-max="emit('update:originalVolumeMax', $event)"
          @update:original-fade-enabled="emit('update:originalFadeEnabled', $event)"
          @update:dynamic-adjust-enabled="emit('update:dynamicAudioAdjustEnabled', $event)"
          @update:bgm-volume="emit('update:bgmVolume', $event)"
          @update:bgm-volume-min="emit('update:bgmVolumeMin', $event)"
          @update:bgm-volume-max="emit('update:bgmVolumeMax', $event)"
          @update:bgm-fade-enabled="emit('update:bgmFadeEnabled', $event)"
          @update:loop-playback-enabled="emit('update:bgmLoopPlaybackEnabled', $event)"
          @update:bgm-fade-in-seconds="emit('update:bgmFadeInSeconds', $event)"
          @update:bgm-fade-out-seconds="emit('update:bgmFadeOutSeconds', $event)"
          @reset="emit('reset', activeTool)"
        />

        <EntranceSettingsPanel
          v-else-if="activeTool === 'entrance'"
          :effect="entranceEffect"
          @update:effect="emit('update:entranceEffect', $event)"
          @reset="emit('reset', activeTool)"
        />

        <TtsSettingsPanel
          v-else-if="activeTool === 'tts'"
          :text="ttsText"
          :speaker="ttsSpeaker"
          :resource-id="ttsResourceId"
          :configured="ttsConfigured"
          :is-loading-config="isLoadingTtsConfig"
          :is-generating="isGeneratingTts"
          :is-generating-video="isGeneratingNarratedVideo"
          :config-error="ttsConfigError"
          :error="ttsError"
          :narrated-video-error="narratedVideoError"
          :narration-progress-text="narrationProgressText"
          :video-enabled="ttsVideoEnabled"
          :keep-original-audio="ttsKeepOriginalAudio"
          :original-audio-volume="ttsOriginalAudioVolume"
          :subtitle-enabled="ttsSubtitleEnabled"
          :subtitle-position="ttsSubtitlePosition"
          :subtitle-size="ttsSubtitleSize"
          :result="ttsResult"
          :audio-url="ttsAudioUrl"
          @generate="emit('generateTts')"
          @update:text="emit('update:ttsText', $event)"
          @update:speaker="emit('update:ttsSpeaker', $event)"
          @update:video-enabled="emit('update:ttsVideoEnabled', $event)"
          @update:keep-original-audio="emit('update:ttsKeepOriginalAudio', $event)"
          @update:original-audio-volume="emit('update:ttsOriginalAudioVolume', $event)"
          @update:subtitle-enabled="emit('update:ttsSubtitleEnabled', $event)"
          @update:subtitle-position="emit('update:ttsSubtitlePosition', $event)"
          @update:subtitle-size="emit('update:ttsSubtitleSize', $event)"
        />

        <AsrSettingsPanel
          v-else-if="activeTool === 'asr'"
          :configured="asrConfigured"
          :resource-id="asrResourceId"
          :source-file-path="asrSourceFilePath"
          :source-file-name="asrSourceFileName"
          :is-loading-config="isLoadingAsrConfig"
          :is-recognizing="isRecognizingAsr"
          :error="asrError"
          :result="asrResult"
          :is-saving-to-library="isSavingAsrToLibrary"
          :library-feedback="asrLibraryFeedback"
          @select-source="emit('selectAsrSourceFile')"
          @recognize="emit('recognizeAsr')"
          @clear="emit('clearAsrSelection')"
          @save-to-library="emit('saveAsrToLibrary')"
          @save-and-use="emit('saveAndUseAsrScript')"
        />

        <WatermarkToolPanel
          v-else-if="activeTool === 'watermark'"
          :watermark-settings="watermarkSettings"
          :removal-settings="watermarkRemovalSettings"
          :preview-url="watermarkPreviewUrl"
          :disabled="isMixing || isBatchMixing || isExporting || isGeneratingNarratedVideo"
          @select-image="emit('selectWatermarkImage')"
          @update:enabled="emit('update:watermarkEnabled', $event)"
          @update:kind="emit('update:watermarkKind', $event)"
          @update:text="emit('update:watermarkText', $event)"
          @update:position="emit('update:watermarkPosition', $event)"
          @update:opacity="emit('update:watermarkOpacity', $event)"
          @update:margin="emit('update:watermarkMargin', $event)"
          @update:text-font-size="emit('update:watermarkTextFontSize', $event)"
          @update:text-color="emit('update:watermarkTextColor', $event)"
          @update:image-size-ratio="emit('update:watermarkImageSizeRatio', $event)"
          @update:removal-enabled="emit('update:watermarkRemovalEnabled', $event)"
          @update:removal-mode="emit('update:watermarkRemovalMode', $event)"
          @update:removal-position="emit('update:watermarkRemovalPosition', $event)"
          @update:removal-size="emit('update:watermarkRemovalSize', $event)"
          @update:removal-margin="emit('update:watermarkRemovalMargin', $event)"
          @update:removal-strength="emit('update:watermarkRemovalStrength', $event)"
          @update:removal-cover-color="emit('update:watermarkRemovalCoverColor', $event)"
          @update:removal-cover-opacity="emit('update:watermarkRemovalCoverOpacity', $event)"
          @update:removal-tracking-enabled="emit('update:watermarkRemovalTrackingEnabled', $event)"
          @update:removal-tracking-region-width-ratio="emit('update:watermarkRemovalTrackingRegionWidthRatio', $event)"
          @update:removal-tracking-region-height-ratio="emit('update:watermarkRemovalTrackingRegionHeightRatio', $event)"
          @update:removal-tracking-keyframes="emit('update:watermarkRemovalTrackingKeyframes', $event)"
          @auto-detect-watermark="emit('autoDetectWatermark')"
        />

        <CoverSettingsPanel
          v-else-if="activeTool === 'cover'"
          :selected-cover-url="selectedCoverUrl"
          :selected-cover-path="selectedCoverPath"
          :cover-frame-seconds="coverFrameSeconds"
          :is-generating-cover="isGeneratingCover"
          :cover-error="coverError"
            @generate-cover-frame="emit('generateCoverFrame')"
            @update:cover-frame-seconds="emit('update:coverFrameSeconds', $event)"
            @select-cover-image-file="emit('selectCoverImageFile')"
            @select-cover-image-folder="emit('selectCoverImageFolder')"
            @reset="emit('reset', activeTool)"
        />

        <FrameSettingsPanel
          v-else-if="activeTool === 'frame'"
          :material-file-path="frameMaterialFilePath"
          :is-mixing="isMixing"
          :is-batch-mixing="isBatchMixing"
          :settings="frameOperationSettings"
          @update:settings="emit('update:frameOperationSettings', $event)"
          @select-material-file="emit('selectFrameMaterialFile')"
          @clear-material-file="emit('clearFrameMaterialFile')"
          @reset="emit('reset', activeTool)"
        />

        <FusionSettingsPanel
          v-else-if="activeTool === 'fusion'"
          :material-file-path="fusionMaterialFilePath"
          :settings="fusionSettings"
          @update:settings="emit('update:fusionSettings', $event)"
          @select-material-file="emit('selectFusionMaterialFile')"
          @clear-material-file="emit('clearFusionMaterialFile')"
          @reset="emit('reset', activeTool)"
        />

        <VideoEffectsSettingsPanel
          v-else-if="videoEffectTools.includes(activeTool)"
          :active-tool="activeTool"
          :apply-horizontal-mirror="applyHorizontalMirror"
          :apply-vertical-mirror="applyVerticalMirror"
          :playback-speed="playbackSpeed"
          :rotation-mode="rotationMode"
          :brightness="brightness"
          :contrast="contrast"
          :saturation="saturation"
          :effect-scale="effectScale"
          :zoom-enabled="zoomEnabled"
          :zoom-mode="zoomMode"
          :zoom-min-scale="zoomMinScale"
          :zoom-max-scale="zoomMaxScale"
          :zoom-min-duration-seconds="zoomMinDurationSeconds"
          :zoom-max-duration-seconds="zoomMaxDurationSeconds"
          :crop-black-bars="cropBlackBars"
          :random-rotation-min-degrees="randomRotationMinDegrees"
          :random-rotation-max-degrees="randomRotationMaxDegrees"
          :sharpness="sharpness"
          :noise-reduction="noiseReduction"
          :temperature="temperature"
          :visual-style="visualStyle"
          :glow-enabled="glowEnabled"
          :grain-enabled="grainEnabled"
          :vignette-enabled="vignetteEnabled"
          :playback-speed-mode="playbackSpeedMode"
          :playback-speed-min="playbackSpeedMin"
          :playback-speed-max="playbackSpeedMax"
          :playback-segment-min-seconds="playbackSegmentMinSeconds"
          :playback-segment-max-seconds="playbackSegmentMaxSeconds"
          :is-mixing="isMixing"
          :is-batch-mixing="isBatchMixing"
          @update:apply-horizontal-mirror="emit('update:applyHorizontalMirror', $event)"
          @update:apply-vertical-mirror="emit('update:applyVerticalMirror', $event)"
          @update:playback-speed="emit('update:playbackSpeed', $event)"
          @update:rotation-mode="emit('update:rotationMode', $event)"
          @update:brightness="emit('update:brightness', $event)"
          @update:contrast="emit('update:contrast', $event)"
          @update:saturation="emit('update:saturation', $event)"
          @update:effect-scale="emit('update:effectScale', $event)"
          @update:zoom-enabled="emit('update:zoomEnabled', $event)"
          @update:zoom-mode="emit('update:zoomMode', $event)"
          @update:zoom-min-scale="emit('update:zoomMinScale', $event)"
          @update:zoom-max-scale="emit('update:zoomMaxScale', $event)"
          @update:zoom-min-duration-seconds="emit('update:zoomMinDurationSeconds', $event)"
          @update:zoom-max-duration-seconds="emit('update:zoomMaxDurationSeconds', $event)"
          @update:crop-black-bars="emit('update:cropBlackBars', $event)"
          @update:random-rotation-min-degrees="emit('update:randomRotationMinDegrees', $event)"
          @update:random-rotation-max-degrees="emit('update:randomRotationMaxDegrees', $event)"
          @update:sharpness="emit('update:sharpness', $event)"
          @update:noise-reduction="emit('update:noiseReduction', $event)"
          @update:temperature="emit('update:temperature', $event)"
          @update:visual-style="emit('update:visualStyle', $event)"
          @update:glow-enabled="emit('update:glowEnabled', $event)"
          @update:grain-enabled="emit('update:grainEnabled', $event)"
          @update:vignette-enabled="emit('update:vignetteEnabled', $event)"
          @update:playback-speed-mode="emit('update:playbackSpeedMode', $event)"
          @update:playback-speed-min="emit('update:playbackSpeedMin', $event)"
          @update:playback-speed-max="emit('update:playbackSpeedMax', $event)"
          @update:playback-segment-min-seconds="emit('update:playbackSegmentMinSeconds', $event)"
          @update:playback-segment-max-seconds="emit('update:playbackSegmentMaxSeconds', $event)"
          @reset="emit('reset', activeTool)"
        />

        <ExportSettingsPanel
          v-else-if="activeTool === 'export'"
          :output-directory="outputDirectory"
          :output-directory-error="outputDirectoryError"
          :is-exporting="isExporting"
          :is-processing="isExporting || isMixing || isBatchMixing"
          :output-resolution="outputResolution"
          :output-frame-rate="outputFrameRate"
          :output-quality="outputQuality"
          :output-encoder="outputEncoder"
          :encoder-capabilities="encoderCapabilities"
          :encoder-detection-error="encoderDetectionError"
          :is-detecting-encoders="isDetectingEncoders"
          @select-output-directory="emit('selectOutputDirectory')"
          @open-output-directory="emit('openOutputDirectory')"
          @export-selected-video="emit('exportSelectedVideo')"
          @detect-encoders="emit('detectEncoders')"
          @update:output-resolution="emit('update:outputResolution', $event)"
          @update:output-frame-rate="emit('update:outputFrameRate', $event)"
          @update:output-quality="emit('update:outputQuality', $event)"
          @update:output-encoder="emit('update:outputEncoder', $event)"
        />

        <p v-else class="empty-text">这个入口先定版 UI 位置，本次不开发真实功能。</p>
      </div>

      <footer v-if="activeTool !== 'apiKeys' && activeTool !== 'asr' && !replicaParameterTools.includes(activeTool)" class="tool-modal__footer">
        <button class="ghost-button" type="button" @click="emit('reset', activeTool)">重置</button>
        <button class="primary-button" type="button" @click="emit('close')">应用</button>
      </footer>
    </section>
  </div>
</template>
