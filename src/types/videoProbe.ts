export interface ToolProbeResult {
  available: boolean;
  version: string | null;
  error: string | null;
}

export interface FfmpegEnvironmentResult {
  ffmpeg: ToolProbeResult;
  ffprobe: ToolProbeResult;
  available: boolean;
  message: string;
}
