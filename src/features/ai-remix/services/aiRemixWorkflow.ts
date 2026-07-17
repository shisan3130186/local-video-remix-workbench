import { convertFileSrc } from "@tauri-apps/api/core";
import { readVideoMetadata } from "../../../services/videoProbeService";
import { generateThumbnail } from "../../../services/videoThumbnailService";
import { runRetryableRequest } from "../../../services/retryableRequest";
import type { TaskLogLevel } from "../../../types/workbench";
import { analyzeAiRemixSegments, buildAiRemixVariants } from "./aiRemixService";
import type {
  AiRemixPlanResult,
  AiRemixPlannedShot,
  AiRemixSegment,
  AiRemixVariant,
} from "../types";

interface PrepareAiRemixSegmentsOptions {
  segmentPaths: string[];
  outputDirectory: string;
  onSegmentError: (message: string) => void;
}

export interface PrepareAiRemixSegmentsResult {
  preparedSegments: AiRemixSegment[];
  thumbnailEntries: Record<string, string>;
  preparationErrors: string[];
}

interface AnalyzeAiRemixDescriptionsOptions {
  appendLog: (message: string, level: TaskLogLevel) => void;
  updateProgress: (message: string) => void;
}

export async function prepareAiRemixSegments({
  segmentPaths,
  outputDirectory,
  onSegmentError,
}: PrepareAiRemixSegmentsOptions): Promise<PrepareAiRemixSegmentsResult> {
  const thumbnailEntries: Record<string, string> = {};
  const preparedSegments: AiRemixSegment[] = [];
  const preparationErrors: string[] = [];

  for (const [index, segmentPath] of segmentPaths.entries()) {
    try {
      const [thumbnailResult, metadata] = await Promise.all([
        generateThumbnail(segmentPath, outputDirectory, 0.1, `segment_${index + 1}`),
        readVideoMetadata(segmentPath),
      ]);
      const durationSeconds = metadata.durationSeconds;

      if (durationSeconds === null || !Number.isFinite(durationSeconds) || durationSeconds <= 0) {
        throw new Error("无法读取有效片段时长。");
      }

      thumbnailEntries[segmentPath] = thumbnailResult.thumbnailPath;
      preparedSegments.push({
        segmentId: `segment-${String(index + 1).padStart(3, "0")}`,
        path: segmentPath,
        durationSeconds,
        thumbnailPath: thumbnailResult.thumbnailPath,
        thumbnailUrl: convertFileSrc(thumbnailResult.thumbnailPath),
        description: null,
      });
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error ?? "未知错误");
      preparationErrors.push(`${formatFileName(segmentPath)}：${errorMessage}`);
      onSegmentError(
        `片段 AI 信息准备失败：${formatFileName(segmentPath)}，${errorMessage}`,
      );
    }
  }

  return { preparedSegments, thumbnailEntries, preparationErrors };
}

