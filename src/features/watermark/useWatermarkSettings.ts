import { computed, ref } from "vue";
import { DEFAULT_WATERMARK_SETTINGS } from "./types";
import type { WatermarkKind, WatermarkPosition, WatermarkSettings } from "./types";

export function useWatermarkSettings() {
  const watermarkEnabled = ref(DEFAULT_WATERMARK_SETTINGS.enabled);
  const watermarkKind = ref<WatermarkKind>(DEFAULT_WATERMARK_SETTINGS.kind);
  const watermarkAssetType = ref(DEFAULT_WATERMARK_SETTINGS.assetType);
  const watermarkText = ref(DEFAULT_WATERMARK_SETTINGS.text);
  const watermarkImageFilePath = ref<string | null>(DEFAULT_WATERMARK_SETTINGS.imageFilePath);
  const watermarkPosition = ref<WatermarkPosition>(DEFAULT_WATERMARK_SETTINGS.position);
  const watermarkOpacity = ref(DEFAULT_WATERMARK_SETTINGS.opacity);
  const watermarkMargin = ref(DEFAULT_WATERMARK_SETTINGS.margin);
  const watermarkTextFontSize = ref(DEFAULT_WATERMARK_SETTINGS.textFontSize);
  const watermarkTextColor = ref(DEFAULT_WATERMARK_SETTINGS.textColor);
  const watermarkImageSizeRatio = ref(DEFAULT_WATERMARK_SETTINGS.imageSizeRatio);
  const watermarkImagePositionXRatio = ref(DEFAULT_WATERMARK_SETTINGS.imagePositionXRatio);
  const watermarkImagePositionYRatio = ref(DEFAULT_WATERMARK_SETTINGS.imagePositionYRatio);
  const watermarkTrajectory = ref(DEFAULT_WATERMARK_SETTINGS.trajectory);

  const watermarkSettings = computed(
    (): WatermarkSettings => ({
      enabled: watermarkEnabled.value,
      kind: watermarkKind.value,
      assetType: watermarkAssetType.value,
      text: watermarkText.value,
      imageFilePath: watermarkImageFilePath.value,
      position: watermarkPosition.value,
      opacity: watermarkOpacity.value,
      margin: watermarkMargin.value,
      textFontSize: watermarkTextFontSize.value,
      textColor: watermarkTextColor.value,
      imageSizeRatio: watermarkImageSizeRatio.value,
      imagePositionXRatio: watermarkImagePositionXRatio.value,
      imagePositionYRatio: watermarkImagePositionYRatio.value,
      trajectory: watermarkTrajectory.value,
    }),
  );

  function restoreWatermarkSettings(settings?: WatermarkSettings) {
    const value = settings ?? DEFAULT_WATERMARK_SETTINGS;
    watermarkEnabled.value = value.enabled;
    watermarkKind.value = value.kind;
    watermarkAssetType.value = value.assetType ?? DEFAULT_WATERMARK_SETTINGS.assetType;
    watermarkText.value = value.text;
    watermarkImageFilePath.value = value.imageFilePath;
    watermarkPosition.value = value.position;
    watermarkOpacity.value = value.opacity;
    watermarkMargin.value = value.margin;
    watermarkTextFontSize.value = value.textFontSize;
    watermarkTextColor.value = value.textColor;
    watermarkImageSizeRatio.value = value.imageSizeRatio;
    watermarkImagePositionXRatio.value = value.imagePositionXRatio ?? DEFAULT_WATERMARK_SETTINGS.imagePositionXRatio;
    watermarkImagePositionYRatio.value = value.imagePositionYRatio ?? DEFAULT_WATERMARK_SETTINGS.imagePositionYRatio;
    watermarkTrajectory.value = value.trajectory ?? DEFAULT_WATERMARK_SETTINGS.trajectory;
  }

  function resetWatermarkSettings() {
    restoreWatermarkSettings(DEFAULT_WATERMARK_SETTINGS);
  }

  function validateWatermarkSettings() {
    if (!watermarkEnabled.value) return null;
    if (watermarkKind.value === "text" && !watermarkText.value.trim()) {
      return "请输入文字水印内容。";
    }
    if (watermarkKind.value === "image" && !watermarkImageFilePath.value) {
      return "请先选择图片水印文件。";
    }
    if (!Number.isFinite(watermarkOpacity.value) || watermarkOpacity.value < 0.1 || watermarkOpacity.value > 1) {
      return "水印透明度必须在10%到100%之间。";
    }
    if (!Number.isInteger(watermarkMargin.value) || watermarkMargin.value < 0 || watermarkMargin.value > 240) {
      return "水印边距必须是0到240之间的整数。";
    }
    if (watermarkKind.value === "text") {
      if (!Number.isInteger(watermarkTextFontSize.value) || watermarkTextFontSize.value < 16 || watermarkTextFontSize.value > 120) {
        return "文字水印字号必须是16到120之间的整数。";
      }
      if (!/^#[0-9a-f]{6}$/i.test(watermarkTextColor.value)) {
        return "请选择有效的文字颜色。";
      }
    }
    if (
      watermarkKind.value === "image" &&
      (!Number.isFinite(watermarkImageSizeRatio.value) || watermarkImageSizeRatio.value < 0.08 || watermarkImageSizeRatio.value > 0.5)
    ) {
      return "图片水印大小必须在8%到50%之间。";
    }
    if (
      watermarkKind.value === "image" &&
      (!Number.isFinite(watermarkImagePositionXRatio.value) || watermarkImagePositionXRatio.value < 0 || watermarkImagePositionXRatio.value > 1 ||
        !Number.isFinite(watermarkImagePositionYRatio.value) || watermarkImagePositionYRatio.value < 0 || watermarkImagePositionYRatio.value > 1)
    ) {
      return "图片水印位置必须位于视频画面内部。";
    }
    return null;
  }

  return {
    resetWatermarkSettings,
    restoreWatermarkSettings,
    validateWatermarkSettings,
    watermarkEnabled,
    watermarkAssetType,
    watermarkImageFilePath,
    watermarkImagePositionXRatio,
    watermarkImagePositionYRatio,
    watermarkImageSizeRatio,
    watermarkKind,
    watermarkMargin,
    watermarkOpacity,
    watermarkPosition,
    watermarkSettings,
    watermarkText,
    watermarkTextColor,
    watermarkTextFontSize,
    watermarkTrajectory,
  };
}
