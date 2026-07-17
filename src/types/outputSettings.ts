export type OutputFormat = "mp4";
export type OutputResolution = "followCanvas" | "hd720" | "fullHd1080";
export type OutputFrameRate = "source" | "fps24" | "fps25" | "fps30" | "fps50" | "fps60";
export type OutputQuality = "compact" | "standard" | "high";
export type VideoEncoder = "auto" | "cpu" | "nvidia" | "intel" | "amd";

export interface OutputSettings {
  format: OutputFormat;
  resolution: OutputResolution;
  frameRate: OutputFrameRate;
  quality: OutputQuality;
  encoder: VideoEncoder;
}

export interface EncoderCapability {
  encoder: Exclude<VideoEncoder, "auto">;
  label: string;
  available: boolean;
  detail: string;
}

export interface EncoderCapabilities {
  encoders: EncoderCapability[];
  recommendedEncoder: Exclude<VideoEncoder, "auto">;
  message: string;
}

export const DEFAULT_OUTPUT_SETTINGS: OutputSettings = {
  format: "mp4",
  resolution: "followCanvas",
  frameRate: "source",
  quality: "standard",
  encoder: "auto",
};
