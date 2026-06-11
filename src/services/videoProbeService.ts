import { invoke } from "@tauri-apps/api/core";
import type { FfmpegEnvironmentResult } from "../types/videoProbe";

export function checkFfmpegEnvironment(): Promise<FfmpegEnvironmentResult> {
  return invoke<FfmpegEnvironmentResult>("check_ffmpeg_environment");
}
