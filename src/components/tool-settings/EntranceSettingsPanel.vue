<script setup lang="ts">
import { ref } from "vue";

const effects = [
  ["smooth-up", "平滑上移"], ["smooth-down", "平滑下移"], ["horizontal-squeeze", "水平挤压"], ["vertical-squeeze", "垂直挤压"], ["circle-crop", "圆形裁剪"], ["rectangle-crop", "矩形裁剪"],
  ["circle-close", "圆形闭合"], ["circle-open", "圆形展开"], ["horizontal-close", "水平闭合"], ["horizontal-open", "水平展开"], ["vertical-close", "垂直闭合"], ["vertical-open", "垂直展开"],
  ["left-bottom", "左下对角"], ["right-bottom", "右下对角"], ["left-top", "左上对角"], ["right-top", "右上对角"], ["horizontal-slice", "水平切片"], ["vertical-slice", "垂直切片"],
] as const;

const selectedEffect = ref<(typeof effects)[number][0]>("smooth-up");
const enabled = ref(false);

const emit = defineEmits<{
  reset: [];
}>();

function resetSettings() {
  selectedEffect.value = "smooth-up";
  enabled.value = false;
  emit("reset");
}
</script>

<template>
  <div class="replica-parameter-panel replica-entrance-panel">
    <header class="replica-parameter-panel__header">
      <h2>入场效果 - 参数设置</h2>
      <div class="replica-parameter-panel__actions">
        <button class="replica-parameter-button replica-parameter-button--reset" type="button" @click="resetSettings">重置</button>
        <button class="replica-parameter-button replica-parameter-button--enable" type="button" :class="{ 'is-enabled': enabled }" @click="enabled = true">{{ enabled ? "已启用" : "启用" }}</button>
      </div>
    </header>

    <div class="replica-entrance-options" role="radiogroup" aria-label="入场效果">
      <label v-for="[value, label] in effects" :key="value" class="replica-entrance-option" :class="{ 'is-selected': selectedEffect === value }">
        <input v-model="selectedEffect" type="radio" name="entrance-effect" :value="value" />
        <span>{{ label }}</span>
      </label>
    </div>

    <div class="replica-entrance-preview" aria-live="polite">
      <div :key="selectedEffect" class="replica-entrance-preview__stage" :class="`is-${selectedEffect}`">
        <span>fade</span>
      </div>
    </div>
  </div>
</template>
