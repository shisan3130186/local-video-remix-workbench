import { invoke } from "@tauri-apps/api/core";

export function listVideoFilesInFolder(folderPath: string): Promise<string[]> {
  return invoke<string[]>("list_video_files_in_folder", { folderPath });
}
