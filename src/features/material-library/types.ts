import type { ImportedVideo } from "../../types/videoProbe";
import type { SegmentCategory } from "../../services/videoMixService";

export interface MaterialLibraryMaterial {
  video: ImportedVideo;
  coverPath: string | null;
}

export interface MaterialLibrarySegment {
  path: string;
  category: SegmentCategory | "";
  thumbnailPath: string | null;
  segmentId: string | null;
  durationSeconds: number | null;
  description: string | null;
}

export interface MaterialLibraryState {
  materials: MaterialLibraryMaterial[];
  selectedVideoPath: string | null;
  segmentDurationSeconds: number;
  splitOutputDirectory: string | null;
  selectedSegmentPath: string | null;
  coverFrameSeconds: number;
  segments: MaterialLibrarySegment[];
}

export interface MaterialLibrarySnapshot extends MaterialLibraryState {
  version: 1;
  savedAt: string;
}

export interface MaterialLibraryLoadResult {
  snapshot: MaterialLibrarySnapshot | null;
  missingFilePaths: string[];
  databasePath: string;
}

export type MaterialLibraryStatus = "idle" | "loading" | "saving" | "saved" | "error";
