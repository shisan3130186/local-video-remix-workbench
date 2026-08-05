import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";

export type ThemePreference = "system" | "light" | "dark";
export type ResolvedTheme = "light" | "dark";

const THEME_STORAGE_KEY = "smartcut:theme-preference";
const SYSTEM_THEME_QUERY = "(prefers-color-scheme: dark)";

function isThemePreference(value: string | null): value is ThemePreference {
  return value === "system" || value === "light" || value === "dark";
}

function readStoredPreference(): ThemePreference {
  try {
    const storedPreference = window.localStorage.getItem(THEME_STORAGE_KEY);
    return isThemePreference(storedPreference) ? storedPreference : "system";
  } catch {
    return "system";
  }
}

function readSystemTheme(): ResolvedTheme {
  return window.matchMedia(SYSTEM_THEME_QUERY).matches ? "dark" : "light";
}

export function useTheme() {
  const preference = ref<ThemePreference>(readStoredPreference());
  const systemTheme = ref<ResolvedTheme>(readSystemTheme());
  const resolvedTheme = computed<ResolvedTheme>(() =>
    preference.value === "system" ? systemTheme.value : preference.value,
  );
  let systemThemeMedia: MediaQueryList | null = null;

  function applyTheme(theme: ResolvedTheme) {
    document.documentElement.dataset.theme = theme;
    document.documentElement.style.colorScheme = theme;
  }

  function handleSystemThemeChange(event: MediaQueryListEvent) {
    systemTheme.value = event.matches ? "dark" : "light";
  }

  function setThemePreference(nextPreference: ThemePreference) {
    preference.value = nextPreference;
  }

  watch(
    resolvedTheme,
    (nextTheme) => {
      applyTheme(nextTheme);
    },
    { immediate: true },
  );

  watch(preference, (nextPreference) => {
    try {
      window.localStorage.setItem(THEME_STORAGE_KEY, nextPreference);
    } catch {
      // 本地存储不可用时仍保留当前会话内的主题切换。
    }
  });

  onMounted(() => {
    systemThemeMedia = window.matchMedia(SYSTEM_THEME_QUERY);
    systemTheme.value = systemThemeMedia.matches ? "dark" : "light";
    systemThemeMedia.addEventListener("change", handleSystemThemeChange);
  });

  onBeforeUnmount(() => {
    systemThemeMedia?.removeEventListener("change", handleSystemThemeChange);
  });

  return {
    preference,
    resolvedTheme,
    setThemePreference,
  };
}
