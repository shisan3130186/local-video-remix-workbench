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

export interface VideoMetadata {
  fileName: string;
  filePath: string;
  durationSeconds: number | null;
  width: number | null;
  height: number | null;
  frameRate: number | null;
  hasAudio: boolean;
  fileSizeBytes: number;
}

export interface ImportedVideo extends VideoMetadata {
  id: string;
}
