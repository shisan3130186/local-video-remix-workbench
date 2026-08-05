<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed } from "vue";
import { openPathInFileManager } from "../../../services/fileManagerService";
import { useImageToVideo } from "../useImageToVideo";

const tool = useImageToVideo();
const previewUrl = computed(() => tool.selectedPath.value ? convertFileSrc(tool.selectedPath.value) : null);
</script>

<template>
  <section class="feature-workspace feature-workspace--three image-video-workspace" aria-label="图片转视频工作台">
    <aside class="feature-pane feature-file-list">
      <div class="feature-toolbar"><button class="is-primary" type="button" @click="tool.selectImages">导入图片</button><button type="button" @click="tool.selectFolder">文件夹</button><button type="button" :disabled="tool.imagePaths.value.length === 0" @click="tool.imagePaths.value = []">清空</button></div>
      <div v-if="tool.imagePaths.value.length === 0" class="feature-empty-state"><strong>暂无图片</strong><p>点击导入按钮或拖拽添加图片文件。</p></div>
      <ol v-else class="image-source-list"><li v-for="(path, index) in tool.imagePaths.value" :key="path"><button type="button" :class="{ 'is-active': tool.selectedIndex.value === index }" @click="tool.selectedIndex.value = index"><img :src="convertFileSrc(path)" alt="" /><span>{{ path.split(/[\\/]/).pop() }}</span></button></li></ol>
      <button class="output-path-button" type="button" @click="tool.selectOutputDirectory"><span>输出路径</span><strong>{{ tool.outputDirectory.value ?? '请选择输出目录' }}</strong></button>
    </aside>

    <section class="feature-pane media-preview-pane">
      <header class="feature-pane__title"><h2>预览</h2><span>{{ tool.imagePaths.value.length }} 张</span></header>
      <div v-if="!previewUrl" class="feature-empty-state"><strong>选择图片后在此预览</strong></div><img v-else class="image-main-preview" :src="previewUrl" alt="当前待转换图片" />
    </section>

    <aside class="feature-pane feature-preview-list">
      <header class="feature-pane__title"><h2>输出列表</h2><span v-if="tool.result.value">成功 {{ tool.result.value.items.length }}</span></header>
      <div v-if="!tool.result.value" class="feature-empty-state"><strong>暂无输出</strong><p>处理完成后视频将显示在此。</p></div>
      <ol v-else class="output-result-list"><li v-for="item in tool.result.value.items" :key="item.outputPath"><strong>{{ item.outputPath.split(/[\\/]/).pop() }}</strong><button type="button" @click="openPathInFileManager(item.outputPath)">打开位置</button></li><li v-for="item in tool.result.value.failures" :key="item.sourcePath" class="has-error"><strong>{{ item.sourcePath.split(/[\\/]/).pop() }}</strong><span>{{ item.message }}</span></li></ol>
    </aside>

    <footer class="image-video-control-bar">
      <label>目标时长<input v-model.number="tool.durationSeconds.value" type="number" min="0.5" max="120" step="0.5" /></label>
      <label>分辨率<select v-model="tool.aspectRatio.value"><option value="portrait916">1080×1920 竖屏</option><option value="square11">1080×1080 方形</option><option value="landscape169">1920×1080 横屏</option><option value="original">跟随原图</option></select></label>
      <span>帧率 30fps</span><span>格式 MP4</span><span>编码 自动选择</span>
      <button class="feature-primary-action" type="button" :disabled="tool.isRunning.value || !tool.outputDirectory.value || tool.imagePaths.value.length === 0" @click="tool.runConversion">{{ tool.isRunning.value ? '正在处理' : '开始处理' }}</button>
      <p v-if="tool.error.value" class="feature-message feature-message--error" role="alert">{{ tool.error.value }}</p>
    </footer>
  </section>
</template>
