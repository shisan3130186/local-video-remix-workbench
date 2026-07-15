<script setup lang="ts">
import AudioSettingsPanel from "./tool-settings/AudioSettingsPanel.vue";
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

defineProps<ToolSettingModalProps>();
const emit = defineEmits<ToolSettingModalEmits>();

const titles: Record<ToolKey, string> = {
  remix: "混剪设置",
  canvas: "画布设置",
  audio: "音频设置",
  bgm: "背景音乐",
  tts: "语音合成",
  subtitleStyle: "字幕样式",
  watermark: "去除水印",
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
    <section class="tool-modal" role="dialog" aria-modal="true" aria-labelledby="tool-setting-title">
      <header class="tool-modal__header">
        <div>
          <p class="panel__label">二级设置</p>
          <h2 id="tool-setting-title">{{ titles[activeTool] }}</h2>
        </div>
        <button class="panel-toggle" type="button" @click="emit('close')">关闭</button>
      </header>

      <div class="tool-modal__body">
        <RemixSettingsPanel
          v-if="activeTool === 'remix'"
          :segment-duration-seconds="segmentDurationSeconds"
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
          :config-error="ttsConfigError"
          :error="ttsError"
          :result="ttsResult"
          :audio-url="ttsAudioUrl"
          @generate="emit('generateTts')"
          @update:text="emit('update:ttsText', $event)"
          @update:speaker="emit('update:ttsSpeaker', $event)"
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
          @select-output-directory="emit('selectOutputDirectory')"
          @open-output-directory="emit('openOutputDirectory')"
          @export-selected-video="emit('exportSelectedVideo')"
        />

        <p v-else class="empty-text">这个入口先定版 UI 位置，本次不开发真实功能。</p>
      </div>

      <footer class="tool-modal__footer">
        <button class="ghost-button" type="button" @click="emit('reset', activeTool)">重置</button>
        <button class="primary-button" type="button" @click="emit('close')">应用</button>
      </footer>
    </section>
  </div>
</template>
