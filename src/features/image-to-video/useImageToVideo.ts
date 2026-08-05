import { open } from "@tauri-apps/plugin-dialog";
import { computed, ref } from "vue";
import { listFilesInFolder } from "../file-renamer/services/fileRenamerService";
import { convertImagesToVideos } from "./services/imageToVideoService";
import type { ImageVideoAspect, ImageVideoResult } from "./types";

const IMAGE_PATTERN = /\.(png|jpe?g|webp|bmp|gif)$/i;

export function useImageToVideo() {
  const imagePaths = ref<string[]>([]);
  const selectedIndex = ref(0);
  const outputDirectory = ref<string | null>(null);
  const durationSeconds = ref(5);
  const aspectRatio = ref<ImageVideoAspect>("portrait916");
  const result = ref<ImageVideoResult | null>(null);
  const error = ref<string | null>(null);
  const isRunning = ref(false);
  const selectedPath = computed(() => imagePaths.value[selectedIndex.value] ?? null);

  async function selectImages() {
    const selected = await open({ multiple: true, directory: false, filters: [{ name: "图片", extensions: ["png", "jpg", "jpeg", "webp", "bmp", "gif"] }] });
    if (!selected) return;
    imagePaths.value = (Array.isArray(selected) ? selected : [selected]).filter((path) => IMAGE_PATTERN.test(path));
    selectedIndex.value = 0; result.value = null;
  }
  async function selectFolder() {
    const folder = await open({ directory: true, multiple: false }); if (!folder || Array.isArray(folder)) return;
    imagePaths.value = (await listFilesInFolder(folder)).filter((path) => IMAGE_PATTERN.test(path)); selectedIndex.value = 0; result.value = null;
  }
  async function selectOutputDirectory() {
    const folder = await open({ directory: true, multiple: false }); if (folder && !Array.isArray(folder)) outputDirectory.value = folder;
  }
  async function runConversion() {
    if (!outputDirectory.value || imagePaths.value.length === 0) return;
    isRunning.value = true; error.value = null; result.value = null;
    try { result.value = await convertImagesToVideos({ imagePaths: imagePaths.value, outputDirectory: outputDirectory.value, durationSeconds: durationSeconds.value, aspectRatio: aspectRatio.value }); }
    catch (value) { error.value = value instanceof Error ? value.message : String(value); }
    finally { isRunning.value = false; }
  }
  return { aspectRatio, durationSeconds, error, imagePaths, isRunning, outputDirectory, result, selectedIndex, selectedPath, runConversion, selectFolder, selectImages, selectOutputDirectory };
}
