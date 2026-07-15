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
