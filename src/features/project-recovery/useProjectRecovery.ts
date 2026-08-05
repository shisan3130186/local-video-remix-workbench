import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import type { Ref } from "vue";
import {
  deleteProjectSnapshot,
  loadProjectSnapshot,
  saveProjectSnapshot,
} from "./services/projectRecoveryService";
import {
  assertValidLoadedSnapshot,
  buildProjectSnapshotEnvelope,
  formatSavedTime,
} from "./snapshotPersistence";
import type {
  ProjectAutoSaveStatus,
  ProjectSnapshotLoadResult,
  ProjectStateSnapshot,
} from "./types";

interface UseProjectRecoveryOptions {
  projectState: Readonly<Ref<ProjectStateSnapshot>>;
  hasMeaningfulState: () => boolean;
  applySnapshot: (result: ProjectSnapshotLoadResult) => string[];
  onNoSnapshot?: () => Promise<void>;
  enabled?: boolean;
}

const AUTO_SAVE_DELAY_MS = 1200;

export function useProjectRecovery(options: UseProjectRecoveryOptions) {
  const pendingResult = ref<ProjectSnapshotLoadResult | null>(null);
  const loadError = ref<string | null>(null);
  const restoreError = ref<string | null>(null);
  const recoveryNotices = ref<string[]>([]);
  const isRestoring = ref(false);
  const status = ref<ProjectAutoSaveStatus>(options.enabled === false ? "idle" : "loading");
  const saveError = ref<string | null>(null);
  const lastSavedAt = ref<string | null>(null);
  let autoSaveEnabled = false;
  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  const statusText = computed(() => {
    if (status.value === "loading") return "正在读取上次项目";
    if (status.value === "saving") return "正在自动保存";
    if (status.value === "error") return "自动保存失败";
    if (status.value === "saved" && lastSavedAt.value) {
      return `已自动保存 ${formatSavedTime(lastSavedAt.value)}`;
    }
    return "自动保存已开启";
  });

  watch(
    options.projectState,
    () => {
      if (autoSaveEnabled) scheduleSave();
    },
    { deep: true },
  );

  onMounted(() => {
    if (options.enabled !== false) void loadLatestSnapshot();
  });

  onBeforeUnmount(() => {
    if (saveTimer) clearTimeout(saveTimer);
  });

  async function loadLatestSnapshot() {
    status.value = "loading";
    loadError.value = null;

    try {
      const result = await loadProjectSnapshot();
      if (result.snapshot) {
        assertValidLoadedSnapshot(result.snapshot);
        pendingResult.value = result;
        lastSavedAt.value = result.snapshot.savedAt;
        return;
      }

      await options.onNoSnapshot?.();
      enableAutoSave();
    } catch (error) {
      loadError.value = formatError(error, "读取上次项目失败。");
      status.value = "error";
    }
  }

  async function restorePendingSnapshot() {
    if (!pendingResult.value?.snapshot) return;

    isRestoring.value = true;
    restoreError.value = null;

    try {
      recoveryNotices.value = options.applySnapshot(pendingResult.value);
      pendingResult.value = null;
      enableAutoSave();
      scheduleSave(0);
    } catch (error) {
      restoreError.value = formatError(error, "恢复上次项目失败。");
    } finally {
      isRestoring.value = false;
    }
  }

  async function discardPendingSnapshot() {
    isRestoring.value = true;
    restoreError.value = null;

    try {
      await deleteProjectSnapshot();
      pendingResult.value = null;
      loadError.value = null;
      recoveryNotices.value = [];
      lastSavedAt.value = null;
      enableAutoSave();
    } catch (error) {
      restoreError.value = formatError(error, "清除上次项目记录失败。");
    } finally {
      isRestoring.value = false;
    }
  }

  function enableAutoSave() {
    autoSaveEnabled = true;
    status.value = "idle";
  }

  function scheduleSave(delayMs = AUTO_SAVE_DELAY_MS) {
    if (!options.hasMeaningfulState()) return;
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      saveTimer = null;
      void persistCurrentState();
    }, delayMs);
  }

  async function persistCurrentState() {
    if (!autoSaveEnabled || !options.hasMeaningfulState()) return;

    status.value = "saving";
    saveError.value = null;
    const savedAt = new Date().toISOString();
    const snapshot = buildProjectSnapshotEnvelope(options.projectState.value, savedAt);

    try {
      await saveProjectSnapshot(snapshot);
      lastSavedAt.value = savedAt;
      status.value = "saved";
    } catch (error) {
      saveError.value = formatError(error, "项目自动保存失败。");
      status.value = "error";
    }
  }

  return {
    discardPendingSnapshot,
    isRestoring,
    lastSavedAt,
    loadError,
    pendingResult,
    recoveryNotices,
    restoreError,
    restorePendingSnapshot,
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
