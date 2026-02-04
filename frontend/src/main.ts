/**
 * Application entry point
 */

import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import { router } from './router';
import { useThemeStore } from './stores/theme';
import { useConfigStore } from './stores/config';

// Create Vue app
const app = createApp(App);

// Create Pinia store
const pinia = createPinia();
app.use(pinia);

// Use router
app.use(router);

// Global error handler
app.config.errorHandler = (err, instance, info) => {
  console.error('Global error:', err, info);
};

// Mount app
app.mount('#app');

// Initialize theme and config after mounting
const themeStore = useThemeStore();
const configStore = useConfigStore();

themeStore.loadTheme();
configStore.loadConfig();
