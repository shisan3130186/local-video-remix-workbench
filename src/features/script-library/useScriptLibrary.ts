import { ref } from "vue";
import type { AsrRecognitionResult } from "../asr";
import {
  deleteScriptLibraryEntry,
  loadScriptLibrary,
  saveScriptLibraryEntry,
} from "./services/scriptLibraryService";
import type { SaveScriptLibraryEntryResult, ScriptLibraryEntry } from "./types";

export function useScriptLibrary() {
  const entries = ref<ScriptLibraryEntry[]>([]);
  const error = ref<string | null>(null);
  const feedback = ref<string | null>(null);
  const isLoading = ref(false);
  const isSaving = ref(false);
  const deletingId = ref<string | null>(null);

  async function loadLibrary() {
    isLoading.value = true;
    error.value = null;
    try {
      entries.value = await loadScriptLibrary();
      return entries.value;
    } catch (loadError) {
      error.value = formatError(loadError, "读取文案库失败。");
      return null;
    } finally {
      isLoading.value = false;
    }
  }

  async function saveAsrResult(
    result: AsrRecognitionResult,
  ): Promise<SaveScriptLibraryEntryResult | null> {
    isSaving.value = true;
    error.value = null;
    feedback.value = null;
    try {
      const saved = await saveScriptLibraryEntry({
        title: buildTitle(result.sourceFileName),
        text: result.text,
        sourcePath: result.sourcePath,
        sourceFileName: result.sourceFileName,
        durationSeconds: result.durationSeconds,
        utterances: result.utterances,
      });
      feedback.value = saved.message;
      if (saved.created) {
        entries.value = [saved.entry, ...entries.value];
      }
      return saved;
    } catch (saveError) {
      error.value = formatError(saveError, "保存文案失败。");
      return null;
    } finally {
      isSaving.value = false;
    }
  }

  async function removeEntry(id: string) {
    deletingId.value = id;
    error.value = null;
    feedback.value = null;
    try {
      entries.value = await deleteScriptLibraryEntry(id);
      feedback.value = "文案已从本机删除。";
      return true;
    } catch (deleteError) {
      error.value = formatError(deleteError, "删除文案失败。");
      return false;
    } finally {
      deletingId.value = null;
    }
  }

  return {
    deleteScriptEntry: removeEntry,
    deletingScriptId: deletingId,
    isLoadingScriptLibrary: isLoading,
    isSavingScript: isSaving,
    loadScriptLibrary: loadLibrary,
    saveAsrToScriptLibrary: saveAsrResult,
    scriptLibraryEntries: entries,
    scriptLibraryError: error,
    scriptLibraryFeedback: feedback,
  };
}

function buildTitle(fileName: string) {
  const baseName = fileName.replace(/\.[^.]+$/, "").trim();
  return `${baseName || "语音识别"}文案`;
}

function formatError(value: unknown, fallback: string) {
  if (value instanceof Error) return value.message;
  return String(value ?? "").trim() || fallback;
}
