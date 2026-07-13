export interface AiRemixVisualSegmentInput {
  segmentId: string;
  durationSeconds: number;
  thumbnailPath: string;
}

export interface AiRemixSegmentInput {
  segmentId: string;
  durationSeconds: number;
  description: string;
}

export interface AiRemixSegment extends AiRemixVisualSegmentInput {
  path: string;
  thumbnailUrl: string;
  description: string | null;
}

export interface AiRemixSegmentAnalysis {
  segmentId: string;
  description: string;
}

export interface AiRemixSegmentAnalysisResult {
  segments: AiRemixSegmentAnalysis[];
}

export interface AiRemixShotPlan {
  text: string;
  segmentId: string;
  alternativeSegmentIds: string[];
}

export interface AiRemixPlanResult {
  shots: AiRemixShotPlan[];
}

export interface AiRemixPlannedShot {
  shotId: string;
  text: string;
  segment: AiRemixSegment;
  alternativeSegments: AiRemixSegment[];
}
