/**
 * SQL Editor Store
 * 
 * Manages SQL editor state including:
 * - Editor tabs (create, close, switch)
 * - Query history (add, clear, filter by database)
 * - Query execution state
 * 
 * Validates: Requirements 1.1, 4.1
 */

import { defineStore } from 'pinia';
import type { EditorTab, QueryHistoryItem, QueryResult } from '@/types/sql-editor';

const HISTORY_STORAGE_KEY = 'sql-editor-query-history';
const MAX_HISTORY_ITEMS = 100;

interface SQLEditorState {
  /** All open editor tabs */
  tabs: EditorTab[];
  /** ID of the currently active tab */
  activeTabId: string | null;
  /** Query execution history */
  queryHistory: QueryHistoryItem[];
}

export const useSQLEditorStore = defineStore('sql-editor', {
  state: (): SQLEditorState => ({
    tabs: [],
    activeTabId: null,
    queryHistory: [],
  }),

  getters: {
    /**
     * Get the currently active tab
     */
    activeTab: (state): EditorTab | undefined => {
      return state.tabs.find(tab => tab.id === state.activeTabId);
    },

    /**
     * Get query history filtered by database
     */
    getHistoryByDatabase: (state) => {
      return (database: string): QueryHistoryItem[] => {
        return state.queryHistory.filter(item => item.database === database);
      };
    },

    /**
     * Check if there are any tabs open
     */
    hasTabs: (state): boolean => {
      return state.tabs.length > 0;
    },

    /**
     * Get tab by ID
     */
    getTabById: (state) => {
      return (tabId: string): EditorTab | undefined => {
        return state.tabs.find(tab => tab.id === tabId);
      };
    },
  },

  actions: {
    /**
     * Create a new editor tab
     * @param content - Initial SQL content for the tab
     * @param database - Database to associate with the tab
     * @returns The ID of the newly created tab
     */
    createTab(content: string = '', database?: string): string {
      const tabId = `tab-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
      const tabNumber = this.tabs.length + 1;
      
      const newTab: EditorTab = {
        id: tabId,
        label: `Query ${tabNumber}`,
        content,
        database,
        isExecuting: false,
        isDirty: false,
      };

      this.tabs.push(newTab);
      this.activeTabId = tabId;

      return tabId;
    },

    /**
     * Close an editor tab
     * @param tabId - ID of the tab to close
     */
    closeTab(tabId: string): void {
      const index = this.tabs.findIndex(tab => tab.id === tabId);
      
      if (index === -1) {
        return;
      }

      // Remove the tab
      this.tabs.splice(index, 1);

      // Update active tab if necessary
      if (this.activeTabId === tabId) {
        if (this.tabs.length > 0) {
          // Set active to the previous tab, or the first tab if we closed the first one
          const newIndex = Math.max(0, index - 1);
          this.activeTabId = this.tabs[newIndex]?.id || null;
        } else {
          this.activeTabId = null;
        }
      }
    },

    /**
     * Update the content of a tab
     * @param tabId - ID of the tab to update
     * @param content - New SQL content
     */
    updateTabContent(tabId: string, content: string): void {
      const tab = this.tabs.find(t => t.id === tabId);
      
      if (tab) {
        tab.content = content;
        tab.isDirty = true;
      }
    },

    /**
     * Set the active tab
     * @param tabId - ID of the tab to activate
     */
    setActiveTab(tabId: string): void {
      const tab = this.tabs.find(t => t.id === tabId);
      
      if (tab) {
        this.activeTabId = tabId;
      }
    },

    /**
     * Update the label of a tab
     * @param tabId - ID of the tab to update
     * @param label - New label for the tab
     */
    updateTabLabel(tabId: string, label: string): void {
      const tab = this.tabs.find(t => t.id === tabId);
      
      if (tab) {
        tab.label = label;
      }
    },

    /**
     * Set the database for a tab
     * @param tabId - ID of the tab to update
     * @param database - Database name
     */
    setTabDatabase(tabId: string, database: string): void {
      const tab = this.tabs.find(t => t.id === tabId);
      
      if (tab) {
        tab.database = database;
      }
    },

    /**
     * Set the executing state of a tab
     * @param tabId - ID of the tab to update
     * @param isExecuting - Whether the tab is executing a query
     */
    setTabExecuting(tabId: string, isExecuting: boolean): void {
      const tab = this.tabs.find(t => t.id === tabId);
      
      if (tab) {
        tab.isExecuting = isExecuting;
      }
    },

    /**
     * Set the result of a query execution
     * @param tabId - ID of the tab to update
     * @param result - Query result
     */
    setTabResult(tabId: string, result: QueryResult): void {
      const tab = this.tabs.find(t => t.id === tabId);
      
      if (tab) {
        tab.result = result;
        tab.isDirty = false;
      }
    },

    /**
     * Add a query to the history
     * @param item - Query history item to add
     */
    addToHistory(item: QueryHistoryItem): void {
      // Add to the beginning of the array (most recent first)
      this.queryHistory.unshift(item);

      // Limit history size
      if (this.queryHistory.length > MAX_HISTORY_ITEMS) {
        this.queryHistory = this.queryHistory.slice(0, MAX_HISTORY_ITEMS);
      }

      // Persist to localStorage
      this.saveHistoryToStorage();
    },

    /**
     * Clear all query history
     */
    clearHistory(): void {
      this.queryHistory = [];
      this.saveHistoryToStorage();
    },

    /**
     * Remove a specific history item
     * @param itemId - ID of the history item to remove
     */
    removeHistoryItem(itemId: string): void {
      const index = this.queryHistory.findIndex(item => item.id === itemId);
      
      if (index !== -1) {
        this.queryHistory.splice(index, 1);
        this.saveHistoryToStorage();
      }
    },

    /**
     * Delete a history item (alias for removeHistoryItem)
     * @param itemId - ID of the history item to delete
     */
    deleteHistoryItem(itemId: string): void {
      this.removeHistoryItem(itemId);
    },

    /**
     * Load query history from localStorage
     */
    loadHistoryFromStorage(): void {
      try {
        const stored = localStorage.getItem(HISTORY_STORAGE_KEY);
        
        if (stored) {
          const parsed = JSON.parse(stored);
          
          // Convert date strings back to Date objects
          this.queryHistory = parsed.map((item: any) => ({
            ...item,
            executedAt: new Date(item.executedAt),
          }));
        }
      } catch (error) {
        console.error('Failed to load query history from storage:', error);
        this.queryHistory = [];
      }
    },

    /**
     * Save query history to localStorage
     */
    saveHistoryToStorage(): void {
      try {
        localStorage.setItem(HISTORY_STORAGE_KEY, JSON.stringify(this.queryHistory));
      } catch (error) {
        console.error('Failed to save query history to storage:', error);
      }
    },

    /**
     * Search query history
     * @param searchText - Text to search for in queries
     * @returns Filtered history items
     */
    searchHistory(searchText: string): QueryHistoryItem[] {
      if (!searchText.trim()) {
        return this.queryHistory;
      }

      const lowerSearch = searchText.toLowerCase();
      
      return this.queryHistory.filter(item =>
        item.query.toLowerCase().includes(lowerSearch) ||
        item.database.toLowerCase().includes(lowerSearch)
      );
    },

    /**
     * Close all tabs
     */
    closeAllTabs(): void {
      this.tabs = [];
      this.activeTabId = null;
    },

    /**
     * Close all tabs except the specified one
     * @param tabId - ID of the tab to keep open
     */
    closeOtherTabs(tabId: string): void {
      const tab = this.tabs.find(t => t.id === tabId);
      
      if (tab) {
        this.tabs = [tab];
        this.activeTabId = tabId;
      }
    },

    /**
     * Get tabs that have unsaved changes
     */
    getDirtyTabs(): EditorTab[] {
      return this.tabs.filter(tab => tab.isDirty);
    },

    /**
     * Initialize the store (load history from storage)
     */
    initialize(): void {
      this.loadHistoryFromStorage();
    },
  },
});
