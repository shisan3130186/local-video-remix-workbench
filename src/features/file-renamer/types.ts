export type RenameMode = "uniform" | "replace" | "affix" | "datetime";

export interface RenamePreviewItem {
  sourcePath: string;
  targetPath: string;
  oldName: string;
  newName: string;
  conflict: string | null;
}
