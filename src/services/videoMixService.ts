import { invoke } from "@tauri-apps/api/core";

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
  | "ending";

export interface SegmentCategoryOption {
  key: SegmentCategory;
  label: string;
}

export interface CategorizedPickedSegment {
  category: SegmentCategory;
  label: string;
  path: string;
}

export interface CategorizedPickResult {
  pickedSegments: CategorizedPickedSegment[];
  skippedCategories: string[];
}

export function pickRandomSegments(
  segmentPaths: string[],
  pickCount: number,
): string[] {
  if (!Number.isInteger(pickCount) || pickCount <= 0) {
    throw new Error("随机抽取数量必须大于 0。");
  }

  if (segmentPaths.length === 0) {
    throw new Error("请先完成一次视频切片。");
  }

  if (pickCount > segmentPaths.length) {
    throw new Error(`抽取数量不能超过当前片段数量：${segmentPaths.length}。`);
  }

  return shuffleSegments(segmentPaths).slice(0, pickCount);
}

export function pickCategorizedSegments(
  segmentPaths: string[],
  segmentCategories: Record<string, SegmentCategory | "">,
  categoryOptions: SegmentCategoryOption[],
): CategorizedPickResult {
  if (segmentPaths.length === 0) {
    throw new Error("请先完成一次视频切片。");
  }

  const pickedSegments: CategorizedPickedSegment[] = [];
  const skippedCategories: string[] = [];

  for (const categoryOption of categoryOptions) {
    const candidates = segmentPaths.filter(
      (segmentPath) => segmentCategories[segmentPath] === categoryOption.key,
    );

    if (candidates.length === 0) {
      skippedCategories.push(categoryOption.label);
      continue;
    }

    const [pickedPath] = shuffleSegments(candidates);
    pickedSegments.push({
      category: categoryOption.key,
      label: categoryOption.label,
      path: pickedPath,
    });
  }

  if (pickedSegments.length < 2) {
    throw new Error("至少需要 2 个分类各有 1 个片段，才能生成分类混剪。");
  }

  return {
    pickedSegments,
    skippedCategories,
  };
}

export function concatSelectedSegments(
  segmentPaths: string[],
  outputDirectory: string,
  applyHorizontalMirror: boolean,
  playbackSpeed: number,
  canvasAspectRatio: CanvasAspectRatio,
  canvasBackgroundMode: CanvasBackgroundMode,
  smoothRemixEnabled: boolean,
  videoEffectSettings: VideoEffectSettings,
  pictureInPictureSettings: PictureInPictureSettings,
  bgmSettings: BgmSettings,
): Promise<MixVideoResult> {
  return invoke<MixVideoResult>("concat_selected_segments", {
    segmentPaths,
    outputDirectory,
    applyHorizontalMirror,
    playbackSpeed,
    canvasAspectRatio,
    canvasBackgroundMode,
    smoothRemixEnabled,
    videoEffectSettings,
    pictureInPictureSettings,
    bgmSettings,
  });
}

function shuffleSegments(segmentPaths: string[]) {
  const shuffled = [...segmentPaths];

  for (let index = shuffled.length - 1; index > 0; index -= 1) {
    const randomIndex = Math.floor(Math.random() * (index + 1));
    [shuffled[index], shuffled[randomIndex]] = [shuffled[randomIndex], shuffled[index]];
  }

  return shuffled;
}
