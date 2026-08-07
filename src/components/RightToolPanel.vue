<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { FfmpegEnvironmentResult } from "../types/videoProbe";
import type { DrawerKey, ToolKey, WorkspaceMode } from "../types/workbench";
import { findTtsVoice, TTS_LANGUAGE_LABELS, TTS_SCENE_LABELS } from "../features/tts/voiceCatalog";

const props = defineProps<{
  workspaceMode: Exclude<WorkspaceMode, "tools">;
  isAdvancedMode: boolean;
  environment: FfmpegEnvironmentResult | null;
  statusText: string;
  bgmAudioFilePath: string | null;
}>();

const emit = defineEmits<{
  openTool: [tool: ToolKey];
  openDrawer: [drawer: DrawerKey];
  toggleAdvancedMode: [enabled: boolean];
  selectBgmAudioFile: [];
}>();

const originalVolume = defineModel<number>("originalVolume", { required: true });
const bgmEnabled = defineModel<boolean>("bgmEnabled", { required: true });
const bgmVolume = defineModel<number>("bgmVolume", { required: true });
const ttsEnabled = defineModel<boolean>("ttsEnabled", { required: true });
const ttsSpeaker = defineModel<string>("ttsSpeaker", { required: true });
const speechVolume = defineModel<number>("speechVolume", { required: true });
const speechSpeed = defineModel<number>("speechSpeed", { required: true });
const subtitleEnabled = defineModel<boolean>("subtitleEnabled", { required: true });
const subtitlePosition = defineModel<string>("subtitlePosition", { required: true });
const subtitleSize = defineModel<string>("subtitleSize", { required: true });

const activeTab = ref<"basic" | "visual">("basic");
const subtitlePreset = ref(7);
const subtitleFont = ref("MiSans");
const subtitleOpacity = ref(100);
const subtitleStyleExpanded = ref(true);
const subtitleColors = ["#f3f3f3", "#f4d22f", "#50d7b0", "#f08c57", "#76a9ff", "#ef6f91", "#111317", "#ffffff", "#2cc7b1", "#f6b83f", "#db4267", "#9be3d2"];
const bgmFileName = computed(() => props.bgmAudioFilePath?.split(/[\\/]/).pop() ?? "未选择");
const selectedVoice = computed(() => findTtsVoice(ttsSpeaker.value));
const selectedVoiceName = computed(() => selectedVoice.value?.name ?? "小问 2.0");
const selectedVoiceMeta = computed(() => {
  const voice = selectedVoice.value;
  if (!voice) return "中文 · 通用场景";
  const language = voice.languages.length > 1
    ? "多语种"
    : (TTS_LANGUAGE_LABELS[voice.languages[0]] ?? voice.languages[0]);
  const scene = TTS_SCENE_LABELS[voice.scenes[0]] ?? voice.scenes[0];
  return `${language} · ${scene}`;
});
const avatarFailed = ref(false);
const avatarLabel = computed(() => {
  const name = selectedVoiceName.value.trim();
  if (!name) return "?";
  const first = name.charAt(0);
  return /[\u4e00-\u9fff]/.test(first) ? first : name.slice(0, 2).toUpperCase();
});
const avatarGradient = computed(() => {
  const id = selectedVoice.value?.id ?? ttsSpeaker.value;
  let hash = 0;
  for (let index = 0; index < id.length; index += 1) {
    hash = (hash * 31 + id.charCodeAt(index)) >>> 0;
  }
  const palettes = [
    ["#3b6b58", "#65c5a3"],
    ["#315e78", "#6ab3da"],
    ["#7c6031", "#e0b264"],
    ["#7c4857", "#e07a99"],
    ["#5f507f", "#9c8bce"],
    ["#4c5a56", "#9eafaa"],
  ];
  const [from, to] = palettes[hash % palettes.length];
  return `linear-gradient(135deg, ${from}, ${to})`;
});

watch(() => selectedVoice.value?.id ?? ttsSpeaker.value, () => {
  avatarFailed.value = false;
}, { immediate: true });
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
        <div class="replica-bgm-file-row">
          <span :title="props.bgmAudioFilePath ?? '未选择文件'">{{ bgmFileName }}</span>
          <button type="button" :disabled="!bgmEnabled" aria-label="选择本地音乐" title="选择本地音乐" @click="emit('selectBgmAudioFile')">▱</button>
        </div>
        <label class="replica-range-row"><span>BGM音量</span><input v-model.number="bgmVolume" :disabled="!bgmEnabled" type="range" min="0" max="1" step="0.05" /><output>{{ Math.round(bgmVolume * 100) }}%</output></label>
      </section>

      <section class="replica-setting-block">
        <header><strong>语音合成</strong><label class="replica-switch"><input v-model="ttsEnabled" type="checkbox" /><span></span></label></header>
        <button class="replica-voice-row" type="button" :disabled="!ttsEnabled" @click="emit('openTool', 'tts')">
          <span class="replica-avatar" :style="{ background: avatarGradient }">
            <img
              v-if="selectedVoice?.avatarUrl && !avatarFailed"
              :src="selectedVoice.avatarUrl"
              :alt="selectedVoiceName"
              @error="avatarFailed = true"
            />
            <span v-else>{{ avatarLabel }}</span>
          </span>
          <span class="replica-voice-copy"><strong>{{ selectedVoiceName }}</strong><small>{{ selectedVoiceMeta }}</small></span>
          <b aria-hidden="true">⇄</b>
        </button>
        <label class="replica-range-row"><span>原声音量</span><input v-model.number="speechVolume" :disabled="!ttsEnabled" type="range" min="0" max="1" step="0.05" /><output>{{ Math.round(speechVolume * 100) }}%</output></label>
        <label class="replica-range-row"><span>语速调整</span><input v-model.number="speechSpeed" :disabled="!ttsEnabled" type="range" min="0.5" max="2" step="0.05" /><output>{{ speechSpeed.toFixed(1) }}x</output></label>
      </section>

      <section class="replica-setting-block replica-subtitle-block">
        <header><strong>文本/字幕样式</strong><button type="button" @click="subtitleStyleExpanded = !subtitleStyleExpanded">{{ subtitleStyleExpanded ? '收起' : '字幕样式' }}</button></header>
        <div v-show="subtitleStyleExpanded" class="replica-subtitle-controls">
          <label class="replica-select-row"><span>选择字体</span><select v-model="subtitleFont"><option>MiSans</option><option>Noto Sans SC</option><option>Noto Serif SC</option><option>仿宋</option><option>宋体</option><option>微软雅黑</option><option>楷体</option><option>等线</option><option>黑体</option></select></label>
          <label class="replica-select-row"><span>字号大小</span><select v-model="subtitleSize"><option value="small">小号</option><option value="medium">中号</option><option value="large">大号</option></select></label>
          <label class="replica-select-row"><span>字幕位置</span><select v-model="subtitlePosition"><option value="top">顶部</option><option value="middle">居中</option><option value="bottom">底部</option></select></label>
          <label class="replica-range-row"><span>字体透明</span><input v-model.number="subtitleOpacity" type="range" min="10" max="100" /><output>{{ subtitleOpacity }}%</output></label>
          <div class="replica-style-swatches" aria-label="字幕颜色预设">
            <button v-for="(color, index) in subtitleColors" :key="color + index" type="button" :class="{ 'is-active': subtitlePreset === index }" :style="{ color }" :aria-label="`选择字幕颜色 ${index + 1}`" @click="subtitlePreset = index">A</button>
          </div>
          <label class="replica-checkbox-row"><input v-model="subtitleEnabled" type="checkbox" />生成字幕</label>
        </div>
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
