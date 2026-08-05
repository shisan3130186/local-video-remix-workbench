import { invoke } from "@tauri-apps/api/core";

export function writeUtf8TextFile(path: string, content: string): Promise<void> {
  return invoke("write_utf8_text_file", { path, content });
}
