import { computed, ref } from "vue";
import { DEFAULT_WATERMARK_REMOVAL_SETTINGS } from "./types";
import type {
  WatermarkPosition,
  WatermarkRemovalMode,
  WatermarkRemovalSettings,
  WatermarkRemovalSize,
  WatermarkTrackingKeyframe,
} from "./types";

export function useWatermarkRemovalSettings() {
  const watermarkRemovalEnabled = ref(DEFAULT_WATERMARK_REMOVAL_SETTINGS.enabled);
  const watermarkRemovalMode = ref<WatermarkRemovalMode>(DEFAULT_WATERMARK_REMOVAL_SETTINGS.mode);
  const watermarkRemovalPosition = ref<WatermarkPosition>(DEFAULT_WATERMARK_REMOVAL_SETTINGS.position);
  const watermarkRemovalSize = ref<WatermarkRemovalSize>(DEFAULT_WATERMARK_REMOVAL_SETTINGS.size);
  const watermarkRemovalMargin = ref(DEFAULT_WATERMARK_REMOVAL_SETTINGS.margin);
  const watermarkRemovalStrength = ref(DEFAULT_WATERMARK_REMOVAL_SETTINGS.strength);
  const watermarkRemovalCoverColor = ref(DEFAULT_WATERMARK_REMOVAL_SETTINGS.coverColor);
  const watermarkRemovalCoverOpacity = ref(DEFAULT_WATERMARK_REMOVAL_SETTINGS.coverOpacity);
  const watermarkRemovalTrackingEnabled = ref(DEFAULT_WATERMARK_REMOVAL_SETTINGS.trackingEnabled);
  const watermarkRemovalTrackingRegionWidthRatio = ref(DEFAULT_WATERMARK_REMOVAL_SETTINGS.trackingRegionWidthRatio);
  const watermarkRemovalTrackingRegionHeightRatio = ref(DEFAULT_WATERMARK_REMOVAL_SETTINGS.trackingRegionHeightRatio);
  const watermarkRemovalTrackingKeyframes = ref<WatermarkTrackingKeyframe[]>([]);

  const watermarkRemovalSettings = computed(
    (): WatermarkRemovalSettings => ({
      enabled: watermarkRemovalEnabled.value,
      mode: watermarkRemovalMode.value,
      position: watermarkRemovalPosition.value,
      size: watermarkRemovalSize.value,
      margin: watermarkRemovalMargin.value,
      strength: watermarkRemovalStrength.value,
      coverColor: watermarkRemovalCoverColor.value,
      coverOpacity: watermarkRemovalCoverOpacity.value,
      trackingEnabled: watermarkRemovalTrackingEnabled.value,
      trackingRegionWidthRatio: watermarkRemovalTrackingRegionWidthRatio.value,
      trackingRegionHeightRatio: watermarkRemovalTrackingRegionHeightRatio.value,
      trackingKeyframes: watermarkRemovalTrackingKeyframes.value.map((keyframe) => ({ ...keyframe })),
    }),
  );

  function restoreWatermarkRemovalSettings(settings?: WatermarkRemovalSettings) {
    const value = settings ?? DEFAULT_WATERMARK_REMOVAL_SETTINGS;
    watermarkRemovalEnabled.value = value.enabled;
    watermarkRemovalMode.value = value.mode;
    watermarkRemovalPosition.value = value.mode === "crop" && value.position === "center" ? "topRight" : value.position;
    watermarkRemovalSize.value = value.size;
    watermarkRemovalMargin.value = value.margin;
    watermarkRemovalStrength.value = value.strength;
    watermarkRemovalCoverColor.value = value.coverColor;
    watermarkRemovalCoverOpacity.value = value.coverOpacity;
    watermarkRemovalTrackingEnabled.value = value.trackingEnabled ?? false;
    watermarkRemovalTrackingRegionWidthRatio.value = value.trackingRegionWidthRatio ?? DEFAULT_WATERMARK_REMOVAL_SETTINGS.trackingRegionWidthRatio;
    watermarkRemovalTrackingRegionHeightRatio.value = value.trackingRegionHeightRatio ?? DEFAULT_WATERMARK_REMOVAL_SETTINGS.trackingRegionHeightRatio;
    watermarkRemovalTrackingKeyframes.value = Array.isArray(value.trackingKeyframes)
      ? value.trackingKeyframes.map((keyframe) => ({ ...keyframe }))
      : [];
  }

  function resetWatermarkRemovalSettings() {
    restoreWatermarkRemovalSettings(DEFAULT_WATERMARK_REMOVAL_SETTINGS);
  }

  function validateWatermarkRemovalSettings() {
    if (!watermarkRemovalEnabled.value) return null;
    if (watermarkRemovalMode.value === "crop" && watermarkRemovalPosition.value === "center") {
      return "裁剪去水印只适用于画面顶部或底部，请选择四个边角之一。";
    }
    if (watermarkRemovalTrackingEnabled.value) {
      if (!(["blur", "mosaic", "cover"] as WatermarkRemovalMode[]).includes(watermarkRemovalMode.value)) {
        return "移动水印跟踪只支持区域模糊、马赛克或色块遮盖。";
      }
      if (
        !Number.isFinite(watermarkRemovalTrackingRegionWidthRatio.value) ||
        watermarkRemovalTrackingRegionWidthRatio.value < 0.04 ||
        watermarkRemovalTrackingRegionWidthRatio.value > 0.8 ||
        !Number.isFinite(watermarkRemovalTrackingRegionHeightRatio.value) ||
        watermarkRemovalTrackingRegionHeightRatio.value < 0.04 ||
        watermarkRemovalTrackingRegionHeightRatio.value > 0.8
      ) {
        return "移动水印框的宽度和高度必须在画面的4%到80%之间。";
      }
      const keyframes = watermarkRemovalTrackingKeyframes.value;
      if (keyframes.length < 2 || keyframes.length > 8) {
        return "移动水印至少需要2个、最多支持8个关键位置。";
      }
      for (let index = 0; index < keyframes.length; index += 1) {
        const keyframe = keyframes[index];
        const previous = keyframes[index - 1];
        if (
          !Number.isFinite(keyframe.timeSeconds) || keyframe.timeSeconds < 0 ||
          !Number.isFinite(keyframe.xRatio) || keyframe.xRatio < 0 ||
          !Number.isFinite(keyframe.yRatio) || keyframe.yRatio < 0 ||
          keyframe.xRatio + watermarkRemovalTrackingRegionWidthRatio.value > 1.0001 ||
          keyframe.yRatio + watermarkRemovalTrackingRegionHeightRatio.value > 1.0001
        ) {
          return `第${index + 1}个移动水印关键位置超出画面。`;
        }
        if (previous && keyframe.timeSeconds - previous.timeSeconds < 0.05) {
          return "移动水印关键位置的时间必须依次递增，且至少间隔0.05秒。";
        }
      }
    }
    if (!Number.isInteger(watermarkRemovalMargin.value) || watermarkRemovalMargin.value < 0 || watermarkRemovalMargin.value > 160) {
      return "原水印处理边距必须是0到160之间的整数。";
    }
    if (
      (watermarkRemovalMode.value === "blur" || watermarkRemovalMode.value === "mosaic") &&
      (!Number.isInteger(watermarkRemovalStrength.value) || watermarkRemovalStrength.value < 4 || watermarkRemovalStrength.value > 24)
    ) {
      return "模糊或马赛克强度必须是4到24之间的整数。";
    }
    if (watermarkRemovalMode.value === "cover") {
      if (!/^#[0-9a-f]{6}$/i.test(watermarkRemovalCoverColor.value)) {
        return "请选择有效的遮盖颜色。";
      }
      if (
        !Number.isFinite(watermarkRemovalCoverOpacity.value) ||
        watermarkRemovalCoverOpacity.value < 0.1 ||
        watermarkRemovalCoverOpacity.value > 1
      ) {
        return "遮盖透明度必须在10%到100%之间。";
      }
    }
    return null;
  }

  return {
    resetWatermarkRemovalSettings,
    restoreWatermarkRemovalSettings,
    validateWatermarkRemovalSettings,
    watermarkRemovalCoverColor,
    watermarkRemovalCoverOpacity,
    watermarkRemovalEnabled,
    watermarkRemovalMargin,
    watermarkRemovalMode,
    watermarkRemovalPosition,
    watermarkRemovalSettings,
    watermarkRemovalSize,
    watermarkRemovalStrength,
    watermarkRemovalTrackingEnabled,
    watermarkRemovalTrackingKeyframes,
    watermarkRemovalTrackingRegionHeightRatio,
    watermarkRemovalTrackingRegionWidthRatio,
  };
}
