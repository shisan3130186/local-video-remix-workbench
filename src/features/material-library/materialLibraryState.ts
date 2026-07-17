import { convertFileSrc } from "@tauri-apps/api/core";
import type { SegmentCategory } from "../../services/videoMixService";
import type { ImportedVideo } from "../../types/videoProbe";
import type { AiRemixSegment } from "../ai-remix/types";
import type { ProjectMaterialsSnapshot } from "../project-recovery/types";
import type { MaterialLibrarySnapshot, MaterialLibraryState } from "./types";

interface BuildMaterialLibraryStateInput {
  importedVideos: ImportedVideo[];
  selectedVideoPath: string | null;
  segmentDurationSeconds: number;
  splitOutputDirectory: string | null;
  selectedSegmentPath: string | null;
  coverFrameSeconds: number;
  videoCoverPaths: Record<string, string>;
  splitSegmentPaths: string[];
  segmentCategories: Record<string, SegmentCategory | "">;
  segmentThumbnailPaths: Record<string, string>;
  preparedSegments: AiRemixSegment[];
}

export interface SanitizedMaterialLibrary {
  materials: ProjectMaterialsSnapshot;
  preparedSegments: AiRemixSegment[];
  restoredMaterials: number;
  restoredSegments: number;
}

export function buildMaterialLibraryState(
  input: BuildMaterialLibraryStateInput,
): MaterialLibraryState {
  const preparedByPath = new Map(
    input.preparedSegments.map((segment) => [segment.path, segment]),
  );
  return {
    materials: input.importedVideos.map((video) => ({
      video,
      coverPath: input.videoCoverPaths[video.id] ?? null,
    })),
    selectedVideoPath: input.selectedVideoPath,
    segmentDurationSeconds: input.segmentDurationSeconds,
    splitOutputDirectory: input.splitOutputDirectory,
    selectedSegmentPath: input.selectedSegmentPath,
    coverFrameSeconds: input.coverFrameSeconds,
    segments: input.splitSegmentPaths.map((path) => {
      const prepared = preparedByPath.get(path);
      return {
        path,
        category: input.segmentCategories[path] ?? "",
        thumbnailPath:
          input.segmentThumbnailPaths[path] ?? prepared?.thumbnailPath ?? null,
        segmentId: prepared?.segmentId ?? null,
        durationSeconds: prepared?.durationSeconds ?? null,
        description: prepared?.description ?? null,
      };
    }),
  };
}

export function sanitizeMaterialLibrarySnapshot(
  snapshot: MaterialLibrarySnapshot,
  missingFilePaths: string[],
): SanitizedMaterialLibrary {
  const missingSet = new Set(missingFilePaths);
  const validMaterials = snapshot.materials.filter(
    (material) => !missingSet.has(material.video.filePath),
  );
  const validSegments = snapshot.segments.filter(
    (segment) => !missingSet.has(segment.path),
  );
  const segmentPaths = validSegments.map((segment) => segment.path);
  const segmentThumbnailPaths = Object.fromEntries(
    validSegments
      .filter(
        (segment) =>
          segment.thumbnailPath && !missingSet.has(segment.thumbnailPath),
      )
      .map((segment) => [segment.path, segment.thumbnailPath as string]),
  );

  return {
    materials: {
      importedVideos: validMaterials.map((material) => material.video),
      selectedVideoPath: snapshot.selectedVideoPath,
      segmentDurationSeconds: snapshot.segmentDurationSeconds,
      splitOutputDirectory: snapshot.splitOutputDirectory,
      splitSegmentPaths: segmentPaths,
      segmentCategories: Object.fromEntries(
        validSegments.map((segment) => [segment.path, segment.category]),
      ),
      segmentThumbnailPaths,
      selectedSegmentPath: segmentPaths.includes(snapshot.selectedSegmentPath ?? "")
        ? snapshot.selectedSegmentPath
        : null,
      videoCoverPaths: Object.fromEntries(
        validMaterials
          .filter(
            (material) => material.coverPath && !missingSet.has(material.coverPath),
          )
          .map((material) => [material.video.id, material.coverPath as string]),
      ),
      selectedCoverPath: null,
      coverFrameSeconds: snapshot.coverFrameSeconds,
    },
    preparedSegments: validSegments
      .filter(
        (segment) =>
          Boolean(segment.segmentId) &&
          Boolean(segment.thumbnailPath) &&
          !missingSet.has(segment.thumbnailPath as string) &&
          typeof segment.durationSeconds === "number" &&
          segment.durationSeconds > 0,
      )
      .map((segment) => ({
        segmentId: segment.segmentId as string,
        path: segment.path,
        durationSeconds: segment.durationSeconds as number,
        thumbnailPath: segment.thumbnailPath as string,
        thumbnailUrl: convertFileSrc(segment.thumbnailPath as string),
        analysisThumbnailPaths: [segment.thumbnailPath as string],
        description: segment.description,
      })),
    restoredMaterials: validMaterials.length,
    restoredSegments: validSegments.length,
  };
}
