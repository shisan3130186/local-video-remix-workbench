import { computed, ref } from "vue";
import { saveScriptLibraryEntry } from "../script-library/services/scriptLibraryService";
import { rewriteScripts } from "./services/copyRewriteService";
import type { RewriteRequest, RewriteResult } from "./types";

export function useCopyRewrite() {
  const mode = ref<"single" | "batch">("single");
  const inputText = ref("");
  const style = ref<RewriteRequest["style"]>("conservative");
  const targetLength = ref<RewriteRequest["targetLength"]>("similar");
  const customPrompt = ref("");
  const count = ref(3);
  const result = ref<RewriteResult | null>(null);
  const error = ref<string | null>(null);
  const feedback = ref<string | null>(null);
  const isRunning = ref(false);
  const isSaving = ref(false);

  const scripts = computed(() => {
    const text = inputText.value.trim();
    if (!text) return [];
    return mode.value === "single"
      ? [text]
      : text.split(/\n\s*\n/).map((item) => item.trim()).filter(Boolean);
  });

  async function runRewrite() {
    error.value = null;
    feedback.value = null;
    if (scripts.value.length === 0) {
      error.value = "请先输入需要改写的文案。";
      return;
    }
    isRunning.value = true;
    try {
      result.value = await rewriteScripts({
        scripts: scripts.value,
        style: style.value,
        targetLength: targetLength.value,
        customPrompt: customPrompt.value,
        count: count.value,
      });
      feedback.value = `已完成 ${result.value.items.length} 条文案改写。`;
    } catch (value) {
      error.value = formatError(value, "文案改写失败。");
    } finally {
      isRunning.value = false;
    }
  }

  async function saveVersion(text: string) {
    isSaving.value = true;
    error.value = null;
    try {
      const saved = await saveScriptLibraryEntry({
        title: `AI改写文案 ${new Date().toLocaleString("zh-CN")}`,
        text,
        sourcePath: null,
        sourceFileName: null,
        durationSeconds: null,
        utterances: [],
      });
      feedback.value = saved.message;
    } catch (value) {
      error.value = formatError(value, "保存到文案库失败。");
    } finally {
      isSaving.value = false;
    }
  }

  return {
    count,
    customPrompt,
    error,
    feedback,
    inputText,
    isRunning,
    isSaving,
    mode,
    result,
    scripts,
    style,
    targetLength,
    runRewrite,
    saveVersion,
  };
}

function formatError(value: unknown, fallback: string) {
  if (value instanceof Error) return value.message;
  return String(value ?? "").trim() || fallback;
}
