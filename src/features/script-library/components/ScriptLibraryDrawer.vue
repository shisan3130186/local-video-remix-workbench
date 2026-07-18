<script setup lang="ts">
import type { ScriptLibraryEntry } from "../types";
import "../script-library.css";

defineProps<{
  open: boolean;
  entries: ScriptLibraryEntry[];
  isLoading: boolean;
  deletingId: string | null;
  error: string | null;
  feedback: string | null;
}>();

defineEmits<{
  close: [];
  use: [entry: ScriptLibraryEntry];
  delete: [entry: ScriptLibraryEntry];
}>();

function formatTime(milliseconds: number) {
  return new Date(milliseconds).toLocaleString("zh-CN", { hour12: false });
}
</script>

<template>
  <aside v-if="open" class="info-drawer script-library-drawer" aria-label="本地文案库">
      <header class="drawer-header">
        <div>
          <p class="panel__label">本机保存</p>
          <h2>文案库</h2>
        </div>
        <button class="panel-toggle" type="button" @click="$emit('close')">关闭</button>
      </header>

      <div class="script-library-body">
        <p v-if="isLoading" class="empty-text" role="status">正在读取文案库...</p>
        <p v-if="error" class="error-text" role="alert">{{ error }}</p>
        <p v-if="feedback" class="script-library-feedback" role="status">{{ feedback }}</p>

        <div v-if="!isLoading && entries.length === 0" class="script-library-empty">
          <strong>文案库还是空的</strong>
          <span>完成语音识别后，点击“保存到文案库”即可在这里找到。</span>
        </div>

        <ol v-else class="script-library-list">
          <li v-for="entry in entries" :key="entry.id">
            <div class="script-library-entry__heading">
              <span>
                <strong>{{ entry.title }}</strong>
                <small>{{ formatTime(entry.createdAtMs) }} · {{ entry.text.length }} 字</small>
              </span>
              <small v-if="entry.sourceFileName">来源：{{ entry.sourceFileName }}</small>
            </div>
            <p>{{ entry.text }}</p>
            <div class="script-library-entry__actions">
              <button class="primary-button" type="button" @click="$emit('use', entry)">使用这条文案</button>
              <button
                class="ghost-button"
                type="button"
                :disabled="deletingId === entry.id"
                @click="$emit('delete', entry)"
              >{{ deletingId === entry.id ? "正在删除..." : "删除" }}</button>
            </div>
          </li>
        </ol>
      </div>
  </aside>
</template>
