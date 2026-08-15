import { invoke } from "@tauri-apps/api/core";
import type { TaskProgressContext } from "../features/task-center";
import type { OutputSettings } from "../types/outputSettings";
import type { WatermarkRemovalSettings, WatermarkSettings } from "../features/watermark";

export interface MixVideoResult {
  outputPath: string;
  inputCount: number;
  message: string;
  outputAspectRatio: string;
  outputResolution: string;
  backgroundMode: string;
  appliedToRemixExport: boolean;
  smoothRemixEnabled: boolean;
  skippedShortSegmentCount: number;
  outputEncoder: string;
  outputFrameRate: string;
  outputQuality: string;
  outputVideoBitrateKbps: number;
}

export type RotationMode = "none" | "clockwise90" | "counterclockwise90" | "rotate180";
export type DynamicZoomMode = "push" | "pull" | "random";

export interface CanvasCropSettings {
  enabled: boolean;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface VideoEffectSettings {
  verticalMirror: boolean;
  rotation: RotationMode;
  hslEnabled: boolean;
  hue: number;
  brightness: number;
  contrast: number;
  saturation: number;
  scale: number;
  crop: CanvasCropSettings;
  zoomEnabled: boolean;
  zoomMode: DynamicZoomMode;
  zoomMinScale: number;
  zoomMaxScale: number;
  zoomMinDurationSeconds: number;
  zoomMaxDurationSeconds: number;
  cropBlackBars: boolean;
  randomRotationMinDegrees: number;
  randomRotationMaxDegrees: number;
  sharpness: number;
  noiseReduction: number;
  temperature: number;
  visualStyle: VideoVisualStyle;
  glowEnabled: boolean;
  grainEnabled: boolean;
  vignetteEnabled: boolean;
}

export type VideoVisualStyle = "none" | "random" | "bw" | "invert" | "retro" | "cross" | "cartoon" | "emboss" | "pixel" | "outline";
export type PlaybackSpeedMode = "global" | "segment";

export interface PlaybackSpeedSettings {
  mode: PlaybackSpeedMode;
  min: number;
  max: number;
  segmentMinSeconds: number;
  segmentMaxSeconds: number;
}

export type PipPosition = "topLeft" | "topRight" | "bottomLeft" | "bottomRight" | "center";

export interface PictureInPictureSettings {
  enabled: boolean;
  overlayFilePath: string | null;
  mode: "main" | "external";
  mainSizeMin: number;
  mainSizeMax: number;
  blurMin: number;
  blurMax: number;
  offsetXMin: number;
  offsetXMax: number;
  offsetYMin: number;
  offsetYMax: number;
  position: PipPosition;
  sizeRatio: number;
  opacity: number;
  margin: number;
}

export interface BgmSettings {
  enabled: boolean;
  audioFilePath: string | null;
  originalVolume: number;
  originalVolumeMin: number;
  originalVolumeMax: number;
  originalFadeEnabled: boolean;
  dynamicAdjustEnabled: boolean;
  bgmVolume: number;
  bgmVolumeMin: number;
  bgmVolumeMax: number;
  bgmFadeEnabled: boolean;
  loopPlaybackEnabled: boolean;
  fadeInSeconds: number;
  fadeOutSeconds: number;
}

export type SubtitlePosition = "top" | "middle" | "bottom";
export type SubtitleSize = "small" | "medium" | "large";

export interface SubtitleSettings {
  enabled: boolean;
  text: string;
  position: SubtitlePosition;
  size: SubtitleSize;
  fontSize: number;
  fontFamily: string;
  textColor: string;
  opacity: number;
  backgroundEnabled: boolean;
}

export type EntranceEffect = "none" | "smooth-up" | "smooth-down" | "horizontal-squeeze" | "vertical-squeeze" | "circle-crop" | "rectangle-crop" | "circle-close" | "circle-open" | "horizontal-close" | "horizontal-open" | "vertical-close" | "vertical-open" | "left-bottom" | "right-bottom" | "left-top" | "right-top" | "horizontal-slice" | "vertical-slice";
export type FrameOperationMode = "extract" | "insert" | "mixed";
export interface FrameOperationSettings {
  enabled: boolean;
  mode: FrameOperationMode;
  intervalMin: number;
  intervalMax: number;
  frameMin: number;
  frameMax: number;
  opacity: number;
  materialFilePath: string | null;
}
export interface FusionSettings {
  enabled: boolean;
  materialFilePath: string | null;
  intervalMin: number;
  intervalMax: number;
  strength: number;
}

export interface RemixExportSettings {
  applyHorizontalMirror: boolean;
  playbackSpeed: number;
  playbackSpeedSettings: PlaybackSpeedSettings;
  canvasAspectRatio: CanvasAspectRatio;
  canvasBackgroundMode: CanvasBackgroundMode;
  smoothRemixEnabled: boolean;
  videoEffectSettings: VideoEffectSettings;
  pictureInPictureSettings: PictureInPictureSettings;
  bgmSettings: BgmSettings;
  watermarkSettings: WatermarkSettings;
  watermarkRemovalSettings: WatermarkRemovalSettings;
  subtitleSettings: SubtitleSettings;
  outputSettings: OutputSettings;
  outputName: string | null;
  entranceEffect: EntranceEffect;
  frameOperationSettings: FrameOperationSettings;
  fusionSettings: FusionSettings;
}

/** A source video, optionally restricted to a real time range before it enters the mix. */
export interface RemixSegmentInput {
  path: string;
  startSeconds?: number;
  durationSeconds?: number;
}

export type CanvasAspectRatio =
  | "original"
  | "portrait916"
  | "square11"
  | "landscape169";

export type CanvasBackgroundMode = "black" | "blur";

export type SegmentCategory =
  | "hook"
  | "product"
  | "usage"
  | "detail"
  | "result"
  | "ending"
  | "talking"
  | "environment";

export interface SegmentCategoryOption {
  key: SegmentCategory;
  label: string;
}

export function concatSelectedSegments(
  segmentPaths: string[],
  outputDirectory: string,
  settings: RemixExportSettings,
  taskContext?: TaskProgressContext,
  segmentInputs?: RemixSegmentInput[],
): Promise<MixVideoResult> {
  return invoke<MixVideoResult>("concat_selected_segments", {
    segmentPaths,
    segmentInputs: segmentInputs ?? null,
    outputDirectory,
    settings,
    taskContext: taskContext ?? null,
  });
}
