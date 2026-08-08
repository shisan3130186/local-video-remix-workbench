import type { ProjectSnapshotEnvelope, ProjectStateSnapshot } from "./types";

export function buildProjectSnapshotEnvelope(
  projectState: ProjectStateSnapshot,
  savedAt: string,
): ProjectSnapshotEnvelope {
  const project = cloneProjectState(projectState);
  return {
    version: 1,
    savedAt,
    project,
    filePaths: collectFilePaths(project),
    directoryPaths: collectDirectoryPaths(project),
  };
}

export function assertValidLoadedSnapshot(snapshot: ProjectSnapshotEnvelope) {
  const project = snapshot.project;
  if (
    snapshot.version !== 1 ||
    typeof snapshot.savedAt !== "string" ||
    !isRecord(project) ||
    !isRecord(project.materials) ||
    !Array.isArray(project.materials.importedVideos) ||
    !Array.isArray(project.materials.splitSegmentPaths) ||
    !isRecord(project.materials.segmentCategories) ||
    !isRecord(project.materials.segmentThumbnailPaths) ||
    !isRecord(project.materials.videoCoverPaths) ||
    !isRecord(project.ai) ||
    typeof project.ai.script !== "string" ||
    !Array.isArray(project.ai.preparedSegments) ||
    !Array.isArray(project.ai.plannedShots) ||
    !isRecord(project.remixSettings) ||
    !isRecord(project.remixSettings.videoEffectSettings) ||
    !isRecord(project.remixSettings.pictureInPictureSettings) ||
    !isRecord(project.remixSettings.bgmSettings) ||
    ("watermarkSettings" in project.remixSettings &&
      !isRecord(project.remixSettings.watermarkSettings)) ||
    ("watermarkRemovalSettings" in project.remixSettings &&
      !isRecord(project.remixSettings.watermarkRemovalSettings)) ||
    !isRecord(project.tts)
  ) {
    throw new Error("上次项目记录内容不完整，请清除记录后重新开始。");
  }
}

export function formatSavedTime(value: string) {
  const date = new Date(value);
  return Number.isNaN(date.getTime())
    ? ""
    : date.toLocaleTimeString("zh-CN", { hour12: false });
}

function cloneProjectState(project: ProjectStateSnapshot) {
  return JSON.parse(JSON.stringify(project)) as ProjectStateSnapshot;
}

function collectFilePaths(project: ProjectStateSnapshot) {
  return uniquePaths([
    ...project.materials.importedVideos.map((video) => video.filePath),
    ...(project.materials.materialFolders ?? []).flatMap((folder) => [
      ...folder.videoPaths,
      folder.settings.fixedFirstMaterialKind === "file"
        ? folder.settings.fixedFirstMaterialPath
        : null,
    ]),
    ...project.materials.splitSegmentPaths,
    ...Object.values(project.materials.segmentThumbnailPaths),
    ...Object.values(project.materials.videoCoverPaths),
    ...project.ai.preparedSegments.flatMap((segment) => [
      segment.path,
      segment.thumbnailPath,
      ...(segment.analysisThumbnailPaths ?? []),
    ]),
    project.remixSettings.pictureInPictureSettings.overlayFilePath,
    project.remixSettings.bgmSettings.audioFilePath,
    project.remixSettings.watermarkSettings?.imageFilePath,
  ]);
}

function collectDirectoryPaths(project: ProjectStateSnapshot) {
  return uniquePaths([
    project.outputDirectory,
    project.materials.splitOutputDirectory,
    ...(project.materials.materialFolders ?? []).flatMap((folder) => [
      folder.folderPath,
      folder.settings.fixedFirstMaterialKind === "folder"
        ? folder.settings.fixedFirstMaterialPath
        : null,
    ]),
  ]);
}

function uniquePaths(paths: Array<string | null | undefined>) {
  return Array.from(
    new Set(paths.filter((path): path is string => Boolean(path?.trim()))),
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
