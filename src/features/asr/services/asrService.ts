import { invoke } from "@tauri-apps/api/core";
import type { TaskProgressContext } from "../../task-center";
import type { AsrConfigStatus, AsrRecognitionResult } from "../types";

export function getAsrConfigStatus(): Promise<AsrConfigStatus> {
  return invoke<AsrConfigStatus>("get_asr_config_status");
}

export function recognizeSpeech(
  filePath: string,
  taskContext: TaskProgressContext,
): Promise<AsrRecognitionResult> {
  return invoke<AsrRecognitionResult>("recognize_speech", {
    filePath,
    taskContext,
  });
}
