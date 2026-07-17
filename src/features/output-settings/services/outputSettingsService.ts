import { invoke } from "@tauri-apps/api/core";
import type { EncoderCapabilities } from "../../../types/outputSettings";

export function getVideoEncoderCapabilities(forceRefresh: boolean): Promise<EncoderCapabilities> {
  return invoke<EncoderCapabilities>("get_video_encoder_capabilities", { forceRefresh });
}
