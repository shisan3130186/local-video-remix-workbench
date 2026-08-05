export type ImageVideoAspect = "original" | "portrait916" | "square11" | "landscape169";

export interface ImageVideoResult {
  items: Array<{ sourcePath: string; outputPath: string }>;
  failures: Array<{ sourcePath: string; message: string }>;
}
