/**
 * Configuration API
 */

import type { AppConfig } from '@/types/config';
import { loadConfigFromStorage, saveConfigToStorage } from '@/utils/storage';

/**
 * Load application configuration from local storage
 */
export async function loadConfig(): Promise<AppConfig> {
  return await loadConfigFromStorage();
}

/**
 * Save application configuration to local storage
 */
export async function saveConfig(config: AppConfig): Promise<void> {
  await saveConfigToStorage(config);
}
