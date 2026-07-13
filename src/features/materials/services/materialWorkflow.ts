import { readVideoMetadata } from "../../../services/videoProbeService";
import type { ImportedVideo } from "../../../types/videoProbe";
import { splitCurrentVideo } from "./materialService";

interface SplitImportedVideosOptions {
  videos: ImportedVideo[];
  outputDirectory: string;
  segmentDurationSeconds: number;
  appendLog: (message: string, level: "info" | "success" | "error") => void;
}

export interface SplitImportedVideosResult {
  segmentPaths: string[];
  failedVideoNames: string[];
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
  segmentDurationSeconds,
  appendLog,
}: SplitImportedVideosOptions): Promise<SplitImportedVideosResult> {
  const segmentPaths: string[] = [];
  const failedVideoNames: string[] = [];

  for (const [index, video] of videos.entries()) {
    appendLog(`正在切片 ${index + 1}/${videos.length}：${video.fileName}`, "info");

    try {
      const result = await splitCurrentVideo(
        video.filePath,
        outputDirectory,
        segmentDurationSeconds,
      );
      segmentPaths.push(...result.segmentPaths);
      appendLog(`${video.fileName} 切片完成：${result.segmentCount} 个片段。`, "success");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error ?? "视频切片失败");
      failedVideoNames.push(video.fileName);
      appendLog(`${video.fileName} 切片失败：${message}`, "error");
    }
  }

  return { segmentPaths, failedVideoNames };
}
