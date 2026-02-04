/**
 * Database Store
 */

import { defineStore } from 'pinia';

export const useDatabaseStore = defineStore('database', {
  state: () => ({
    currentConnectionId: undefined as string | undefined,
    currentDatabase: undefined as string | undefined,
    databases: [] as string[],
    isLoading: false,
  }),

  actions: {
    setCurrentConnection(id: string) {
      this.currentConnectionId = id;
      this.currentDatabase = undefined;
      this.databases = [];
    },

    setCurrentDatabase(name: string) {
      this.currentDatabase = name;
    },

    setDatabases(databases: string[]) {
      this.databases = databases;
    },

    setLoading(loading: boolean) {
      this.isLoading = loading;
    },
  },
});
