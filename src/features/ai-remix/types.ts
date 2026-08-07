export interface AiRemixVisualSegmentInput {
  segmentId: string;
  durationSeconds: number;
  thumbnailPaths: string[];
}

export type AiRemixMatchMode = "local" | "cloud";

export interface AiRemixSegmentInput {
  segmentId: string;
  durationSeconds: number;
  description: string;
}

export interface AiRemixSegment {
  segmentId: string;
  path: string;
  durationSeconds: number;
  thumbnailPath: string;
  thumbnailUrl: string;
  analysisThumbnailPaths?: string[];
  description: string | null;
  contentAnalysis: AiRemixContentAnalysis | null;
}

export interface AiRemixContentAnalysis {
  theme: string;
  sellingPoints: string[];
  action: string;
  tags: string[];
}

export interface AiRemixSegmentAnalysis {
  segmentId: string;
  description: string;
}

export interface AiRemixSegmentAnalysisResult {
  segments: AiRemixSegmentAnalysis[];
}

export interface AiRemixSegmentContentAnalysis extends AiRemixContentAnalysis {
  segmentId: string;
  category: import("../../services/videoMixService").SegmentCategory;
}

export interface AiRemixSegmentContentAnalysisResult {
  segments: AiRemixSegmentContentAnalysis[];
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

export interface AiRemixVariantShotInput {
  segmentId: string;
  alternativeSegmentIds: string[];
}

export interface AiRemixVariantPlan {
  segmentIds: string[];
}

export interface AiRemixVariantPlanResult {
  variants: AiRemixVariantPlan[];
}

export interface AiRemixVariant {
  version: number;
  shots: AiRemixPlannedShot[];
}

export interface AiRemixGenerationFailure {
  version: number;
  message: string;
}
