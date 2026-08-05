<script setup lang="ts">
import { useCopyRewrite } from "../useCopyRewrite";

const emit = defineEmits<{ useText: [text: string] }>();
const {
  count, customPrompt, error, feedback, inputText, isRunning, isSaving,
  mode, result, scripts, style, targetLength, runRewrite, saveVersion,
} = useCopyRewrite();

async function copyText(text: string) {
  await navigator.clipboard.writeText(text);
}
</script>

<template>
  <section class="feature-workspace feature-workspace--split" aria-label="文案改写工作台">
    <section class="feature-pane feature-pane--editor">
      <div class="segmented-tabs" role="tablist" aria-label="文案输入模式">
        <button type="button" :class="{ 'is-active': mode === 'single' }" @click="mode = 'single'">单文案模式</button>
        <button type="button" :class="{ 'is-active': mode === 'batch' }" @click="mode = 'batch'">多文案模式</button>
      </div>
      <textarea
        v-model="inputText"
        class="feature-main-textarea"
        :placeholder="mode === 'single' ? '在这里粘贴需要改写的文案，至少10个字' : '每条文案之间空一行，一次最多30条'"
        aria-label="待改写文案"
      ></textarea>
      <div class="field-meta"><span>{{ inputText.length }} 字</span><span>{{ scripts.length }} 条</span></div>
      <div class="feature-settings-row">
        <fieldset>
          <legend>改写风格</legend>
          <div class="choice-buttons">
            <button type="button" :class="{ 'is-active': style === 'conservative' }" @click="style = 'conservative'">保守型</button>
            <button type="button" :class="{ 'is-active': style === 'creative' }" @click="style = 'creative'">创作型</button>
            <button type="button" :class="{ 'is-active': style === 'custom' }" @click="style = 'custom'">自定义</button>
          </div>
        </fieldset>
        <label>目标长度
          <select v-model="targetLength"><option value="shorter">更精简</option><option value="similar">接近原文</option><option value="longer">适当扩写</option></select>
        </label>
        <label>改写数量
          <input v-model.number="count" type="number" min="1" max="10" />
        </label>
      </div>
      <label v-if="style === 'custom'" class="feature-field">自定义提示词
        <input v-model="customPrompt" type="text" placeholder="例如：口语化，保留数字和产品名称" />
      </label>
      <button class="feature-primary-action feature-primary-action--wide" type="button" :disabled="isRunning || scripts.length === 0" @click="runRewrite">
        {{ isRunning ? "正在改写" : "开始改写" }}
      </button>
      <p v-if="error" class="feature-message feature-message--error" role="alert">{{ error }}</p>
      <p v-else-if="feedback" class="feature-message" role="status">{{ feedback }}</p>
    </section>

    <section class="feature-pane feature-pane--results">
      <header class="feature-pane__title"><h2>改写结果</h2><span v-if="result">{{ result.items.length }} 组</span></header>
      <div v-if="!result" class="feature-empty-state"><strong>改写结果将显示在这里</strong><p>每个版本都可以复制、保存到文案库或直接用于AI成片。</p></div>
      <div v-else class="rewrite-result-list">
        <article v-for="item in result.items" :key="item.sourceIndex" class="rewrite-result-group">
          <details :open="result.items.length === 1"><summary>原文 {{ item.sourceIndex + 1 }}</summary><p>{{ item.sourceText }}</p></details>
          <section v-for="(version, index) in item.versions" :key="index" class="rewrite-version">
            <header><strong>版本 {{ index + 1 }}</strong><span>{{ version.length }} 字</span></header>
            <p>{{ version }}</p>
            <footer>
              <button type="button" @click="copyText(version)">复制</button>
              <button type="button" :disabled="isSaving" @click="saveVersion(version)">保存文案库</button>
              <button class="is-primary" type="button" @click="emit('useText', version)">用于AI成片</button>
            </footer>
          </section>
        </article>
      </div>
    </section>
  </section>
</template>
