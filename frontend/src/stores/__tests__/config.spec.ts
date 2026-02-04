/**
 * Config Store Unit Tests
 * Using Bun's built-in test runner
 */

import { describe, it, expect, beforeEach } from 'bun:test';
import { setActivePinia, createPinia } from 'pinia';
import { useConfigStore } from '../config';
import type { DatabaseConnection } from '@/types/config';

describe('Config Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('should initialize with empty connections', () => {
    const store = useConfigStore();
    expect(store.connections).toEqual([]);
    expect(store.defaultConnectionId).toBeUndefined();
  });

  it('should add connection', () => {
    const store = useConfigStore();
    const connection: DatabaseConnection = {
      id: 'test-1',
      name: 'Test Connection',
      host: 'localhost',
      port: 5432,
      username: 'postgres',
      password: 'password',
      isDefault: false,
    };

    store.addConnection(connection);
    expect(store.connections).toHaveLength(1);
    expect(store.connections[0]).toEqual(connection);
  });

  it('should update connection', () => {
    const store = useConfigStore();
    const connection: DatabaseConnection = {
      id: 'test-1',
      name: 'Test Connection',
      host: 'localhost',
      port: 5432,
      username: 'postgres',
      password: 'password',
      isDefault: false,
    };

    store.addConnection(connection);
    store.updateConnection('test-1', { name: 'Updated Connection', port: 5433 });

    expect(store.connections[0].name).toBe('Updated Connection');
    expect(store.connections[0].port).toBe(5433);
    expect(store.connections[0].host).toBe('localhost'); // unchanged
  });

  it('should delete connection', () => {
    const store = useConfigStore();
    const connection: DatabaseConnection = {
      id: 'test-1',
      name: 'Test Connection',
      host: 'localhost',
      port: 5432,
      username: 'postgres',
      password: 'password',
      isDefault: false,
    };

    store.addConnection(connection);
    expect(store.connections).toHaveLength(1);

    store.deleteConnection('test-1');
    expect(store.connections).toHaveLength(0);
  });

  it('should set default connection', () => {
    const store = useConfigStore();
    const connection: DatabaseConnection = {
      id: 'test-1',
      name: 'Test Connection',
      host: 'localhost',
      port: 5432,
      username: 'postgres',
      password: 'password',
      isDefault: false,
    };

    store.addConnection(connection);
    store.setDefaultConnection('test-1');

    expect(store.defaultConnectionId).toBe('test-1');
  });

  it('should get default connection', () => {
    const store = useConfigStore();
    const connection: DatabaseConnection = {
      id: 'test-1',
      name: 'Test Connection',
      host: 'localhost',
      port: 5432,
      username: 'postgres',
      password: 'password',
      isDefault: false,
    };

    store.addConnection(connection);
    store.setDefaultConnection('test-1');

    expect(store.defaultConnection).toEqual(connection);
  });

  it('should get connection by id', () => {
    const store = useConfigStore();
    const connection: DatabaseConnection = {
      id: 'test-1',
      name: 'Test Connection',
      host: 'localhost',
      port: 5432,
      username: 'postgres',
      password: 'password',
      isDefault: false,
    };

    store.addConnection(connection);

    const found = store.getConnectionById('test-1');
    expect(found).toEqual(connection);
  });

  it('should clear default connection when deleted', () => {
    const store = useConfigStore();
    const connection: DatabaseConnection = {
      id: 'test-1',
      name: 'Test Connection',
      host: 'localhost',
      port: 5432,
      username: 'postgres',
      password: 'password',
      isDefault: false,
    };

    store.addConnection(connection);
    store.setDefaultConnection('test-1');
    expect(store.defaultConnectionId).toBe('test-1');

    store.deleteConnection('test-1');
    expect(store.defaultConnectionId).toBeUndefined();
  });
});
