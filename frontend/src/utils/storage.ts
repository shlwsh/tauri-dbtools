/**
 * Local storage utilities
 */

import type { AppConfig } from '@/types/config';

const CONFIG_KEY = 'app-config';

/**
 * Get default configuration
 */
export function getDefaultConfig(): AppConfig {
  return {
    connections: [],
    theme: 'light',
  };
}

/**
 * Validate configuration structure
 */
export function isValidConfig(config: any): config is AppConfig {
  if (!config || typeof config !== 'object') {
    return false;
  }

  return (
    Array.isArray(config.connections) &&
    (config.theme === 'light' || config.theme === 'dark')
  );
}

/**
 * Load configuration from local storage
 */
export async function loadConfigFromStorage(): Promise<AppConfig> {
  try {
    const configStr = localStorage.getItem(CONFIG_KEY);
    if (!configStr) {
      return getDefaultConfig();
    }

    const config = JSON.parse(configStr);

    // Validate configuration structure
    if (!isValidConfig(config)) {
      console.warn('Invalid config structure, using default');
      return getDefaultConfig();
    }

    return config;
  } catch (error) {
    console.error('Failed to load config', error);
    return getDefaultConfig();
  }
}

/**
 * Save configuration to local storage
 */
export async function saveConfigToStorage(config: AppConfig): Promise<void> {
  try {
    const configStr = JSON.stringify(config, null, 2);
    localStorage.setItem(CONFIG_KEY, configStr);
  } catch (error) {
    console.error('Failed to save config', error);
    throw error;
  }
}
