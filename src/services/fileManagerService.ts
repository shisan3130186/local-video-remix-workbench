import { invoke } from "@tauri-apps/api/core";

export function openPathInFileManager(path: string): Promise<void> {
  return invoke("open_path_in_file_manager", { path });
}

export function isExistingDirectory(path: string): Promise<boolean> {
  return invoke<boolean>("is_existing_directory", { path });
}

export function createAiRemixOutputDirectory(
  outputDirectory: string,
  label: string,
): Promise<string> {
  return invoke<string>("create_ai_remix_output_directory_command", {
    outputDirectory,
    label,
  });
}

export function exportJianyingDraftPackage(
  outputDirectory: string,
  projectName: string,
  videoPaths: string[],
  script: string,
): Promise<string> {
  return invoke<string>("export_jianying_draft_package_command", {
    outputDirectory,
    projectName,
    videoPaths,
    script,
  });
}
