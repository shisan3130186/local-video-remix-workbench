import { convertFileSrc } from "@tauri-apps/api/core";
import type { AiRemixPlannedShot, AiRemixSegment } from "../ai-remix/types";
import { sanitizeAiRemixContentAnalysis } from "../ai-remix/analysisCache";
import { DEFAULT_WATERMARK_REMOVAL_SETTINGS, DEFAULT_WATERMARK_SETTINGS } from "../watermark";
import { DEFAULT_MATERIAL_FOLDER_SETTINGS } from "../materials/types";
import type { ProjectStateSnapshot } from "./types";

export interface SanitizedProjectSnapshot {
  project: ProjectStateSnapshot;
  preparedSegments: AiRemixSegment[];
  plannedShots: AiRemixPlannedShot[];
  notices: string[];
}

export function sanitizeProjectSnapshot(
  project: ProjectStateSnapshot,
  missingFilePaths: string[],
  missingDirectoryPaths: string[],
): SanitizedProjectSnapshot {
  const missingFiles = new Set(missingFilePaths);
  const missingDirectories = new Set(missingDirectoryPaths);
  const notices: string[] = [];

  const importedVideos = project.materials.importedVideos.filter(
    (video) => !missingFiles.has(video.filePath),
  );
  const removedVideoCount = project.materials.importedVideos.length - importedVideos.length;
  if (removedVideoCount > 0) {
    notices.push(`${removedVideoCount} 个原始视频已失效，恢复时自动跳过。`);
  }

  const splitSegmentPaths = project.materials.splitSegmentPaths.filter(
    (path) => !missingFiles.has(path),
  );
  const validSegmentPaths = new Set(splitSegmentPaths);
  const removedSegmentCount =
    project.materials.splitSegmentPaths.length - splitSegmentPaths.length;
  if (removedSegmentCount > 0) {
    notices.push(`${removedSegmentCount} 个切片文件已失效，恢复时自动跳过。`);
  }

  const segmentThumbnailPaths = filterPathMap(
    project.materials.segmentThumbnailPaths,
    (key, value) => validSegmentPaths.has(key) && !missingFiles.has(value),
  );
  const videoCoverPaths = filterPathMap(
    project.materials.videoCoverPaths,
    (_key, value) => !missingFiles.has(value),
  );

  const preparedSegments = project.ai.preparedSegments
    .filter(
      (segment) =>
        validSegmentPaths.has(segment.path) && !missingFiles.has(segment.thumbnailPath),
    )
    .map((segment) => ({
      ...segment,
      contentAnalysis: sanitizeAiRemixContentAnalysis(segment.contentAnalysis),
      analysisThumbnailPaths:
        segment.analysisThumbnailPaths?.filter((path) => !missingFiles.has(path)) ?? [segment.thumbnailPath],
      thumbnailUrl: convertFileSrc(segment.thumbnailPath),
    }));
  const preparedSegmentMap = new Map(
    preparedSegments.map((segment) => [segment.segmentId, segment]),
  );

  const allPlannedShotsValid = project.ai.plannedShots.every(
    (shot) =>
      preparedSegmentMap.has(shot.segmentId) &&
      shot.alternativeSegmentIds.every((segmentId) => preparedSegmentMap.has(segmentId)),
  );
  const plannedShots = allPlannedShotsValid
    ? project.ai.plannedShots.map((shot) => ({
        shotId: shot.shotId,
        text: shot.text,
        segment: preparedSegmentMap.get(shot.segmentId) as AiRemixSegment,
        alternativeSegments: shot.alternativeSegmentIds.map(
          (segmentId) => preparedSegmentMap.get(segmentId) as AiRemixSegment,
        ),
      }))
    : [];

  if (!allPlannedShotsValid && project.ai.plannedShots.length > 0) {
    notices.push("部分分镜引用的片段已失效，原分镜已清空，请重新生成。");
  }

  const outputDirectory =
    project.outputDirectory && !missingDirectories.has(project.outputDirectory)
      ? project.outputDirectory
      : null;
  if (project.outputDirectory && !outputDirectory) {
    notices.push("原输出目录已失效，请重新选择输出目录。");
  }

  const splitOutputDirectory =
    project.materials.splitOutputDirectory &&
    !missingDirectories.has(project.materials.splitOutputDirectory)
      ? project.materials.splitOutputDirectory
      : null;

  const materialFolders = (project.materials.materialFolders ?? [])
    .map((folder) => {
      const videoPaths = folder.videoPaths.filter((path) => !missingFiles.has(path));
      const fixedPath = folder.settings.fixedFirstMaterialPath;
      const fixedMissing = fixedPath
        ? folder.settings.fixedFirstMaterialKind === "folder"
          ? missingDirectories.has(fixedPath)
          : missingFiles.has(fixedPath)
        : false;
      return {
        ...folder,
        videoPaths,
        settings: {
          ...DEFAULT_MATERIAL_FOLDER_SETTINGS,
          ...folder.settings,
          ...(fixedMissing
            ? {
                fixedFirstMaterialEnabled: false,
                fixedFirstMaterialPath: null,
                fixedFirstMaterialKind: null,
              }
            : {}),
        },
      };
    })
    .filter(
      (folder) =>
        !missingDirectories.has(folder.folderPath) && folder.videoPaths.length > 0,
    );

  const remixSettings = cloneJsonValue(project.remixSettings);
  remixSettings.watermarkSettings = {
    ...DEFAULT_WATERMARK_SETTINGS,
    ...(remixSettings.watermarkSettings ?? {}),
  };
  remixSettings.watermarkRemovalSettings = {
    ...DEFAULT_WATERMARK_REMOVAL_SETTINGS,
    ...(remixSettings.watermarkRemovalSettings ?? {}),
  };
  remixSettings.watermarkRemovalSettings.trackingRegionWidthRatio = sanitizeTrackingSize(
    remixSettings.watermarkRemovalSettings.trackingRegionWidthRatio,
    DEFAULT_WATERMARK_REMOVAL_SETTINGS.trackingRegionWidthRatio,
  );
  remixSettings.watermarkRemovalSettings.trackingRegionHeightRatio = sanitizeTrackingSize(
    remixSettings.watermarkRemovalSettings.trackingRegionHeightRatio,
    DEFAULT_WATERMARK_REMOVAL_SETTINGS.trackingRegionHeightRatio,
  );
  remixSettings.watermarkRemovalSettings.trackingKeyframes = Array.isArray(
    remixSettings.watermarkRemovalSettings.trackingKeyframes,
  )
    ? remixSettings.watermarkRemovalSettings.trackingKeyframes
        .filter(
          (keyframe) =>
            isRecord(keyframe) &&
            typeof keyframe.timeSeconds === "number" &&
            Number.isFinite(keyframe.timeSeconds) &&
            keyframe.timeSeconds >= 0 &&
            typeof keyframe.xRatio === "number" &&
            Number.isFinite(keyframe.xRatio) &&
            keyframe.xRatio >= 0 &&
            typeof keyframe.yRatio === "number" &&
            Number.isFinite(keyframe.yRatio) &&
            keyframe.yRatio >= 0,
        )
        .map((keyframe) => ({
          timeSeconds: keyframe.timeSeconds as number,
          xRatio: keyframe.xRatio as number,
          yRatio: keyframe.yRatio as number,
        }))
        .sort((left, right) => left.timeSeconds - right.timeSeconds)
        .slice(0, 8)
    : [];
  if (
    remixSettings.watermarkRemovalSettings.trackingEnabled &&
    remixSettings.watermarkRemovalSettings.trackingKeyframes.length < 2
  ) {
    remixSettings.watermarkRemovalSettings.trackingEnabled = false;
    notices.push("移动水印轨迹记录不完整，关键帧跟踪已自动关闭。");
  }
  const pipPath = remixSettings.pictureInPictureSettings.overlayFilePath;
  if (pipPath && missingFiles.has(pipPath)) {
    remixSettings.pictureInPictureSettings.enabled = false;
    remixSettings.pictureInPictureSettings.overlayFilePath = null;
    notices.push("原画中画素材已失效，画中画已自动关闭。");
  }

  const bgmPath = remixSettings.bgmSettings.audioFilePath;
  if (bgmPath && missingFiles.has(bgmPath)) {
    remixSettings.bgmSettings.enabled = false;
    remixSettings.bgmSettings.audioFilePath = null;
    notices.push("原BGM文件已失效，BGM已自动关闭。");
  }

  const watermarkPath = remixSettings.watermarkSettings.imageFilePath;
  if (watermarkPath && missingFiles.has(watermarkPath)) {
    remixSettings.watermarkSettings.enabled = false;
    remixSettings.watermarkSettings.imageFilePath = null;
    notices.push("原图片水印文件已失效，水印已自动关闭。");
  }

  const sanitizedProject: ProjectStateSnapshot = {
    ...project,
    outputDirectory,
    materials: {
      ...project.materials,
      importedVideos,
      materialFolders,
      selectedVideoPath: importedVideos.some(
        (video) => video.filePath === project.materials.selectedVideoPath,
      )
        ? project.materials.selectedVideoPath
        : importedVideos[0]?.filePath ?? null,
      splitOutputDirectory,
      splitSegmentPaths,
      segmentCategories: filterPathMap(
        project.materials.segmentCategories,
        (key) => validSegmentPaths.has(key),
      ),
      segmentThumbnailPaths,
      selectedSegmentPath:
        project.materials.selectedSegmentPath &&
        validSegmentPaths.has(project.materials.selectedSegmentPath)
          ? project.materials.selectedSegmentPath
          : null,
      videoCoverPaths,
      selectedCoverPath:
        project.materials.selectedCoverPath &&
        !missingFiles.has(project.materials.selectedCoverPath)
          ? project.materials.selectedCoverPath
          : null,
    },
    ai: {
      ...project.ai,
      preparedSegments: preparedSegments.map(({ thumbnailUrl: _thumbnailUrl, ...segment }) =>
        segment,
      ),
      plannedShots: allPlannedShotsValid ? project.ai.plannedShots : [],
    },
    remixSettings,
  };

  return { project: sanitizedProject, preparedSegments, plannedShots, notices };
}

function filterPathMap<T>(
  entries: Record<string, T>,
  predicate: (key: string, value: T) => boolean,
) {
  return Object.fromEntries(
    Object.entries(entries).filter(([key, value]) => predicate(key, value)),
  ) as Record<string, T>;
}

function sanitizeTrackingSize(value: unknown, fallback: number) {
  return typeof value === "number" && Number.isFinite(value) && value >= 0.04 && value <= 0.8
    ? value
    : fallback;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function cloneJsonValue<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}
