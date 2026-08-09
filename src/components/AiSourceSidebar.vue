<script setup lang="ts">
import { ref } from "vue";
import type { ImportedVideo } from "../types/videoProbe";
import type { AiRemixMatchMode } from "../features/ai-remix";
import MaterialFolderSettingsPanel from "../features/materials/components/MaterialFolderSettingsPanel.vue";
import type { FixedMaterialKind, MaterialFolder, MaterialFolderSettings } from "../features/materials/types";
import ToolIcon from "./ToolIcon.vue";

const props = defineProps<{
  importedVideos: ImportedVideo[];
  selectedVideo: ImportedVideo | null;
  videoCoverUrls: Record<string, string>;
  splitSegmentPaths: string[];
  selectedSegmentPath: string | null;
  segmentThumbnailUrls: Record<string, string>;
  formatDuration: (durationSeconds: number | null) => string;
  formatFileName: (path: string) => string;
  matchMode: AiRemixMatchMode;
  materialFolders: MaterialFolder[];
}>();

const emit = defineEmits<{
  importVideos: [];
  importVideoFolder: [];
  clearVideos: [];
  selectVideo: [video: ImportedVideo];
  selectSegment: [segmentPath: string];
  updateMatchMode: [mode: AiRemixMatchMode];
  removeMaterialFolder: [folderId: string];
  updateMaterialFolderSettings: [folderId: string, patch: Partial<MaterialFolderSettings>];
  selectFixedMaterial: [folderId: string, kind: FixedMaterialKind];
  applyFolderSettingsToAll: [settings: MaterialFolderSettings];
}>();

const expandedFolderIds = ref<string[]>([]);
const settingsFolderId = ref<string | null>(null);

const folderVideos = (folder: MaterialFolder) =>
  props.importedVideos.filter((video) => folder.videoPaths.includes(video.filePath));

function toggleFolder(folderId: string) {
  expandedFolderIds.value = expandedFolderIds.value.includes(folderId)
    ? expandedFolderIds.value.filter((id) => id !== folderId)
    : [...expandedFolderIds.value, folderId];
}

function openFolderSettings(folderId: string) {
  settingsFolderId.value = settingsFolderId.value === folderId ? null : folderId;
  if (!expandedFolderIds.value.includes(folderId)) expandedFolderIds.value.push(folderId);
}

function removeFolder(folder: MaterialFolder) {
  if (window.confirm(`确定移除素材文件夹“${folder.folderName}”吗？`)) {
    emit("removeMaterialFolder", folder.id);
    if (settingsFolderId.value === folder.id) settingsFolderId.value = null;
  }
}
</script>

<template>
  <aside class="replica-source-sidebar" aria-label="素材文件夹">
    <div class="replica-source-toolbar">
       <button class="is-primary" type="button" @click="emit('importVideoFolder')"><ToolIcon name="folder" />导入文件夹</button>
       <button type="button" :disabled="importedVideos.length === 0" aria-label="清空列表" @click="emit('clearVideos')"><ToolIcon name="trash" /></button>
    </div>

    <div v-if="materialFolders.length === 0" class="replica-source-empty">
       <span aria-hidden="true"><ToolIcon name="folder" /></span>
      <strong>暂无文件夹</strong>
      <small>点击导入按钮或拖拽添加素材文件夹</small>
    </div>

    <div v-else class="replica-source-list replica-source-list--folders">
      <article v-for="folder in materialFolders" :key="folder.id" class="replica-material-folder">
        <header class="replica-material-folder__header">
          <button class="replica-material-folder__toggle" type="button" @click="toggleFolder(folder.id)">
             <span class="replica-material-folder__caret" :class="{ 'is-open': expandedFolderIds.includes(folder.id) }"><ToolIcon name="chevron" /></span>
             <span class="replica-material-folder__icon" aria-hidden="true"><ToolIcon name="folder" /></span>
            <strong :title="folder.folderPath">{{ folder.folderName }}</strong>
            <small>{{ folderVideos(folder).length }}</small>
          </button>
          <div class="replica-material-folder__actions">
            <button
              type="button"
              aria-label="打开文件夹详细设置"
              title="文件夹详细设置"
              :class="{ 'is-active': settingsFolderId === folder.id }"
              @click.stop="openFolderSettings(folder.id)"
            ><ToolIcon name="settings" /></button>
             <button type="button" aria-label="移除素材文件夹" title="移除文件夹" @click.stop="removeFolder(folder)"><ToolIcon name="trash" /></button>
          </div>
        </header>

        <div v-if="expandedFolderIds.includes(folder.id) && settingsFolderId !== folder.id" class="replica-material-folder__body">
          <button
            v-for="video in folderVideos(folder)"
            :key="video.id"
            type="button"
            class="replica-material-video"
            :class="{ 'is-active': selectedVideo?.id === video.id }"
            @click="emit('selectVideo', video)"
          >
            <img v-if="videoCoverUrls[video.id]" :src="videoCoverUrls[video.id]" alt="" />
             <span v-else aria-hidden="true"><ToolIcon name="video" /></span>
            <b>{{ video.fileName }}</b>
            <small>{{ formatDuration(video.durationSeconds) }}</small>
          </button>
        </div>

        <MaterialFolderSettingsPanel
          v-if="settingsFolderId === folder.id"
          :settings="folder.settings"
          :apply-to-all-disabled="materialFolders.length < 2"
          @update="emit('updateMaterialFolderSettings', folder.id, $event)"
          @select-fixed-material="emit('selectFixedMaterial', folder.id, $event)"
          @apply-to-all="emit('applyFolderSettingsToAll', folder.settings)"
        />
      </article>

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
           <span v-else aria-hidden="true"><ToolIcon name="video" /></span>
          <b>{{ formatFileName(segmentPath) }}</b>
        </button>
      </div>
    </div>

    <footer class="replica-match-mode">
      <div><strong>匹配模式：</strong><small>{{ props.matchMode === 'local' ? '本地快速匹配，不消耗云端额度。' : '云端模型分析，画面匹配更准确。' }}</small></div>
      <div>
        <button type="button" :class="{ 'is-active': props.matchMode === 'local' }" @click="emit('updateMatchMode', 'local')">本地模型</button>
        <button type="button" :class="{ 'is-active': props.matchMode === 'cloud' }" @click="emit('updateMatchMode', 'cloud')">云端模型</button>
      </div>
    </footer>
  </aside>
</template>
