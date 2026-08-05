<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed } from "vue";
import type { AsrRecognitionResult } from "../../asr";
import { useSubtitleEditor } from "../useSubtitleEditor";

const props = defineProps<{
  configured: boolean;
  sourceFilePath: string | null;
  sourceFileName: string | null;
  isLoadingConfig: boolean;
  isRecognizing: boolean;
  error: string | null;
  result: AsrRecognitionResult | null;
}>();
const emit = defineEmits<{ selectSource: []; recognize: []; clear: []; useText: [text: string] }>();
const editor = useSubtitleEditor(() => props.result);
const previewUrl = computed(() => props.sourceFilePath ? convertFileSrc(props.sourceFilePath) : null);
const isAudio = computed(() => /\.(mp3|wav|m4a|aac|flac|ogg|opus|wma)$/i.test(props.sourceFilePath ?? ""));
</script>

<template>
  <section class="feature-workspace feature-workspace--three subtitle-workspace" aria-label="字幕识别与编辑工作台">
    <aside class="feature-pane feature-file-list">
      <div class="feature-toolbar"><button class="is-primary" type="button" :disabled="isRecognizing" @click="emit('selectSource')">导入文件</button><button type="button" :disabled="!sourceFilePath || isRecognizing" @click="emit('clear')">清空</button></div>
      <div v-if="!sourceFilePath" class="feature-empty-state"><strong>暂无文件</strong><p>导入视频或音频后进行语音识别。</p></div>
      <article v-else class="selected-source-card"><strong>{{ sourceFileName }}</strong><small>{{ sourceFilePath }}</small><span>{{ configured ? '语音服务已配置' : '请先配置语音密钥' }}</span></article>
      <button class="feature-primary-action feature-primary-action--bottom" type="button" :disabled="!configured || !sourceFilePath || isLoadingConfig || isRecognizing" @click="emit('recognize')">{{ isRecognizing ? '正在识别' : result ? '重新识别' : '开始识别' }}</button>
    </aside>

    <section class="feature-pane media-preview-pane">
      <header class="feature-pane__title"><h2>预览</h2><span v-if="result">{{ result.utterances.length }} 条字幕</span></header>
      <div v-if="!previewUrl" class="feature-empty-state"><strong>选择文件后在此预览</strong></div>
      <audio v-else-if="isAudio" class="audio-preview" :src="previewUrl" controls></audio>
      <video v-else class="video-preview" :src="previewUrl" controls></video>
      <section v-if="result" class="subtitle-full-editor">
        <label>完整文字<textarea :value="editor.fullText.value" rows="7" @input="editor.updateFullText(($event.target as HTMLTextAreaElement).value)"></textarea></label>
      </section>
      <p v-if="error || editor.error.value" class="feature-message feature-message--error" role="alert">{{ error ?? editor.error.value }}</p>
      <p v-else-if="editor.feedback.value" class="feature-message" role="status">{{ editor.feedback.value }}</p>
    </section>

    <aside class="feature-pane subtitle-list-pane">
      <header class="feature-pane__title"><h2>字幕列表</h2><span>{{ editor.utterances.value.length }} 条</span></header>
      <div v-if="editor.utterances.value.length === 0" class="feature-empty-state"><strong>暂无数据</strong><p>识别完成后可逐句编辑文字和时间。</p></div>
      <ol v-else class="subtitle-edit-list">
        <li v-for="(item, index) in editor.utterances.value" :key="index">
          <div><label>开始<input v-model.number="item.startTimeMs" type="number" min="0" step="100" /></label><label>结束<input v-model.number="item.endTimeMs" type="number" min="0" step="100" /></label></div>
          <textarea v-model="item.text" rows="2" :aria-label="`第${index + 1}条字幕`"></textarea>
        </li>
      </ol>
      <div class="subtitle-export-bar">
        <button type="button" :disabled="!result" @click="editor.exportSrt">导出 SRT</button>
        <button type="button" :disabled="!result" @click="editor.exportText">导出文案</button>
        <button type="button" :disabled="!result || editor.isSaving.value" @click="editor.saveToLibrary">保存文案库</button>
        <button class="is-primary" type="button" :disabled="!result" @click="emit('useText', editor.fullText.value)">用于AI成片</button>
      </div>
    </aside>
  </section>
</template>
