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

export type CanvasAspectRatio =
  | "original"
  | "portrait916"
  | "square11"
  | "landscape169";

export type CanvasBackgroundMode = "black" | "blur";

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

export function concatSelectedSegments(
  segmentPaths: string[],
  outputDirectory: string,
  applyHorizontalMirror: boolean,
  playbackSpeed: number,
  canvasAspectRatio: CanvasAspectRatio,
  canvasBackgroundMode: CanvasBackgroundMode,
  smoothRemixEnabled: boolean,
): Promise<MixVideoResult> {
  return invoke<MixVideoResult>("concat_selected_segments", {
    segmentPaths,
    outputDirectory,
    applyHorizontalMirror,
    playbackSpeed,
    canvasAspectRatio,
    canvasBackgroundMode,
    smoothRemixEnabled,
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
