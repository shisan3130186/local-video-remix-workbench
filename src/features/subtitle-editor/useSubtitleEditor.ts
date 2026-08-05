import { save } from "@tauri-apps/plugin-dialog";
import { computed, ref, watch, type MaybeRefOrGetter, toValue } from "vue";
import { saveScriptLibraryEntry } from "../script-library/services/scriptLibraryService";
import type { AsrRecognitionResult, AsrUtterance } from "../asr";
import { writeUtf8TextFile } from "./services/subtitleExportService";

export function useSubtitleEditor(resultSource: MaybeRefOrGetter<AsrRecognitionResult | null>) {
  const utterances = ref<AsrUtterance[]>([]);
  const error = ref<string | null>(null);
  const feedback = ref<string | null>(null);
  const isSaving = ref(false);
  const fullText = computed(() => utterances.value.map((item) => item.text.trim()).filter(Boolean).join("\n"));

  watch(
    () => toValue(resultSource),
    (result) => {
      utterances.value = result?.utterances.map((item) => ({ ...item })) ?? [];
      error.value = null;
      feedback.value = null;
    },
    { immediate: true },
  );

  function updateFullText(text: string) {
    const lines = text.split(/\r?\n/);
    if (utterances.value.length === 0) {
      utterances.value = lines.filter(Boolean).map((line, index) => ({ startTimeMs: index * 3000, endTimeMs: (index + 1) * 3000, text: line }));
      return;
    }
    utterances.value = utterances.value.map((item, index) => ({ ...item, text: lines[index] ?? item.text }));
  }

  async function exportSrt() {
    const path = await save({ defaultPath: "智剪字幕.srt", filters: [{ name: "SRT 字幕", extensions: ["srt"] }] });
    if (!path) return;
    await writeOutput(path, buildSrt(utterances.value), "SRT字幕已导出。");
  }

  async function exportText() {
    const path = await save({ defaultPath: "智剪文案.txt", filters: [{ name: "文本文档", extensions: ["txt"] }] });
    if (!path) return;
    await writeOutput(path, fullText.value, "文案已导出。");
  }

  async function saveToLibrary() {
    const result = toValue(resultSource);
    if (!result || !fullText.value) return;
    isSaving.value = true;
    error.value = null;
    try {
      const saved = await saveScriptLibraryEntry({
        title: `${result.sourceFileName.replace(/\.[^.]+$/, "")}字幕文案`,
        text: fullText.value,
        sourcePath: result.sourcePath,
        sourceFileName: result.sourceFileName,
        durationSeconds: result.durationSeconds,
        utterances: utterances.value,
      });
      feedback.value = saved.message;
    } catch (value) {
      error.value = formatError(value, "保存文案库失败。");
    } finally {
      isSaving.value = false;
    }
  }

  async function writeOutput(path: string, content: string, message: string) {
    error.value = null;
    try { await writeUtf8TextFile(path, content); feedback.value = message; }
    catch (value) { error.value = formatError(value, "导出失败。"); }
  }

  return { error, feedback, fullText, isSaving, utterances, exportSrt, exportText, saveToLibrary, updateFullText };
}

function buildSrt(items: AsrUtterance[]) {
  return items.map((item, index) => `${index + 1}\n${formatSrtTime(item.startTimeMs)} --> ${formatSrtTime(item.endTimeMs)}\n${item.text.trim()}\n`).join("\n");
}
function formatSrtTime(ms: number) {
  const safe = Math.max(0, Math.round(ms)); const hours = Math.floor(safe / 3600000); const minutes = Math.floor((safe % 3600000) / 60000); const seconds = Math.floor((safe % 60000) / 1000); const millis = safe % 1000;
  return `${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")},${String(millis).padStart(3, "0")}`;
}
function formatError(value: unknown, fallback: string) { return value instanceof Error ? value.message : String(value ?? "").trim() || fallback; }
