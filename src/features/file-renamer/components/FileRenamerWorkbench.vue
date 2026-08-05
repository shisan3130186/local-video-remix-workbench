<script setup lang="ts">
import { useFileRenamer } from "../useFileRenamer";

const {
  canRun, datePattern, error, feedback, filePaths, findText, isRunning, mode, padding,
  prefix, preview, replaceText, sequenceFirst, startNumber, suffix, template,
  executeRename, selectFiles, selectFolder,
} = useFileRenamer();
</script>

<template>
  <section class="feature-workspace feature-workspace--three" aria-label="文件批量改名工作台">
    <aside class="feature-pane feature-file-list">
      <div class="feature-toolbar"><button class="is-primary" type="button" @click="selectFiles">选择文件</button><button type="button" @click="selectFolder">文件夹</button><button type="button" :disabled="filePaths.length === 0" @click="filePaths = []">清空</button></div>
      <div v-if="filePaths.length === 0" class="feature-empty-state"><strong>拖拽文件到此处或点击导入</strong><p>支持任意本地文件，不会覆盖已有文件。</p></div>
      <ol v-else class="compact-file-list"><li v-for="path in filePaths" :key="path"><strong>{{ path.split(/[\\/]/).pop() }}</strong><small>{{ path }}</small></li></ol>
    </aside>

    <section class="feature-pane feature-rule-pane">
      <div class="segmented-tabs segmented-tabs--four">
        <button type="button" :class="{ 'is-active': mode === 'uniform' }" @click="mode = 'uniform'">统一命名</button>
        <button type="button" :class="{ 'is-active': mode === 'replace' }" @click="mode = 'replace'">查找替换</button>
        <button type="button" :class="{ 'is-active': mode === 'affix' }" @click="mode = 'affix'">添加前后缀</button>
        <button type="button" :class="{ 'is-active': mode === 'datetime' }" @click="mode = 'datetime'">日期时间</button>
      </div>
      <div v-if="mode === 'uniform'" class="feature-form-grid">
        <label class="is-wide">文件名模板<input v-model="template" type="text" /></label>
        <label>序号位置<select v-model="sequenceFirst"><option :value="false">文件名_001</option><option :value="true">001_文件名</option></select></label>
        <label>起始序号<input v-model.number="startNumber" type="number" min="0" /></label>
        <label>补零位数<input v-model.number="padding" type="number" min="1" max="8" /></label>
      </div>
      <div v-else-if="mode === 'replace'" class="feature-form-grid"><label>查找内容<input v-model="findText" type="text" /></label><label>替换为<input v-model="replaceText" type="text" /></label></div>
      <div v-else-if="mode === 'affix'" class="feature-form-grid"><label>前缀<input v-model="prefix" type="text" /></label><label>后缀<input v-model="suffix" type="text" /></label></div>
      <div v-else class="feature-form-grid"><label>日期格式<select v-model="datePattern"><option value="date">20260804</option><option value="datetime">20260804_153012</option></select></label></div>
      <section class="rename-example"><span>改名示例</span><strong>{{ preview[0]?.newName ?? '产品展示_001.mp4' }}</strong></section>
      <p v-if="error" class="feature-message feature-message--error" role="alert">{{ error }}</p><p v-else-if="feedback" class="feature-message" role="status">{{ feedback }}</p>
    </section>

    <aside class="feature-pane feature-preview-list">
      <header class="feature-pane__title"><h2>改名预览</h2><span>{{ preview.length }} 项</span></header>
      <div v-if="preview.length === 0" class="feature-empty-state"><strong>请先在左侧添加文件</strong></div>
      <ol v-else class="rename-preview-items"><li v-for="item in preview" :key="item.sourcePath" :class="{ 'has-error': item.conflict }"><span>{{ item.oldName }}</span><strong>{{ item.newName }}</strong><small v-if="item.conflict">{{ item.conflict }}</small></li></ol>
      <button class="feature-primary-action feature-primary-action--bottom" type="button" :disabled="!canRun || isRunning" @click="executeRename">{{ isRunning ? '正在执行' : '执行改名' }}</button>
    </aside>
  </section>
</template>
