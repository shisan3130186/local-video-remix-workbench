import { invoke } from "@tauri-apps/api/core";
import type { WatermarkDetectionResult } from "../types";

export function detectWatermarkRegions(inputFilePath: string, maxRegions: number): Promise<WatermarkDetectionResult> {
  return invoke<WatermarkDetectionResult>("detect_watermark_regions", {
    inputFilePath,
    maxRegions,
  });
}
