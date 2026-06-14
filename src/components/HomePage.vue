<script setup lang="ts">
interface ModuleCard {
  title: string;
  description: string;
  tags: string[];
  available: boolean;
}

defineProps<{
  moduleCards: ModuleCard[];
}>();

defineEmits<{
  openWorkspace: [];
}>();
</script>

<template>
  <section class="home-screen">
    <div class="home-hero">
      <p class="eyebrow">本地桌面端视频处理</p>
      <h2>本地短视频批量混剪工作台</h2>
      <p>本地处理 / 批量混剪 / API Key 自带</p>
    </div>

    <div class="module-grid">
      <article
        v-for="card in moduleCards"
        :key="card.title"
        class="module-card"
        :class="{ 'module-card--disabled': !card.available }"
      >
        <div class="module-card__header">
          <h3>{{ card.title }}</h3>
          <span>{{ card.available ? "功能可用" : "敬请期待" }}</span>
        </div>
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
  </section>
</template>
