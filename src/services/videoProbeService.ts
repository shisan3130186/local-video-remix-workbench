import { invoke } from "@tauri-apps/api/core";
import type { FfmpegEnvironmentResult, VideoMetadata } from "../types/videoProbe";

export function checkFfmpegEnvironment(): Promise<FfmpegEnvironmentResult> {
  return invoke<FfmpegEnvironmentResult>("check_ffmpeg_environment");
}

export function readVideoMetadata(filePath: string): Promise<VideoMetadata> {
  return invoke<VideoMetadata>("read_video_metadata", { filePath });
}
