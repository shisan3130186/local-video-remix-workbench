import { invoke } from "@tauri-apps/api/core";
import type {
  MaterialLibraryLoadResult,
  MaterialLibrarySnapshot,
} from "../types";

export function loadMaterialLibrary(): Promise<MaterialLibraryLoadResult> {
  return invoke<MaterialLibraryLoadResult>("load_material_library");
}

export function saveMaterialLibrary(snapshot: MaterialLibrarySnapshot): Promise<void> {
  return invoke<void>("save_material_library", { snapshot });
}
