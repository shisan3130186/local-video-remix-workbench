export interface AsrConfigStatus {
  configured: boolean;
  resourceId: string;
}

export interface AsrUtterance {
  startTimeMs: number;
  endTimeMs: number;
  text: string;
}

export interface AsrRecognitionResult {
  sourcePath: string;
  sourceFileName: string;
  text: string;
  utterances: AsrUtterance[];
  durationSeconds: number | null;
  normalizedAudioBytes: number;
  cacheHit: boolean;
  modelVersion: string;
}
