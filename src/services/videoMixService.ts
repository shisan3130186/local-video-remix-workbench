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

export interface VideoEffectSettings {
  verticalMirror: boolean;
  rotation: RotationMode;
  brightness: number;
  contrast: number;
  saturation: number;
  scale: number;
}

export type PipPosition = "topLeft" | "topRight" | "bottomLeft" | "bottomRight" | "center";

export interface PictureInPictureSettings {
  enabled: boolean;
  overlayFilePath: string | null;
  position: PipPosition;
  sizeRatio: number;
  opacity: number;
  margin: number;
}

export interface BgmSettings {
  enabled: boolean;
  audioFilePath: string | null;
  originalVolume: number;
  bgmVolume: number;
  fadeInSeconds: number;
  fadeOutSeconds: number;
}

export type SubtitlePosition = "top" | "middle" | "bottom";

export interface SubtitleSettings {
  enabled: boolean;
  text: string;
  position: SubtitlePosition;
  fontSize: number;
  textColor: string;
  backgroundEnabled: boolean;
}

export interface RemixExportSettings {
  applyHorizontalMirror: boolean;
  playbackSpeed: number;
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
): Promise<MixVideoResult> {
  return invoke<MixVideoResult>("concat_selected_segments", {
    segmentPaths,
    outputDirectory,
    settings,
    taskContext: taskContext ?? null,
  });
}
