/**
 * Storage Utils Unit Tests
 * Using Bun's built-in test runner
 */

import { describe, it, expect, beforeEach } from 'bun:test';
import {
  getDefaultConfig,
  isValidConfig,
  loadConfigFromStorage,
  saveConfigToStorage,
} from '../storage';
import type { AppConfig } from '@/types/config';

describe('Storage Utils', () => {
  beforeEach(() => {
    if (typeof localStorage !== 'undefined') {
      localStorage.clear();
    }
  });

  describe('getDefaultConfig', () => {
    it('should return default config', () => {
      const config = getDefaultConfig();
      expect(config).toEqual({
        connections: [],
        theme: 'light',
      });
    });
  });

  describe('isValidConfig', () => {
    it('should validate correct config', () => {
      const config: AppConfig = {
        connections: [],
        theme: 'light',
      };
      expect(isValidConfig(config)).toBe(true);
    });

    it('should validate config with connections', () => {
      const config: AppConfig = {
        connections: [
          {
            id: 'test-1',
            name: 'Test',
            host: 'localhost',
            port: 5432,
            username: 'postgres',
            password: 'password',
            isDefault: false,
          },
        ],
        theme: 'dark',
      };
      expect(isValidConfig(config)).toBe(true);
    });

    it('should reject invalid config - missing connections', () => {
      const config = {
        theme: 'light',
      };
      expect(isValidConfig(config)).toBe(false);
    });

    it('should reject invalid config - invalid theme', () => {
      const config = {
        connections: [],
        theme: 'invalid',
      };
      expect(isValidConfig(config)).toBe(false);
    });

    it('should reject invalid config - connections not array', () => {
      const config = {
        connections: 'not-array',
        theme: 'light',
      };
      expect(isValidConfig(config)).toBe(false);
    });

    it('should reject null config', () => {
      expect(isValidConfig(null)).toBe(false);
    });

    it('should reject undefined config', () => {
      expect(isValidConfig(undefined)).toBe(false);
    });
  });

  describe('loadConfigFromStorage', () => {
    it('should return default config when localStorage is empty', async () => {
      const config = await loadConfigFromStorage();
      expect(config).toEqual(getDefaultConfig());
    });

    it('should load config from localStorage', async () => {
      if (typeof localStorage !== 'undefined') {
        const testConfig: AppConfig = {
          connections: [
            {
              id: 'test-1',
              name: 'Test',
              host: 'localhost',
              port: 5432,
              username: 'postgres',
              password: 'password',
              isDefault: true,
            },
          ],
          theme: 'dark',
          defaultConnectionId: 'test-1',
        };

        localStorage.setItem('app-config', JSON.stringify(testConfig));

        const config = await loadConfigFromStorage();
        expect(config).toEqual(testConfig);
      }
    });

    it('should return default config when stored config is invalid', async () => {
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem('app-config', 'invalid-json');

        const config = await loadConfigFromStorage();
        expect(config).toEqual(getDefaultConfig());
      }
    });

    it('should return default config when stored config structure is invalid', async () => {
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem('app-config', JSON.stringify({ invalid: 'structure' }));

        const config = await loadConfigFromStorage();
        expect(config).toEqual(getDefaultConfig());
      }
    });
  });

  describe('saveConfigToStorage', () => {
    it('should save config to localStorage', async () => {
      if (typeof localStorage !== 'undefined') {
        const testConfig: AppConfig = {
          connections: [
            {
              id: 'test-1',
              name: 'Test',
              host: 'localhost',
              port: 5432,
              username: 'postgres',
              password: 'password',
              isDefault: true,
            },
          ],
          theme: 'dark',
          defaultConnectionId: 'test-1',
        };

        await saveConfigToStorage(testConfig);

        const stored = localStorage.getItem('app-config');
        expect(stored).toBeTruthy();

        const parsed = JSON.parse(stored!);
        expect(parsed).toEqual(testConfig);
      }
    });

    it('should format JSON with indentation', async () => {
      if (typeof localStorage !== 'undefined') {
        const testConfig: AppConfig = {
          connections: [],
          theme: 'light',
        };

        await saveConfigToStorage(testConfig);

        const stored = localStorage.getItem('app-config');
        expect(stored).toContain('\n'); // Check for newlines (formatted JSON)
      }
    });
  });
});
