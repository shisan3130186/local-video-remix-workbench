<script setup lang="ts">
import { onMounted, ref } from "vue";
import { posterTemplates, usePosterMaker } from "../usePosterMaker";

const canvas = ref<HTMLCanvasElement | null>(null);
const maker = usePosterMaker(canvas);
onMounted(maker.render);
</script>

<template>
  <section class="feature-workspace poster-workspace" aria-label="大字报设计工作台">
    <aside class="feature-pane poster-templates">
      <header class="feature-pane__title"><h2>排版预设</h2><button type="button" @click="maker.templateId.value = posterTemplates[Math.floor(Math.random() * posterTemplates.length)].id">随机</button></header>
      <div class="poster-template-grid"><button v-for="item in posterTemplates" :key="item.id" type="button" :class="{ 'is-active': maker.templateId.value === item.id }" :style="{ background: item.background, color: item.primary }" @click="maker.templateId.value = item.id"><strong>{{ item.name }}</strong><span :style="{ color: item.accent }">主标题</span><small>副标题<br />行动号召</small></button></div>
    </aside>

    <section class="feature-pane poster-preview-pane">
      <header class="feature-pane__title"><h2>设计预览</h2><div class="feature-toolbar"><select v-model="maker.aspect.value"><option value="portrait">9:16 竖屏</option><option value="square">1:1 方形</option><option value="landscape">16:9 横屏</option></select><button class="is-primary" type="button" @click="maker.exportPng">导出 PNG</button></div></header>
      <div class="poster-canvas-stage" :class="`is-${maker.aspect.value}`"><canvas ref="canvas"></canvas></div>
      <p v-if="maker.error.value" class="feature-message feature-message--error" role="alert">{{ maker.error.value }}</p><p v-else-if="maker.feedback.value" class="feature-message" role="status">{{ maker.feedback.value }}</p>
    </section>

    <aside class="feature-pane poster-controls">
      <section><h2>文案</h2><textarea v-model="maker.text.value" rows="10" placeholder="每行一段文字"></textarea><small>文案字数：{{ maker.text.value.length }}</small></section>
      <section class="poster-adjustments"><h2>调整</h2><label>整体字号<input v-model.number="maker.fontSize.value" type="range" min="32" max="120" /></label><label>字间距<input v-model.number="maker.letterSpacing.value" type="range" min="0" max="20" /></label><label>行间距<input v-model.number="maker.lineHeight.value" type="range" min="0.9" max="2" step="0.05" /></label><fieldset><legend>对齐方式</legend><div class="choice-buttons"><button type="button" :class="{ 'is-active': maker.align.value === 'left' }" @click="maker.align.value = 'left'">左对齐</button><button type="button" :class="{ 'is-active': maker.align.value === 'center' }" @click="maker.align.value = 'center'">居中</button><button type="button" :class="{ 'is-active': maker.align.value === 'right' }" @click="maker.align.value = 'right'">右对齐</button></div></fieldset></section>
    </aside>
  </section>
</template>
