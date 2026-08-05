import { invoke } from "@tauri-apps/api/core";
import type { ImageVideoAspect, ImageVideoResult } from "../types";

export function convertImagesToVideos(input: {
  imagePaths: string[];
  outputDirectory: string;
  durationSeconds: number;
  aspectRatio: ImageVideoAspect;
}): Promise<ImageVideoResult> {
  return invoke<ImageVideoResult>("convert_images_to_videos", {
    ...input,
    outputSettings: {
      format: "mp4",
      resolution: "followCanvas",
      frameRate: "fps30",
      quality: "standard",
      encoder: "auto",
    },
    taskContext: null,
  });
}
