<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { SegmentCategory, SegmentCategoryOption } from "../../../services/videoMixService";
import type { MaterialLibraryMaterial } from "../../material-library";
import type { ImportedVideo } from "../../../types/videoProbe";
import type { AiRemixContentAnalysis, AiRemixSegment } from "../../ai-remix/types";
import SegmentContentAnalysisPanel from "./SegmentContentAnalysisPanel.vue";

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
  preparedSegments: AiRemixSegment[];
  isAnalyzingAiContent: boolean;
  aiContentAnalysisProgressText: string | null;
  aiContentAnalysisError: string | null;
  videoCoverUrls: Record<string, string>;
  segmentThumbnailUrls: Record<string, string>;
  materialLibraryStatusText: string;
  materialLibraryError: string | null;
  hasAvailableMaterialLibrary: boolean;
  isLoadingMaterialLibrary: boolean;
  missingMaterials: MaterialLibraryMaterial[];
  missingSegmentCount: number;
  relinkingMaterialPath: string | null;
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
  relinkMissingMaterial: [filePath: string];
  loadMaterialLibrary: [];
  analyzeAiContent: [];
  updateSegmentContentAnalysis: [segmentPath: string, analysis: AiRemixContentAnalysis];
}>();

const activeTab = ref<"materials" | "segments">("materials");
const selectedPreparedSegment = computed(
  () =>
    props.preparedSegments.find((segment) => segment.path === props.selectedSegmentPath) ?? null,
);
const analyzedSegmentCount = computed(
  () => props.preparedSegments.filter((segment) => segment.contentAnalysis).length,
);

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

function updateContentAnalysis(
  segmentPath: string,
  analysis: AiRemixContentAnalysis,
) {
  emit("updateSegmentContentAnalysis", segmentPath, analysis);
}

function updateContentCategory(
  segmentPath: string,
  category: SegmentCategory | "",
) {
  emit("updateSegmentCategory", segmentPath, category);
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

      <div class="material-library-meta">
        <div class="material-library-persistence" role="status">
          <span aria-hidden="true">◉</span>
          <p>{{ materialLibraryStatusText }}</p>
          <button
            v-if="hasAvailableMaterialLibrary && importedVideos.length === 0 && splitSegmentPaths.length === 0"
            class="panel-toggle"
            type="button"
            :disabled="isLoadingMaterialLibrary"
            @click="$emit('loadMaterialLibrary')"
          >
            {{ isLoadingMaterialLibrary ? "正在读取..." : "载入素材库" }}
          </button>
        </div>
        <p v-if="materialLibraryError" class="error-text">{{ materialLibraryError }}</p>

        <details
          v-if="missingMaterials.length > 0 || missingSegmentCount > 0"
          class="missing-materials"
        >
          <summary>
            {{ missingMaterials.length }} 个原视频失效
            <span v-if="missingSegmentCount > 0">，{{ missingSegmentCount }} 个切片失效</span>
          </summary>
          <p>文件可能被移动或删除。重新选择对应视频后，素材库会记住新位置。</p>
          <ul v-if="missingMaterials.length > 0">
            <li v-for="material in missingMaterials" :key="material.video.filePath">
              <span>
                <strong>{{ material.video.fileName }}</strong>
                <small>{{ material.video.filePath }}</small>
              </span>
              <button
                class="panel-toggle"
                type="button"
                :disabled="Boolean(relinkingMaterialPath)"
                @click="$emit('relinkMissingMaterial', material.video.filePath)"
              >
                {{ relinkingMaterialPath === material.video.filePath ? "正在读取..." : "重新定位" }}
              </button>
            </li>
          </ul>
          <p v-if="missingSegmentCount > 0">失效切片不会载入，需要时重新执行切片即可。</p>
        </details>
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
          <strong>{{ hasAvailableMaterialLibrary ? "当前是空白项目" : "先添加用于混剪的视频" }}</strong>
          <p>
            {{ hasAvailableMaterialLibrary ? "可载入本机素材库，或导入一批新视频。" : "支持一次导入多个视频或整个素材文件夹。" }}
          </p>
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

        <template v-else>
          <SegmentContentAnalysisPanel
            :selected-segment="selectedPreparedSegment"
            :selected-category="selectedSegmentPath ? segmentCategories[selectedSegmentPath] ?? '' : ''"
            :category-options="segmentCategoryOptions"
            :analyzed-count="analyzedSegmentCount"
            :total-count="preparedSegments.length"
            :is-analyzing="isAnalyzingAiContent"
            :progress-text="aiContentAnalysisProgressText"
            :error="aiContentAnalysisError"
            @analyze-all="$emit('analyzeAiContent')"
            @update-analysis="updateContentAnalysis"
            @update-category="updateContentCategory"
          />

        <ol class="segment-library-list">
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
        </template>
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
