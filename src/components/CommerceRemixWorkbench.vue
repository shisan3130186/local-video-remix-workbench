<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { AiRemixSegment } from "../features/ai-remix/types";
import type { ImportedVideo } from "../types/videoProbe";
import type { SegmentCategory, SegmentCategoryOption } from "../services/videoMixService";

type Step = 1 | 2 | 3 | 4;

interface CommerceResult {
  path: string;
  label: string;
}

interface CommerceFailure {
  label: string;
  segmentPaths: string[];
}

interface CommerceGenerationPayload {
  variants: Array<{ label: string; segmentPaths: string[] }>;
  subtitle: string;
  subtitleEnabled: boolean;
}

const props = defineProps<{
  importedVideos: ImportedVideo[];
  splitSegmentPaths: string[];
  aiPreparedSegments: AiRemixSegment[];
  segmentCategories: Record<string, SegmentCategory | "">;
  categoryOptions: SegmentCategoryOption[];
  outputDirectory: string | null;
  isImporting: boolean;
  isSplitting: boolean;
  isGenerating: boolean;
  splitError: string | null;
  commerceError: string | null;
  commerceResults: CommerceResult[];
  commerceFailures: CommerceFailure[];
}>();

const emit = defineEmits<{
  (event: "import-videos"): void;
  (event: "select-output-directory"): void;
  (event: "split-materials"): void;
  (event: "update-category", path: string, category: SegmentCategory | ""): void;
  (event: "generate", payload: CommerceGenerationPayload): void;
  (event: "retry-failures", payload: Omit<CommerceGenerationPayload, "variants">): void;
  (event: "open-result", path: string): void;
}>();

const step = ref<Step>(1);
const selectedTemplate = ref("problem");
const subtitleEnabled = ref(true);
const productName = ref("玻璃清洁喷雾");
const audience = ref("家里有玻璃水渍、想快速清洁的家庭用户");
const painPoint = ref("玻璃上的水渍和手印很难擦干净，普通抹布擦完还会留下痕迹。");
const sellingPoints = ref("喷一下就能软化水渍；擦完不容易留痕；适合窗户、镜子和淋浴房。");
const callToAction = ref("现在下单，家里的玻璃马上亮起来");
const shotOrder = ref<string[]>([]);

const stepTitle = computed(() => ({
  1: "先说清楚要卖什么",
  2: "让系统知道每段素材在做什么",
  3: "先看懂，再生成",
  4: "多条版本，直接挑最能卖的",
}[step.value]));

const preparedByPath = computed(() => new Map(props.aiPreparedSegments.map((segment) => [segment.path, segment])));
const visiblePaths = computed(() => props.splitSegmentPaths.length > 0
  ? props.splitSegmentPaths
  : props.importedVideos.map((video) => video.filePath));
const shots = computed(() => shotOrder.value
  .map((path) => {
    const prepared = preparedByPath.value.get(path);
    return {
      path,
      title: prepared?.description?.trim() || path.split(/[\\/]/).pop() || "未命名素材",
      duration: prepared?.durationSeconds ? `${prepared.durationSeconds.toFixed(1)} 秒` : "待读取时长",
      thumbnailUrl: prepared?.thumbnailUrl ?? "",
      category: props.segmentCategories[path] ?? "",
    };
  })
  .filter((shot) => shot.path));

const hasCompleteProductBrief = computed(() => [
  productName.value,
  audience.value,
  painPoint.value,
  sellingPoints.value,
  callToAction.value,
].every((value) => value.trim().length > 0));
const canContinueFromStep1 = computed(() => props.importedVideos.length > 0 && hasCompleteProductBrief.value);
const canContinueFromStep2 = computed(() => props.splitSegmentPaths.length >= 2);
const hasOutputDirectory = computed(() => Boolean(props.outputDirectory));
const previewVideoUrl = computed(() => shots.value[0]?.thumbnailUrl ?? "");
const previewMediaUrl = computed(() => {
  const path = shots.value[0]?.path ?? props.importedVideos[0]?.filePath;
  if (!path) return "";
  try {
    return convertFileSrc(path);
  } catch {
    return "";
  }
});

watch(visiblePaths, (paths) => {
  const existing = new Set(paths);
  shotOrder.value = [
    ...shotOrder.value.filter((path) => existing.has(path)),
    ...paths.filter((path) => !shotOrder.value.includes(path)),
  ];
}, { immediate: true });

