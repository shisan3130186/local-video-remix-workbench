export type SplitMode = "scene" | "duration";

export type SceneSensitivity = "stable" | "balanced" | "sensitive";

export interface SmartSplitSettings {
  sensitivity: SceneSensitivity;
  minimumSegmentSeconds: number;
  maximumSegmentSeconds: number;
}

export type MaterialFolderVideoMode = "script" | "audio";
export type MaterialFolderOutputMode = "byScript" | "flat";
export type FixedMaterialMode = "default" | "independentJoin";
export type FixedMaterialKind = "file" | "folder";

export interface MaterialFolderSettings {
  videoMode: MaterialFolderVideoMode;
  extractionMode: "auto";
  materialCountMode: "auto";
  variantCount: number;
  allowMaterialRepeat: boolean;
  outputMode: MaterialFolderOutputMode;
  fixedFirstMaterialEnabled: boolean;
  fixedFirstMaterialPath: string | null;
  fixedFirstMaterialKind: FixedMaterialKind | null;
  fixedMaterialMode: FixedMaterialMode;
}

export interface MaterialFolder {
  id: string;
  folderPath: string;
  folderName: string;
  videoPaths: string[];
  settings: MaterialFolderSettings;
}

export const DEFAULT_MATERIAL_FOLDER_SETTINGS: MaterialFolderSettings = {
  videoMode: "script",
  extractionMode: "auto",
  materialCountMode: "auto",
  variantCount: 1,
  allowMaterialRepeat: true,
  outputMode: "byScript",
  fixedFirstMaterialEnabled: false,
  fixedFirstMaterialPath: null,
  fixedFirstMaterialKind: null,
  fixedMaterialMode: "default",
};

export function cloneMaterialFolderSettings(
  settings: MaterialFolderSettings,
): MaterialFolderSettings {
  return { ...settings };
}
