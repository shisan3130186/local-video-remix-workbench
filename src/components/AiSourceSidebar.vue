<script setup lang="ts">
import { ref } from "vue";
import type { ImportedVideo } from "../types/videoProbe";

defineProps<{
  importedVideos: ImportedVideo[];
  selectedVideo: ImportedVideo | null;
  videoCoverUrls: Record<string, string>;
  splitSegmentPaths: string[];
  selectedSegmentPath: string | null;
  segmentThumbnailUrls: Record<string, string>;
  formatDuration: (durationSeconds: number | null) => string;
  formatFileName: (path: string) => string;
}>();

const emit = defineEmits<{
  importVideos: [];
  importVideoFolder: [];
  clearVideos: [];
  selectVideo: [video: ImportedVideo];
  selectSegment: [segmentPath: string];
}>();

const matchMode = ref<"local" | "cloud">("local");
</script>

<template>
  <aside class="replica-source-sidebar" aria-label="素材文件夹">
    <div class="replica-source-toolbar">
      <button class="is-primary" type="button" @click="emit('importVideoFolder')">▱ 导入文件夹</button>
      <button type="button" :disabled="importedVideos.length === 0" aria-label="清空列表" @click="emit('clearVideos')">⌫</button>
    </div>

    <div v-if="importedVideos.length === 0" class="replica-source-empty">
      <span aria-hidden="true">▱</span>
      <strong>暂无文件夹</strong>
      <small>点击导入按钮或拖拽添加素材文件夹</small>
    </div>

    <div v-else class="replica-source-list">
      <button
        v-for="video in importedVideos"
        :key="video.id"
        type="button"
        :class="{ 'is-active': selectedVideo?.id === video.id }"
        @click="emit('selectVideo', video)"
      >
        <img v-if="videoCoverUrls[video.id]" :src="videoCoverUrls[video.id]" alt="" />
        <span v-else aria-hidden="true">▶</span>
        <b>{{ video.fileName }}</b>
        <small>{{ formatDuration(video.durationSeconds) }}</small>
      </button>

      <div v-if="splitSegmentPaths.length" class="replica-source-segments">
        <p>已生成片段 {{ splitSegmentPaths.length }}</p>
        <button
          v-for="segmentPath in splitSegmentPaths"
          :key="segmentPath"
          type="button"
          :class="{ 'is-active': selectedSegmentPath === segmentPath }"
          @click="emit('selectSegment', segmentPath)"
        >
          <img v-if="segmentThumbnailUrls[segmentPath]" :src="segmentThumbnailUrls[segmentPath]" alt="" />
          <span v-else aria-hidden="true">▶</span>
          <b>{{ formatFileName(segmentPath) }}</b>
        </button>
      </div>
    </div>

    <footer class="replica-match-mode">
      <div><strong>匹配模式：</strong><small>{{ matchMode === 'local' ? '本地模型处理，精度稍低。' : '云端模型分析，画面匹配更准确。' }}</small></div>
      <div>
        <button type="button" :class="{ 'is-active': matchMode === 'local' }" @click="matchMode = 'local'">本地模型</button>
        <button type="button" :class="{ 'is-active': matchMode === 'cloud' }" @click="matchMode = 'cloud'">云端模型</button>
      </div>
    </footer>
  </aside>
</template>
