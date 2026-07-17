import { invoke } from "@tauri-apps/api/core";
import type { TaskProgressContext } from "../../task-center";
import type { SceneSensitivity } from "../types";

export interface SplitVideoResult {
  outputDirectory: string;
  segmentPaths: string[];
  segmentCount: number;
  detectedSceneCount: number;
  splitMode: "duration" | "scene";
  message: string;
}

export function listVideoFilesInFolder(folderPath: string): Promise<string[]> {
  return invoke<string[]>("list_video_files_in_folder", { folderPath });
}

export function splitCurrentVideo(
  inputFilePath: string,
  outputDirectory: string,
  segmentDurationSeconds: number,
  inputDurationSeconds: number | null,
  taskContext?: TaskProgressContext,
): Promise<SplitVideoResult> {
  return invoke<SplitVideoResult>("split_current_video", {
    inputFilePath,
    outputDirectory,
    segmentDurationSeconds,
    inputDurationSeconds,
    taskContext: taskContext ?? null,
  });
}

export function splitVideoByScenes(
  inputFilePath: string,
  outputDirectory: string,
  sensitivity: SceneSensitivity,
  minimumSegmentSeconds: number,
  maximumSegmentSeconds: number,
  inputDurationSeconds: number | null,
  taskContext?: TaskProgressContext,
): Promise<SplitVideoResult> {
  return invoke<SplitVideoResult>("split_video_by_scenes", {
    inputFilePath,
    outputDirectory,
    sensitivity,
    minimumSegmentSeconds,
    maximumSegmentSeconds,
    inputDurationSeconds,
    taskContext: taskContext ?? null,
  });
}
