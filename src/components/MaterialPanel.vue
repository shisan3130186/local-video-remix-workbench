<script setup lang="ts">
import type { SegmentCategory, SegmentCategoryOption } from "../services/videoMixService";
import type { ImportedVideo } from "../types/videoProbe";

defineProps<{
  isAdvancedMode: boolean;
  importedVideos: ImportedVideo[];
  selectedVideo: ImportedVideo | null;
  isImporting: boolean;
  importError: string | null;
  outputDirectory: string | null;
  outputDirectoryError: string | null;
  splitSegmentPaths: string[];
  randomSelectedSegments: string[];
  segmentCategories: Record<string, SegmentCategory | "">;
  segmentCategoryOptions: SegmentCategoryOption[];
  videoCoverUrls: Record<string, string>;
  segmentThumbnailUrls: Record<string, string>;
  formatDuration: (durationSeconds: number | null) => string;
  formatResolution: (video: ImportedVideo) => string;
  formatFileName: (path: string) => string;
}>();

const emit = defineEmits<{
  importVideos: [];
  importVideoFolder: [];
  selectVideo: [video: ImportedVideo];
  selectOutputDirectory: [];
  openOutputDirectory: [];
  updateSegmentCategory: [segmentPath: string, category: SegmentCategory | ""];
}>();

function updateCategory(event: Event, segmentPath: string) {
  emit("updateSegmentCategory", segmentPath, (event.target as HTMLSelectElement).value as SegmentCategory | "");
}
</script>

<template>
  <aside class="left-rail" :class="{ 'left-rail--simple': !isAdvancedMode }" aria-label="素材和片段">
    <section class="panel material-browser-panel">
      <div class="panel__header material-browser-panel__header">
        <div>
          <p class="panel__label">素材</p>
          <h2>素材树</h2>
        </div>
        <span class="count-badge">{{ importedVideos.length }}</span>
      </div>

      <div class="button-row">
        <button class="ghost-button" type="button" :disabled="isImporting" @click="$emit('importVideos')">
          {{ isImporting ? "正在导入..." : "导入视频" }}
        </button>
        <button class="ghost-button" type="button" :disabled="isImporting" @click="$emit('importVideoFolder')">
          导入文件夹
        </button>
      </div>

      <p v-if="importError" class="error-text">{{ importError }}</p>
      <div v-else-if="importedVideos.length === 0" class="empty-state material-empty-state">
        <span class="empty-state__icon">□</span>
        <strong>暂无视频</strong>
        <p>点击导入视频或导入文件夹添加素材。</p>
      </div>

      <div v-else class="asset-tree">
        <div class="asset-tree__folder">
          <span>素材库</span>
          <strong>{{ importedVideos.length }}</strong>
        </div>
        <button
          v-for="video in importedVideos"
          :key="video.id"
          class="asset-row"
          :class="{ 'asset-row--active': selectedVideo?.id === video.id }"
          type="button"
          @click="$emit('selectVideo', video)"
        >
          <span class="asset-row__icon">▣</span>
          <span class="asset-row__name">{{ video.fileName }}</span>
          <small>{{ formatDuration(video.durationSeconds) }}</small>
        </button>
      </div>

      <div v-if="isAdvancedMode" class="material-summary">
        <div>
          <span>已切片</span>
          <strong>{{ splitSegmentPaths.length }}</strong>
        </div>
        <div>
          <span>已抽取</span>
          <strong>{{ randomSelectedSegments.length }}</strong>
        </div>
      </div>
    </section>

    <section v-if="isAdvancedMode" class="panel segment-browser-panel">
      <div class="panel__header">
        <div>
          <p class="panel__label">切片列表</p>
          <h2>片段分类</h2>
        </div>
        <span class="count-badge">{{ splitSegmentPaths.length }}</span>
      </div>

      <p v-if="splitSegmentPaths.length === 0" class="empty-text">完成固定切片后，这里会出现片段列表。</p>
      <ol v-else class="compact-list compact-list--with-select">
        <li v-for="segmentPath in splitSegmentPaths" :key="segmentPath">
          <img v-if="segmentThumbnailUrls[segmentPath]" :src="segmentThumbnailUrls[segmentPath]" alt="" />
          <span>{{ formatFileName(segmentPath) }}</span>
          <select
            class="segment-category-select"
            :value="segmentCategories[segmentPath] ?? ''"
            aria-label="片段分类"
            @change="updateCategory($event, segmentPath)"
          >
            <option value="">未分类</option>
            <option
              v-for="categoryOption in segmentCategoryOptions"
              :key="categoryOption.key"
              :value="categoryOption.key"
            >
              {{ categoryOption.label }}
            </option>
          </select>
        </li>
      </ol>

      <div v-if="randomSelectedSegments.length > 0" class="sub-block">
        <div class="section-title">
          <span>已抽取</span>
          <strong>{{ randomSelectedSegments.length }}</strong>
        </div>
        <ol class="compact-list compact-list--selected">
          <li v-for="segmentPath in randomSelectedSegments" :key="segmentPath">
            <img v-if="segmentThumbnailUrls[segmentPath]" :src="segmentThumbnailUrls[segmentPath]" alt="" />
            <span>{{ formatFileName(segmentPath) }}</span>
          </li>
        </ol>
      </div>
    </section>

    <section class="panel output-mini-panel">
      <div class="output-dock">
        <span>输出路径：</span>
        <p v-if="outputDirectory" class="output-path">{{ outputDirectory }}</p>
        <p v-else class="output-path">请选择输出目录</p>
        <button class="icon-button" type="button" aria-label="选择输出目录" @click="$emit('selectOutputDirectory')">□</button>
        <button
          class="icon-button"
          type="button"
          aria-label="打开输出目录"
          :disabled="!outputDirectory"
          @click="$emit('openOutputDirectory')"
        >
          ↗
        </button>
      </div>
      <p v-if="outputDirectoryError" class="error-text">{{ outputDirectoryError }}</p>
    </section>
  </aside>
</template>
