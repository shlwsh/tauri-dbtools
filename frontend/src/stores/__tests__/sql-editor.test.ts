/**
 * Unit tests for SQL Editor Store
 * 
 * Tests tab management, query history, and state management functionality
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useSQLEditorStore } from '../sql-editor';
import type { QueryHistoryItem, QueryResult } from '@/types/sql-editor';
import fc from 'fast-check';

describe('SQL Editor Store', () => {
  beforeEach(() => {
    // Create a fresh pinia instance for each test
    setActivePinia(createPinia());
    
    // Clear localStorage
    localStorage.clear();
  });

  afterEach(() => {
    localStorage.clear();
  });

  describe('Tab Management', () => {
    it('should create a new tab with default values', () => {
      const store = useSQLEditorStore();
      
      const tabId = store.createTab();
      
      expect(store.tabs).toHaveLength(1);
      expect(store.activeTabId).toBe(tabId);
      
      const tab = store.tabs[0];
      expect(tab.id).toBe(tabId);
      expect(tab.label).toBe('Query 1');
      expect(tab.content).toBe('');
      expect(tab.isExecuting).toBe(false);
      expect(tab.isDirty).toBe(false);
    });

    it('should create a tab with initial content and database', () => {
      const store = useSQLEditorStore();
      
      const tabId = store.createTab('SELECT * FROM users', 'testdb');
      
      const tab = store.getTabById(tabId);
      expect(tab?.content).toBe('SELECT * FROM users');
      expect(tab?.database).toBe('testdb');
    });

    it('should create multiple tabs with incrementing labels', () => {
      const store = useSQLEditorStore();
      
      store.createTab();
      store.createTab();
      store.createTab();
      
      expect(store.tabs).toHaveLength(3);
      expect(store.tabs[0].label).toBe('Query 1');
      expect(store.tabs[1].label).toBe('Query 2');
      expect(store.tabs[2].label).toBe('Query 3');
    });

    it('should set the newly created tab as active', () => {
      const store = useSQLEditorStore();
      
      const tab1 = store.createTab();
      expect(store.activeTabId).toBe(tab1);
      
      const tab2 = store.createTab();
      expect(store.activeTabId).toBe(tab2);
    });

    it('should close a tab', () => {
      const store = useSQLEditorStore();
      
      const tab1 = store.createTab();
      const tab2 = store.createTab();
      
      store.closeTab(tab1);
      
      expect(store.tabs).toHaveLength(1);
      expect(store.tabs[0].id).toBe(tab2);
    });

    it('should update active tab when closing the active tab', () => {
      const store = useSQLEditorStore();
      
      const tab1 = store.createTab();
      const tab2 = store.createTab();
      const tab3 = store.createTab();
      
      // Close the active tab (tab3)
      store.closeTab(tab3);
      
      // Should activate the previous tab (tab2)
      expect(store.activeTabId).toBe(tab2);
    });

    it('should set activeTabId to null when closing the last tab', () => {
      const store = useSQLEditorStore();
      
      const tabId = store.createTab();
      store.closeTab(tabId);
      
      expect(store.tabs).toHaveLength(0);
      expect(store.activeTabId).toBeNull();
    });

    it('should switch active tab', () => {
      const store = useSQLEditorStore();
      
      const tab1 = store.createTab();
      const tab2 = store.createTab();
      
      store.setActiveTab(tab1);
      
      expect(store.activeTabId).toBe(tab1);
    });

    it('should update tab content and mark as dirty', () => {
      const store = useSQLEditorStore();
      
      const tabId = store.createTab();
      store.updateTabContent(tabId, 'SELECT * FROM users');
      
      const tab = store.getTabById(tabId);
      expect(tab?.content).toBe('SELECT * FROM users');
      expect(tab?.isDirty).toBe(true);
    });

    it('should update tab label', () => {
      const store = useSQLEditorStore();
      
      const tabId = store.createTab();
      store.updateTabLabel(tabId, 'My Custom Query');
      
      const tab = store.getTabById(tabId);
      expect(tab?.label).toBe('My Custom Query');
    });

    it('should set tab database', () => {
      const store = useSQLEditorStore();
      
      const tabId = store.createTab();
      store.setTabDatabase(tabId, 'production');
      
      const tab = store.getTabById(tabId);
      expect(tab?.database).toBe('production');
    });

    it('should set tab executing state', () => {
      const store = useSQLEditorStore();
      
      const tabId = store.createTab();
      store.setTabExecuting(tabId, true);
      
      const tab = store.getTabById(tabId);
      expect(tab?.isExecuting).toBe(true);
    });

    it('should set tab result and clear dirty flag', () => {
      const store = useSQLEditorStore();
      
      const tabId = store.createTab();
      store.updateTabContent(tabId, 'SELECT 1');
      
      const result: QueryResult = {
        type: 'select',
        columns: [{ name: '?column?', typeName: 'integer', nullable: false, isPrimaryKey: false }],
        rows: [{ '?column?': 1 }],
        rowCount: 1,
        duration: 10,
      };
      
      store.setTabResult(tabId, result);
      
      const tab = store.getTabById(tabId);
      expect(tab?.result).toEqual(result);
      expect(tab?.isDirty).toBe(false);
    });

    it('should close all tabs', () => {
      const store = useSQLEditorStore();
      
      store.createTab();
      store.createTab();
      store.createTab();
      
      store.closeAllTabs();
      
      expect(store.tabs).toHaveLength(0);
      expect(store.activeTabId).toBeNull();
    });

    it('should close other tabs', () => {
      const store = useSQLEditorStore();
      
      const tab1 = store.createTab();
      const tab2 = store.createTab();
      store.createTab();
      
      store.closeOtherTabs(tab2);
      
      expect(store.tabs).toHaveLength(1);
      expect(store.tabs[0].id).toBe(tab2);
      expect(store.activeTabId).toBe(tab2);
    });

    it('should get dirty tabs', () => {
      const store = useSQLEditorStore();
      
      const tab1 = store.createTab();
      const tab2 = store.createTab();
      const tab3 = store.createTab();
      
      store.updateTabContent(tab1, 'SELECT 1');
      store.updateTabContent(tab3, 'SELECT 3');
      
      const dirtyTabs = store.getDirtyTabs();
      
      expect(dirtyTabs).toHaveLength(2);
      expect(dirtyTabs.map(t => t.id)).toContain(tab1);
      expect(dirtyTabs.map(t => t.id)).toContain(tab3);
    });
  });

  describe('Getters', () => {
    it('should get active tab', () => {
      const store = useSQLEditorStore();
      
      const tabId = store.createTab('SELECT 1');
      const activeTab = store.activeTab;
      
      expect(activeTab?.id).toBe(tabId);
      expect(activeTab?.content).toBe('SELECT 1');
    });

    it('should return undefined when no active tab', () => {
      const store = useSQLEditorStore();
      
      expect(store.activeTab).toBeUndefined();
    });

    it('should check if there are tabs', () => {
      const store = useSQLEditorStore();
      
      expect(store.hasTabs).toBe(false);
      
      store.createTab();
      
      expect(store.hasTabs).toBe(true);
    });

    it('should get tab by ID', () => {
      const store = useSQLEditorStore();
      
      const tabId = store.createTab('SELECT 1');
      const tab = store.getTabById(tabId);
      
      expect(tab?.id).toBe(tabId);
      expect(tab?.content).toBe('SELECT 1');
    });

    it('should return undefined for non-existent tab ID', () => {
      const store = useSQLEditorStore();
      
      const tab = store.getTabById('non-existent');
      
      expect(tab).toBeUndefined();
    });
  });

  describe('Query History', () => {
    it('should add query to history', () => {
      const store = useSQLEditorStore();
      
      const historyItem: QueryHistoryItem = {
        id: 'hist-1',
        query: 'SELECT * FROM users',
        database: 'testdb',
        executedAt: new Date(),
        duration: 100,
        success: true,
      };
      
      store.addToHistory(historyItem);
      
      expect(store.queryHistory).toHaveLength(1);
      expect(store.queryHistory[0]).toEqual(historyItem);
    });

    it('should add new items to the beginning of history', () => {
      const store = useSQLEditorStore();
      
      const item1: QueryHistoryItem = {
        id: 'hist-1',
        query: 'SELECT 1',
        database: 'testdb',
        executedAt: new Date(),
        duration: 10,
        success: true,
      };
      
      const item2: QueryHistoryItem = {
        id: 'hist-2',
        query: 'SELECT 2',
        database: 'testdb',
        executedAt: new Date(),
        duration: 20,
        success: true,
      };
      
      store.addToHistory(item1);
      store.addToHistory(item2);
      
      expect(store.queryHistory[0].id).toBe('hist-2');
      expect(store.queryHistory[1].id).toBe('hist-1');
    });

    it('should limit history to 100 items', () => {
      const store = useSQLEditorStore();
      
      // Add 150 items
      for (let i = 0; i < 150; i++) {
        store.addToHistory({
          id: `hist-${i}`,
          query: `SELECT ${i}`,
          database: 'testdb',
          executedAt: new Date(),
          duration: 10,
          success: true,
        });
      }
      
      expect(store.queryHistory).toHaveLength(100);
      // Most recent item should be hist-149
      expect(store.queryHistory[0].id).toBe('hist-149');
      // Oldest item should be hist-50
      expect(store.queryHistory[99].id).toBe('hist-50');
    });

    it('should clear all history', () => {
      const store = useSQLEditorStore();
      
      store.addToHistory({
        id: 'hist-1',
        query: 'SELECT 1',
        database: 'testdb',
        executedAt: new Date(),
        duration: 10,
        success: true,
      });
      
      store.clearHistory();
      
      expect(store.queryHistory).toHaveLength(0);
    });

    it('should remove specific history item', () => {
      const store = useSQLEditorStore();
      
      store.addToHistory({
        id: 'hist-1',
        query: 'SELECT 1',
        database: 'testdb',
        executedAt: new Date(),
        duration: 10,
        success: true,
      });
      
      store.addToHistory({
        id: 'hist-2',
        query: 'SELECT 2',
        database: 'testdb',
        executedAt: new Date(),
        duration: 20,
        success: true,
      });
      
      store.removeHistoryItem('hist-1');
      
      expect(store.queryHistory).toHaveLength(1);
      expect(store.queryHistory[0].id).toBe('hist-2');
    });

    it('should filter history by database', () => {
      const store = useSQLEditorStore();
      
      store.addToHistory({
        id: 'hist-1',
        query: 'SELECT 1',
        database: 'db1',
        executedAt: new Date(),
        duration: 10,
        success: true,
      });
      
      store.addToHistory({
        id: 'hist-2',
        query: 'SELECT 2',
        database: 'db2',
        executedAt: new Date(),
        duration: 20,
        success: true,
      });
      
      store.addToHistory({
        id: 'hist-3',
        query: 'SELECT 3',
        database: 'db1',
        executedAt: new Date(),
        duration: 30,
        success: true,
      });
      
      const db1History = store.getHistoryByDatabase('db1');
      
      expect(db1History).toHaveLength(2);
      expect(db1History.map(h => h.id)).toContain('hist-1');
      expect(db1History.map(h => h.id)).toContain('hist-3');
    });

    it('should search history by query text', () => {
      const store = useSQLEditorStore();
      
      store.addToHistory({
        id: 'hist-1',
        query: 'SELECT * FROM users',
        database: 'testdb',
        executedAt: new Date(),
        duration: 10,
        success: true,
      });
      
      store.addToHistory({
        id: 'hist-2',
        query: 'SELECT * FROM orders',
        database: 'testdb',
        executedAt: new Date(),
        duration: 20,
        success: true,
      });
      
      const results = store.searchHistory('users');
      
      expect(results).toHaveLength(1);
      expect(results[0].id).toBe('hist-1');
    });

    it('should search history by database name', () => {
      const store = useSQLEditorStore();
      
      store.addToHistory({
        id: 'hist-1',
        query: 'SELECT 1',
        database: 'production',
        executedAt: new Date(),
        duration: 10,
        success: true,
      });
      
      store.addToHistory({
        id: 'hist-2',
        query: 'SELECT 2',
        database: 'development',
        executedAt: new Date(),
        duration: 20,
        success: true,
      });
      
      const results = store.searchHistory('prod');
      
      expect(results).toHaveLength(1);
      expect(results[0].id).toBe('hist-1');
    });

    it('should return all history when search text is empty', () => {
      const store = useSQLEditorStore();
      
      store.addToHistory({
        id: 'hist-1',
        query: 'SELECT 1',
        database: 'testdb',
        executedAt: new Date(),
        duration: 10,
        success: true,
      });
      
      store.addToHistory({
        id: 'hist-2',
        query: 'SELECT 2',
        database: 'testdb',
        executedAt: new Date(),
        duration: 20,
        success: true,
      });
      
      const results = store.searchHistory('');
      
      expect(results).toHaveLength(2);
    });
  });

  describe('LocalStorage Persistence', () => {
    it('should save history to localStorage', () => {
      const store = useSQLEditorStore();
      
      const historyItem: QueryHistoryItem = {
        id: 'hist-1',
        query: 'SELECT * FROM users',
        database: 'testdb',
        executedAt: new Date(),
        duration: 100,
        success: true,
      };
      
      store.addToHistory(historyItem);
      
      const stored = localStorage.getItem('sql-editor-query-history');
      expect(stored).toBeTruthy();
      
      const parsed = JSON.parse(stored!);
      expect(parsed).toHaveLength(1);
      expect(parsed[0].id).toBe('hist-1');
    });

    it('should load history from localStorage', () => {
      const historyItem: QueryHistoryItem = {
        id: 'hist-1',
        query: 'SELECT * FROM users',
        database: 'testdb',
        executedAt: new Date(),
        duration: 100,
        success: true,
      };
      
      localStorage.setItem('sql-editor-query-history', JSON.stringify([historyItem]));
      
      const store = useSQLEditorStore();
      store.loadHistoryFromStorage();
      
      expect(store.queryHistory).toHaveLength(1);
      expect(store.queryHistory[0].id).toBe('hist-1');
      expect(store.queryHistory[0].executedAt).toBeInstanceOf(Date);
    });

    it('should handle corrupted localStorage data gracefully', () => {
      localStorage.setItem('sql-editor-query-history', 'invalid json');
      
      const store = useSQLEditorStore();
      store.loadHistoryFromStorage();
      
      expect(store.queryHistory).toHaveLength(0);
    });

    it('should clear localStorage when clearing history', () => {
      const store = useSQLEditorStore();
      
      store.addToHistory({
        id: 'hist-1',
        query: 'SELECT 1',
        database: 'testdb',
        executedAt: new Date(),
        duration: 10,
        success: true,
      });
      
      store.clearHistory();
      
      const stored = localStorage.getItem('sql-editor-query-history');
      const parsed = JSON.parse(stored!);
      expect(parsed).toHaveLength(0);
    });

    it('should initialize store by loading history', () => {
      const historyItem: QueryHistoryItem = {
        id: 'hist-1',
        query: 'SELECT * FROM users',
        database: 'testdb',
        executedAt: new Date(),
        duration: 100,
        success: true,
      };
      
      localStorage.setItem('sql-editor-query-history', JSON.stringify([historyItem]));
      
      const store = useSQLEditorStore();
      store.initialize();
      
      expect(store.queryHistory).toHaveLength(1);
      expect(store.queryHistory[0].id).toBe('hist-1');
    });
  });

  describe('Edge Cases', () => {
    it('should handle operations on non-existent tabs gracefully', () => {
      const store = useSQLEditorStore();
      
      // These should not throw errors
      expect(() => {
        store.closeTab('non-existent');
        store.updateTabContent('non-existent', 'SELECT 1');
        store.updateTabLabel('non-existent', 'Label');
        store.setTabDatabase('non-existent', 'db');
        store.setTabExecuting('non-existent', true);
      }).not.toThrow();
    });

    it('should handle removing non-existent history item', () => {
      const store = useSQLEditorStore();
      
      expect(() => {
        store.removeHistoryItem('non-existent');
      }).not.toThrow();
      
      expect(store.queryHistory).toHaveLength(0);
    });

    it('should handle setting active tab to non-existent ID', () => {
      const store = useSQLEditorStore();
      
      store.createTab();
      const originalActiveId = store.activeTabId;
      
      store.setActiveTab('non-existent');
      
      // Active tab should not change
      expect(store.activeTabId).toBe(originalActiveId);
    });
  });

  /**
   * Property-Based Tests
   * 
   * These tests use fast-check to verify properties hold across many randomly generated inputs
   */
  describe('Property-Based Tests', () => {
    /**
     * Feature: database-advanced-features, Property 6: 查询历史完整性
     * **Validates: Requirements 4.1**
     * 
     * For any executed SQL query, the query history should save a record containing:
     * - Query text
     * - Execution time
     * - Database name
     * - Execution result (success/failure)
     */
    it('Property 6: Query history integrity - all executed queries are saved with complete information', () => {
      fc.assert(
        fc.property(
          // Generate arbitrary query history items
          fc.array(
            fc.record({
              id: fc.string({ minLength: 1, maxLength: 50 }),
              query: fc.string({ minLength: 1, maxLength: 500 }),
              database: fc.string({ minLength: 1, maxLength: 100 }),
              executedAt: fc.date(),
              duration: fc.integer({ min: 0, max: 60000 }), // 0-60 seconds
              success: fc.boolean(),
              error: fc.option(fc.string({ minLength: 1, maxLength: 200 }), { nil: undefined }),
            }),
            { minLength: 1, maxLength: 50 }
          ),
          (queryHistoryItems) => {
            // Create a fresh store for each test
            setActivePinia(createPinia());
            localStorage.clear();
            const store = useSQLEditorStore();

            // Add all generated query history items to the store
            for (const item of queryHistoryItems) {
              store.addToHistory(item);
            }

            // Property: All added items should be in the history (up to MAX_HISTORY_ITEMS = 100)
            const expectedCount = Math.min(queryHistoryItems.length, 100);
            expect(store.queryHistory).toHaveLength(expectedCount);

            // Property: Each history item must contain all required fields
            for (const historyItem of store.queryHistory) {
              // Must have query text
              expect(historyItem.query).toBeDefined();
              expect(typeof historyItem.query).toBe('string');
              expect(historyItem.query.length).toBeGreaterThan(0);

              // Must have execution time
              expect(historyItem.executedAt).toBeDefined();
              expect(historyItem.executedAt).toBeInstanceOf(Date);

              // Must have database name
              expect(historyItem.database).toBeDefined();
              expect(typeof historyItem.database).toBe('string');
              expect(historyItem.database.length).toBeGreaterThan(0);

              // Must have execution result (success/failure)
              expect(historyItem.success).toBeDefined();
              expect(typeof historyItem.success).toBe('boolean');

              // Must have duration
              expect(historyItem.duration).toBeDefined();
              expect(typeof historyItem.duration).toBe('number');
              expect(historyItem.duration).toBeGreaterThanOrEqual(0);

              // Must have unique ID
              expect(historyItem.id).toBeDefined();
              expect(typeof historyItem.id).toBe('string');
              expect(historyItem.id.length).toBeGreaterThan(0);

              // If success is false, error may be present
              if (!historyItem.success && historyItem.error !== undefined) {
                expect(typeof historyItem.error).toBe('string');
              }
            }

            // Property: Most recent items should be first (LIFO order)
            // The first item in history should be the last item we added (within the 100 limit)
            if (queryHistoryItems.length > 0) {
              const lastAddedItem = queryHistoryItems[queryHistoryItems.length - 1];
              const firstHistoryItem = store.queryHistory[0];
              
              expect(firstHistoryItem.id).toBe(lastAddedItem.id);
              expect(firstHistoryItem.query).toBe(lastAddedItem.query);
              expect(firstHistoryItem.database).toBe(lastAddedItem.database);
            }

            // Property: History should be persisted to localStorage
            const storedHistory = localStorage.getItem('sql-editor-query-history');
            expect(storedHistory).toBeTruthy();
            
            const parsedHistory = JSON.parse(storedHistory!);
            expect(parsedHistory).toHaveLength(expectedCount);

            // Property: Each stored item should have all required fields
            for (const storedItem of parsedHistory) {
              expect(storedItem.query).toBeDefined();
              expect(storedItem.database).toBeDefined();
              expect(storedItem.executedAt).toBeDefined();
              expect(storedItem.duration).toBeDefined();
              expect(storedItem.success).toBeDefined();
              expect(storedItem.id).toBeDefined();
            }

            // Cleanup
            localStorage.clear();
          }
        ),
        { numRuns: 100 } // Run 100 iterations as specified
      );
    });

    /**
     * Property: Query history filtering by database preserves all required fields
     */
    it('Property 6 (extended): Filtered query history maintains data integrity', () => {
      fc.assert(
        fc.property(
          // Generate queries for multiple databases
          fc.array(
            fc.record({
              id: fc.string({ minLength: 1, maxLength: 50 }),
              query: fc.string({ minLength: 1, maxLength: 500 }),
              database: fc.constantFrom('db1', 'db2', 'db3', 'testdb', 'production'),
              executedAt: fc.date(),
              duration: fc.integer({ min: 0, max: 60000 }),
              success: fc.boolean(),
              error: fc.option(fc.string({ minLength: 1, maxLength: 200 }), { nil: undefined }),
            }),
            { minLength: 5, maxLength: 30 }
          ),
          fc.constantFrom('db1', 'db2', 'db3', 'testdb', 'production'),
          (queryHistoryItems, filterDatabase) => {
            // Create a fresh store for each test
            setActivePinia(createPinia());
            localStorage.clear();
            const store = useSQLEditorStore();

            // Add all items
            for (const item of queryHistoryItems) {
              store.addToHistory(item);
            }

            // Filter by database
            const filteredHistory = store.getHistoryByDatabase(filterDatabase);

            // Property: All filtered items must belong to the specified database
            for (const item of filteredHistory) {
              expect(item.database).toBe(filterDatabase);
            }

            // Property: All filtered items must have complete information
            for (const item of filteredHistory) {
              expect(item.query).toBeDefined();
              expect(item.database).toBe(filterDatabase);
              expect(item.executedAt).toBeInstanceOf(Date);
              expect(item.duration).toBeGreaterThanOrEqual(0);
              expect(typeof item.success).toBe('boolean');
              expect(item.id).toBeDefined();
            }

            // Property: Count of filtered items should match expected count
            const expectedCount = queryHistoryItems
              .slice(-100) // Only last 100 items are kept
              .filter(item => item.database === filterDatabase)
              .length;
            
            expect(filteredHistory).toHaveLength(expectedCount);

            // Cleanup
            localStorage.clear();
          }
        ),
        { numRuns: 100 }
      );
    });

    /**
     * Property: Query history search maintains data integrity
     */
    it('Property 6 (extended): Search results maintain complete query information', () => {
      fc.assert(
        fc.property(
          fc.array(
            fc.record({
              id: fc.string({ minLength: 1, maxLength: 50 }),
              query: fc.oneof(
                fc.constant('SELECT * FROM users'),
                fc.constant('SELECT * FROM orders'),
                fc.constant('INSERT INTO products'),
                fc.constant('UPDATE customers SET'),
                fc.constant('DELETE FROM logs')
              ),
              database: fc.constantFrom('testdb', 'production', 'development'),
              executedAt: fc.date(),
              duration: fc.integer({ min: 0, max: 60000 }),
              success: fc.boolean(),
              error: fc.option(fc.string({ minLength: 1, maxLength: 200 }), { nil: undefined }),
            }),
            { minLength: 5, maxLength: 30 }
          ),
          fc.constantFrom('users', 'orders', 'products', 'SELECT', 'testdb'),
          (queryHistoryItems, searchTerm) => {
            // Create a fresh store
            setActivePinia(createPinia());
            localStorage.clear();
            const store = useSQLEditorStore();

            // Add all items
            for (const item of queryHistoryItems) {
              store.addToHistory(item);
            }

            // Search history
            const searchResults = store.searchHistory(searchTerm);

            // Property: All search results must contain the search term (case-insensitive)
            const lowerSearchTerm = searchTerm.toLowerCase();
            for (const item of searchResults) {
              const matchesQuery = item.query.toLowerCase().includes(lowerSearchTerm);
              const matchesDatabase = item.database.toLowerCase().includes(lowerSearchTerm);
              expect(matchesQuery || matchesDatabase).toBe(true);
            }

            // Property: All search results must have complete information
            for (const item of searchResults) {
              expect(item.query).toBeDefined();
              expect(item.database).toBeDefined();
              expect(item.executedAt).toBeInstanceOf(Date);
              expect(item.duration).toBeGreaterThanOrEqual(0);
              expect(typeof item.success).toBe('boolean');
              expect(item.id).toBeDefined();
            }

            // Cleanup
            localStorage.clear();
          }
        ),
        { numRuns: 100 }
      );
    });
  });
});
