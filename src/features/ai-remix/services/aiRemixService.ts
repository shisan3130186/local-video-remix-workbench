import { invoke } from "@tauri-apps/api/core";
import type {
  AiRemixPlanResult,
  AiRemixSegmentAnalysisResult,
  AiRemixSegmentInput,
  AiRemixVisualSegmentInput,
} from "../types";

export function analyzeAiRemixSegments(
  segments: AiRemixVisualSegmentInput[],
): Promise<AiRemixSegmentAnalysisResult> {
  return invoke<AiRemixSegmentAnalysisResult>("analyze_ai_remix_segments", {
    segments,
  });
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
