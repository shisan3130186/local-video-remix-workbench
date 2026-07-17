import { readVideoMetadata } from "../../../services/videoProbeService";
import type { ImportedVideo } from "../../../types/videoProbe";
import type { TaskRunHandle } from "../../task-center";
import type { SmartSplitSettings, SplitMode } from "../types";
import { splitCurrentVideo, splitVideoByScenes } from "./materialService";

interface SplitImportedVideosOptions {
  videos: ImportedVideo[];
  outputDirectory: string;
  splitMode: SplitMode;
  segmentDurationSeconds: number;
  smartSplitSettings: SmartSplitSettings;
  appendLog: (message: string, level: "info" | "success" | "error") => void;
  task: TaskRunHandle;
}

export interface SplitImportedVideosResult {
  segmentPaths: string[];
  failedVideoNames: string[];
  detectedSceneCount: number;
}

export async function loadImportedVideos(filePaths: string[]): Promise<ImportedVideo[]> {
  return Promise.all(
    filePaths.map(async (filePath) => {
      const metadata = await readVideoMetadata(filePath);

      return {
        ...metadata,
        id: `${metadata.filePath}-${metadata.fileSizeBytes}`,
      };
    }),
  );
}

export async function splitImportedVideos({
  videos,
  outputDirectory,
  splitMode,
  segmentDurationSeconds,
  smartSplitSettings,
  appendLog,
  task,
}: SplitImportedVideosOptions): Promise<SplitImportedVideosResult> {
  const segmentPaths: string[] = [];
  const failedVideoNames: string[] = [];
  let detectedSceneCount = 0;

  for (const [index, video] of videos.entries()) {
    task.throwIfCancelled();
    appendLog(`正在切片 ${index + 1}/${videos.length}：${video.fileName}`, "info");

    try {
      const progress = task.progress(
        (index / videos.length) * 70,
        ((index + 1) / videos.length) * 70,
        `${splitMode === "scene" ? "正在智能分析" : "正在切片"} ${index + 1}/${videos.length}：${video.fileName}`,
      );
      const result =
        splitMode === "scene"
          ? await splitVideoByScenes(
              video.filePath,
              outputDirectory,
              smartSplitSettings.sensitivity,
              smartSplitSettings.minimumSegmentSeconds,
              smartSplitSettings.maximumSegmentSeconds,
              video.durationSeconds,
              progress,
            )
          : await splitCurrentVideo(
              video.filePath,
              outputDirectory,
              segmentDurationSeconds,
              video.durationSeconds,
              progress,
            );
      segmentPaths.push(...result.segmentPaths);
      detectedSceneCount += result.detectedSceneCount;
      appendLog(
        splitMode === "scene"
          ? `${video.fileName} 智能切片完成：${result.segmentCount} 个片段，${result.message}`
          : `${video.fileName} 切片完成：${result.segmentCount} 个片段。`,
        "success",
      );
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error ?? "视频切片失败");
      failedVideoNames.push(video.fileName);
      appendLog(`${video.fileName} 切片失败：${message}`, "error");
    }
  }

  return { segmentPaths, failedVideoNames, detectedSceneCount };
}
