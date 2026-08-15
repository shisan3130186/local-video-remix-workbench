import { generateThumbnail } from "../../services/videoThumbnailService";
import type { ImportedVideo } from "../../types/videoProbe";
import type { ToolKey } from "../../types/workbench";
import type { AiRemixMatchMode } from "../ai-remix/types";
import { analyzeAiRemixSegments } from "../ai-remix/services/aiRemixService";

export type SmartEffectMode = AiRemixMatchMode;

export interface SmartEffectRecommendation {
  key: ToolKey;
  title: string;
  status: "enabled" | "needs-material";
  reason: string;
}

export interface SmartEffectPlan {
  mode: SmartEffectMode;
  sourceVideoName: string;
  description: string;
  summary: string;
  recommendations: SmartEffectRecommendation[];
  enabledEffectKeys: ToolKey[];
  pendingEffectKeys: ToolKey[];
  watermarkHint: boolean;
}

const effectTitles: Record<string, string> = {
  bgm: "音频设置",
  cover: "视频封面",
  entrance: "入场效果",
  frame: "视频帧操作",
  pip: "画中画",
  effects: "画面调整",
  fusion: "像素融合",
  rotate: "旋转换像",
  speed: "视频变速",
  zoom: "动态缩放",
};

export async function buildSmartEffectPlan(
  video: ImportedVideo,
  outputDirectory: string,
  mode: SmartEffectMode,
): Promise<SmartEffectPlan> {
  const thumbnailPaths = await createAnalysisThumbnails(video, outputDirectory);
  const description =
    mode === "cloud"
      ? await analyzeCloudDescription(video, thumbnailPaths)
      : buildLocalDescription(video);
  const normalizedDescription = description.trim();
  const enabledEffectKeys = selectSupportedEffects(video, normalizedDescription);
  const pendingEffectKeys: ToolKey[] = ["pip", "fusion"];
  const watermarkHint = /水印|标识|logo|logo|文字叠加/i.test(normalizedDescription);
  const recommendations = createRecommendations(
    enabledEffectKeys,
    pendingEffectKeys,
    normalizedDescription,
    watermarkHint,
  );

  return {
    mode,
    sourceVideoName: video.fileName,
    description: normalizedDescription,
    summary:
      mode === "cloud"
        ? `云端视觉理解完成，已推荐 ${enabledEffectKeys.length} 项安全效果，另有 ${pendingEffectKeys.length} 项等待素材。`
        : `本地视频特征分析完成，已推荐 ${enabledEffectKeys.length} 项安全效果，另有 ${pendingEffectKeys.length} 项等待素材。`,
    recommendations,
    enabledEffectKeys,
    pendingEffectKeys,
    watermarkHint,
  };
}

async function createAnalysisThumbnails(video: ImportedVideo, outputDirectory: string) {
  const duration = Math.max(video.durationSeconds ?? 1, 1);
  const sampleTimes = [0.08, 0.5, 0.88].map((ratio) =>
    Math.min(Math.max(duration * ratio, 0.1), Math.max(duration - 0.1, 0.1)),
  );
  const results = [];

  for (const [index, timeSeconds] of sampleTimes.entries()) {
    results.push(
      await generateThumbnail(
        video.filePath,
        outputDirectory,
        timeSeconds,
        `smart_effect_${video.id}_${index + 1}`,
        "contain",
      ),
    );
  }

  return results.map((result) => result.thumbnailPath);
}

async function analyzeCloudDescription(video: ImportedVideo, thumbnailPaths: string[]) {
  const result = await analyzeAiRemixSegments([
    {
      segmentId: video.id,
      durationSeconds: video.durationSeconds ?? 0,
      thumbnailPaths,
    },
  ]);
  return result.segments[0]?.description ?? "未获得有效的画面描述。";
}

function buildLocalDescription(video: ImportedVideo) {
  const width = video.width ?? 0;
  const height = video.height ?? 0;
  const orientation = width > height ? "横屏" : width === height ? "方形" : "竖屏";
  const audio = video.hasAudio ? "包含原始音频" : "未检测到原始音频";
  return `本地分析：${orientation}视频，${audio}，时长约 ${Math.round(video.durationSeconds ?? 0)} 秒。已根据画布方向、时长和素材处理安全性生成效果建议。`;
}

function selectSupportedEffects(_video: ImportedVideo, _description: string): ToolKey[] {
  return selectSafeEffects(_video, _description).filter((key) =>
    ["effects", "speed", "zoom", "rotate", "frame", "entrance"].includes(key),
  );
}

function selectSafeEffects(video: ImportedVideo, description: string): ToolKey[] {
  const normalized = description.toLowerCase();
  const keys: ToolKey[] = ["effects", "speed", "zoom"];

  if ((video.width ?? 0) <= (video.height ?? 0) || /人物|产品|主体|portrait|person|product/.test(normalized)) {
    keys.push("rotate");
  }

  if ((video.durationSeconds ?? 0) >= 6) {
    keys.push("frame");
  }

  if (/人物|产品|商品|展示|细节|person|product|detail/.test(normalized)) {
    keys.push("cover");
  }

  if (video.hasAudio) {
    keys.push("bgm");
  }

  keys.push("entrance");
  return [...new Set(keys)];
}

function createRecommendations(
  enabledEffectKeys: ToolKey[],
  pendingEffectKeys: ToolKey[],
  description: string,
  watermarkHint: boolean,
) {
  const recommendations: SmartEffectRecommendation[] = enabledEffectKeys.map((key) => ({
    key,
    title: effectTitles[key] ?? key,
    status: "enabled",
    reason: reasonForEffect(key, description),
  }));

  recommendations.push(
    ...pendingEffectKeys.map((key) => ({
      key,
      title: effectTitles[key] ?? key,
      status: "needs-material" as const,
      reason: key === "pip" ? "需要上传背景素材后才能加入处理队列。" : "需要上传融合素材后才能加入处理队列。",
    })),
  );

  if (watermarkHint) {
    recommendations.push({
      key: "watermark",
      title: "去除水印",
      status: "needs-material",
      reason: "AI 发现疑似文字或标识，请打开贴画与水印并人工确认框选区域。",
    });
  }

  return recommendations;
}

function reasonForEffect(key: ToolKey, description: string) {
  if (key === "effects") return "根据画面特征准备轻量调色，避免改变主体识别。";
  if (key === "speed") return "使用轻微变速，降低连续素材的时间指纹。";
  if (key === "zoom") return "根据画面方向加入轻量动态缩放，减少静态重复。";
  if (key === "rotate") return "按画布方向预留镜像/旋转差异化处理。";
  if (key === "frame") return "素材时长足够，允许使用安全的抽帧策略。";
  if (key === "cover") return "从当前视频帧生成封面，不要求额外上传素材。";
  if (key === "bgm") return "保留音频设置入口，只有选择本地音乐后才会加入背景音乐。";
  if (key === "entrance") return "使用轻量入场效果，避免画面突然出现。";
  return description ? "由 AI 画面描述综合判断。" : "按默认视频策略处理。";
}