function moveShot(index: number, direction: -1 | 1) {
  const target = index + direction;
  if (target < 0 || target >= shotOrder.value.length) return;
  const next = [...shotOrder.value];
  [next[index], next[target]] = [next[target], next[index]];
  shotOrder.value = next;
}

function nextStep() {
  if (step.value === 1 && canContinueFromStep1.value) step.value = 2;
  else if (step.value === 2 && canContinueFromStep2.value) step.value = 3;
  else if (step.value === 3 && canContinueFromStep2.value) step.value = 4;
}

function previousStep() {
  if (step.value > 1) step.value = (step.value - 1) as Step;
}

function goToStep(target: Step) {
  if (props.isGenerating || props.isSplitting) return;
  if (target === 1 || (target === 2 && canContinueFromStep1.value) || (target >= 3 && canContinueFromStep2.value)) {
    step.value = target;
  }
}

function orderByCategory(paths: string[]) {
  const priority: SegmentCategory[] = ["hook", "talking", "product", "usage", "detail", "result", "ending", "environment"];
  return paths
    .map((path, index) => ({ path, index, priority: priority.indexOf(props.segmentCategories[path] as SegmentCategory) }))
    .sort((left, right) => {
      const leftPriority = left.priority === -1 ? priority.length : left.priority;
      const rightPriority = right.priority === -1 ? priority.length : right.priority;
      return leftPriority - rightPriority || left.index - right.index;
    })
    .map((entry) => entry.path);
}

function movePathFirst(paths: string[], preferredPath: string | undefined) {
  if (!preferredPath) return paths;
  return [preferredPath, ...paths.filter((path) => path !== preferredPath)];
}

function buildVariants(): CommerceGenerationPayload["variants"] {
  const paths = shotOrder.value.length > 0 ? [...shotOrder.value] : [...props.splitSegmentPaths];
  if (paths.length < 3) return [];
  const base = orderByCategory(paths);
  const resultLead = base.find((path) => ["result", "usage"].includes(props.segmentCategories[path] ?? "")) ?? base[base.length - 1];
  const talkingLead = base.find((path) => ["talking", "hook"].includes(props.segmentCategories[path] ?? "")) ?? base[1];
  const candidates = {
    problem: base,
    result: movePathFirst(base, resultLead),
    talking: movePathFirst(base, talkingLead),
    rotateOne: [...base.slice(1), base[0]],
    rotateTwo: [...base.slice(2), ...base.slice(0, 2)],
    reverse: [...base].reverse(),
  };
  const priority = selectedTemplate.value === "result"
    ? [candidates.result, candidates.problem, candidates.talking, candidates.rotateOne, candidates.rotateTwo, candidates.reverse]
    : selectedTemplate.value === "talking"
      ? [candidates.talking, candidates.problem, candidates.result, candidates.rotateOne, candidates.rotateTwo, candidates.reverse]
      : [candidates.problem, candidates.result, candidates.talking, candidates.rotateOne, candidates.rotateTwo, candidates.reverse];
  const uniqueOrders: string[][] = [];
  const seen = new Set<string>();
  for (const candidate of priority) {
    const key = candidate.join("\u0001");
    if (!seen.has(key)) {
      seen.add(key);
      uniqueOrders.push(candidate);
    }
    if (uniqueOrders.length === 3) break;
  }
  return uniqueOrders.map((segmentPaths, index) => ({
    label: String.fromCharCode(65 + index),
    segmentPaths,
  }));
}

function buildSubtitle() {
  return [
    `${productName.value.trim()}，面向${audience.value.trim()}。`,
    painPoint.value.trim(),
    sellingPoints.value.trim(),
    callToAction.value.trim(),
  ].filter(Boolean).join(" ");
}

function generateVideos() {
  const variants = buildVariants();
  if (variants.length !== 3 || !hasCompleteProductBrief.value) return;
  emit("generate", {
    variants,
    subtitle: buildSubtitle(),
    subtitleEnabled: subtitleEnabled.value,
  });
}

function retryFailedVideos() {
  if (props.commerceFailures.length === 0) return;
  emit("retry-failures", { subtitle: buildSubtitle(), subtitleEnabled: subtitleEnabled.value });
}

function resultVideoUrl(path: string) {
  try {
    return convertFileSrc(path);
  } catch {
    return "";
  }
}
</script>

