/**
 * Theme Store
 */

import { defineStore } from 'pinia';
import { darkTheme, lightTheme, type GlobalTheme } from 'naive-ui';

export const useThemeStore = defineStore('theme', {
  state: () => ({
    isDark: false,
  }),

  getters: {
    currentTheme: (state): GlobalTheme | null => (state.isDark ? darkTheme : null),
    themeName: (state): 'dark' | 'light' => (state.isDark ? 'dark' : 'light'),
  },

  actions: {
    toggleTheme() {
      this.isDark = !this.isDark;
      this.saveTheme();
      this.applyTheme();
    },

    setTheme(isDark: boolean) {
      this.isDark = isDark;
      this.saveTheme();
      this.applyTheme();
    },

    loadTheme() {
      const saved = localStorage.getItem('theme');
      if (saved) {
        this.isDark = saved === 'dark';
      }
      this.applyTheme();
    },

    saveTheme() {
      localStorage.setItem('theme', this.themeName);
    },

    applyTheme() {
      if (this.isDark) {
        document.documentElement.setAttribute('data-theme', 'dark');
      } else {
        document.documentElement.removeAttribute('data-theme');
      }
    },
  },
});
