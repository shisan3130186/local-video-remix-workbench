import { invoke } from "@tauri-apps/api/core";

export interface AiRemixSegmentInput {
  segmentId: string;
  durationSeconds: number;
  thumbnailPath: string;
}

export interface AiRemixSegment extends AiRemixSegmentInput {
  path: string;
  thumbnailUrl: string;
}

export interface AiRemixPlanResult {
  orderedSegmentIds: string[];
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