<template>
  <main class="commerce-workbench">
    <header class="commerce-workbench__header">
      <div>
        <p class="commerce-workbench__eyebrow">SmartCut / 创作中心</p>
        <h1>信息流带货成片</h1>
        <p>把商品信息、素材分类和信息流节奏放进同一条成片流程。</p>
      </div>
      <span class="commerce-workbench__status">真实处理 · {{ hasOutputDirectory ? '已选择输出目录' : '先选择输出目录' }}</span>
    </header>

    <nav class="commerce-steps" aria-label="信息流带货成片步骤">
      <button v-for="item in [{ value: 1, label: '商品信息' }, { value: 2, label: '素材归类' }, { value: 3, label: '自动分镜' }, { value: 4, label: '批量成片' }]" :key="item.value" type="button" :disabled="props.isGenerating || props.isSplitting" :class="{ 'is-active': step === item.value, 'is-done': step > item.value }" @click="goToStep(item.value as Step)">
        <span>{{ item.value }}</span>{{ item.label }}
      </button>
    </nav>

    <section class="commerce-workbench__heading">
      <div>
        <h2>{{ stepTitle }}</h2>
        <p v-if="step === 1">只填写影响成片的关键信息，复杂剪辑设置放到后面。</p>
        <p v-else-if="step === 2">先导入并切片，再对系统识别的分类进行人工确认。</p>
        <p v-else-if="step === 3">每一行就是成片中的一个镜头，顺序会真实影响导出结果。</p>
        <p v-else>每条版本会沿用当前商品信息和镜头顺序，并输出到已选目录。</p>
      </div>
      <span>原素材不会被覆盖</span>
    </section>

    <section v-if="step === 1" class="commerce-layout">
      <article class="commerce-panel commerce-panel--form">
        <header><strong>商品与目标</strong><small>1 / 4</small></header>
        <div class="commerce-panel__body">
          <label>商品名称<input v-model="productName" /></label>
          <label>给谁看<input v-model="audience" /></label>
          <label>用户痛点<textarea v-model="painPoint" rows="3" /></label>
          <label>必须讲清楚的卖点<textarea v-model="sellingPoints" rows="3" /></label>
          <label>结尾引导<input v-model="callToAction" /></label>
          <div class="commerce-import-actions"><button type="button" class="commerce-button" :disabled="props.isImporting || props.isSplitting || props.isGenerating" @click="emit('import-videos')">{{ props.isImporting ? '导入中…' : '导入视频素材' }}</button><button type="button" class="commerce-button" :disabled="props.isSplitting || props.isGenerating" @click="emit('select-output-directory')">{{ hasOutputDirectory ? '更换输出目录' : '选择输出目录' }}</button></div>
        </div>
      </article>
      <article class="commerce-panel commerce-preview-panel">
        <header><strong>信息流结构预览</strong><small>不随机拼接</small></header>
        <div class="commerce-preview commerce-preview--portrait">
          <video v-if="previewMediaUrl" :src="previewMediaUrl" autoplay muted loop playsinline />
          <img v-else-if="previewVideoUrl" :src="previewVideoUrl" alt="当前素材预览" />
          <span>9:16</span><strong>{{ props.importedVideos.length > 0 ? '已导入原视频预览' : '导入素材后显示原视频预览' }}</strong><small>会先切片，再按信息流顺序成片</small>
        </div>
        <footer><span>预计 {{ shots.length > 0 ? `${shots.length * 4} 秒` : '待切片' }}</span><span>字幕 + 原声</span></footer>
      </article>
      <article class="commerce-panel commerce-panel--wide">
        <header><strong>选择带货结构</strong><small>模板会影响镜头顺序</small></header>
        <div class="commerce-template-grid">
          <button v-for="template in [{ key: 'problem', title: '问题 → 解决方案', text: '适合日用品和功能型商品', flow: '痛点 / 产品 / 演示 / 效果 / 引导' }, { key: 'result', title: '结果先行', text: '先展示变化，再解释为什么有效', flow: '结果 / 原因 / 卖点 / 对比 / 引导' }, { key: 'talking', title: '人物口播', text: '适合达人出镜和直播切片', flow: '口播 / 特写 / 使用 / 反馈 / 引导' }]" :key="template.key" type="button" :class="{ 'is-selected': selectedTemplate === template.key }" @click="selectedTemplate = template.key">
            <strong>{{ template.title }}</strong><small>{{ template.text }}</small><em>{{ template.flow }}</em>
          </button>
        </div>
      </article>
    </section>

    <section v-else-if="step === 2" class="commerce-layout commerce-layout--two">
      <article class="commerce-panel">
        <header><strong>素材片段</strong><button type="button" class="commerce-button" :disabled="props.isImporting || props.isSplitting || props.isGenerating" @click="emit('import-videos')">{{ props.isImporting ? '导入中…' : '导入更多' }}</button></header>
        <div class="commerce-panel__body commerce-assets">
          <div v-if="props.importedVideos.length === 0" class="commerce-empty">还没有导入视频，请先导入多条原素材。</div>
          <div v-for="video in props.importedVideos" :key="video.id" class="commerce-asset"><span class="commerce-asset__thumb">视频</span><span><strong>{{ video.fileName }}</strong><small>{{ video.durationSeconds ? `${video.durationSeconds.toFixed(1)} 秒` : '时长读取中' }}</small></span><em>原素材</em></div>
        </div>
      </article>
      <article class="commerce-panel">
        <header><strong>信息流镜头分类</strong><small>{{ props.splitSegmentPaths.length }} 个切片</small></header>
        <div class="commerce-panel__body commerce-categories">
          <div v-if="props.splitSegmentPaths.length === 0" class="commerce-empty">点击下方“开始切片并初步归类”，使用现有 FFmpeg 切片。</div>
          <div v-for="shot in shots" :key="shot.path"><strong>{{ shot.title }}</strong><select :value="shot.category" aria-label="镜头分类" @change="emit('update-category', shot.path, ($event.target as HTMLSelectElement).value as SegmentCategory)"><option value="">待确认</option><option v-for="option in props.categoryOptions" :key="option.key" :value="option.key">{{ option.label }}</option></select></div>
        </div>
      </article>
      <p v-if="props.splitError || props.commerceError" class="commerce-results__notice">{{ props.splitError || props.commerceError }}</p>
      <div class="commerce-inline-actions"><button type="button" class="commerce-button" :disabled="props.isSplitting || props.isGenerating || props.importedVideos.length === 0 || !hasOutputDirectory" @click="emit('split-materials')">{{ props.isSplitting ? '切片与归类中…' : '开始切片并初步归类' }}</button><button type="button" class="commerce-button" :disabled="props.isSplitting || props.isGenerating" @click="emit('select-output-directory')">{{ hasOutputDirectory ? '更换输出目录' : '选择输出目录' }}</button></div>
    </section>

    <section v-else-if="step === 3" class="commerce-layout commerce-layout--two">
      <article class="commerce-panel">
        <header><strong>自动分镜 · {{ shots.length }} 个镜头</strong><small>拖动顺序会参与导出</small></header>
        <div class="commerce-panel__body commerce-shots">
          <div v-if="shots.length < 2" class="commerce-empty">至少切出 2 个片段后才能生成信息流成片。</div>
          <div v-for="(shot, index) in shots" :key="shot.path" class="commerce-shot"><span class="commerce-shot__number">{{ String(index + 1).padStart(2, '0') }}</span><span class="commerce-shot__thumb"><img v-if="shot.thumbnailUrl" :src="shot.thumbnailUrl" alt="镜头缩略图" /><span v-else>视频</span></span><span><strong>{{ shot.title }}</strong><small>{{ shot.category || '待确认分类' }} · {{ shot.duration }}</small></span><span class="commerce-shot__actions"><button type="button" :disabled="index === 0" aria-label="上移镜头" @click="moveShot(index, -1)">↑</button><button type="button" :disabled="index === shots.length - 1" aria-label="下移镜头" @click="moveShot(index, 1)">↓</button></span></div>
        </div>
      </article>
      <article class="commerce-panel commerce-preview-panel"><header><strong>当前成片预览</strong><button type="button" class="commerce-button" @click="subtitleEnabled = !subtitleEnabled">{{ subtitleEnabled ? '隐藏字幕' : '显示字幕' }}</button></header><div class="commerce-preview commerce-preview--portrait"><video v-if="previewMediaUrl" :src="previewMediaUrl" autoplay muted loop playsinline /><img v-else-if="previewVideoUrl" :src="previewVideoUrl" alt="当前成片镜头" /><span>9:16</span><strong>{{ subtitleEnabled ? painPoint : '字幕已隐藏' }}</strong><small>镜头顺序和字幕会写入真实导出</small></div></article>
    </section>

    <section v-else class="commerce-results">
      <article v-if="props.commerceResults.length === 0" class="commerce-results__empty">还没有成片结果。确认分镜后点击下方“生成 3 条成片”。</article>
      <article v-for="(result, index) in props.commerceResults" :key="result.path" class="commerce-result"><div class="commerce-preview commerce-preview--result"><video v-if="resultVideoUrl(result.path)" :src="resultVideoUrl(result.path)" muted loop playsinline controls /><span>版本 {{ String.fromCharCode(65 + index) }}</span><strong>{{ result.label }}</strong></div><div><strong>{{ result.label }}</strong><small>{{ result.path }}</small><button type="button" class="commerce-button" @click="emit('open-result', result.path)">打开结果</button></div></article>
      <div v-if="props.commerceFailures.length > 0" class="commerce-results__retry"><strong>{{ props.commerceFailures.length }} 个版本导出失败</strong><button type="button" class="commerce-button commerce-button--primary" :disabled="props.isGenerating" @click="retryFailedVideos">重试失败版本（{{ props.commerceFailures.length }}）</button></div>
      <p v-if="props.commerceError" class="commerce-results__notice">{{ props.commerceError }}</p>
    </section>

    <footer class="commerce-workbench__footer">
      <button type="button" class="commerce-button" :disabled="step === 1" @click="previousStep">← 返回上一步</button>
      <span>第 {{ step }} / 4 步</span>
      <button v-if="step < 3" type="button" class="commerce-button commerce-button--primary" :disabled="step === 1 ? !canContinueFromStep1 : !canContinueFromStep2" @click="nextStep">继续下一步 →</button>
      <button v-else-if="step === 3" type="button" class="commerce-button commerce-button--primary" :disabled="props.isGenerating || shots.length < 2 || !hasOutputDirectory" @click="step = 4">确认分镜，查看成片 →</button>
      <button v-else type="button" class="commerce-button commerce-button--primary" :disabled="props.isGenerating || shots.length < 2 || !hasOutputDirectory" @click="generateVideos">{{ props.isGenerating ? '正在生成…' : '生成 3 条成片' }}</button>
    </footer>
  </main>
