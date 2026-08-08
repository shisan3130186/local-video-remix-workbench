import { computed, ref } from "vue";
import type {
  BgmSettings,
  CanvasAspectRatio,
  CanvasBackgroundMode,
  DynamicZoomMode,
  PictureInPictureSettings,
  PipPosition,
  RemixExportSettings,
  RotationMode,
  SubtitlePosition,
  SubtitleSize,
  SubtitleSettings,
  VideoEffectSettings,
} from "../services/videoMixService";
import { useOutputSettings } from "../features/output-settings";
import { DEFAULT_WATERMARK_REMOVAL_SETTINGS, useWatermarkRemovalSettings, useWatermarkSettings } from "../features/watermark";
import type { WatermarkRemovalRegion } from "../features/watermark/types";

export function useRemixSettings() {
  const output = useOutputSettings();
  const watermark = useWatermarkSettings();
  const watermarkRemoval = useWatermarkRemovalSettings();
  const watermarkRemovalRegionCount = ref(DEFAULT_WATERMARK_REMOVAL_SETTINGS.regionCount);
  const watermarkRemovalManualRegions = ref<WatermarkRemovalRegion[]>(DEFAULT_WATERMARK_REMOVAL_SETTINGS.manualRegions.map((region) => ({ ...region })));
  const applyHorizontalMirror = ref(false);
  const playbackSpeed = ref(1.0);
  const smoothRemixEnabled = ref(false);
  const applyVerticalMirror = ref(false);
  const rotationMode = ref<RotationMode>("none");
  const hslEnabled = ref(false);
  const hue = ref(0);
  const brightness = ref(0);
  const contrast = ref(1);
  const saturation = ref(1);
  const effectScale = ref(1);
  const zoomEnabled = ref(false);
  const zoomMode = ref<DynamicZoomMode>("push");
  const zoomMinScale = ref(1.02);
  const zoomMaxScale = ref(1.08);
  const zoomMinDurationSeconds = ref(8);
  const zoomMaxDurationSeconds = ref(10);
  const pipEnabled = ref(false);
  const pipOverlayFilePath = ref<string | null>(null);
  const pipPosition = ref<PipPosition>("topRight");
  const pipSizeRatio = ref(0.3);
  const pipOpacity = ref(1);
  const pipMargin = ref(24);
  const bgmEnabled = ref(false);
  const bgmAudioFilePath = ref<string | null>(null);
  const originalVolume = ref(1);
  const bgmVolume = ref(0.35);
  const bgmFadeInSeconds = ref(0.5);
  const bgmFadeOutSeconds = ref(0.5);
  const canvasAspectRatio = ref<CanvasAspectRatio>("original");
  const canvasBackgroundMode = ref<CanvasBackgroundMode>("black");
  const subtitleEnabled = ref(false);
  const subtitleText = ref("");
  const subtitlePosition = ref<SubtitlePosition>("bottom");
  const subtitleSize = ref<SubtitleSize>("medium");

  const videoEffectSettings = computed(
    (): VideoEffectSettings => ({
      verticalMirror: applyVerticalMirror.value,
      rotation: rotationMode.value,
      hslEnabled: hslEnabled.value,
      hue: hue.value,
      brightness: brightness.value,
      contrast: contrast.value,
      saturation: saturation.value,
      scale: effectScale.value,
      zoomEnabled: zoomEnabled.value,
      zoomMode: zoomMode.value,
      zoomMinScale: zoomMinScale.value,
      zoomMaxScale: zoomMaxScale.value,
      zoomMinDurationSeconds: zoomMinDurationSeconds.value,
      zoomMaxDurationSeconds: zoomMaxDurationSeconds.value,
    }),
  );

  const pictureInPictureSettings = computed(
    (): PictureInPictureSettings => ({
      enabled: pipEnabled.value,
      overlayFilePath: pipOverlayFilePath.value,
      position: pipPosition.value,
      sizeRatio: pipSizeRatio.value,
      opacity: pipOpacity.value,
      margin: pipMargin.value,
    }),
  );

  const bgmSettings = computed(
    (): BgmSettings => ({
      enabled: bgmEnabled.value,
      audioFilePath: bgmAudioFilePath.value,
      originalVolume: originalVolume.value,
      bgmVolume: bgmVolume.value,
      fadeInSeconds: bgmFadeInSeconds.value,
      fadeOutSeconds: bgmFadeOutSeconds.value,
    }),
  );

  const subtitleSettings = computed(
    (): SubtitleSettings => ({
      enabled: subtitleEnabled.value,
      text: subtitleText.value,
      position: subtitlePosition.value,
      size: subtitleSize.value,
      fontSize: subtitleSize.value === "small" ? 38 : subtitleSize.value === "large" ? 56 : 46,
      textColor: "#ffffff",
      backgroundEnabled: false,
    }),
  );

  const remixExportSettings = computed(
    (): RemixExportSettings => ({
      applyHorizontalMirror: applyHorizontalMirror.value,
      playbackSpeed: playbackSpeed.value,
      canvasAspectRatio: canvasAspectRatio.value,
      canvasBackgroundMode: canvasBackgroundMode.value,
      smoothRemixEnabled: smoothRemixEnabled.value,
      videoEffectSettings: videoEffectSettings.value,
      pictureInPictureSettings: pictureInPictureSettings.value,
      bgmSettings: bgmSettings.value,
      watermarkSettings: watermark.watermarkSettings.value,
      watermarkRemovalSettings: {
        ...watermarkRemoval.watermarkRemovalSettings.value,
        regionCount: watermarkRemovalRegionCount.value,
        manualRegions: watermarkRemovalManualRegions.value.map((region) => ({ ...region })),
      },
      subtitleSettings: subtitleSettings.value,
      outputSettings: output.outputSettings.value,
    }),
  );

  function restoreRemixSettings(settings: RemixExportSettings) {
    applyHorizontalMirror.value = settings.applyHorizontalMirror;
    playbackSpeed.value = settings.playbackSpeed;
    smoothRemixEnabled.value = settings.smoothRemixEnabled;
    canvasAspectRatio.value = settings.canvasAspectRatio;
    canvasBackgroundMode.value = settings.canvasBackgroundMode;
    subtitleEnabled.value = settings.subtitleSettings.enabled;
    subtitleText.value = settings.subtitleSettings.text;
    subtitlePosition.value = settings.subtitleSettings.position;
    subtitleSize.value = settings.subtitleSettings.size ?? "medium";
    applyVerticalMirror.value = settings.videoEffectSettings.verticalMirror;
    rotationMode.value = settings.videoEffectSettings.rotation;
    hslEnabled.value = settings.videoEffectSettings.hslEnabled ?? false;
    hue.value = settings.videoEffectSettings.hue ?? 0;
    brightness.value = settings.videoEffectSettings.brightness;
    contrast.value = settings.videoEffectSettings.contrast;
    saturation.value = settings.videoEffectSettings.saturation;
    effectScale.value = settings.videoEffectSettings.scale;
    zoomEnabled.value = settings.videoEffectSettings.zoomEnabled ?? false;
    zoomMode.value = settings.videoEffectSettings.zoomMode ?? "push";
    zoomMinScale.value = settings.videoEffectSettings.zoomMinScale ?? 1.02;
    zoomMaxScale.value = settings.videoEffectSettings.zoomMaxScale ?? 1.08;
    zoomMinDurationSeconds.value = settings.videoEffectSettings.zoomMinDurationSeconds ?? 8;
    zoomMaxDurationSeconds.value = settings.videoEffectSettings.zoomMaxDurationSeconds ?? 10;
    pipEnabled.value = settings.pictureInPictureSettings.enabled;
    pipOverlayFilePath.value = settings.pictureInPictureSettings.overlayFilePath;
    pipPosition.value = settings.pictureInPictureSettings.position;
    pipSizeRatio.value = settings.pictureInPictureSettings.sizeRatio;
    pipOpacity.value = settings.pictureInPictureSettings.opacity;
    pipMargin.value = settings.pictureInPictureSettings.margin;
    bgmEnabled.value = settings.bgmSettings.enabled;
    bgmAudioFilePath.value = settings.bgmSettings.audioFilePath;
    originalVolume.value = settings.bgmSettings.originalVolume;
    bgmVolume.value = settings.bgmSettings.bgmVolume;
    bgmFadeInSeconds.value = settings.bgmSettings.fadeInSeconds;
    bgmFadeOutSeconds.value = settings.bgmSettings.fadeOutSeconds;
    watermark.restoreWatermarkSettings(settings.watermarkSettings);
    watermarkRemoval.restoreWatermarkRemovalSettings(settings.watermarkRemovalSettings);
    watermarkRemovalRegionCount.value = settings.watermarkRemovalSettings.regionCount ?? DEFAULT_WATERMARK_REMOVAL_SETTINGS.regionCount;
    watermarkRemovalManualRegions.value = (settings.watermarkRemovalSettings.manualRegions ?? DEFAULT_WATERMARK_REMOVAL_SETTINGS.manualRegions).map((region) => ({ ...region }));
    output.restoreOutputSettings(settings.outputSettings);
  }

  return {
    ...output,
    ...watermark,
    ...watermarkRemoval,
    applyHorizontalMirror,
    applyVerticalMirror,
    bgmAudioFilePath,
    bgmEnabled,
    bgmFadeInSeconds,
    bgmFadeOutSeconds,
    bgmSettings,
    bgmVolume,
    brightness,
    canvasAspectRatio,
    canvasBackgroundMode,
    contrast,
    effectScale,
    hslEnabled,
    hue,
    watermarkAssetType: watermark.watermarkAssetType,
    watermarkTrajectory: watermark.watermarkTrajectory,
    watermarkRemovalRegionCount,
    watermarkRemovalManualRegions,
    originalVolume,
    pictureInPictureSettings,
    pipEnabled,
    pipMargin,
    pipOpacity,
    pipOverlayFilePath,
    pipPosition,
    pipSizeRatio,
    playbackSpeed,
    remixExportSettings,
    rotationMode,
    restoreRemixSettings,
    saturation,
    zoomEnabled,
    zoomMode,
    zoomMinScale,
    zoomMaxScale,
    zoomMinDurationSeconds,
    zoomMaxDurationSeconds,
    subtitleEnabled,
    subtitlePosition,
    subtitleSettings,
    subtitleSize,
    subtitleText,
    smoothRemixEnabled,
    videoEffectSettings,
  };
}
