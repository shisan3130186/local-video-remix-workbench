import { computed, ref } from "vue";
import type {
  BgmSettings,
  CanvasCropSettings,
  CanvasAspectRatio,
  CanvasBackgroundMode,
  DynamicZoomMode,
  PlaybackSpeedMode,
  PictureInPictureSettings,
  PipPosition,
  RemixExportSettings,
  RotationMode,
  SubtitlePosition,
  SubtitleSize,
  SubtitleSettings,
  VideoEffectSettings,
  EntranceEffect,
  FrameOperationSettings,
  FusionSettings,
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
  const playbackSpeedMode = ref<PlaybackSpeedMode>("global");
  const playbackSpeedMin = ref(1.0);
  const playbackSpeedMax = ref(1.0);
  const playbackSegmentMinSeconds = ref(2);
  const playbackSegmentMaxSeconds = ref(4);
  const smoothRemixEnabled = ref(false);
  const applyVerticalMirror = ref(false);
  const rotationMode = ref<RotationMode>("none");
  const hslEnabled = ref(false);
  const hue = ref(0);
  const brightness = ref(0);
  const contrast = ref(1);
  const saturation = ref(1);
  const effectScale = ref(1);
  const cropSettings = ref<CanvasCropSettings>({ enabled: false, x: 0, y: 0, width: 100, height: 100 });
  const zoomEnabled = ref(false);
  const zoomMode = ref<DynamicZoomMode>("push");
  const zoomMinScale = ref(1.02);
  const zoomMaxScale = ref(1.08);
  const zoomMinDurationSeconds = ref(8);
  const zoomMaxDurationSeconds = ref(10);
  const cropBlackBars = ref(false);
  const randomRotationMinDegrees = ref(0);
  const randomRotationMaxDegrees = ref(0);
  const sharpness = ref(0);
  const noiseReduction = ref(0);
  const temperature = ref(6500);
  const visualStyle = ref<VideoEffectSettings["visualStyle"]>("none");
  const glowEnabled = ref(false);
  const grainEnabled = ref(false);
  const vignetteEnabled = ref(false);
  const pipEnabled = ref(false);
  const pipOverlayFilePath = ref<string | null>(null);
  const pipMode = ref<"main" | "external">("external");
  const pipMainSizeMin = ref(0.96);
  const pipMainSizeMax = ref(0.99);
  const pipBlurMin = ref(40);
  const pipBlurMax = ref(50);
  const pipOffsetXMin = ref(0.5);
  const pipOffsetXMax = ref(0.5);
  const pipOffsetYMin = ref(0.5);
  const pipOffsetYMax = ref(0.5);
  const pipPosition = ref<PipPosition>("topRight");
  const pipSizeRatio = ref(0.3);
  const pipOpacity = ref(1);
  const pipMargin = ref(24);
  const bgmEnabled = ref(false);
  const bgmAudioFilePath = ref<string | null>(null);
  const originalVolume = ref(1);
  const originalVolumeMin = ref(0.95);
  const originalVolumeMax = ref(1.05);
  const originalFadeEnabled = ref(false);
  const dynamicAudioAdjustEnabled = ref(true);
  const bgmVolume = ref(0.35);
  const bgmVolumeMin = ref(0.25);
  const bgmVolumeMax = ref(0.35);
  const bgmFadeEnabled = ref(false);
  const bgmLoopPlaybackEnabled = ref(true);
  const bgmFadeInSeconds = ref(0.5);
  const bgmFadeOutSeconds = ref(0.5);
  const canvasAspectRatio = ref<CanvasAspectRatio>("original");
  const canvasBackgroundMode = ref<CanvasBackgroundMode>("black");
  const subtitleEnabled = ref(false);
  const subtitleText = ref("");
  const subtitlePosition = ref<SubtitlePosition>("bottom");
  const subtitleSize = ref<SubtitleSize>("medium");
  const subtitleFontFamily = ref("Microsoft YaHei");
  const subtitleTextColor = ref("#ffffff");
  const subtitleOpacity = ref(1);
  const entranceEffect = ref<EntranceEffect>("none");
  const frameOperationSettings = ref<FrameOperationSettings>({ enabled: false, mode: "extract", intervalMin: 30, intervalMax: 60, frameMin: 1, frameMax: 1, opacity: 2, materialFilePath: null });
  const fusionSettings = ref<FusionSettings>({ enabled: false, materialFilePath: null, intervalMin: 15, intervalMax: 60, strength: 0.2 });

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
      crop: { ...cropSettings.value },
      zoomEnabled: zoomEnabled.value,
      zoomMode: zoomMode.value,
      zoomMinScale: zoomMinScale.value,
      zoomMaxScale: zoomMaxScale.value,
      zoomMinDurationSeconds: zoomMinDurationSeconds.value,
      zoomMaxDurationSeconds: zoomMaxDurationSeconds.value,
      cropBlackBars: cropBlackBars.value,
      randomRotationMinDegrees: randomRotationMinDegrees.value,
      randomRotationMaxDegrees: randomRotationMaxDegrees.value,
      sharpness: sharpness.value,
      noiseReduction: noiseReduction.value,
      temperature: temperature.value,
      visualStyle: visualStyle.value,
      glowEnabled: glowEnabled.value,
      grainEnabled: grainEnabled.value,
      vignetteEnabled: vignetteEnabled.value,
    }),
  );

  const pictureInPictureSettings = computed(
    (): PictureInPictureSettings => ({
      enabled: pipEnabled.value,
      overlayFilePath: pipOverlayFilePath.value,
      mode: pipMode.value,
      mainSizeMin: pipMainSizeMin.value,
      mainSizeMax: pipMainSizeMax.value,
      blurMin: pipBlurMin.value,
      blurMax: pipBlurMax.value,
      offsetXMin: pipOffsetXMin.value,
      offsetXMax: pipOffsetXMax.value,
      offsetYMin: pipOffsetYMin.value,
      offsetYMax: pipOffsetYMax.value,
      position: pipPosition.value,
      sizeRatio: pipSizeRatio.value,
      opacity: pipOpacity.value,
      margin: pipMargin.value,
    }),
  );

  const bgmSettings = computed(
    (): BgmSettings => {
      const originalRange = originalVolume.value >= originalVolumeMin.value && originalVolume.value <= originalVolumeMax.value
        ? [originalVolumeMin.value, originalVolumeMax.value]
        : [originalVolume.value, originalVolume.value];
      const bgmRange = bgmVolume.value >= bgmVolumeMin.value && bgmVolume.value <= bgmVolumeMax.value
        ? [bgmVolumeMin.value, bgmVolumeMax.value]
        : [bgmVolume.value, bgmVolume.value];
      return ({
      enabled: bgmEnabled.value,
      audioFilePath: bgmAudioFilePath.value,
      originalVolume: originalVolume.value,
      originalVolumeMin: originalRange[0],
      originalVolumeMax: originalRange[1],
      originalFadeEnabled: originalFadeEnabled.value,
      dynamicAdjustEnabled: dynamicAudioAdjustEnabled.value,
      bgmVolume: bgmVolume.value,
      bgmVolumeMin: bgmRange[0],
      bgmVolumeMax: bgmRange[1],
      bgmFadeEnabled: bgmFadeEnabled.value,
      loopPlaybackEnabled: bgmLoopPlaybackEnabled.value,
      fadeInSeconds: bgmFadeInSeconds.value,
      fadeOutSeconds: bgmFadeOutSeconds.value,
      });
    },
  );

  const subtitleSettings = computed(
    (): SubtitleSettings => ({
      enabled: subtitleEnabled.value,
      text: subtitleText.value,
      position: subtitlePosition.value,
      size: subtitleSize.value,
      fontSize: subtitleSize.value === "small" ? 38 : subtitleSize.value === "large" ? 56 : 46,
      fontFamily: subtitleFontFamily.value,
      textColor: subtitleTextColor.value,
      opacity: subtitleOpacity.value,
      backgroundEnabled: false,
    }),
  );

  const remixExportSettings = computed(
    (): RemixExportSettings => {
      const usesLegacyGlobalSpeed =
        playbackSpeedMode.value === "global" &&
        Math.abs(playbackSpeedMin.value - 1) < 0.001 &&
        Math.abs(playbackSpeedMax.value - 1) < 0.001 &&
        Math.abs(playbackSpeed.value - 1) > 0.001;
      return ({
      applyHorizontalMirror: applyHorizontalMirror.value,
      playbackSpeed: playbackSpeed.value,
      playbackSpeedSettings: {
        mode: playbackSpeedMode.value,
        min: usesLegacyGlobalSpeed ? playbackSpeed.value : playbackSpeedMin.value,
        max: usesLegacyGlobalSpeed ? playbackSpeed.value : playbackSpeedMax.value,
        segmentMinSeconds: playbackSegmentMinSeconds.value,
        segmentMaxSeconds: playbackSegmentMaxSeconds.value,
      },
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
      outputName: null,
      entranceEffect: entranceEffect.value,
      frameOperationSettings: { ...frameOperationSettings.value },
      fusionSettings: { ...fusionSettings.value },
      });
    },
  );

  function restoreRemixSettings(settings: RemixExportSettings) {
    applyHorizontalMirror.value = settings.applyHorizontalMirror;
    playbackSpeed.value = settings.playbackSpeed;
    playbackSpeedMode.value = settings.playbackSpeedSettings?.mode ?? "global";
    playbackSpeedMin.value = settings.playbackSpeedSettings?.min ?? settings.playbackSpeed;
    playbackSpeedMax.value = settings.playbackSpeedSettings?.max ?? settings.playbackSpeed;
    playbackSegmentMinSeconds.value = settings.playbackSpeedSettings?.segmentMinSeconds ?? 2;
    playbackSegmentMaxSeconds.value = settings.playbackSpeedSettings?.segmentMaxSeconds ?? 4;
    smoothRemixEnabled.value = settings.smoothRemixEnabled;
    canvasAspectRatio.value = settings.canvasAspectRatio;
    canvasBackgroundMode.value = settings.canvasBackgroundMode;
    subtitleEnabled.value = settings.subtitleSettings.enabled;
    subtitleText.value = settings.subtitleSettings.text;
    subtitlePosition.value = settings.subtitleSettings.position;
    subtitleSize.value = settings.subtitleSettings.size ?? "medium";
    subtitleFontFamily.value = settings.subtitleSettings.fontFamily ?? "Microsoft YaHei";
    subtitleTextColor.value = settings.subtitleSettings.textColor ?? "#ffffff";
    subtitleOpacity.value = settings.subtitleSettings.opacity ?? 1;
    applyVerticalMirror.value = settings.videoEffectSettings.verticalMirror;
    rotationMode.value = settings.videoEffectSettings.rotation;
    hslEnabled.value = settings.videoEffectSettings.hslEnabled ?? false;
    hue.value = settings.videoEffectSettings.hue ?? 0;
    brightness.value = settings.videoEffectSettings.brightness;
    contrast.value = settings.videoEffectSettings.contrast;
    saturation.value = settings.videoEffectSettings.saturation;
    effectScale.value = settings.videoEffectSettings.scale;
    cropSettings.value = settings.videoEffectSettings.crop ?? { enabled: false, x: 0, y: 0, width: 100, height: 100 };
    zoomEnabled.value = settings.videoEffectSettings.zoomEnabled ?? false;
    zoomMode.value = settings.videoEffectSettings.zoomMode ?? "push";
    zoomMinScale.value = settings.videoEffectSettings.zoomMinScale ?? 1.02;
    zoomMaxScale.value = settings.videoEffectSettings.zoomMaxScale ?? 1.08;
    zoomMinDurationSeconds.value = settings.videoEffectSettings.zoomMinDurationSeconds ?? 8;
    zoomMaxDurationSeconds.value = settings.videoEffectSettings.zoomMaxDurationSeconds ?? 10;
    cropBlackBars.value = settings.videoEffectSettings.cropBlackBars ?? false;
    randomRotationMinDegrees.value = settings.videoEffectSettings.randomRotationMinDegrees ?? 0;
    randomRotationMaxDegrees.value = settings.videoEffectSettings.randomRotationMaxDegrees ?? 0;
    sharpness.value = settings.videoEffectSettings.sharpness ?? 0;
    noiseReduction.value = settings.videoEffectSettings.noiseReduction ?? 0;
    temperature.value = settings.videoEffectSettings.temperature ?? 6500;
    visualStyle.value = settings.videoEffectSettings.visualStyle ?? "none";
    glowEnabled.value = settings.videoEffectSettings.glowEnabled ?? false;
    grainEnabled.value = settings.videoEffectSettings.grainEnabled ?? false;
    vignetteEnabled.value = settings.videoEffectSettings.vignetteEnabled ?? false;
    pipEnabled.value = settings.pictureInPictureSettings.enabled;
    pipOverlayFilePath.value = settings.pictureInPictureSettings.overlayFilePath;
    pipMode.value = settings.pictureInPictureSettings.mode ?? "external";
    pipMainSizeMin.value = settings.pictureInPictureSettings.mainSizeMin ?? 0.96;
    pipMainSizeMax.value = settings.pictureInPictureSettings.mainSizeMax ?? 0.99;
    pipBlurMin.value = settings.pictureInPictureSettings.blurMin ?? 40;
    pipBlurMax.value = settings.pictureInPictureSettings.blurMax ?? 50;
    pipOffsetXMin.value = settings.pictureInPictureSettings.offsetXMin ?? 0.5;
    pipOffsetXMax.value = settings.pictureInPictureSettings.offsetXMax ?? 0.5;
    pipOffsetYMin.value = settings.pictureInPictureSettings.offsetYMin ?? 0.5;
    pipOffsetYMax.value = settings.pictureInPictureSettings.offsetYMax ?? 0.5;
    pipPosition.value = settings.pictureInPictureSettings.position;
    pipSizeRatio.value = settings.pictureInPictureSettings.sizeRatio;
    pipOpacity.value = settings.pictureInPictureSettings.opacity;
    pipMargin.value = settings.pictureInPictureSettings.margin;
    bgmEnabled.value = settings.bgmSettings.enabled;
    bgmAudioFilePath.value = settings.bgmSettings.audioFilePath;
    originalVolume.value = settings.bgmSettings.originalVolume;
    originalVolumeMin.value = settings.bgmSettings.originalVolumeMin ?? settings.bgmSettings.originalVolume;
    originalVolumeMax.value = settings.bgmSettings.originalVolumeMax ?? settings.bgmSettings.originalVolume;
    originalFadeEnabled.value = settings.bgmSettings.originalFadeEnabled ?? false;
    dynamicAudioAdjustEnabled.value = settings.bgmSettings.dynamicAdjustEnabled ?? false;
    bgmVolume.value = settings.bgmSettings.bgmVolume;
    bgmVolumeMin.value = settings.bgmSettings.bgmVolumeMin ?? settings.bgmSettings.bgmVolume;
    bgmVolumeMax.value = settings.bgmSettings.bgmVolumeMax ?? settings.bgmSettings.bgmVolume;
    bgmFadeEnabled.value = settings.bgmSettings.bgmFadeEnabled ?? false;
    bgmLoopPlaybackEnabled.value = settings.bgmSettings.loopPlaybackEnabled ?? true;
    bgmFadeInSeconds.value = settings.bgmSettings.fadeInSeconds;
    bgmFadeOutSeconds.value = settings.bgmSettings.fadeOutSeconds;
    watermark.restoreWatermarkSettings(settings.watermarkSettings);
    watermarkRemoval.restoreWatermarkRemovalSettings(settings.watermarkRemovalSettings);
    watermarkRemovalRegionCount.value = settings.watermarkRemovalSettings.regionCount ?? DEFAULT_WATERMARK_REMOVAL_SETTINGS.regionCount;
    watermarkRemovalManualRegions.value = (settings.watermarkRemovalSettings.manualRegions ?? DEFAULT_WATERMARK_REMOVAL_SETTINGS.manualRegions).map((region) => ({ ...region }));
    output.restoreOutputSettings(settings.outputSettings);
    entranceEffect.value = settings.entranceEffect ?? "none";
    frameOperationSettings.value = { ...frameOperationSettings.value, ...(settings.frameOperationSettings ?? {}) };
    fusionSettings.value = { ...fusionSettings.value, ...(settings.fusionSettings ?? {}) };
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
    cropSettings,
    effectScale,
    entranceEffect,
    frameOperationSettings,
    fusionSettings,
    hslEnabled,
    hue,
    watermarkAssetType: watermark.watermarkAssetType,
    watermarkTrajectory: watermark.watermarkTrajectory,
    watermarkRemovalRegionCount,
    watermarkRemovalManualRegions,
    originalVolume,
    originalVolumeMin,
    originalVolumeMax,
    originalFadeEnabled,
    dynamicAudioAdjustEnabled,
    bgmVolumeMin,
    bgmVolumeMax,
    bgmFadeEnabled,
    bgmLoopPlaybackEnabled,
    pictureInPictureSettings,
    pipEnabled,
    pipMode,
    pipMainSizeMin,
    pipMainSizeMax,
    pipBlurMin,
    pipBlurMax,
    pipOffsetXMin,
    pipOffsetXMax,
    pipOffsetYMin,
    pipOffsetYMax,
    pipMargin,
    pipOpacity,
    pipOverlayFilePath,
    pipPosition,
    pipSizeRatio,
    playbackSpeed,
    playbackSpeedMode,
    playbackSpeedMin,
    playbackSpeedMax,
    playbackSegmentMinSeconds,
    playbackSegmentMaxSeconds,
    cropBlackBars,
    randomRotationMinDegrees,
    randomRotationMaxDegrees,
    sharpness,
    noiseReduction,
    temperature,
    visualStyle,
    glowEnabled,
    grainEnabled,
    vignetteEnabled,
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
    subtitleFontFamily,
    subtitleOpacity,
    subtitlePosition,
    subtitleSettings,
    subtitleSize,
    subtitleText,
    subtitleTextColor,
    smoothRemixEnabled,
    videoEffectSettings,
  };
}
