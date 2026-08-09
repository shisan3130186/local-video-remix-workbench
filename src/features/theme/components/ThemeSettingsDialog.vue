<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
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
  { value: "system", label: "跟随系统", description: "自动匹配电脑的明暗外观" },
  { value: "pearl", label: "珍珠雾白", description: "通透、柔和，适合长时间工作" },
  { value: "sand", label: "暖砂晨光", description: "低对比暖白，减轻视觉紧张" },
  { value: "mist", label: "冰雾银蓝", description: "克制冷白，信息层级更清晰" },
  { value: "night", label: "夜航墨色", description: "夜间使用的低亮度深色界面" },
];

const activeThemeLabel = computed(() => {
  if (props.preference === "system") {
    return props.resolvedTheme === "dark" ? "跟随系统 · 夜航墨色" : "跟随系统 · 珍珠雾白";
  }
  return themeOptions.find((option) => option.value === props.preference)?.label ?? "珍珠雾白";
});

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
              <span>当前生效：{{ activeThemeLabel }}</span>
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
