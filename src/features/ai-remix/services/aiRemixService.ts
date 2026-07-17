import { invoke } from "@tauri-apps/api/core";
import type {
  AiRemixPlanResult,
  AiRemixSegmentAnalysisResult,
  AiRemixSegmentContentAnalysisResult,
  AiRemixSegmentInput,
  AiRemixVariantPlanResult,
  AiRemixVariantShotInput,
  AiRemixVisualSegmentInput,
} from "../types";

export function analyzeAiRemixSegments(
  segments: AiRemixVisualSegmentInput[],
): Promise<AiRemixSegmentAnalysisResult> {
  return invoke<AiRemixSegmentAnalysisResult>("analyze_ai_remix_segments", {
    segments,
  });
}

export function extractAiRemixSegmentContent(
  segments: AiRemixSegmentInput[],
): Promise<AiRemixSegmentContentAnalysisResult> {
  return invoke<AiRemixSegmentContentAnalysisResult>(
    "extract_ai_remix_segment_content",
    { segments },
  );
}

export function planAiRemix(
  script: string,
  segments: AiRemixSegmentInput[],
): Promise<AiRemixPlanResult> {
  return invoke<AiRemixPlanResult>("plan_ai_remix", {
    script,
    segments,
  });
}

export function buildAiRemixVariants(
  shots: AiRemixVariantShotInput[],
  requestedCount: number,
): Promise<AiRemixVariantPlanResult> {
  return invoke<AiRemixVariantPlanResult>("build_ai_remix_variants", {
    shots,
    requestedCount,
  });
}
