/**
 * Configuration composable
 */

import { computed } from 'vue';
import { useConfigStore } from '@/stores/config';
import type { DatabaseConnection } from '@/types/config';

export function useConfig() {
  const configStore = useConfigStore();

  const connections = computed(() => configStore.connections);
  const defaultConnection = computed(() => configStore.defaultConnection);
  const defaultConnectionId = computed(() => configStore.defaultConnectionId);

  const getConnectionById = (id: string) => {
    return configStore.getConnectionById(id);
  };

  const addConnection = (connection: DatabaseConnection) => {
    configStore.addConnection(connection);
  };

  const updateConnection = (id: string, updates: Partial<DatabaseConnection>) => {
    configStore.updateConnection(id, updates);
  };

  const deleteConnection = (id: string) => {
    configStore.deleteConnection(id);
  };

  const setDefaultConnection = (id: string) => {
    configStore.setDefaultConnection(id);
  };

  const loadConfig = async () => {
    await configStore.loadConfig();
  };

  return {
    connections,
    defaultConnection,
    defaultConnectionId,
    getConnectionById,
    addConnection,
    updateConnection,
    deleteConnection,
    setDefaultConnection,
    loadConfig,
  };
}
