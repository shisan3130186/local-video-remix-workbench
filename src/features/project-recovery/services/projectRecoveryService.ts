import { invoke } from "@tauri-apps/api/core";
import type { ProjectSnapshotEnvelope, ProjectSnapshotLoadResult } from "../types";

export function loadProjectSnapshot(): Promise<ProjectSnapshotLoadResult> {
  return invoke<ProjectSnapshotLoadResult>("load_project_snapshot");
}

export function saveProjectSnapshot(snapshot: ProjectSnapshotEnvelope): Promise<void> {
  return invoke<void>("save_project_snapshot", { snapshot });
}

export function deleteProjectSnapshot(): Promise<void> {
  return invoke<void>("delete_project_snapshot");
}
