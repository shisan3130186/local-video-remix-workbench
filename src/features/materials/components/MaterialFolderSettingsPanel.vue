<script setup lang="ts">
import { computed } from "vue";
import type {
  FixedMaterialKind,
  MaterialFolderSettings,
} from "../types";
import ToolIcon from "../../../components/ToolIcon.vue";

const props = defineProps<{
  settings: MaterialFolderSettings;
  mode?: "ai" | "batch" | "category";
  disabled?: boolean;
  applyToAllDisabled?: boolean;
}>();

const emit = defineEmits<{
  update: [patch: Partial<MaterialFolderSettings>];
  selectFixedMaterial: [kind: FixedMaterialKind];
  applyToAll: [];
}>();

const fixedMaterialName = computed(() => {
  const path = props.settings.fixedFirstMaterialPath;
  if (!path) return "未选择";
  return path.split(/[\\/]/).filter(Boolean).pop() ?? path;
});

function updateNumber(event: Event) {
  const value = Number((event.target as HTMLInputElement).value);
  emit("update", { variantCount: Number.isFinite(value) ? value : 1 });
}

function updateSettingNumber(key: "clipMinSeconds" | "clipMaxSeconds" | "materialCount" | "exportCount", event: Event) {
  const value = Number((event.target as HTMLInputElement).value);
  emit("update", { [key]: Number.isFinite(value) ? Math.max(0, Math.floor(value)) : 0 });
}
</script>

