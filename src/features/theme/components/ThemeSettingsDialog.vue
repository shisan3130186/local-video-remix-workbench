<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import type { ResolvedTheme, ThemePreference } from "../useTheme";

const props = defineProps<{
  visible: boolean;
  preference: ThemePreference;
  resolvedTheme: ResolvedTheme;
}>();

const emit = defineEmits<{
  close: [];
  select: [preference: ThemePreference];
}>();

const closeButton = ref<HTMLButtonElement | null>(null);
let previouslyFocusedElement: HTMLElement | null = null;

const themeOptions: Array<{
  value: ThemePreference;
  label: string;
  description: string;
}> = [
  { value: "system", label: "跟随系统", description: "自动跟随电脑的外观设置" },
  { value: "light", label: "浅色", description: "适合明亮环境与白天使用" },
  { value: "dark", label: "深色", description: "降低眩光，适合长时间剪辑" },
];

function closeDialog() {
  emit("close");
}

function handleKeydown(event: KeyboardEvent) {
  if (!props.visible) return;
  if (event.key === "Escape") {
    event.preventDefault();
    closeDialog();
  }
}

function moveSelection(event: KeyboardEvent, currentIndex: number) {
  if (!(["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"] as string[]).includes(event.key)) return;
  event.preventDefault();
  const offset = event.key === "ArrowLeft" || event.key === "ArrowUp" ? -1 : 1;
  const nextIndex = (currentIndex + offset + themeOptions.length) % themeOptions.length;
  emit("select", themeOptions[nextIndex].value);
  const buttons = document.querySelectorAll<HTMLButtonElement>(".theme-option-card");
  buttons[nextIndex]?.focus();
}

watch(
  () => props.visible,
  async (visible) => {
    if (visible) {
      previouslyFocusedElement = document.activeElement as HTMLElement | null;
      await nextTick();
      closeButton.value?.focus();
      return;
    }
    previouslyFocusedElement?.focus();
    previouslyFocusedElement = null;
  },
);

onMounted(() => document.addEventListener("keydown", handleKeydown));
onBeforeUnmount(() => document.removeEventListener("keydown", handleKeydown));
</script>

<template>
  <Teleport to="body">
    <Transition name="theme-dialog">
      <div v-if="visible" class="theme-dialog-backdrop" @mousedown.self="closeDialog">
        <section
          class="theme-dialog"
          role="dialog"
          aria-modal="true"
          aria-labelledby="theme-dialog-title"
          aria-describedby="theme-dialog-description"
        >
          <header class="theme-dialog__header">
            <div>
              <h2 id="theme-dialog-title">主题设置</h2>
              <p id="theme-dialog-description">选择适合当前环境的界面外观。</p>
            </div>
            <button ref="closeButton" class="theme-dialog__close" type="button" aria-label="关闭主题设置" @click="closeDialog">×</button>
          </header>

          <div class="theme-dialog__body">
            <div class="theme-dialog__section-heading">
              <strong>主题</strong>
              <span>当前生效：{{ resolvedTheme === "dark" ? "深色" : "浅色" }}</span>
            </div>

            <div class="theme-option-grid" role="radiogroup" aria-label="主题模式">
              <button
                v-for="(option, index) in themeOptions"
                :key="option.value"
                class="theme-option-card"
                :class="[`theme-option-card--${option.value}`, { 'is-selected': preference === option.value }]"
                type="button"
                role="radio"
                :aria-checked="preference === option.value"
                :title="option.description"
                @click="emit('select', option.value)"
                @keydown="moveSelection($event, index)"
              >
                <span class="theme-option-card__preview" aria-hidden="true">
                  <span class="theme-preview__sidebar"></span>
                  <span class="theme-preview__line theme-preview__line--one"></span>
                  <span class="theme-preview__line theme-preview__line--two"></span>
                  <span class="theme-preview__chart"></span>
                  <span class="theme-preview__canvas"></span>
                </span>
                <span class="theme-option-card__copy">
                  <strong>{{ option.label }}</strong>
                  <small>{{ option.description }}</small>
                </span>
                <span v-if="preference === option.value" class="theme-option-card__check" aria-hidden="true">✓</span>
              </button>
            </div>
          </div>

          <footer class="theme-dialog__footer">
            <span>选择会自动保存，下次打开继续使用。</span>
            <button type="button" :disabled="preference === 'system'" @click="emit('select', 'system')">恢复跟随系统</button>
          </footer>
        </section>
      </div>
    </Transition>
  </Teleport>
</template>
