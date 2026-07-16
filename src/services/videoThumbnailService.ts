import { invoke } from "@tauri-apps/api/core";

export interface VideoThumbnailResult {
  thumbnailPath: string;
  sourcePath: string;
  timeSeconds: number;
  message: string;
}

export type ThumbnailFitMode = "source" | "contain";

export function generateThumbnail(
  inputFilePath: string,
  outputDirectory: string | null,
  timeSeconds: number,
  label: string,
  fitMode: ThumbnailFitMode = "source",
): Promise<VideoThumbnailResult> {
  return invoke<VideoThumbnailResult>("generate_thumbnail", {
    inputFilePath,
    outputDirectory,
    timeSeconds,
    label,
    fitMode,
  });
}
