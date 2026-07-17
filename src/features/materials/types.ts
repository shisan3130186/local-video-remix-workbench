export type SplitMode = "scene" | "duration";

export type SceneSensitivity = "stable" | "balanced" | "sensitive";

export interface SmartSplitSettings {
  sensitivity: SceneSensitivity;
  minimumSegmentSeconds: number;
  maximumSegmentSeconds: number;
}
