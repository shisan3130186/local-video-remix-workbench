import { invoke } from "@tauri-apps/api/core";
import type {
  SaveScriptLibraryEntryInput,
  SaveScriptLibraryEntryResult,
  ScriptLibraryEntry,
} from "../types";

export function loadScriptLibrary(): Promise<ScriptLibraryEntry[]> {
  return invoke<ScriptLibraryEntry[]>("load_script_library");
}

export function saveScriptLibraryEntry(
  input: SaveScriptLibraryEntryInput,
): Promise<SaveScriptLibraryEntryResult> {
  return invoke<SaveScriptLibraryEntryResult>("save_script_library_entry", { input });
}

export function deleteScriptLibraryEntry(id: string): Promise<ScriptLibraryEntry[]> {
  return invoke<ScriptLibraryEntry[]>("delete_script_library_entry", { id });
}
