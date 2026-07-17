import type { AiRemixSegment } from "../ai-remix/types";
import type { NarratedSubtitlePosition, NarratedSubtitleSize } from "../tts/types";
import type { RemixExportSettings, SegmentCategory } from "../../services/videoMixService";
import type { ImportedVideo } from "../../types/videoProbe";

export interface SavedAiRemixSegment
  extends Omit<AiRemixSegment, "thumbnailUrl"> {}

export interface SavedAiRemixShot {
  shotId: string;
  text: string;
  segmentId: string;
  alternativeSegmentIds: string[];
}

export interface ProjectMaterialsSnapshot {
  importedVideos: ImportedVideo[];
  selectedVideoPath: string | null;
  segmentDurationSeconds: number;
  splitOutputDirectory: string | null;
  splitSegmentPaths: string[];
  segmentCategories: Record<string, SegmentCategory | "">;
  segmentThumbnailPaths: Record<string, string>;
  selectedSegmentPath: string | null;
  videoCoverPaths: Record<string, string>;
  selectedCoverPath: string | null;
  coverFrameSeconds: number;
}

export interface ProjectAiSnapshot {
  script: string;
  generateCount: number;
  preparedSegments: SavedAiRemixSegment[];
  plannedShots: SavedAiRemixShot[];
}

export interface ProjectTtsSnapshot {
  speaker: string;
  videoEnabled: boolean;
  keepOriginalAudio: boolean;
  originalAudioVolume: number;
  subtitleEnabled: boolean;
  subtitlePosition: NarratedSubtitlePosition;
  subtitleSize: NarratedSubtitleSize;
}

export interface ProjectStateSnapshot {
  outputDirectory: string | null;
  materials: ProjectMaterialsSnapshot;
  ai: ProjectAiSnapshot;
  remixSettings: RemixExportSettings;
  tts: ProjectTtsSnapshot;
}

export interface ProjectSnapshotEnvelope {
  version: 1;
  savedAt: string;
  project: ProjectStateSnapshot;
  filePaths: string[];
  directoryPaths: string[];
}

export interface ProjectSnapshotLoadResult {
  snapshot: ProjectSnapshotEnvelope | null;
  missingFilePaths: string[];
  missingDirectoryPaths: string[];
}

export type ProjectAutoSaveStatus = "loading" | "idle" | "saving" | "saved" | "error";
