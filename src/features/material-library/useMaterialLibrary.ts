import { open } from "@tauri-apps/plugin-dialog";
import { computed, onBeforeUnmount, ref, watch } from "vue";
import type { Ref } from "vue";
import { loadImportedVideos } from "../materials/services/materialWorkflow";
import { loadMaterialLibrary, saveMaterialLibrary } from "./services/materialLibraryService";
import type {
  MaterialLibraryMaterial,
  MaterialLibrarySnapshot,
  MaterialLibraryState,
  MaterialLibraryStatus,
} from "./types";

interface UseMaterialLibraryOptions {
  currentState: Readonly<Ref<MaterialLibraryState>>;
  applySnapshot: (
    snapshot: MaterialLibrarySnapshot,
    missingFilePaths: string[],
  ) => { restoredMaterials: number; restoredSegments: number };
  onMaterialRelinked: (
    missingMaterial: MaterialLibraryMaterial,
    replacement: MaterialLibraryMaterial,
  ) => void;
}

const AUTO_SAVE_DELAY_MS = 900;

export function useMaterialLibrary(options: UseMaterialLibraryOptions) {
  const status = ref<MaterialLibraryStatus>("idle");
  const loadError = ref<string | null>(null);
  const saveError = ref<string | null>(null);
  const lastSavedAt = ref<string | null>(null);
  const missingMaterials = ref<MaterialLibraryMaterial[]>([]);
  const missingSegmentCount = ref(0);
  const restoredSummary = ref<string | null>(null);
  const relinkingPath = ref<string | null>(null);
  const availableSnapshot = ref<MaterialLibrarySnapshot | null>(null);
  const availableMissingFilePaths = ref<string[]>([]);
  let autoSaveEnabled = false;
  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  const statusText = computed(() => {
    if (status.value === "loading") return "正在读取本地素材库";
    if (status.value === "saving") return "正在保存素材库";
    if (status.value === "error") return "素材库保存异常";
    if (restoredSummary.value) return restoredSummary.value;
    if (availableSnapshot.value && !hasActiveState()) {
      return `本机素材库可载入：${availableSnapshot.value.materials.length} 个视频、${availableSnapshot.value.segments.length} 个片段`;
    }
    if (status.value === "saved") return "素材库已自动保存";
    return "素材库将在本机自动保存";
  });

  const hasAvailableLibrary = computed(() => Boolean(availableSnapshot.value));
  const isLoadingLibrary = computed(() => status.value === "loading");

  watch(
    options.currentState,
    () => {
      if (autoSaveEnabled) scheduleSave();
    },
    { deep: true },
  );

  onBeforeUnmount(() => {
    if (saveTimer) clearTimeout(saveTimer);
  });

  async function inspectLibrary() {
    status.value = "loading";
    loadError.value = null;

    try {
      const result = await loadMaterialLibrary();
      availableSnapshot.value = result.snapshot;
      availableMissingFilePaths.value = result.missingFilePaths;
      lastSavedAt.value = result.snapshot?.savedAt ?? null;
      autoSaveEnabled = true;
      status.value = "idle";
    } catch (error) {
      loadError.value = formatError(error, "读取本地素材库失败。");
      status.value = "error";
      autoSaveEnabled = true;
    }
  }

  async function loadAvailableLibrary() {
    if (!availableSnapshot.value) {
      await inspectLibrary();
    }
    const snapshot = availableSnapshot.value;
    if (!snapshot) return;

    const missingSet = new Set(availableMissingFilePaths.value);
    missingMaterials.value = snapshot.materials
      .filter((material) => missingSet.has(material.video.filePath))
      .map((material) => ({
        ...material,
        coverPath:
          material.coverPath && !missingSet.has(material.coverPath)
            ? material.coverPath
            : null,
      }));
    missingSegmentCount.value = snapshot.segments.filter((segment) =>
      missingSet.has(segment.path),
    ).length;
    const restored = options.applySnapshot(snapshot, availableMissingFilePaths.value);
    const unavailableCount = missingMaterials.value.length + missingSegmentCount.value;
    restoredSummary.value = unavailableCount > 0
      ? `已载入 ${restored.restoredMaterials} 个视频、${restored.restoredSegments} 个片段，${unavailableCount} 项失效`
      : `已载入 ${restored.restoredMaterials} 个视频、${restored.restoredSegments} 个片段`;
    availableSnapshot.value = null;
    availableMissingFilePaths.value = [];
    status.value = "idle";
  }

  function activateWithCurrentState() {
    autoSaveEnabled = true;
    availableSnapshot.value = null;
    availableMissingFilePaths.value = [];
    restoredSummary.value = "当前项目已同步到本地素材库";
    scheduleSave(0);
  }

  async function relinkMissingMaterial(filePath: string) {
    const missingMaterial = missingMaterials.value.find(
      (material) => material.video.filePath === filePath,
    );
    if (!missingMaterial || relinkingPath.value) return;

    relinkingPath.value = filePath;
    loadError.value = null;
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "视频文件", extensions: ["mp4", "mov", "avi", "mkv"] }],
      });
      if (!selected || Array.isArray(selected)) return;

      const [video] = await loadImportedVideos([selected]);
      if (!video) throw new Error("无法读取重新选择的视频。");
      const replacement: MaterialLibraryMaterial = {
        video,
        coverPath: missingMaterial.coverPath,
      };
      options.onMaterialRelinked(missingMaterial, replacement);
      missingMaterials.value = missingMaterials.value.filter(
        (material) => material.video.filePath !== filePath,
      );
      restoredSummary.value = `已重新定位：${video.fileName}`;
      scheduleSave(0);
    } catch (error) {
      loadError.value = formatError(error, "重新定位素材失败。");
      status.value = "error";
    } finally {
      relinkingPath.value = null;
    }
  }

  function scheduleSave(delayMs = AUTO_SAVE_DELAY_MS) {
    if (!hasMeaningfulState()) return;
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      saveTimer = null;
      void persistCurrentState();
    }, delayMs);
  }

  async function persistCurrentState() {
    if (!autoSaveEnabled || !hasMeaningfulState()) return;
    status.value = "saving";
    saveError.value = null;
    const savedAt = new Date().toISOString();
    const activePaths = new Set(
      options.currentState.value.materials.map((material) => material.video.filePath),
    );
    const retainedMissingMaterials = missingMaterials.value.filter(
      (material) => !activePaths.has(material.video.filePath),
    );
    const snapshot: MaterialLibrarySnapshot = {
      version: 1,
      savedAt,
      ...options.currentState.value,
      materials: [
        ...options.currentState.value.materials,
        ...retainedMissingMaterials,
      ],
    };

    try {
      await saveMaterialLibrary(snapshot);
      lastSavedAt.value = savedAt;
      restoredSummary.value = null;
      status.value = "saved";
    } catch (error) {
      saveError.value = formatError(error, "素材库自动保存失败。");
      status.value = "error";
    }
  }

  function hasMeaningfulState() {
    return (
      options.currentState.value.materials.length > 0 ||
      options.currentState.value.segments.length > 0 ||
      missingMaterials.value.length > 0
    );
  }

  function hasActiveState() {
    return (
      options.currentState.value.materials.length > 0 ||
      options.currentState.value.segments.length > 0
    );
  }

  return {
    activateWithCurrentState,
    hasAvailableLibrary,
    inspectLibrary,
    isLoadingLibrary,
    lastSavedAt,
    loadAvailableLibrary,
    loadError,
    missingMaterials,
    missingSegmentCount,
    relinkingPath,
    relinkMissingMaterial,
    saveError,
    status,
    statusText,
  };
}

function formatError(error: unknown, fallback: string) {
  if (error instanceof Error) return error.message;
  const value = String(error ?? "").trim();
  return value || fallback;
}
