<script setup lang="ts">
import { ref, watch } from "vue";
import type { SegmentCategory, SegmentCategoryOption } from "../../../services/videoMixService";
import type { ImportedVideo } from "../../../types/videoProbe";

const props = defineProps<{
  isAdvancedMode: boolean;
  importedVideos: ImportedVideo[];
  selectedVideo: ImportedVideo | null;
  isImporting: boolean;
  importError: string | null;
  outputDirectory: string | null;
  outputDirectoryError: string | null;
  splitSegmentPaths: string[];
  selectedSegmentPath: string | null;
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
  selectSegment: [segmentPath: string];
  selectOutputDirectory: [];
  openOutputDirectory: [];
  updateSegmentCategory: [segmentPath: string, category: SegmentCategory | ""];
}>();

const activeTab = ref<"materials" | "segments">("materials");

watch(
  () => props.splitSegmentPaths.length,
  (count, previousCount) => {
    if (count > 0 && previousCount === 0) {
      activeTab.value = "segments";
    }
  },
);

function updateCategory(event: Event, segmentPath: string) {
  emit(
    "updateSegmentCategory",
    segmentPath,
    (event.target as HTMLSelectElement).value as SegmentCategory | "",
  );
}
</script>

<template>
  <aside class="left-rail" aria-label="素材库">
    <section class="panel material-library-panel">
      <div class="material-library-panel__header">
        <div>
          <p class="panel__label">创作素材</p>
          <h2>素材库</h2>
        </div>
        <span class="count-badge">{{ importedVideos.length }} 个视频</span>
      </div>

      <div class="library-tabs" role="tablist" aria-label="素材类型">
        <button
          type="button"
          role="tab"
          :aria-selected="activeTab === 'materials'"
          :class="{ 'library-tab--active': activeTab === 'materials' }"
          @click="activeTab = 'materials'"
        >
          原始素材 <span>{{ importedVideos.length }}</span>
        </button>
        <button
          type="button"
          role="tab"
          :aria-selected="activeTab === 'segments'"
          :class="{ 'library-tab--active': activeTab === 'segments' }"
          @click="activeTab = 'segments'"
        >
          视频片段 <span>{{ splitSegmentPaths.length }}</span>
        </button>
      </div>

      <div v-if="activeTab === 'materials'" class="library-tab-panel" role="tabpanel">
        <div class="material-import-actions">
          <button class="primary-button" type="button" :disabled="isImporting" @click="$emit('importVideos')">
            {{ isImporting ? "正在导入..." : "导入视频" }}
          </button>
          <button class="ghost-button" type="button" :disabled="isImporting" @click="$emit('importVideoFolder')">
            导入文件夹
          </button>
        </div>

        <p v-if="importError" class="error-text">{{ importError }}</p>
        <div v-else-if="importedVideos.length === 0" class="empty-state material-empty-state">
          <span class="empty-state__icon" aria-hidden="true">＋</span>
          <strong>先添加用于混剪的视频</strong>
          <p>支持一次导入多个视频或整个素材文件夹。</p>
        </div>

        <div v-else class="asset-tree">
          <button
            v-for="video in importedVideos"
            :key="video.id"
            class="asset-row"
            :class="{ 'asset-row--active': selectedVideo?.id === video.id }"
            type="button"
            @click="$emit('selectVideo', video)"
          >
            <span class="asset-row__thumb">
              <img v-if="videoCoverUrls[video.id]" :src="videoCoverUrls[video.id]" alt="" />
              <span v-else aria-hidden="true">▶</span>
            </span>
            <span class="asset-row__content">
              <strong>{{ video.fileName }}</strong>
              <small>{{ formatDuration(video.durationSeconds) }} · {{ formatResolution(video) }}</small>
            </span>
          </button>
        </div>
      </div>

      <div v-else class="library-tab-panel" role="tabpanel">
        <div v-if="splitSegmentPaths.length === 0" class="empty-state material-empty-state">
          <span class="empty-state__icon" aria-hidden="true">✂</span>
          <strong>还没有视频片段</strong>
          <p>选择一个视频并完成切片后，片段会自动出现在这里。</p>
        </div>

        <ol v-else class="segment-library-list">
          <li v-for="segmentPath in splitSegmentPaths" :key="segmentPath">
            <button
              class="segment-library-item"
              :class="{ 'segment-library-item--active': selectedSegmentPath === segmentPath }"
              type="button"
              @click="$emit('selectSegment', segmentPath)"
            >
              <img v-if="segmentThumbnailUrls[segmentPath]" :src="segmentThumbnailUrls[segmentPath]" alt="" />
              <span v-else class="segment-library-item__placeholder" aria-hidden="true">▶</span>
              <span>
                <strong>{{ formatFileName(segmentPath) }}</strong>
                <small>{{ randomSelectedSegments.includes(segmentPath) ? "已加入抽取结果" : "可用于 AI 分镜" }}</small>
              </span>
            </button>
            <select
              v-if="isAdvancedMode"
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
      </div>

      <div class="material-summary">
        <div><span>视频</span><strong>{{ importedVideos.length }}</strong></div>
        <div><span>片段</span><strong>{{ splitSegmentPaths.length }}</strong></div>
        <div><span>已抽取</span><strong>{{ randomSelectedSegments.length }}</strong></div>
      </div>
    </section>

    <section class="panel output-mini-panel">
      <div class="output-mini-panel__heading">
        <span>输出位置</span>
        <button class="panel-toggle" type="button" @click="$emit('selectOutputDirectory')">选择目录</button>
      </div>
      <p class="output-path">{{ outputDirectory ?? "生成视频前请选择输出目录" }}</p>
      <button
        v-if="outputDirectory"
        class="output-open-button"
        type="button"
        @click="$emit('openOutputDirectory')"
      >
        打开输出目录 ↗
      </button>
      <p v-if="outputDirectoryError" class="error-text">{{ outputDirectoryError }}</p>
    </section>
  </aside>
</template>
