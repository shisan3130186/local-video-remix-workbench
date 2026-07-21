<script setup lang="ts">
import type { ImportedVideo } from "../types/videoProbe";
import type { DrawerKey, ToolKey } from "../types/workbench";

type BatchToolKey = Extract<ToolKey, "remix" | "canvas" | "effects" | "transition" | "export">;

defineProps<{
  importedVideoCount: number;
  selectedVideo: ImportedVideo | null;
  previewUrl: string | null;
  splitSegmentCount: number | null;
  randomSelectedCount: number;
  batchGenerateCount: number;
  batchMixResultCount: number;
  isSplitting: boolean;
  isMixing: boolean;
  isBatchMixing: boolean;
  splitError: string | null;
  mixError: string | null;
  batchMixError: string | null;
}>();

defineEmits<{
  splitSelectedVideo: [];
  pickSegmentsRandomly: [];
  concatRandomSegments: [];
  concatCategorizedSegments: [];
  generateBatchMixes: [];
  openTool: [tool: BatchToolKey];
  openDrawer: [drawer: DrawerKey];
  "update:batchGenerateCount": [count: number];
}>();
</script>

<template>
  <section class="batch-workspace" aria-label="批量混剪工作区">
    <header class="workspace-page-heading">
      <div>
        <p class="panel__label">批量生产</p>
        <h2>用一套规则生成多个差异版本</h2>
        <small>适合素材量较多、需要一次导出多条视频的任务。</small>
      </div>
      <button class="panel-toggle" type="button" @click="$emit('openDrawer', 'batch')">
        批量结果 {{ batchMixResultCount }}
      </button>
    </header>

    <nav class="batch-steps" aria-label="批量混剪步骤">
      <span :class="{ 'batch-steps__item--done': importedVideoCount > 0 }"><b>1</b><small>导入多份素材</small></span>
      <i aria-hidden="true"></i>
      <span :class="{ 'batch-steps__item--done': (splitSegmentCount ?? 0) > 1 }"><b>2</b><small>切成可组合片段</small></span>
      <i aria-hidden="true"></i>
      <span :class="{ 'batch-steps__item--done': randomSelectedCount > 0 }"><b>3</b><small>确认组合规则</small></span>
      <i aria-hidden="true"></i>
      <span :class="{ 'batch-steps__item--active': batchMixResultCount > 0 }"><b>4</b><small>批量生成</small></span>
    </nav>

    <div class="batch-workspace__body">
      <section class="panel batch-preview-card">
        <div class="panel__header">
          <div>
            <p class="panel__label">素材预览</p>
            <h3>{{ selectedVideo?.fileName ?? "等待选择素材" }}</h3>
          </div>
          <span class="count-badge">{{ importedVideoCount }} 个视频</span>
        </div>
        <div v-if="previewUrl" class="batch-preview-card__video">
          <span class="preview-fit-badge">完整画面</span>
          <video :src="previewUrl" controls playsinline preload="metadata" aria-label="当前批量素材完整画面预览"></video>
        </div>
        <div v-else class="batch-preview-card__empty">
          <strong>先从左侧导入一组视频</strong>
          <small>这里用于抽查当前素材，批量规则在右侧设置。</small>
        </div>
      </section>

      <section class="panel batch-recipe-card">
        <div class="batch-recipe-card__heading">
          <div>
            <p class="panel__label">本次生成方案</p>
            <h3>组合规则</h3>
          </div>
          <button class="ghost-button" type="button" @click="$emit('openTool', 'remix')">详细参数</button>
        </div>

        <div class="batch-recipe-stats">
          <article><small>源视频</small><strong>{{ importedVideoCount }}</strong></article>
          <article><small>可用片段</small><strong>{{ splitSegmentCount ?? 0 }}</strong></article>
          <article><small>已选片段</small><strong>{{ randomSelectedCount }}</strong></article>
        </div>

        <label class="batch-count-field">
          <span><strong>生成数量</strong><small>建议先生成 3 条检查效果</small></span>
          <input
            :value="batchGenerateCount"
            type="number"
            min="1"
            max="20"
            @input="$emit('update:batchGenerateCount', Number(($event.target as HTMLInputElement).value))"
          />
        </label>

        <div class="batch-rule-actions">
          <button type="button" :disabled="isSplitting || importedVideoCount === 0" @click="$emit('splitSelectedVideo')">
            <span>01</span><strong>{{ isSplitting ? "正在切片" : "智能切片全部素材" }}</strong><small>把长视频拆成可重新组合的片段</small>
          </button>
          <button type="button" :disabled="(splitSegmentCount ?? 0) < 2" @click="$emit('pickSegmentsRandomly')">
            <span>02</span><strong>随机选择一组片段</strong><small>先预览一次组合结构是否合适</small>
          </button>
          <button type="button" :disabled="randomSelectedCount === 0 || isMixing" @click="$emit('concatRandomSegments')">
            <span>03</span><strong>生成单条样片</strong><small>正式批量前先输出一条检查节奏</small>
          </button>
        </div>

        <div class="batch-enhance-row" aria-label="批量增强设置">
          <button type="button" @click="$emit('openTool', 'canvas')">画布比例</button>
          <button type="button" @click="$emit('openTool', 'effects')">画面效果</button>
          <button type="button" @click="$emit('openTool', 'transition')">平滑转场</button>
          <button type="button" @click="$emit('openTool', 'export')">输出设置</button>
        </div>

        <button
          class="primary-button batch-generate-button"
          type="button"
          :disabled="isBatchMixing || (splitSegmentCount ?? 0) < 2"
          @click="$emit('generateBatchMixes')"
        >
          {{ isBatchMixing ? "正在批量生成..." : `生成 ${batchGenerateCount} 条差异视频` }}
        </button>

        <p v-if="splitError || mixError || batchMixError" class="workflow-error" role="alert">
          {{ splitError || mixError || batchMixError }}
        </p>
      </section>
    </div>
  </section>
</template>
