/**
 * Database Store Unit Tests
 * Using Bun's built-in test runner
 */

import { describe, it, expect, beforeEach } from 'bun:test';
import { setActivePinia, createPinia } from 'pinia';
import { useDatabaseStore } from '../database';

describe('Database Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('should initialize with undefined values', () => {
    const store = useDatabaseStore();
    expect(store.currentConnectionId).toBeUndefined();
    expect(store.currentDatabase).toBeUndefined();
    expect(store.databases).toEqual([]);
    expect(store.isLoading).toBe(false);
  });

  it('should set current connection', () => {
    const store = useDatabaseStore();
    store.setCurrentConnection('test-conn-1');

    expect(store.currentConnectionId).toBe('test-conn-1');
    expect(store.currentDatabase).toBeUndefined();
    expect(store.databases).toEqual([]);
  });

  it('should clear database when changing connection', () => {
    const store = useDatabaseStore();
    store.setCurrentConnection('test-conn-1');
    store.setCurrentDatabase('test-db');
    store.setDatabases(['db1', 'db2']);

    store.setCurrentConnection('test-conn-2');

    expect(store.currentConnectionId).toBe('test-conn-2');
    expect(store.currentDatabase).toBeUndefined();
    expect(store.databases).toEqual([]);
  });

  it('should set current database', () => {
    const store = useDatabaseStore();
    store.setCurrentDatabase('test-db');

    expect(store.currentDatabase).toBe('test-db');
  });

  it('should set databases list', () => {
    const store = useDatabaseStore();
    const databases = ['db1', 'db2', 'db3'];

    store.setDatabases(databases);

    expect(store.databases).toEqual(databases);
  });

  it('should set loading state', () => {
    const store = useDatabaseStore();

    store.setLoading(true);
    expect(store.isLoading).toBe(true);

    store.setLoading(false);
    expect(store.isLoading).toBe(false);
  });
});