export async function analyzeAiRemixSegmentDescriptions(
  segments: AiRemixSegment[],
  { appendLog, updateProgress }: AnalyzeAiRemixDescriptionsOptions,
) {
  const pendingSegments = segments.filter((segment) => !segment.description?.trim());

  if (pendingSegments.length === 0) {
    appendLog("复用当前切片已缓存的画面理解结果。", "info");
    return segments;
  }

  const batches = pendingSegments.map((segment) => [segment]);
  let completedCount = segments.length - pendingSegments.length;
  let updatedSegments = segments;
  const failedAnalyses: string[] = [];
  updateProgress(`正在理解片段画面 ${completedCount}/${segments.length}`);
  appendLog(
    `开始分批理解 ${pendingSegments.length} 个片段画面：每次只发送 1 张预览图，最多同时处理 2 张。`,
    "info",
  );

  for (let waveIndex = 0; waveIndex < batches.length; waveIndex += 2) {
    const wave = batches.slice(waveIndex, waveIndex + 2);
    const waveSegmentIds = wave.flatMap((batch) => batch.map((segment) => segment.segmentId));
    appendLog(
      `正在理解 ${waveSegmentIds.join("、")}（第 ${waveIndex + 1}-${waveIndex + wave.length}/${batches.length} 批）。`,
      "info",
    );
    const results = await Promise.allSettled(
      wave.map((batch) => {
        const batchLabel = batch.map((segment) => segment.segmentId).join("、");
        return runRetryableRequest(
          () =>
            analyzeAiRemixSegments(
              batch.map((segment) => ({
                segmentId: segment.segmentId,
                durationSeconds: segment.durationSeconds,
                thumbnailPath: segment.thumbnailPath,
              })),
            ),
          {
            onRetry({ nextAttempt, maxAttempts, delayMs, message }) {
              updateProgress(`正在重试片段画面理解 ${nextAttempt}/${maxAttempts}`);
              appendLog(
                `${batchLabel} 画面理解遇到临时故障，${Math.ceil(delayMs / 1000)} 秒后进行第 ${nextAttempt}/${maxAttempts} 次尝试：${message}`,
                "info",
              );
            },
          },
        );
      }),
    );
    const descriptions = new Map<string, string>();
    const errors: string[] = [];

    results.forEach((result, resultIndex) => {
      const batch = wave[resultIndex];

      if (result.status === "fulfilled") {
        result.value.segments.forEach((analysis) => {
          descriptions.set(analysis.segmentId, analysis.description.trim());
        });
        completedCount += batch.length;
      } else {
        const message =
          result.reason instanceof Error
            ? result.reason.message
            : String(result.reason ?? "片段画面理解失败");
        errors.push(`${batch.map((segment) => segment.segmentId).join("、")}：${message}`);
      }
    });

    if (descriptions.size > 0) {
      updatedSegments = updatedSegments.map((segment) => ({
        ...segment,
        description: descriptions.get(segment.segmentId) ?? segment.description,
      }));
    }

    updateProgress(`正在理解片段画面 ${completedCount}/${segments.length}`);
    appendLog(
      `片段画面理解进度：${completedCount}/${segments.length}。`,
      errors.length > 0 ? "error" : "success",
    );

    if (errors.length > 0) {
      failedAnalyses.push(...errors);
    }
  }

  if (failedAnalyses.length > 0) {
    appendLog(
      `本轮有 ${failedAnalyses.length} 个片段在自动重试后仍未完成，已保留其他成功结果；再次生成分镜时只会补失败片段。`,
      "error",
    );
  }

  return updatedSegments;
}

export function createAiRemixPlannedShots(
  result: AiRemixPlanResult,
  segments: AiRemixSegment[],
): AiRemixPlannedShot[] {
  const segmentMap = new Map(segments.map((segment) => [segment.segmentId, segment]));
  const primarySegmentIds = new Set(result.shots.map((shot) => shot.segmentId));
  const planTimestamp = Date.now();

  return result.shots.map((shot, index) => {
    const segment = segmentMap.get(shot.segmentId);
    const alternativeSegments = shot.alternativeSegmentIds
      .filter((segmentId) => !primarySegmentIds.has(segmentId))
      .map((segmentId) => segmentMap.get(segmentId));

    if (!segment || alternativeSegments.some((alternative) => !alternative)) {
      throw new Error("AI 分镜结果包含无法识别的片段编号。");
    }

    return {
      shotId: `shot-${planTimestamp}-${index + 1}`,
      text: shot.text.trim(),
      segment,
      alternativeSegments: alternativeSegments as AiRemixSegment[],
    };
  });
}

export async function createAiRemixVariants(
  shots: AiRemixPlannedShot[],
  requestedCount: number,
): Promise<AiRemixVariant[]> {
  const result = await buildAiRemixVariants(
    shots.map((shot) => ({
      segmentId: shot.segment.segmentId,
      alternativeSegmentIds: shot.alternativeSegments.map(
        (segment) => segment.segmentId,
      ),
    })),
    requestedCount,
  );

  return result.variants.map((variant, variantIndex) => ({
    version: variantIndex + 1,
    shots: shots.map((shot, shotIndex) => {
      const selectedSegmentId = variant.segmentIds[shotIndex];
      const candidates = [shot.segment, ...shot.alternativeSegments];
      const selectedSegment = candidates.find(
        (segment) => segment.segmentId === selectedSegmentId,
      );

      if (!selectedSegment) {
        throw new Error(
          `第 ${shotIndex + 1} 个分镜的差异方案包含未知片段 ${selectedSegmentId}。`,
        );
      }

      return {
        ...shot,
        segment: selectedSegment,
        alternativeSegments: candidates.filter(
          (segment) => segment.segmentId !== selectedSegmentId,
        ),
      };
    }),
  }));
}

function formatFileName(filePath: string) {
  return filePath.split(/[\\/]/).pop() ?? filePath;
}
