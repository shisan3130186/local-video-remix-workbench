import { invoke } from "@tauri-apps/api/core";

export interface SplitVideoResult {
  outputDirectory: string;
  segmentPaths: string[];
  segmentCount: number;
  message: string;
}

export function listVideoFilesInFolder(folderPath: string): Promise<string[]> {
  return invoke<string[]>("list_video_files_in_folder", { folderPath });
}

export function splitCurrentVideo(
  inputFilePath: string,
  outputDirectory: string,
  segmentDurationSeconds: number,
): Promise<SplitVideoResult> {
  return invoke<SplitVideoResult>("split_current_video", {
    inputFilePath,
    outputDirectory,
    segmentDurationSeconds,
  });
}
