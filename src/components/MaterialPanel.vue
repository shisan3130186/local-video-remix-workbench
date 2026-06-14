<script setup lang="ts">
import type { ImportedVideo } from "../types/videoProbe";

defineProps<{
  importedVideos: ImportedVideo[];
  selectedVideo: ImportedVideo | null;
  isImporting: boolean;
  importError: string | null;
  outputDirectory: string | null;
  outputDirectoryError: string | null;
  splitSegmentPaths: string[];
  randomSelectedSegments: string[];
  segmentThumbnailUrls: Record<string, string>;
  formatDuration: (durationSeconds: number | null) => string;
  formatResolution: (video: ImportedVideo) => string;
  formatFileName: (path: string) => string;
}>();

defineEmits<{
  importVideos: [];
  importVideoFolder: [];
  selectVideo: [video: ImportedVideo];
  selectOutputDirectory: [];
  openOutputDirectory: [];
}>();
</script>

<template>
  <aside class="left-rail" aria-label="素材和片段">
    <section class="panel panel--stretch">
      <div class="panel__header">
        <div>
          <p class="panel__label">素材</p>
          <h2>视频素材列表</h2>
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
      <p v-else-if="importedVideos.length === 0" class="empty-text">支持 mp4 / mov / avi / mkv。</p>

      <div v-else class="asset-list">
        <article
          v-for="video in importedVideos"
          :key="video.id"
          class="asset-card"
          :class="{ 'asset-card--active': selectedVideo?.id === video.id }"
          tabindex="0"
          role="button"
          @click="$emit('selectVideo', video)"
          @keydown.enter="$emit('selectVideo', video)"
          @keydown.space.prevent="$emit('selectVideo', video)"
        >
          <span class="asset-card__icon">▻</span>
          <div class="asset-card__title">
            <h3>{{ video.fileName }}</h3>
            <span>{{ video.hasAudio ? "有音频" : "无音频" }}</span>
          </div>
        </article>
      </div>
    </section>

    <section class="panel folder-tree-panel">
      <div class="panel__header">
        <div>
          <p class="panel__label">文件夹</p>
          <h2>文件夹树</h2>
        </div>
      </div>
      <div class="folder-tree">
        <div class="folder-tree__item folder-tree__item--active">
          <span>素材库</span>
          <strong>{{ importedVideos.length }}</strong>
        </div>
        <div class="folder-tree__item">
          <span>已切片片段</span>
          <strong>{{ splitSegmentPaths.length }}</strong>
        </div>
        <div class="folder-tree__item">
          <span>已抽取片段</span>
          <strong>{{ randomSelectedSegments.length }}</strong>
        </div>
      </div>
    </section>

    <section class="panel output-mini-panel">
      <div class="panel__header">
        <div>
          <p class="panel__label">输出</p>
          <h2>输出目录</h2>
        </div>
      </div>
      <p v-if="outputDirectory" class="output-path">{{ outputDirectory }}</p>
      <p v-else class="empty-text">请选择导出结果保存位置。</p>
      <button class="ghost-button ghost-button--full" type="button" @click="$emit('selectOutputDirectory')">
        选择输出目录
      </button>
      <button
        class="ghost-button ghost-button--full"
        type="button"
        :disabled="!outputDirectory"
        @click="$emit('openOutputDirectory')"
      >
        打开输出目录
      </button>
      <p v-if="outputDirectoryError" class="error-text">{{ outputDirectoryError }}</p>
    </section>

    <section class="panel">
      <div class="panel__header">
        <div>
          <p class="panel__label">片段</p>
          <h2>切片和抽取结果</h2>
        </div>
        <span class="count-badge">{{ splitSegmentPaths.length }}</span>
      </div>

      <p v-if="splitSegmentPaths.length === 0" class="empty-text">完成固定切片后，这里会出现片段列表。</p>
      <ol v-else class="compact-list">
        <li v-for="segmentPath in splitSegmentPaths" :key="segmentPath">
          <img v-if="segmentThumbnailUrls[segmentPath]" :src="segmentThumbnailUrls[segmentPath]" alt="" />
          <span>{{ formatFileName(segmentPath) }}</span>
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
  </aside>
</template>
