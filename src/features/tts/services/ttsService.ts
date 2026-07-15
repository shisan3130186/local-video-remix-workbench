import { invoke } from "@tauri-apps/api/core";
import type { TtsConfigStatus, TtsSynthesisResult } from "../types";

export function getTtsConfigStatus(): Promise<TtsConfigStatus> {
  return invoke<TtsConfigStatus>("get_tts_config_status");
}

export function synthesizeTts(
  text: string,
  outputDirectory: string,
  speaker: string,
): Promise<TtsSynthesisResult> {
  return invoke<TtsSynthesisResult>("synthesize_tts", {
    text,
    outputDirectory,
    speaker,
  });
}
