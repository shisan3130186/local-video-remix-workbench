import { invoke } from "@tauri-apps/api/core";

export function writePosterImage(path: string, data: string): Promise<void> {
  return invoke("write_base64_file", { path, data });
}
