import type { AiRemixContentAnalysis, AiRemixSegment } from "./types";

interface CachedSegmentAnalysisV1 {
  version: 1;
  description: string | null;
  contentAnalysis: AiRemixContentAnalysis;
}

export function serializeSegmentAnalysisCache(segment: AiRemixSegment) {
  const contentAnalysis = sanitizeAiRemixContentAnalysis(segment.contentAnalysis);
  if (!contentAnalysis) return segment.description?.trim() || null;

  const cache: CachedSegmentAnalysisV1 = {
    version: 1,
    description: segment.description?.trim() || null,
    contentAnalysis,
  };
  return JSON.stringify(cache);
}

export function deserializeSegmentAnalysisCache(value: string | null): {
  description: string | null;
  contentAnalysis: AiRemixContentAnalysis | null;
} {
  const normalized = value?.trim() || null;
  if (!normalized?.startsWith("{")) {
    return { description: normalized, contentAnalysis: null };
  }

  try {
    const parsed = JSON.parse(normalized) as Partial<CachedSegmentAnalysisV1>;
    if (parsed.version !== 1) {
      return { description: normalized, contentAnalysis: null };
    }
    return {
      description:
        typeof parsed.description === "string" && parsed.description.trim()
          ? parsed.description.trim()
          : null,
      contentAnalysis: sanitizeAiRemixContentAnalysis(parsed.contentAnalysis),
    };
  } catch {
    return { description: normalized, contentAnalysis: null };
  }
}

export function sanitizeAiRemixContentAnalysis(
  value: AiRemixContentAnalysis | null | undefined,
): AiRemixContentAnalysis | null {
  if (!value || typeof value !== "object") return null;
  const theme = normalizeText(value.theme, 80);
  const action = normalizeText(value.action, 100);
  const tags = normalizeList(value.tags, 5, 24);
  if (!theme || !action || tags.length === 0) return null;

  return {
    theme,
    sellingPoints: normalizeList(value.sellingPoints, 3, 60),
    action,
    tags,
  };
}

function normalizeList(values: unknown, maximumItems: number, maximumLength: number) {
  if (!Array.isArray(values)) return [];
  return Array.from(
    new Set(
      values
        .filter((value): value is string => typeof value === "string")
        .map((value) => normalizeText(value, maximumLength))
        .filter((value): value is string => Boolean(value)),
    ),
  ).slice(0, maximumItems);
}

function normalizeText(value: unknown, maximumLength: number) {
  if (typeof value !== "string") return null;
  const normalized = value.trim().replace(/\s+/g, " ");
  return normalized ? Array.from(normalized).slice(0, maximumLength).join("") : null;
}
