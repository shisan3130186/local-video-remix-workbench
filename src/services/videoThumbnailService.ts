import { invoke } from "@tauri-apps/api/core";

export interface VideoThumbnailResult {
  thumbnailPath: string;
  sourcePath: string;
  timeSeconds: number;
  message: string;
}

export function generateThumbnail(
  inputFilePath: string,
  outputDirectory: string | null,
  timeSeconds: number,
  label: string,
): Promise<VideoThumbnailResult> {
  return invoke<VideoThumbnailResult>("generate_thumbnail", {
    inputFilePath,
    outputDirectory,
    timeSeconds,
    label,
  });
}
