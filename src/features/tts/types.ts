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
}

export interface NarratedSegmentInput {
  videoPath: string;
  narrationPath: string;
}

export interface NarratedAudioSettings {
  keepOriginalAudio: boolean;
  originalAudioVolume: number;
}
