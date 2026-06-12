import { invoke } from "@tauri-apps/api/core";
import type {
  CanvasAspectRatio,
  CanvasBackgroundMode,
} from "./videoMixService";

export interface RenderVideoResult {
  outputPath: string;
  message: string;
}

export function exportCurrentVideo(
  inputFilePath: string,
  outputDirectory: string,
  canvasAspectRatio: CanvasAspectRatio,
  canvasBackgroundMode: CanvasBackgroundMode,
): Promise<RenderVideoResult> {
  return invoke<RenderVideoResult>("export_current_video", {
    inputFilePath,
    outputDirectory,
    canvasAspectRatio,
    canvasBackgroundMode,
  });
}
