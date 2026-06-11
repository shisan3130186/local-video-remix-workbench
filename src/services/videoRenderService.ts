import { invoke } from "@tauri-apps/api/core";

export interface RenderVideoResult {
  outputPath: string;
  message: string;
}

export function exportCurrentVideo(
  inputFilePath: string,
  outputDirectory: string,
): Promise<RenderVideoResult> {
  return invoke<RenderVideoResult>("export_current_video", {
    inputFilePath,
    outputDirectory,
  });
}
