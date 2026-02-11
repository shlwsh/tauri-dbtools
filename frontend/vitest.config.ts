import { defineConfig } from 'vitest/config';
import vue from '@vitejs/plugin-vue';
import { resolve } from 'path';

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
      '@tauri-apps/api/tauri': resolve(__dirname, 'src/test/mocks/tauri.ts'),
      '@tauri-apps/plugin-dialog': resolve(__dirname, 'src/test/mocks/tauri-dialog.ts'),
      '@tauri-apps/plugin-fs': resolve(__dirname, 'src/test/mocks/tauri-fs.ts'),
    },
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'dist/',
        '**/*.spec.ts',
        '**/*.test.ts',
        '**/types/**',
        '**/test/**',
      ],
    },
  },
});
