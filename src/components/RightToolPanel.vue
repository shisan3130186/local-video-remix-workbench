<script setup lang="ts">
import { ref } from "vue";
import type { FfmpegEnvironmentResult } from "../types/videoProbe";
import type { DrawerKey, ToolKey, WorkspaceMode } from "../types/workbench";

defineProps<{
  workspaceMode: Exclude<WorkspaceMode, "tools">;
  isAdvancedMode: boolean;
  environment: FfmpegEnvironmentResult | null;
  statusText: string;
}>();

const emit = defineEmits<{
  openTool: [tool: ToolKey];
  openDrawer: [drawer: DrawerKey];
  toggleAdvancedMode: [enabled: boolean];
}>();

const originalVolume = defineModel<number>("originalVolume", { required: true });
const bgmEnabled = defineModel<boolean>("bgmEnabled", { required: true });
const bgmVolume = defineModel<number>("bgmVolume", { required: true });
const ttsEnabled = defineModel<boolean>("ttsEnabled", { required: true });
const ttsSpeaker = defineModel<string>("ttsSpeaker", { required: true });
const speechVolume = defineModel<number>("speechVolume", { required: true });
const speechSpeed = defineModel<number>("speechSpeed", { required: true });
const subtitleEnabled = defineModel<boolean>("subtitleEnabled", { required: true });
const subtitleSize = defineModel<string>("subtitleSize", { required: true });

const activeTab = ref<"basic" | "visual">("basic");
const subtitlePreset = ref(7);
const subtitleColors = ["#f3f3f3", "#f4d22f", "#50d7b0", "#f08c57", "#76a9ff", "#ef6f91", "#111317", "#ffffff", "#2cc7b1", "#f6b83f", "#db4267", "#9be3d2"];
</script>

<template>
  <aside class="replica-settings-rail" aria-label="成片设置">
    <div class="replica-settings-tabs" role="tablist" aria-label="参数分类">
      <button type="button" :class="{ 'is-active': activeTab === 'basic' }" @click="activeTab = 'basic'">基础设置</button>
      <button type="button" :class="{ 'is-active': activeTab === 'visual' }" @click="activeTab = 'visual'">画面处理</button>
    </div>

    <div v-if="activeTab === 'basic'" class="replica-settings-scroll">
      <section class="replica-setting-block replica-volume-row">
        <label for="source-volume">原视频音量</label>
        <input id="source-volume" v-model.number="originalVolume" type="range" min="0" max="2" step="0.05" />
        <output>{{ Math.round(originalVolume * 100) }}%</output>
      </section>

      <section class="replica-setting-block">
        <header><strong>背景音乐</strong><label class="replica-switch"><input v-model="bgmEnabled" type="checkbox" /><span></span></label></header>
        <button class="replica-file-field" type="button" :disabled="!bgmEnabled" @click="emit('openTool', 'bgm')"><span>选择本地音乐</span><b>▢</b></button>
        <label class="replica-range-row"><span>BGM音量</span><input v-model.number="bgmVolume" :disabled="!bgmEnabled" type="range" min="0" max="1" step="0.05" /><output>{{ Math.round(bgmVolume * 100) }}%</output></label>
      </section>

      <section class="replica-setting-block">
        <header><strong>语音合成</strong><label class="replica-switch"><input v-model="ttsEnabled" type="checkbox" /><span></span></label></header>
        <button class="replica-voice-row" type="button" :disabled="!ttsEnabled" @click="emit('openTool', 'tts')">
          <span class="replica-avatar">音</span><span><strong>{{ ttsSpeaker || '小问 2.0' }}</strong><small>中文 · 通用场景</small></span><b>⇄</b>
        </button>
        <label class="replica-range-row"><span>原声音量</span><input v-model.number="speechVolume" :disabled="!ttsEnabled" type="range" min="0" max="1" step="0.05" /><output>{{ Math.round(speechVolume * 100) }}%</output></label>
        <label class="replica-range-row"><span>语速调整</span><input v-model.number="speechSpeed" :disabled="!ttsEnabled" type="range" min="0.5" max="2" step="0.05" /><output>{{ speechSpeed.toFixed(1) }}x</output></label>
      </section>

      <section class="replica-setting-block replica-subtitle-block">
        <header><strong>文本/字幕样式</strong><button type="button" @click="emit('openTool', 'tts')">字幕样式</button></header>
        <label class="replica-select-row"><span>选择字体</span><select><option>MiSans</option><option>微软雅黑</option><option>思源黑体</option></select></label>
        <label class="replica-select-row"><span>字号大小</span><select v-model="subtitleSize"><option value="small">小号</option><option value="medium">中号</option><option value="large">大号</option></select></label>
        <label class="replica-range-row"><span>字体透明</span><input type="range" min="10" max="100" value="100" /><output>100%</output></label>
        <div class="replica-style-swatches">
          <button v-for="(color, index) in subtitleColors" :key="color + index" type="button" :class="{ 'is-active': subtitlePreset === index }" :style="{ color }" @click="subtitlePreset = index">A</button>
        </div>
        <label class="replica-checkbox-row"><input v-model="subtitleEnabled" type="checkbox" />生成字幕</label>
      </section>
    </div>

    <div v-else class="replica-settings-scroll replica-visual-tools">
      <button type="button" @click="emit('openTool', 'canvas')"><span>画布比例</span><small>原画、9:16 与背景填充</small><b>›</b></button>
      <button type="button" @click="emit('openTool', 'effects')"><span>画面调整</span><small>镜像、旋转、亮度、色彩与缩放</small><b>›</b></button>
      <button type="button" @click="emit('openTool', 'transition')"><span>片段衔接</span><small>淡入淡出与短片段过滤</small><b>›</b></button>
      <button type="button" @click="emit('openTool', 'pip')"><span>画中画</span><small>叠加视频或图片素材</small><b>›</b></button>
      <button type="button" @click="emit('openTool', 'watermark')"><span>水印处理</span><small>添加、遮盖、模糊与跟踪</small><b>›</b></button>
      <button type="button" @click="emit('openTool', 'export')"><span>输出参数</span><small>分辨率、帧率、编码与质量</small><b>›</b></button>
      <details class="replica-more-settings">
        <summary>展开全部</summary>
        <button type="button" @click="emit('openTool', 'cover')">视频封面 <b>›</b></button>
        <button type="button" @click="emit('toggleAdvancedMode', !isAdvancedMode)">{{ isAdvancedMode ? '关闭高级显示' : '显示高级参数' }} <b>›</b></button>
      </details>
    </div>

  </aside>
</template>
