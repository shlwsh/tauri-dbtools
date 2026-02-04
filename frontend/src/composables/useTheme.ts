/**
 * Theme composable
 */

import { computed } from 'vue';
import { useThemeStore } from '@/stores/theme';

export function useTheme() {
  const themeStore = useThemeStore();

  const isDark = computed(() => themeStore.isDark);
  const currentTheme = computed(() => themeStore.currentTheme);
  const themeName = computed(() => themeStore.themeName);

  const toggleTheme = () => {
    themeStore.toggleTheme();
  };

  const setTheme = (isDark: boolean) => {
    themeStore.setTheme(isDark);
  };

  return {
    isDark,
    currentTheme,
    themeName,
    toggleTheme,
    setTheme,
  };
}
