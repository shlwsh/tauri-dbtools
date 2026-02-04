/**
 * Theme Store Unit Tests
 * Using Bun's built-in test runner
 */

import { describe, it, expect, beforeEach } from 'bun:test';
import { setActivePinia, createPinia } from 'pinia';
import { useThemeStore } from '../theme';

describe('Theme Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    if (typeof localStorage !== 'undefined') {
      localStorage.clear();
    }
  });

  it('should initialize with light theme', () => {
    const store = useThemeStore();
    expect(store.isDark).toBe(false);
    expect(store.themeName).toBe('light');
  });

  it('should toggle theme', () => {
    const store = useThemeStore();
    expect(store.isDark).toBe(false);

    store.toggleTheme();
    expect(store.isDark).toBe(true);
    expect(store.themeName).toBe('dark');

    store.toggleTheme();
    expect(store.isDark).toBe(false);
    expect(store.themeName).toBe('light');
  });

  it('should set theme', () => {
    const store = useThemeStore();

    store.setTheme(true);
    expect(store.isDark).toBe(true);
    expect(store.themeName).toBe('dark');

    store.setTheme(false);
    expect(store.isDark).toBe(false);
    expect(store.themeName).toBe('light');
  });

  it('should save theme to localStorage', () => {
    const store = useThemeStore();

    store.setTheme(true);
    if (typeof localStorage !== 'undefined') {
      expect(localStorage.getItem('theme')).toBe('dark');
    }

    store.setTheme(false);
    if (typeof localStorage !== 'undefined') {
      expect(localStorage.getItem('theme')).toBe('light');
    }
  });

  it('should load theme from localStorage', () => {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('theme', 'dark');
    }

    const store = useThemeStore();
    store.loadTheme();

    if (typeof localStorage !== 'undefined') {
      expect(store.isDark).toBe(true);
      expect(store.themeName).toBe('dark');
    }
  });
});
