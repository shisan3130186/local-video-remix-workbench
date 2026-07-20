<script setup lang="ts">
import { ref } from "vue";
import type { WatermarkRemovalSettings, WatermarkSettings, WatermarkTrackingKeyframe } from "../types";
import WatermarkRemovalSettingsPanel from "./WatermarkRemovalSettingsPanel.vue";
import WatermarkSettingsPanel from "./WatermarkSettingsPanel.vue";
import "../watermark.css";

defineProps<{
  watermarkSettings: WatermarkSettings;
  removalSettings: WatermarkRemovalSettings;
  previewUrl: string | null;
  disabled: boolean;
}>();

const emit = defineEmits<{
  selectImage: [];
  "update:enabled": [value: boolean];
  "update:kind": [value: WatermarkSettings["kind"]];
  "update:text": [value: string];
  "update:position": [value: WatermarkSettings["position"]];
  "update:opacity": [value: number];
  "update:margin": [value: number];
  "update:textFontSize": [value: number];
  "update:textColor": [value: string];
  "update:imageSizeRatio": [value: number];
  "update:removalEnabled": [value: boolean];
  "update:removalMode": [value: WatermarkRemovalSettings["mode"]];
  "update:removalPosition": [value: WatermarkRemovalSettings["position"]];
  "update:removalSize": [value: WatermarkRemovalSettings["size"]];
  "update:removalMargin": [value: number];
  "update:removalStrength": [value: number];
  "update:removalCoverColor": [value: string];
  "update:removalCoverOpacity": [value: number];
  "update:removalTrackingEnabled": [value: boolean];
  "update:removalTrackingRegionWidthRatio": [value: number];
  "update:removalTrackingRegionHeightRatio": [value: number];
  "update:removalTrackingKeyframes": [value: WatermarkTrackingKeyframe[]];
}>();

const activeSection = ref<"add" | "remove">("add");
</script>

<template>
  <div class="watermark-tool-tabs" role="tablist" aria-label="水印工具类型">
    <button
      type="button"
      role="tab"
      :aria-selected="activeSection === 'add'"
      :class="{ 'watermark-tool-tab--active': activeSection === 'add' }"
      @click="activeSection = 'add'"
    >添加水印</button>
    <button
      type="button"
      role="tab"
      :aria-selected="activeSection === 'remove'"
      :class="{ 'watermark-tool-tab--active': activeSection === 'remove' }"
      @click="activeSection = 'remove'"
    >处理原水印</button>
  </div>

  <WatermarkSettingsPanel
    v-if="activeSection === 'add'"
    :enabled="watermarkSettings.enabled"
    :kind="watermarkSettings.kind"
    :text="watermarkSettings.text"
    :image-file-path="watermarkSettings.imageFilePath"
    :position="watermarkSettings.position"
    :opacity="watermarkSettings.opacity"
    :margin="watermarkSettings.margin"
    :text-font-size="watermarkSettings.textFontSize"
    :text-color="watermarkSettings.textColor"
    :image-size-ratio="watermarkSettings.imageSizeRatio"
    :disabled="disabled"
    @select-image="emit('selectImage')"
    @update:enabled="emit('update:enabled', $event)"
    @update:kind="emit('update:kind', $event)"
    @update:text="emit('update:text', $event)"
    @update:position="emit('update:position', $event)"
    @update:opacity="emit('update:opacity', $event)"
    @update:margin="emit('update:margin', $event)"
    @update:text-font-size="emit('update:textFontSize', $event)"
    @update:text-color="emit('update:textColor', $event)"
    @update:image-size-ratio="emit('update:imageSizeRatio', $event)"
  />

  <WatermarkRemovalSettingsPanel
    v-else
    :enabled="removalSettings.enabled"
    :mode="removalSettings.mode"
    :position="removalSettings.position"
    :size="removalSettings.size"
    :margin="removalSettings.margin"
    :strength="removalSettings.strength"
    :cover-color="removalSettings.coverColor"
    :cover-opacity="removalSettings.coverOpacity"
    :preview-url="previewUrl"
    :tracking-enabled="removalSettings.trackingEnabled"
    :tracking-region-width-ratio="removalSettings.trackingRegionWidthRatio"
    :tracking-region-height-ratio="removalSettings.trackingRegionHeightRatio"
    :tracking-keyframes="removalSettings.trackingKeyframes"
    :disabled="disabled"
    @update:enabled="emit('update:removalEnabled', $event)"
    @update:mode="emit('update:removalMode', $event)"
    @update:position="emit('update:removalPosition', $event)"
    @update:size="emit('update:removalSize', $event)"
    @update:margin="emit('update:removalMargin', $event)"
    @update:strength="emit('update:removalStrength', $event)"
    @update:cover-color="emit('update:removalCoverColor', $event)"
    @update:cover-opacity="emit('update:removalCoverOpacity', $event)"
    @update:tracking-enabled="emit('update:removalTrackingEnabled', $event)"
    @update:tracking-region-width-ratio="emit('update:removalTrackingRegionWidthRatio', $event)"
    @update:tracking-region-height-ratio="emit('update:removalTrackingRegionHeightRatio', $event)"
    @update:tracking-keyframes="emit('update:removalTrackingKeyframes', $event)"
  />
</template>
