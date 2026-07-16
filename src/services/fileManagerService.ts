import { invoke } from "@tauri-apps/api/core";

export function openPathInFileManager(path: string): Promise<void> {
  return invoke("open_path_in_file_manager", { path });
}

export function isExistingDirectory(path: string): Promise<boolean> {
  return invoke<boolean>("is_existing_directory", { path });
}
