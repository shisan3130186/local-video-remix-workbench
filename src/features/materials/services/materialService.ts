import { invoke } from "@tauri-apps/api/core";
import type { TaskProgressContext } from "../../task-center";
import type { SceneSensitivity, SplitOutputGrouping } from "../types";

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
  trimStartSeconds: number,
  trimEndSeconds: number,
  outputGrouping: SplitOutputGrouping,
  taskContext?: TaskProgressContext,
): Promise<SplitVideoResult> {
  return invoke<SplitVideoResult>("split_current_video", {
    inputFilePath,
    outputDirectory,
    inputDurationSeconds,
    settings: {
      segmentDurationSeconds,
      trim: { startSeconds: trimStartSeconds, endSeconds: trimEndSeconds },
      outputGrouping,
    },
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
  trimStartSeconds: number,
  trimEndSeconds: number,
  outputGrouping: SplitOutputGrouping,
  taskContext?: TaskProgressContext,
): Promise<SplitVideoResult> {
  return invoke<SplitVideoResult>("split_video_by_scenes", {
    inputFilePath,
    outputDirectory,
    inputDurationSeconds,
    settings: {
      sensitivity,
      minimumSegmentSeconds,
      maximumSegmentSeconds,
      trim: { startSeconds: trimStartSeconds, endSeconds: trimEndSeconds },
      outputGrouping,
    },
    taskContext: taskContext ?? null,
  });
}
