import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, ref } from "vue";
import type { Ref } from "vue";
import { generateThumbnail } from "../../services/videoThumbnailService";
import type { ImportedVideo } from "../../types/videoProbe";

interface UseMaterialCoversOptions {
  selectedVideo: Readonly<Ref<ImportedVideo | null>>;
  outputDirectory: Readonly<Ref<string | null>>;
}

export function useMaterialCovers({ selectedVideo, outputDirectory }: UseMaterialCoversOptions) {
  const videoCoverPaths = ref<Record<string, string>>({});
  const selectedCoverPath = ref<string | null>(null);
  const coverFrameSeconds = ref(1);
  const isGeneratingCover = ref(false);
  const coverError = ref<string | null>(null);

  const videoCoverUrls = computed(() => mapFilePathsToUrls(videoCoverPaths.value));
  const selectedCoverUrl = computed(() =>
    selectedCoverPath.value ? convertFileSrc(selectedCoverPath.value) : null,
  );

  function resetMaterialCovers() {
    videoCoverPaths.value = {};
    selectedCoverPath.value = null;
    coverError.value = null;
  }

  function selectVideoCover(video: ImportedVideo) {
    selectedCoverPath.value = videoCoverPaths.value[video.id] ?? null;
    coverError.value = null;
    void ensureVideoCover(video);
  }

  async function ensureVideoCover(video: ImportedVideo) {
    if (videoCoverPaths.value[video.id]) {
      selectedCoverPath.value = videoCoverPaths.value[video.id];
      return;
    }

    try {
      const result = await generateThumbnail(
        video.filePath,
        outputDirectory.value,
        getDefaultCoverTime(video),
        "cover",
      );
      videoCoverPaths.value = {
        ...videoCoverPaths.value,
        [video.id]: result.thumbnailPath,
      };

      if (selectedVideo.value?.id === video.id) {
        selectedCoverPath.value = result.thumbnailPath;
      }
    } catch (error) {
      if (selectedVideo.value?.id === video.id) {
        coverError.value =
          error instanceof Error ? error.message : String(error ?? "生成封面帧失败。");
      }
    }
  }

  async function generateCoverFrame() {
    coverError.value = null;

    if (!selectedVideo.value) {
      coverError.value = "请先选择一个视频素材。";
      return;
    }

    if (!Number.isFinite(coverFrameSeconds.value) || coverFrameSeconds.value < 0) {
      coverError.value = "抽帧时间必须大于或等于 0。";
      return;
    }

    isGeneratingCover.value = true;

    try {
      const result = await generateThumbnail(
        selectedVideo.value.filePath,
        outputDirectory.value,
        coverFrameSeconds.value,
        "selected_cover",
      );
      videoCoverPaths.value = {
        ...videoCoverPaths.value,
        [selectedVideo.value.id]: result.thumbnailPath,
      };
      selectedCoverPath.value = result.thumbnailPath;
    } catch (error) {
      coverError.value =
        error instanceof Error ? error.message : String(error ?? "生成封面帧失败。");
    } finally {
      isGeneratingCover.value = false;
    }
  }

  return {
    coverError,
    coverFrameSeconds,
    generateCoverFrame,
    isGeneratingCover,
    resetMaterialCovers,
    selectVideoCover,
    selectedCoverPath,
    selectedCoverUrl,
    videoCoverPaths,
    videoCoverUrls,
  };
}

function getDefaultCoverTime(video: ImportedVideo) {
  if (video.durationSeconds === null) {
    return 0.1;
  }

  return Math.min(1, Math.max(0.1, video.durationSeconds / 10));
}

function mapFilePathsToUrls(paths: Record<string, string>) {
  return Object.fromEntries(
    Object.entries(paths).map(([key, path]) => [key, convertFileSrc(path)]),
  );
}
