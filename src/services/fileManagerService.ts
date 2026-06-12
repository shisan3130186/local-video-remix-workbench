import { invoke } from "@tauri-apps/api/core";

export function openPathInFileManager(path: string): Promise<void> {
  return invoke("open_path_in_file_manager", { path });
}
