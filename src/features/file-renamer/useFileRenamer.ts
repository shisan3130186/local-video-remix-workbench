import { open } from "@tauri-apps/plugin-dialog";
import { computed, ref } from "vue";
import { batchRenameFiles, listFilesInFolder } from "./services/fileRenamerService";
import type { RenameMode, RenamePreviewItem } from "./types";

export function useFileRenamer() {
  const filePaths = ref<string[]>([]);
  const mode = ref<RenameMode>("uniform");
  const template = ref("产品展示");
  const sequenceFirst = ref(false);
  const startNumber = ref(1);
  const padding = ref(3);
  const findText = ref("");
  const replaceText = ref("");
  const prefix = ref("");
  const suffix = ref("");
  const datePattern = ref<"date" | "datetime">("date");
  const error = ref<string | null>(null);
  const feedback = ref<string | null>(null);
  const isRunning = ref(false);

  const preview = computed<RenamePreviewItem[]>(() => {
    const seen = new Set<string>();
    return filePaths.value.map((sourcePath, index) => {
      const oldName = baseName(sourcePath);
      const extension = extensionOf(oldName);
      const stem = oldName.slice(0, oldName.length - extension.length);
      const sequence = String(Math.max(0, startNumber.value + index)).padStart(clamp(padding.value, 1, 8), "0");
      let nextStem = stem;
      if (mode.value === "uniform") {
        const base = template.value.trim() || "文件";
        nextStem = sequenceFirst.value ? `${sequence}_${base}` : `${base}_${sequence}`;
      } else if (mode.value === "replace") {
        nextStem = findText.value ? stem.split(findText.value).join(replaceText.value) : stem;
      } else if (mode.value === "affix") {
        nextStem = `${prefix.value}${stem}${suffix.value}`;
      } else {
        const stamp = formatDate(new Date(), datePattern.value === "datetime");
        nextStem = `${stem}_${stamp}`;
      }
      const newName = sanitizeFileName(nextStem) + extension;
      const targetPath = joinPath(directoryOf(sourcePath), newName);
      const key = targetPath.toLowerCase();
      let conflict: string | null = null;
      if (!newName.trim() || newName === extension) conflict = "文件名不能为空";
      if (seen.has(key)) conflict = "改名结果重复";
      seen.add(key);
      return { sourcePath, targetPath, oldName, newName, conflict };
    });
  });

  const canRun = computed(() => preview.value.length > 0 && preview.value.every((item) => !item.conflict));

  async function selectFiles() {
    const selected = await open({ multiple: true, directory: false });
    if (!selected) return;
    filePaths.value = dedupe(Array.isArray(selected) ? selected : [selected]);
    feedback.value = null;
  }

  async function selectFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (!selected || Array.isArray(selected)) return;
    try {
      filePaths.value = await listFilesInFolder(selected);
      feedback.value = `已读取 ${filePaths.value.length} 个文件。`;
    } catch (value) {
      error.value = formatError(value, "读取文件夹失败。");
    }
  }

  async function executeRename() {
    if (!canRun.value) return;
    isRunning.value = true;
    error.value = null;
    feedback.value = null;
    try {
      const result = await batchRenameFiles(preview.value.map(({ sourcePath, targetPath }) => ({ sourcePath, targetPath })));
      filePaths.value = result.map((item) => item.targetPath);
      feedback.value = `已完成 ${result.length} 个文件改名。`;
    } catch (value) {
      error.value = formatError(value, "批量改名失败。");
    } finally {
      isRunning.value = false;
    }
  }

  return {
    canRun, datePattern, error, feedback, filePaths, findText, isRunning, mode, padding,
    prefix, preview, replaceText, sequenceFirst, startNumber, suffix, template,
    executeRename, selectFiles, selectFolder,
  };
}

const clamp = (value: number, min: number, max: number) => Math.min(max, Math.max(min, Number(value) || min));
const baseName = (path: string) => path.split(/[\\/]/).pop() ?? path;
const directoryOf = (path: string) => path.slice(0, Math.max(path.lastIndexOf("\\"), path.lastIndexOf("/")));
const extensionOf = (name: string) => { const index = name.lastIndexOf("."); return index > 0 ? name.slice(index) : ""; };
const joinPath = (directory: string, name: string) => `${directory}${directory.includes("\\") ? "\\" : "/"}${name}`;
const dedupe = (paths: string[]) => [...new Set(paths)];
const sanitizeFileName = (value: string) => value.replace(/[<>:"/\\|?*]/g, "_").trim().replace(/[. ]+$/g, "");
function formatDate(date: Date, includeTime: boolean) {
  const parts = [date.getFullYear(), String(date.getMonth() + 1).padStart(2, "0"), String(date.getDate()).padStart(2, "0")];
  if (includeTime) parts.push(String(date.getHours()).padStart(2, "0"), String(date.getMinutes()).padStart(2, "0"), String(date.getSeconds()).padStart(2, "0"));
  return parts.join("");
}
function formatError(value: unknown, fallback: string) { return value instanceof Error ? value.message : String(value ?? "").trim() || fallback; }
