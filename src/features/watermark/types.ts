export type WatermarkKind = "text" | "image";

export type WatermarkPosition =
  | "topLeft"
  | "topRight"
  | "bottomLeft"
  | "bottomRight"
  | "center";

export interface WatermarkSettings {
  enabled: boolean;
  kind: WatermarkKind;
  text: string;
  imageFilePath: string | null;
  position: WatermarkPosition;
  opacity: number;
  margin: number;
  textFontSize: number;
  textColor: string;
  imageSizeRatio: number;
}

export type WatermarkRemovalMode = "crop" | "delogo" | "blur" | "mosaic" | "cover";
export type WatermarkRemovalSize = "small" | "medium" | "large";

export interface WatermarkTrackingKeyframe {
  timeSeconds: number;
  xRatio: number;
  yRatio: number;
}

export interface WatermarkRemovalSettings {
  enabled: boolean;
  mode: WatermarkRemovalMode;
  position: WatermarkPosition;
  size: WatermarkRemovalSize;
  margin: number;
  strength: number;
  coverColor: string;
  coverOpacity: number;
  trackingEnabled: boolean;
  trackingRegionWidthRatio: number;
  trackingRegionHeightRatio: number;
  trackingKeyframes: WatermarkTrackingKeyframe[];
}

export const DEFAULT_WATERMARK_SETTINGS: WatermarkSettings = {
  enabled: false,
  kind: "text",
  text: "",
  imageFilePath: null,
  position: "topRight",
  opacity: 0.75,
  margin: 24,
  textFontSize: 36,
  textColor: "#ffffff",
  imageSizeRatio: 0.18,
};

export const DEFAULT_WATERMARK_REMOVAL_SETTINGS: WatermarkRemovalSettings = {
  enabled: false,
  mode: "delogo",
  position: "topRight",
  size: "medium",
  margin: 16,
  strength: 12,
  coverColor: "#000000",
  coverOpacity: 0.85,
  trackingEnabled: false,
  trackingRegionWidthRatio: 0.28,
  trackingRegionHeightRatio: 0.12,
  trackingKeyframes: [],
};