<template>
  <section class="replica-folder-settings" aria-label="素材文件夹详细设置">
    <div class="replica-folder-settings__body">
      <div class="replica-folder-setting-row" v-if="props.mode === 'batch'">
        <span>混剪模式</span>
        <div class="replica-folder-segmented">
          <button type="button" :class="{ 'is-active': settings.videoMode === 'custom' }" :disabled="disabled" @click="emit('update', { videoMode: 'custom' })">自定义</button>
          <button type="button" :class="{ 'is-active': settings.videoMode === 'script' }" :disabled="disabled" @click="emit('update', { videoMode: 'script' })">文案模式</button>
          <button type="button" :class="{ 'is-active': settings.videoMode === 'audio' }" :disabled="disabled" @click="emit('update', { videoMode: 'audio' })">音频模式</button>
        </div>
      </div>

      <div class="replica-folder-setting-row" v-else-if="props.mode === 'ai'">
        <span>混剪模式</span>
        <div class="replica-folder-segmented">
          <button type="button" :class="{ 'is-active': settings.videoMode === 'script' }" :disabled="disabled" @click="emit('update', { videoMode: 'script' })">文案模式</button>
          <button type="button" :class="{ 'is-active': settings.videoMode === 'audio' }" :disabled="disabled" @click="emit('update', { videoMode: 'audio' })">音频模式</button>
        </div>
      </div>

      <template v-if="(props.mode === 'batch' && settings.videoMode === 'custom') || props.mode === 'category'">
        <div class="replica-folder-setting-row replica-folder-setting-row--range-inputs">
          <span>素材截取范围</span>
          <span class="replica-number-range"><input :value="settings.clipMinSeconds" type="number" min="0" @input="updateSettingNumber('clipMinSeconds', $event)" /><i>秒 -</i><input :value="settings.clipMaxSeconds" type="number" min="0" @input="updateSettingNumber('clipMaxSeconds', $event)" /><i>秒</i></span>
        </div>
        <label v-if="props.mode === 'batch'" class="replica-folder-setting-row replica-folder-setting-row--input">
          <span>使用素材数量</span>
          <span class="replica-number-field"><input :value="settings.materialCount" type="number" min="1" max="999" @input="updateSettingNumber('materialCount', $event)" /><em>个</em></span>
        </label>
        <label class="replica-folder-setting-row replica-folder-setting-row--input">
          <span>混剪导出数量</span>
          <span class="replica-number-field"><input :value="settings.exportCount" type="number" min="1" max="999" @input="updateSettingNumber('exportCount', $event)" /><em>条</em></span>
        </label>
        <div class="replica-folder-setting-row">
          <span>素材抽取方式</span>
          <div class="replica-folder-segmented replica-folder-segmented--compact">
            <button type="button" :class="{ 'is-active': settings.extractionOrder === 'random' }" :disabled="disabled" @click="emit('update', { extractionOrder: 'random' })">随机抽取</button>
            <button type="button" :class="{ 'is-active': settings.extractionOrder === 'ordered' }" :disabled="disabled" @click="emit('update', { extractionOrder: 'ordered' })">顺序抽取</button>
          </div>
        </div>
      </template>

      <div class="replica-folder-setting-row" v-if="props.mode === 'ai' || (props.mode === 'batch' && settings.videoMode !== 'custom')">
        <span>素材抽取方式</span>
        <b>{{ props.mode === 'batch' ? '由 AI 根据文案自动匹配' : '由模型自动匹配' }}</b>
      </div>

      <div class="replica-folder-setting-row" v-if="props.mode === 'ai' || (props.mode === 'batch' && settings.videoMode !== 'custom')">
        <span>使用素材数量</span>
        <b>由模型自动计算</b>
      </div>

      <label class="replica-folder-setting-row replica-folder-setting-row--input" v-if="props.mode === 'ai' || (props.mode === 'batch' && settings.videoMode !== 'custom')">
        <span>每个文案裂变数量</span>
        <span class="replica-number-field"><input :value="settings.variantCount" type="number" min="1" max="10" step="1" :disabled="disabled" @input="updateNumber" /><em>个</em></span>
      </label>

      <div v-if="props.mode !== 'category'" class="replica-folder-setting-row">
        <span>允许素材重复</span>
        <div class="replica-folder-segmented replica-folder-segmented--compact">
          <button type="button" :class="{ 'is-active': settings.allowMaterialRepeat }" :disabled="disabled" @click="emit('update', { allowMaterialRepeat: true })">允许</button>
          <button type="button" :class="{ 'is-active': !settings.allowMaterialRepeat }" :disabled="disabled" @click="emit('update', { allowMaterialRepeat: false })">不允许</button>
        </div>
      </div>

      <div class="replica-folder-setting-row">
        <span>视频输出方式</span>
        <div class="replica-folder-segmented">
          <button type="button" :class="{ 'is-active': settings.outputMode === 'byScript' }" :disabled="disabled" @click="emit('update', { outputMode: 'byScript' })">按文案分类</button>
          <button type="button" :class="{ 'is-active': settings.outputMode === 'flat' }" :disabled="disabled" @click="emit('update', { outputMode: 'flat' })">不分类</button>
        </div>
      </div>

      <div v-if="props.mode !== 'category'" class="replica-folder-fixed-material">
        <div class="replica-folder-setting-row">
          <span>固定首素材</span>
          <div class="replica-folder-segmented replica-folder-segmented--compact">
            <button type="button" :class="{ 'is-active': settings.fixedFirstMaterialEnabled }" :disabled="disabled" @click="emit('update', { fixedFirstMaterialEnabled: true })">启用</button>
            <button type="button" :class="{ 'is-active': !settings.fixedFirstMaterialEnabled }" :disabled="disabled" @click="emit('update', { fixedFirstMaterialEnabled: false })">关闭</button>
          </div>
        </div>
        <div class="replica-fixed-material-path" :title="settings.fixedFirstMaterialPath ?? '未选择'">
          <span>{{ fixedMaterialName }}</span>
          <small v-if="settings.fixedFirstMaterialKind">{{ settings.fixedFirstMaterialKind === 'file' ? '文件' : '文件夹' }}</small>
        </div>
        <div class="replica-fixed-material-actions">
           <button type="button" :disabled="disabled" @click="emit('selectFixedMaterial', 'file')"><ToolIcon name="file" />选择文件</button>
           <button type="button" :disabled="disabled" @click="emit('selectFixedMaterial', 'folder')"><ToolIcon name="folder" />选择文件夹</button>
        </div>
        <div class="replica-folder-setting-row">
          <span>固定方式</span>
          <div class="replica-folder-segmented">
            <button type="button" :class="{ 'is-active': settings.fixedMaterialMode === 'default' }" :disabled="disabled" @click="emit('update', { fixedMaterialMode: 'default' })">默认</button>
            <button type="button" :class="{ 'is-active': settings.fixedMaterialMode === 'independentJoin' }" :disabled="disabled" @click="emit('update', { fixedMaterialMode: 'independentJoin' })">独立拼接</button>
          </div>
        </div>
      </div>
    </div>

    <footer class="replica-folder-settings__footer">
      <button type="button" :disabled="disabled || applyToAllDisabled" @click="emit('applyToAll')">应用到全部文件夹</button>
    </footer>
  </section>
</template>
