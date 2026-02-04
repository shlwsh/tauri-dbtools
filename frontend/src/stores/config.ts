/**
 * Configuration Store
 */

import { defineStore } from 'pinia';
import type { DatabaseConnection, AppConfig } from '@/types/config';
import { loadConfig, saveConfig } from '@/api/config';

export const useConfigStore = defineStore('config', {
  state: () => ({
    connections: [] as DatabaseConnection[],
    defaultConnectionId: undefined as string | undefined,
  }),

  getters: {
    defaultConnection: state => {
      return state.connections.find(c => c.id === state.defaultConnectionId);
    },

    getConnectionById: state => {
      return (id: string) => state.connections.find(c => c.id === id);
    },
  },

  actions: {
    async loadConfig() {
      const config = await loadConfig();
      this.connections = config.connections;
      this.defaultConnectionId = config.defaultConnectionId;
    },

    async saveConfig() {
      await saveConfig({
        connections: this.connections,
        theme: 'light', // Managed by theme store
        defaultConnectionId: this.defaultConnectionId,
      });
    },

    addConnection(connection: DatabaseConnection) {
      this.connections.push(connection);
      this.saveConfig();
    },

    updateConnection(id: string, updates: Partial<DatabaseConnection>) {
      const index = this.connections.findIndex(c => c.id === id);
      if (index !== -1) {
        this.connections[index] = { ...this.connections[index], ...updates };
        this.saveConfig();
      }
    },

    deleteConnection(id: string) {
      this.connections = this.connections.filter(c => c.id !== id);
      if (this.defaultConnectionId === id) {
        this.defaultConnectionId = undefined;
      }
      this.saveConfig();
    },

    setDefaultConnection(id: string) {
      this.defaultConnectionId = id;
      this.saveConfig();
    },
  },
});
