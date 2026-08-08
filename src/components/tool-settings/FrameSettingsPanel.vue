<script setup lang="ts">
import { computed, ref } from "vue";
import { formatFileName, readChecked, readNumber, readSelectedValue } from "./inputHelpers";
import ToolIcon from "../ToolIcon.vue";

type FrameMode = "extract" | "insert" | "mixed";

const props = defineProps<{
  materialFilePath: string | null;
  isMixing: boolean;
  isBatchMixing: boolean;
}>();

const emit = defineEmits<{
  selectMaterialFile: [];
  clearMaterialFile: [];
  reset: [];
}>();

const enabled = ref(false);
const mode = ref<FrameMode>("extract");
const intervalMin = ref(30);
const intervalMax = ref(60);
const frameMin = ref(1);
const frameMax = ref(1);
const opacity = ref(2);
const isBusy = computed(() => props.isMixing || props.isBatchMixing);
const needsMaterial = computed(() => mode.value !== "extract");
</script>

<template>
  <section class="replica-parameter-panel replica-effect-panel replica-effect-panel--frame">
    <header class="replica-parameter-panel__header">
      <h2>视频帧操作 - 参数设置</h2>
      <div class="replica-parameter-panel__actions">
        <button class="replica-parameter-button" type="button" @click="emit('reset')">重置</button>
        <button
          class="replica-parameter-button replica-parameter-button--enable"
          :class="{ 'is-enabled': enabled }"
          type="button"
          :disabled="isBusy"
          @click="enabled = !enabled"
        >{{ enabled ? "已启用" : "启用" }}</button>
      </div>
    </header>

    <div class="replica-effect-mode-row">
      <strong>操作模式</strong>
      <label v-for="option in [['extract', '仅抽帧'], ['insert', '仅插帧'], ['mixed', '混合模式'] ]" :key="option[0]" class="replica-effect-radio">
        <input v-model="mode" :value="option[0]" type="radio" />
        <span>{{ option[1] }}</span>
      </label>
    </div>

    <div class="replica-effect-row">
      <strong>随机间隔</strong>
      <label class="replica-range-field"><input v-model.number="intervalMin" type="number" min="1" /><span>帧</span></label>
      <em>—</em>
      <label class="replica-range-field"><input v-model.number="intervalMax" type="number" min="1" /><span>帧</span></label>
      <small>每隔随机帧数执行一次操作</small>
    </div>

    <div class="replica-effect-row">
      <strong>操作帧数</strong>
      <label class="replica-range-field"><input v-model.number="frameMin" type="number" min="1" /><span>帧</span></label>
      <em>—</em>
      <label class="replica-range-field"><input v-model.number="frameMax" type="number" min="1" /><span>帧</span></label>
    </div>

    <div class="replica-effect-file-row">
      <strong>素材文件</strong>
      <span class="replica-file-display" :title="props.materialFilePath ?? '未选择素材'">{{ props.materialFilePath ? formatFileName(props.materialFilePath) : "未选择素材" }}</span>
      <button class="replica-icon-button" type="button" aria-label="选择视频帧操作素材" title="选择视频帧操作素材" @click="emit('selectMaterialFile')"><ToolIcon name="file" /></button>
      <button class="replica-clear-button" type="button" :disabled="!props.materialFilePath" @click="emit('clearMaterialFile')">清除</button>
    </div>

    <div class="replica-effect-row">
      <strong>素材透明度</strong>
      <label class="replica-range-field"><input v-model.number="opacity" type="number" min="0" max="100" step="1" /></label>
      <span>%</span>
    </div>

    <p v-if="needsMaterial && !props.materialFilePath" class="replica-parameter-warning">在启用插帧模式时，至少选择一个素材，否则无法进行处理，文件夹模式下系统将随机抽取文件夹中的素材。</p>
    <p v-else class="replica-parameter-help">仅抽帧会在随机间隔内删除画面帧；插帧会使用上传素材插入画面，混合模式会同时执行抽帧和插帧。</p>
  </section>
</template>
