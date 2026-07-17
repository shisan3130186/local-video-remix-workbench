export interface TtsConfigStatus {
  configured: boolean;
  resourceId: string;
  speaker: string;
}

export interface TtsWordTiming {
  confidence: number;
  startTime: number;
  endTime: number;
  word: string;
}

export interface TtsSynthesisResult {
  outputPath: string;
  textLength: number;
  audioByteCount: number;
  speaker: string;
  resourceId: string;
  words: TtsWordTiming[];
}

export interface NarratedShotSource {
  text: string;
  segmentPath: string;
  segmentDurationSeconds: number;
  alternativeSegments: NarratedVideoCandidate[];
}

export interface NarratedVideoCandidate {
  videoPath: string;
  durationSeconds: number;
}

export interface NarratedSegmentInput {
  videoPath: string;
  videoDurationSeconds: number;
  narrationPath: string;
  subtitleText: string;
  alternativeVideos: NarratedVideoCandidate[];
}

export interface NarratedVideoFailure {
  version: number;
  message: string;
}

export interface NarratedAudioSettings {
  keepOriginalAudio: boolean;
  originalAudioVolume: number;
}

export type NarratedSubtitlePosition = "top" | "middle" | "bottom";
export type NarratedSubtitleSize = "small" | "medium" | "large";

export interface NarratedSubtitleSettings {
  enabled: boolean;
  position: NarratedSubtitlePosition;
  size: NarratedSubtitleSize;
}
