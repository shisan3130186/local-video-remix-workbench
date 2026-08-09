import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";

export type ThemePreference = "system" | "pearl" | "sand" | "mist" | "night";
export type ResolvedTheme = "light" | "dark";

const THEME_STORAGE_KEY = "smartcut:theme-preference";
const SYSTEM_THEME_QUERY = "(prefers-color-scheme: dark)";

function isThemePreference(value: string | null): value is ThemePreference {
  return value === "system" || value === "pearl" || value === "sand" || value === "mist" || value === "night";
}

function readStoredPreference(): ThemePreference {
  try {
    const storedPreference = window.localStorage.getItem(THEME_STORAGE_KEY);
    // Keep existing installations usable after replacing the old two-theme selector.
    if (storedPreference === "light" || storedPreference === "dark") return "pearl";
    return isThemePreference(storedPreference) ? storedPreference : "pearl";
  } catch {
    return "pearl";
  }
}

function readSystemTheme(): ResolvedTheme {
  return window.matchMedia(SYSTEM_THEME_QUERY).matches ? "dark" : "light";
}

export function useTheme() {
  const preference = ref<ThemePreference>(readStoredPreference());
  const systemTheme = ref<ResolvedTheme>(readSystemTheme());
  const resolvedTheme = computed<ResolvedTheme>(() =>
    preference.value === "system"
      ? systemTheme.value
      : preference.value === "night"
        ? "dark"
        : "light",
  );
  const activeVariant = computed<Exclude<ThemePreference, "system">>(() =>
    preference.value === "system"
      ? systemTheme.value === "dark"
        ? "night"
        : "pearl"
      : preference.value,
  );
  let systemThemeMedia: MediaQueryList | null = null;

  function applyTheme(theme: ResolvedTheme, variant: Exclude<ThemePreference, "system">) {
    document.documentElement.dataset.theme = theme;
    document.documentElement.dataset.themeVariant = variant;
    document.documentElement.style.colorScheme = theme;
  }

  function handleSystemThemeChange(event: MediaQueryListEvent) {
    systemTheme.value = event.matches ? "dark" : "light";
  }

  function setThemePreference(nextPreference: ThemePreference) {
    preference.value = nextPreference;
  }

  watch(
    [resolvedTheme, activeVariant],
    ([nextTheme, nextVariant]) => {
      applyTheme(nextTheme, nextVariant);
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
    activeVariant,
    setThemePreference,
  };
}
