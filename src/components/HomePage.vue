<script setup lang="ts">
import type { ModuleCard } from "../types/workbench";

const props = defineProps<{
  moduleCards: ModuleCard[];
  showWelcome: boolean;
}>();

defineEmits<{
  openWorkspace: [];
  closeWelcome: [];
}>();

const categories = ["创作中心", "效率工具", "自动化"];
</script>

<template>
  <section class="home-screen">
    <div class="home-hero">
      <p class="eyebrow">本地桌面端视频处理</p>
      <h2>本地短视频批量混剪工作台</h2>
      <p>本地处理 / 批量混剪 / 模块工作台</p>
    </div>

    <nav class="home-tabs" aria-label="首页分类">
      <a v-for="category in categories" :key="category" :href="`#${category}`">
        {{ category }}
      </a>
    </nav>

    <div class="module-grid">
      <article
        v-for="card in props.moduleCards"
        :key="card.title"
        :id="`${card.category}-${card.title}`"
        class="module-card"
        :class="{ 'module-card--disabled': !card.available }"
      >
        <div class="module-card__icon" aria-hidden="true">
          {{ card.title.slice(0, 2) }}
        </div>
        <div class="module-card__header">
          <span>{{ card.available ? "功能可用" : "敬请期待" }}</span>
        </div>
        <h3>{{ card.title }}</h3>
        <p>{{ card.description }}</p>
        <div class="module-card__tags">
          <span v-for="tag in card.tags" :key="tag">{{ tag }}</span>
        </div>
        <button
          class="primary-button primary-button--full"
          type="button"
          :disabled="!card.available"
          @click="$emit('openWorkspace')"
        >
          {{ card.available ? "进入工作台" : "暂未开放" }}
        </button>
      </article>
    </div>

    <div v-if="showWelcome" class="modal-backdrop" @click.self="$emit('closeWelcome')">
      <section class="welcome-modal" role="dialog" aria-modal="true">
        <button class="modal-close" type="button" aria-label="关闭欢迎弹窗" @click="$emit('closeWelcome')">
          ×
        </button>
        <p class="eyebrow">欢迎使用</p>
        <h2>本地短视频批量混剪工作台</h2>
        <div class="welcome-steps">
          <p>1. 选择模块，进入对应工作台。</p>
          <p>2. 导入视频或文件夹。</p>
          <p>3. 设置参数后开始处理，日志和结果可随时展开查看。</p>
        </div>
        <button class="primary-button" type="button" @click="$emit('closeWelcome')">开始使用</button>
      </section>
    </div>
  </section>
</template>
