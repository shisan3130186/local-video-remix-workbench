import { invoke } from "@tauri-apps/api/core";

export function listFilesInFolder(folderPath: string): Promise<string[]> {
  return invoke<string[]>("list_files_in_folder", { folderPath });
}

export function batchRenameFiles(items: Array<{ sourcePath: string; targetPath: string }>) {
  return invoke<Array<{ sourcePath: string; targetPath: string }>>("batch_rename_files", { items });
}
