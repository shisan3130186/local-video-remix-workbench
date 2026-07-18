import type { AsrUtterance } from "../asr";

export interface ScriptLibraryEntry {
  id: string;
  title: string;
  text: string;
  sourcePath: string | null;
  sourceFileName: string | null;
  durationSeconds: number | null;
  utterances: AsrUtterance[];
  createdAtMs: number;
}

export interface SaveScriptLibraryEntryInput {
  title: string;
  text: string;
  sourcePath: string | null;
  sourceFileName: string | null;
  durationSeconds: number | null;
  utterances: AsrUtterance[];
}

export interface SaveScriptLibraryEntryResult {
  entry: ScriptLibraryEntry;
  created: boolean;
  message: string;
}
