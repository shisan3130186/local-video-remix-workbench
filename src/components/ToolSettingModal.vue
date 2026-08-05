<script setup lang="ts">
import AudioSettingsPanel from "./tool-settings/AudioSettingsPanel.vue";
import { ApiConfigSettingsPanel } from "../features/api-config";
import { AsrSettingsPanel } from "../features/asr";
import CanvasSettingsPanel from "./tool-settings/CanvasSettingsPanel.vue";
import CoverSettingsPanel from "./tool-settings/CoverSettingsPanel.vue";
import ExportSettingsPanel from "./tool-settings/ExportSettingsPanel.vue";
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
  bgm: "背景音乐",
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
</script>

<template>
  <div v-if="activeTool" class="modal-backdrop" @click.self="emit('close')">
    <section
      class="tool-modal"
      :class="[
        `tool-modal--${activeTool}`,
        { 'tool-modal--asr': activeTool === 'asr' },
      ]"
      role="dialog"
      aria-modal="true"
      aria-labelledby="tool-setting-title"
    >
      <header class="tool-modal__header">
        <div>
          <p class="panel__label">二级设置</p>
          <h2 id="tool-setting-title">{{ titles[activeTool] }}</h2>
        </div>
        <button class="panel-toggle" type="button" @click="emit('close')">关闭</button>
      </header>

      <div class="tool-modal__body">
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
          :pip-position="pipPosition"
          :pip-size-ratio="pipSizeRatio"
          :pip-opacity="pipOpacity"
          :pip-margin="pipMargin"
          @select-pip-overlay-file="emit('selectPipOverlayFile')"
          @update:pip-enabled="emit('update:pipEnabled', $event)"
          @update:pip-position="emit('update:pipPosition', $event)"
          @update:pip-size-ratio="emit('update:pipSizeRatio', $event)"
          @update:pip-opacity="emit('update:pipOpacity', $event)"
          @update:pip-margin="emit('update:pipMargin', $event)"
        />

        <AudioSettingsPanel
          v-else-if="activeTool === 'bgm' || activeTool === 'audio'"
          :bgm-enabled="bgmEnabled"
          :bgm-audio-file-path="bgmAudioFilePath"
          :original-volume="originalVolume"
          :bgm-volume="bgmVolume"
          :bgm-fade-in-seconds="bgmFadeInSeconds"
          :bgm-fade-out-seconds="bgmFadeOutSeconds"
          :is-mixing="isMixing"
          :is-batch-mixing="isBatchMixing"
          @select-bgm-audio-file="emit('selectBgmAudioFile')"
          @update:bgm-enabled="emit('update:bgmEnabled', $event)"
          @update:original-volume="emit('update:originalVolume', $event)"
          @update:bgm-volume="emit('update:bgmVolume', $event)"
          @update:bgm-fade-in-seconds="emit('update:bgmFadeInSeconds', $event)"
          @update:bgm-fade-out-seconds="emit('update:bgmFadeOutSeconds', $event)"
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
        />

        <CoverSettingsPanel
          v-else-if="activeTool === 'cover' || activeTool === 'frame'"
          :selected-cover-url="selectedCoverUrl"
          :cover-frame-seconds="coverFrameSeconds"
          :is-generating-cover="isGeneratingCover"
          :cover-error="coverError"
          @generate-cover-frame="emit('generateCoverFrame')"
          @update:cover-frame-seconds="emit('update:coverFrameSeconds', $event)"
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

      <footer v-if="activeTool !== 'apiKeys' && activeTool !== 'asr'" class="tool-modal__footer">
        <button class="ghost-button" type="button" @click="emit('reset', activeTool)">重置</button>
        <button class="primary-button" type="button" @click="emit('close')">应用</button>
      </footer>
    </section>
  </div>
</template>
