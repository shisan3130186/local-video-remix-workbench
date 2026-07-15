import { invoke } from "@tauri-apps/api/core";
import type { MixVideoResult, RemixExportSettings } from "../../../services/videoMixService";
import type {
  NarratedAudioSettings,
  NarratedSegmentInput,
  TtsConfigStatus,
  TtsSynthesisResult,
} from "../types";

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

export function synthesizeTtsShot(
  text: string,
  speaker: string,
  sessionId: string,
  shotIndex: number,
): Promise<TtsSynthesisResult> {
  return invoke<TtsSynthesisResult>("synthesize_tts_shot", {
    text,
    speaker,
    sessionId,
    shotIndex,
  });
}

export function cleanupTtsSession(sessionId: string): Promise<void> {
  return invoke<void>("cleanup_tts_session", { sessionId });
}

export function concatNarratedSegments(
  segments: NarratedSegmentInput[],
  outputDirectory: string,
  settings: RemixExportSettings,
  audioSettings: NarratedAudioSettings,
): Promise<MixVideoResult> {
  return invoke<MixVideoResult>("concat_narrated_segments", {
    segments,
    outputDirectory,
    settings,
    audioSettings,
  });
}
