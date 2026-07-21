import { invoke } from "@tauri-apps/api/core";
import type { DiagnosticInfo } from "../types";

export function getDiagnosticInfo() {
  return invoke<DiagnosticInfo>("get_diagnostic_info");
}

export function createDiagnosticReport() {
  return invoke<string>("create_diagnostic_report");
}