</template>

<style scoped>
.commerce-workbench { min-height: calc(100vh - 48px); overflow: auto; padding: 32px clamp(22px, 4vw, 64px) 42px; color: var(--text-primary, #eef0f4); background: var(--app-background, #171a20); }
.commerce-workbench__header, .commerce-workbench__heading, .commerce-workbench__footer, .commerce-steps { max-width: 1240px; margin: 0 auto; }
.commerce-workbench__header { display: flex; align-items: flex-start; justify-content: space-between; gap: 24px; padding-bottom: 20px; border-bottom: 1px solid var(--border-subtle, #353a43); }
.commerce-workbench__eyebrow { margin: 0 0 8px; color: var(--text-muted, #979da8); font-size: 12px; }
.commerce-workbench h1, .commerce-workbench h2 { margin: 0; letter-spacing: -.02em; }
.commerce-workbench h1 { font-size: 25px; }
.commerce-workbench__header p:last-child, .commerce-workbench__heading p { margin: 7px 0 0; color: var(--text-muted, #979da8); font-size: 12px; }
.commerce-workbench__status { padding: 7px 10px; border: 1px solid var(--commerce-accent, var(--theme-accent, #d5b46f)); border-radius: 6px; color: var(--commerce-accent, var(--theme-accent, #d5b46f)); background: var(--commerce-accent-soft, var(--theme-accent-soft, rgba(105, 82, 41, .23))); font-size: 11px; white-space: nowrap; }
.commerce-steps { display: flex; align-items: center; gap: 10px; padding: 18px 0 0; }
.commerce-steps button { border: 0; background: transparent; color: var(--text-muted, #979da8); font-size: 12px; }
.commerce-steps button span { display: inline-grid; place-items: center; width: 25px; height: 25px; margin-right: 7px; border: 1px solid var(--commerce-border-strong, var(--theme-border-strong, #3a404a)); border-radius: 50%; }
.commerce-steps button.is-active { color: var(--text-primary, #eef0f4); }
.commerce-steps button.is-active span { color: var(--theme-panel, #20242b); background: var(--commerce-accent, var(--theme-accent, #d5b46f)); border-color: var(--commerce-accent, var(--theme-accent, #d5b46f)); }
.commerce-steps button.is-done span { color: var(--theme-text-soft, #c7cbd2); border-color: var(--commerce-border-strong, var(--theme-border-strong, #3a404a)); }
.commerce-workbench__heading { display: flex; justify-content: space-between; align-items: end; gap: 20px; padding: 26px 0 17px; }
.commerce-workbench__heading h2 { font-size: 22px; }
.commerce-workbench__heading > span { color: var(--text-muted, #979da8); font-size: 11px; }
.commerce-layout { max-width: 1240px; margin: 0 auto; display: grid; grid-template-columns: minmax(0, 1.12fr) minmax(300px, .88fr); gap: 14px; }
.commerce-layout--two { grid-template-columns: minmax(0, 1fr) minmax(300px, .78fr); }
.commerce-panel { min-width: 0; border: 1px solid var(--border-subtle, #353a43); border-radius: 10px; background: var(--surface, #20242b); }
.commerce-panel--wide { grid-column: 1 / -1; }
.commerce-panel > header { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 14px 16px; border-bottom: 1px solid var(--border-subtle, #353a43); }
.commerce-panel > header strong { font-size: 13px; }
.commerce-panel > header small { color: var(--text-muted, #979da8); font-size: 11px; }
.commerce-panel__body { padding: 16px; }
.commerce-panel--form label { display: grid; gap: 6px; margin-bottom: 12px; color: var(--text-muted, #979da8); font-size: 11px; }
.commerce-panel input, .commerce-panel textarea, .commerce-panel select { width: 100%; border: 1px solid var(--commerce-border-strong, var(--theme-border-strong, #3a404a)); border-radius: 6px; padding: 9px 10px; color: var(--text-primary, #eef0f4); background: var(--commerce-panel-soft, var(--theme-panel-soft, #252729)); font: inherit; font-size: 12px; resize: vertical; }
.commerce-panel input:focus, .commerce-panel textarea:focus, .commerce-panel select:focus { outline: 2px solid var(--commerce-accent, var(--theme-accent, #d5b46f)); outline-offset: 1px; }
.commerce-preview { display: grid; align-content: center; justify-items: center; gap: 8px; border: 1px dashed var(--commerce-border-strong, var(--theme-border-strong, #3a404a)); color: var(--text-muted, #979da8); background: #111318; text-align: center; overflow: hidden; position: relative; }
.commerce-preview img, .commerce-preview video { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; opacity: .55; }
.commerce-preview > span, .commerce-preview > strong, .commerce-preview > small { position: relative; z-index: 1; padding-inline: 12px; }
.commerce-preview--portrait { width: 210px; aspect-ratio: 9 / 16; margin: 19px auto; border-radius: 12px; }
.commerce-preview--portrait span, .commerce-preview--result span { color: var(--commerce-accent, var(--theme-accent, #d5b46f)); font-size: 11px; }
.commerce-preview strong { color: var(--theme-text-soft, #c7cbd2); font-size: 12px; }
.commerce-preview small { font-size: 10px; }
.commerce-preview-panel footer { display: flex; justify-content: space-between; padding: 0 16px 15px; color: var(--text-muted, #979da8); font-size: 10px; }
.commerce-template-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; padding: 15px; }
.commerce-template-grid button { min-height: 108px; border: 1px solid var(--commerce-border-strong, var(--theme-border-strong, #3a404a)); border-radius: 8px; padding: 13px; color: var(--text-primary, #eef0f4); background: var(--commerce-panel-soft, var(--theme-panel-soft, #252729)); text-align: left; }
.commerce-template-grid button:hover, .commerce-template-grid button.is-selected { border-color: var(--commerce-accent, var(--theme-accent, #d5b46f)); background: var(--commerce-accent-soft, var(--theme-accent-soft, rgba(105, 82, 41, .23))); }
.commerce-template-grid strong, .commerce-template-grid small, .commerce-template-grid em { display: block; }
.commerce-template-grid strong { margin-bottom: 7px; font-size: 12px; }
.commerce-template-grid small { min-height: 30px; color: var(--text-muted, #979da8); font-size: 10px; line-height: 1.45; }
.commerce-template-grid em { margin-top: 9px; color: var(--theme-text-soft, #c7cbd2); font-size: 10px; font-style: normal; }
.commerce-button { border: 1px solid var(--commerce-border-strong, var(--theme-border-strong, #3a404a)); border-radius: 6px; padding: 8px 12px; color: var(--theme-text-soft, #c7cbd2); background: transparent; font-size: 11px; }
.commerce-button:hover:not(:disabled) { border-color: var(--commerce-accent, var(--theme-accent, #d5b46f)); color: var(--commerce-accent, var(--theme-accent, #d5b46f)); }
.commerce-button:disabled { cursor: not-allowed; opacity: .48; }
.commerce-button--primary { border-color: var(--commerce-accent, var(--theme-accent, #d5b46f)); color: var(--theme-panel, #20242b); background: var(--commerce-accent, var(--theme-accent, #d5b46f)); }
.commerce-assets, .commerce-categories, .commerce-shots { display: grid; gap: 9px; }
.commerce-asset, .commerce-shot { display: grid; grid-template-columns: 54px minmax(0, 1fr) auto; gap: 10px; align-items: center; padding: 9px; border: 1px solid var(--commerce-border-strong, var(--theme-border-strong, #3a404a)); border-radius: 7px; background: var(--commerce-panel-soft, var(--theme-panel-soft, #252729)); }
.commerce-asset__thumb, .commerce-shot__thumb { display: grid; place-items: center; height: 42px; border-radius: 5px; color: var(--text-muted, #979da8); background: var(--theme-panel-muted, #1b1d1f); font-size: 10px; overflow: hidden; }
.commerce-shot__thumb img, .commerce-asset__thumb img { width: 100%; height: 100%; object-fit: cover; }
.commerce-asset strong, .commerce-asset small, .commerce-shot strong, .commerce-shot small { display: block; }
.commerce-asset strong, .commerce-shot strong { color: var(--text-primary, #eef0f4); font-size: 11px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.commerce-asset small, .commerce-shot small { margin-top: 4px; color: var(--text-muted, #979da8); font-size: 10px; }
.commerce-asset em { color: var(--theme-text-soft, #c7cbd2); font-size: 10px; font-style: normal; }
.commerce-categories > div { display: grid; grid-template-columns: minmax(0, 1fr) 110px; align-items: center; gap: 9px; }
.commerce-categories strong { color: var(--text-primary, #eef0f4); font-size: 11px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.commerce-categories select { padding: 7px 8px; }
.commerce-shot { grid-template-columns: 24px 54px minmax(0, 1fr) auto; }
.commerce-shot__number { color: var(--text-muted, #979da8); font: 12px Georgia, serif; text-align: center; }
.commerce-shot__actions { display: flex; gap: 3px; }
.commerce-shot__actions button { width: 25px; height: 25px; border: 1px solid var(--commerce-border-strong, var(--theme-border-strong, #3a404a)); border-radius: 5px; color: var(--theme-text-soft, #c7cbd2); background: transparent; }
.commerce-shot__actions button:disabled { opacity: .35; }
.commerce-inline-actions { grid-column: 1 / -1; display: flex; justify-content: flex-end; gap: 8px; }
.commerce-import-actions { display: flex; flex-wrap: wrap; gap: 8px; }
.commerce-empty, .commerce-results__empty { padding: 24px; border: 1px dashed var(--commerce-border-strong, var(--theme-border-strong, #3a404a)); color: var(--text-muted, #979da8); font-size: 11px; text-align: center; }
.commerce-results { max-width: 1240px; margin: 0 auto; display: grid; grid-template-columns: repeat(3, 1fr); gap: 13px; }
.commerce-result { overflow: hidden; border: 1px solid var(--border-subtle, var(--theme-border, #353a43)); border-radius: 9px; background: var(--surface, var(--theme-panel, #20242b)); }
.commerce-preview--result { width: 100%; aspect-ratio: 9 / 13; border: 0; border-bottom: 1px solid var(--border-subtle, var(--theme-border, #353a43)); }
.commerce-result > div:last-child { display: grid; gap: 5px; padding: 13px; }
.commerce-result small { overflow: hidden; color: var(--text-muted, #979da8); font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
.commerce-results__notice { grid-column: 1 / -1; margin: 0; padding: 11px 13px; border: 1px solid var(--commerce-accent, var(--theme-accent, #d5b46f)); border-radius: 6px; color: var(--commerce-accent, var(--theme-accent, #d5b46f)); background: var(--commerce-accent-soft, var(--theme-accent-soft, rgba(105, 82, 41, .23))); font-size: 11px; }
.commerce-workbench__footer { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding-top: 18px; }
.commerce-workbench__footer > span { color: var(--text-muted, #979da8); font-size: 11px; }
@media (max-width: 900px) { .commerce-layout, .commerce-layout--two { grid-template-columns: 1fr; } .commerce-panel--wide { grid-column: auto; } .commerce-results { grid-template-columns: 1fr; } }
@media (max-width: 620px) { .commerce-workbench { padding: 20px 14px 28px; } .commerce-workbench__header, .commerce-workbench__heading { display: block; } .commerce-workbench__status { display: inline-block; margin-top: 12px; } .commerce-steps { overflow-x: auto; padding-bottom: 3px; } .commerce-template-grid { grid-template-columns: 1fr; } .commerce-workbench__footer { position: sticky; bottom: 0; padding: 12px 0; background: var(--app-background, var(--theme-page, #171a20)); } }

/* The commerce workbench shares the application theme; only the video canvas stays black. */
.commerce-workbench {
  --app-background: var(--theme-page, #171a20);
  --surface: var(--theme-panel, #20242b);
  --text-primary: var(--theme-text, #eef0f4);
  --text-muted: var(--theme-muted, #979da8);
  --border-subtle: var(--theme-border, #353a43);
  --commerce-border-strong: var(--theme-border-strong, #3a404a);
  --commerce-panel-soft: var(--theme-panel-soft, #252729);
  --commerce-accent: var(--theme-accent, #d5b46f);
  --commerce-accent-soft: var(--theme-accent-soft, rgba(105, 82, 41, .23));
  color: var(--text-primary);
  background: var(--app-background);
}
.commerce-workbench__header,
.commerce-panel,
.commerce-result { border-color: var(--border-subtle); }
.commerce-workbench__status,
.commerce-results__notice { border-color: var(--commerce-accent); color: var(--commerce-accent); background: var(--commerce-accent-soft); }
.commerce-steps button span { border-color: var(--commerce-border-strong); }
.commerce-steps button.is-active { color: var(--text-primary); }
.commerce-steps button.is-active span { color: var(--theme-panel, #20242b); background: var(--commerce-accent); border-color: var(--commerce-accent); }
.commerce-steps button:disabled { cursor: wait; opacity: .62; }
.commerce-panel input,
.commerce-panel textarea,
.commerce-panel select { border-color: var(--commerce-border-strong); color: var(--text-primary); background: var(--commerce-panel-soft); }
.commerce-panel input:focus,
.commerce-panel textarea:focus,
.commerce-panel select:focus { outline-color: var(--commerce-accent); }
.commerce-preview { border-color: var(--commerce-border-strong); }
.commerce-preview--portrait span,
.commerce-preview--result span { color: var(--commerce-accent); }
.commerce-template-grid button { border-color: var(--commerce-border-strong); color: var(--text-primary); background: var(--commerce-panel-soft); }
.commerce-template-grid button:hover,
.commerce-template-grid button.is-selected { border-color: var(--commerce-accent); background: var(--commerce-accent-soft); }
.commerce-template-grid small,
.commerce-template-grid em { color: var(--text-muted); }
.commerce-button { border-color: var(--commerce-border-strong); color: var(--theme-text-soft, #c7cbd2); }
.commerce-button:hover:not(:disabled) { border-color: var(--commerce-accent); color: var(--commerce-accent); }
.commerce-button--primary { border-color: var(--commerce-accent); color: var(--theme-panel, #20242b); background: var(--commerce-accent); }
.commerce-asset,
.commerce-shot { border-color: var(--commerce-border-strong); background: var(--commerce-panel-soft); }
.commerce-asset__thumb,
.commerce-shot__thumb { color: var(--text-muted); background: var(--theme-panel-muted, #1b1d1f); }
.commerce-asset strong,
.commerce-shot strong,
.commerce-categories strong { color: var(--text-primary); }
.commerce-shot__actions button { border-color: var(--commerce-border-strong); color: var(--theme-text-soft, #c7cbd2); }
.commerce-empty,
.commerce-results__empty { border-color: var(--commerce-border-strong); }
.commerce-result { background: var(--surface); }
.commerce-preview--result { border-bottom-color: var(--border-subtle); }
.commerce-results__retry { grid-column: 1 / -1; display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 11px 13px; border: 1px solid var(--border-subtle); border-radius: 6px; color: var(--text-primary); background: var(--commerce-panel-soft); font-size: 11px; }
.commerce-preview--result video { z-index: 0; opacity: .8; }
.commerce-workbench__footer { color: var(--text-muted); }
.commerce-workbench__footer > span { color: var(--text-muted); }
.commerce-workbench__footer .commerce-button,
.commerce-steps button { color: var(--text-muted); }
.commerce-workbench__footer .commerce-button--primary { color: var(--theme-panel, #20242b); }
@media (max-width: 620px) { .commerce-workbench__footer { background: var(--app-background); } .commerce-results__retry { align-items: stretch; flex-direction: column; } }
</style>
