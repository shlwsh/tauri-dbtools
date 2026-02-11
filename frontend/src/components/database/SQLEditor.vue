<template>
  <div class="sql-editor-container">
    <!-- Toolbar -->
    <div class="editor-toolbar">
      <n-space>
        <n-button
          type="primary"
          :disabled="isExecuting || !currentDatabase"
          :loading="isExecuting"
          @click="handleExecute"
        >
          <template #icon>
            <n-icon><PlayCircleOutline /></n-icon>
          </template>
          Execute (Ctrl+Enter)
        </n-button>
        
        <n-button
          :disabled="isExecuting"
          @click="handleClear"
        >
          <template #icon>
            <n-icon><TrashOutline /></n-icon>
          </template>
          Clear
        </n-button>

        <n-divider vertical />

        <n-select
          v-model:value="currentDatabase"
          :options="databaseOptions"
          placeholder="Select Database"
          style="width: 200px"
          :disabled="isExecuting"
        />

        <n-button
          @click="showHistory = true"
        >
          <template #icon>
            <n-icon><TimeOutline /></n-icon>
          </template>
          History
        </n-button>
      </n-space>

      <n-space v-if="!currentDatabase">
        <n-alert
          type="warning"
          :show-icon="false"
        >
          Please select a database to execute queries
        </n-alert>
      </n-space>
    </div>

    <!-- Tabs -->
    <n-tabs
      v-model:value="activeTabId"
      type="card"
      class="editor-tabs"
      closable
      addable
      @add="handleAddTab"
      @close="handleCloseTab"
      @update:value="handleTabChange"
    >
      <n-tab-pane
        v-for="tab in tabs"
        :key="tab.id"
        :name="tab.id"
        :tab="tab.label + (tab.isDirty ? ' *' : '')"
      >
        <div class="tab-content">
          <!-- Monaco Editor -->
          <div
            ref="editorContainer"
            class="monaco-editor-wrapper"
          />
          
          <!-- Result Panel -->
          <div class="result-panel-wrapper">
            <ResultPanel :result="tab.result" />
          </div>
        </div>
      </n-tab-pane>
    </n-tabs>

    <!-- Query History Drawer -->
    <n-drawer
      v-model:show="showHistory"
      :width="400"
      placement="right"
    >
      <n-drawer-content title="Query History">
        <QueryHistoryPanel @load-query="handleLoadQuery" />
      </n-drawer-content>
    </n-drawer>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch, nextTick } from 'vue';
import {
  NButton,
  NSpace,
  NIcon,
  NTabs,
  NTabPane,
  NSelect,
  NAlert,
  NDivider,
  NDrawer,
  NDrawerContent,
} from 'naive-ui';
import {
  PlayCircleOutline,
  TrashOutline,
  TimeOutline,
} from '@vicons/ionicons5';
import * as monaco from 'monaco-editor';
import { useSQLEditorStore } from '@/stores/sql-editor';
import { useDatabaseStore } from '@/stores/database';
import { useNotification } from '@/composables/useNotification';
import { autoCompleter } from '@/services/auto-completer';
import { executeSql } from '@/api/database';
import QueryHistoryPanel from './QueryHistoryPanel.vue';
import ResultPanel from './ResultPanel.vue';

// Store
const sqlEditorStore = useSQLEditorStore();
const databaseStore = useDatabaseStore();
const notification = useNotification();

// Refs
const editorContainer = ref<HTMLElement>();
const showHistory = ref(false);
let editorInstance: monaco.editor.IStandaloneCodeEditor | null = null;

// Computed
const tabs = computed(() => sqlEditorStore.tabs);
const activeTabId = computed({
  get: () => sqlEditorStore.activeTabId || undefined,
  set: (value) => {
    if (value) {
      sqlEditorStore.setActiveTab(value as string);
    }
  },
});

const activeTab = computed(() => sqlEditorStore.activeTab);
const isExecuting = computed(() => activeTab.value?.isExecuting || false);

const currentDatabase = computed({
  get: () => activeTab.value?.database || databaseStore.currentDatabase || '',
  set: (value) => {
    if (activeTab.value && value) {
      sqlEditorStore.setTabDatabase(activeTab.value.id, value as string);
    }
  },
});

const databaseOptions = computed(() => {
  return databaseStore.databases.map(db => ({
    label: db,
    value: db,
  }));
});

// Initialize Monaco Editor
const initializeEditor = async () => {
  if (!editorContainer.value) return;

  // Configure SQL language
  monaco.languages.register({ id: 'pgsql' });

  // Register auto-completion provider
  monaco.languages.registerCompletionItemProvider('pgsql', {
    provideCompletionItems: async (model, position) => {
      const word = model.getWordUntilPosition(position);
      const range = {
        startLineNumber: position.lineNumber,
        endLineNumber: position.lineNumber,
        startColumn: word.startColumn,
        endColumn: word.endColumn,
      };

      // Get completion items
      const items = await autoCompleter.getAllCompletionItems(
        currentDatabase.value || '',
        word.word
      );

      // Convert to Monaco completion items
      const suggestions: monaco.languages.CompletionItem[] = items.map(item => {
        let kind: monaco.languages.CompletionItemKind;
        
        switch (item.kind) {
          case 'keyword':
            kind = monaco.languages.CompletionItemKind.Keyword;
            break;
          case 'table':
            kind = monaco.languages.CompletionItemKind.Class;
            break;
          case 'column':
            kind = monaco.languages.CompletionItemKind.Field;
            break;
          case 'function':
            kind = monaco.languages.CompletionItemKind.Function;
            break;
          default:
            kind = monaco.languages.CompletionItemKind.Text;
        }

        return {
          label: item.label,
          kind,
          detail: item.detail,
          documentation: item.documentation,
          insertText: item.label,
          range,
        };
      });

      return { suggestions };
    },
    triggerCharacters: ['.', ' '],
  });

  // Set SQL syntax highlighting
  monaco.languages.setMonarchTokensProvider('pgsql', {
    defaultToken: '',
    tokenPostfix: '.sql',
    ignoreCase: true,

    keywords: [
      'SELECT', 'FROM', 'WHERE', 'INSERT', 'UPDATE', 'DELETE', 'CREATE', 'ALTER',
      'DROP', 'TABLE', 'INDEX', 'VIEW', 'TRIGGER', 'PROCEDURE', 'FUNCTION',
      'DATABASE', 'SCHEMA', 'CONSTRAINT', 'PRIMARY', 'FOREIGN', 'KEY', 'REFERENCES',
      'UNIQUE', 'CHECK', 'DEFAULT', 'NOT', 'NULL', 'AND', 'OR', 'IN', 'BETWEEN',
      'LIKE', 'IS', 'AS', 'JOIN', 'LEFT', 'RIGHT', 'INNER', 'OUTER', 'ON',
      'GROUP', 'BY', 'HAVING', 'ORDER', 'ASC', 'DESC', 'LIMIT', 'OFFSET',
      'UNION', 'INTERSECT', 'EXCEPT', 'CASE', 'WHEN', 'THEN', 'ELSE', 'END',
      'BEGIN', 'COMMIT', 'ROLLBACK', 'TRANSACTION', 'CASCADE', 'RESTRICT',
      'SET', 'VALUES', 'INTO', 'RETURNING', 'WITH', 'RECURSIVE', 'DISTINCT',
      'ALL', 'ANY', 'SOME', 'EXISTS', 'CAST', 'COALESCE', 'NULLIF',
    ],

    operators: [
      '=', '>', '<', '!', '~', '?', ':', '==', '<=', '>=', '!=',
      '<>', '&&', '||', '++', '--', '+', '-', '*', '/', '&', '|', '^', '%',
      '<<', '>>', '>>>', '+=', '-=', '*=', '/=', '&=', '|=', '^=',
      '%=', '<<=', '>>=', '>>>=',
    ],

    builtinFunctions: [
      'COUNT', 'SUM', 'AVG', 'MIN', 'MAX', 'CONCAT', 'SUBSTRING', 'UPPER',
      'LOWER', 'TRIM', 'LENGTH', 'NOW', 'CURRENT_DATE', 'CURRENT_TIME',
      'CURRENT_TIMESTAMP', 'EXTRACT', 'DATE_PART', 'AGE', 'TO_CHAR',
      'TO_DATE', 'TO_TIMESTAMP', 'ARRAY_AGG', 'STRING_AGG', 'JSON_AGG',
      'JSONB_AGG', 'ROW_NUMBER', 'RANK', 'DENSE_RANK', 'LAG', 'LEAD',
    ],

    builtinTypes: [
      'INTEGER', 'INT', 'BIGINT', 'SMALLINT', 'DECIMAL', 'NUMERIC', 'REAL',
      'DOUBLE', 'PRECISION', 'VARCHAR', 'CHAR', 'TEXT', 'BOOLEAN', 'BOOL',
      'DATE', 'TIME', 'TIMESTAMP', 'TIMESTAMPTZ', 'INTERVAL', 'JSON', 'JSONB',
      'UUID', 'BYTEA', 'ARRAY', 'SERIAL', 'BIGSERIAL', 'SMALLSERIAL',
    ],

    tokenizer: {
      root: [
        { include: '@comments' },
        { include: '@whitespace' },
        { include: '@numbers' },
        { include: '@strings' },
        { include: '@complexIdentifiers' },
        [/[;,.]/, 'delimiter'],
        [/[()]/, '@brackets'],
        [
          /[\w@#$]+/,
          {
            cases: {
              '@keywords': 'keyword',
              '@operators': 'operator',
              '@builtinFunctions': 'predefined',
              '@builtinTypes': 'type',
              '@default': 'identifier',
            },
          },
        ],
        [/[<>=!%&+\-*/|~^]/, 'operator'],
      ],
      whitespace: [[/\s+/, 'white']],
      comments: [
        [/--+.*/, 'comment'],
        [/\/\*/, { token: 'comment.quote', next: '@comment' }],
      ],
      comment: [
        [/[^*/]+/, 'comment'],
        [/\*\//, { token: 'comment.quote', next: '@pop' }],
        [/./, 'comment'],
      ],
      numbers: [
        [/0[xX][0-9a-fA-F]*/, 'number'],
        [/[$][+-]*\d*(\.\d*)?/, 'number'],
        [/((\d+(\.\d*)?)|(\.\d+))([eE][+-]?\d+)?/, 'number'],
      ],
      strings: [
        [/'/, { token: 'string', next: '@string' }],
        [/"/, { token: 'string.double', next: '@stringDouble' }],
      ],
      string: [
        [/[^']+/, 'string'],
        [/''/, 'string'],
        [/'/, { token: 'string', next: '@pop' }],
      ],
      stringDouble: [
        [/[^"]+/, 'string.double'],
        [/""/, 'string.double'],
        [/"/, { token: 'string.double', next: '@pop' }],
      ],
      complexIdentifiers: [[/"/, { token: 'identifier.quote', next: '@quotedIdentifier' }]],
      quotedIdentifier: [
        [/[^"]+/, 'identifier'],
        [/""/, 'identifier'],
        [/"/, { token: 'identifier.quote', next: '@pop' }],
      ],
    },
  });

  // Create editor instance
  editorInstance = monaco.editor.create(editorContainer.value, {
    value: activeTab.value?.content || '',
    language: 'pgsql',
    theme: 'vs-dark',
    automaticLayout: true,
    minimap: { enabled: true },
    lineNumbers: 'on',
    scrollBeyondLastLine: false,
    fontSize: 14,
    tabSize: 2,
    wordWrap: 'on',
    suggest: {
      showKeywords: true,
    },
  });

  // Listen to content changes
  editorInstance.onDidChangeModelContent(() => {
    if (activeTab.value && editorInstance) {
      const content = editorInstance.getValue();
      sqlEditorStore.updateTabContent(activeTab.value.id, content);
    }
  });

  // Add keyboard shortcuts
  editorInstance.addCommand(
    monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter,
    () => {
      handleExecute();
    }
  );

  editorInstance.addCommand(
    monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyK,
    () => {
      handleClear();
    }
  );
};

// Handle tab operations
const handleAddTab = () => {
  sqlEditorStore.createTab('', currentDatabase.value);
  nextTick(() => {
    if (editorInstance) {
      editorInstance.setValue('');
      editorInstance.focus();
    }
  });
};

const handleCloseTab = (tabId: string) => {
  const tab = sqlEditorStore.getTabById(tabId);
  
  if (tab?.isDirty) {
    // TODO: Show confirmation dialog
    // For now, just close
  }
  
  sqlEditorStore.closeTab(tabId);
  
  // Update editor content when switching tabs
  nextTick(() => {
    if (editorInstance && activeTab.value) {
      editorInstance.setValue(activeTab.value.content);
    }
  });
};

const handleTabChange = (tabId: string) => {
  sqlEditorStore.setActiveTab(tabId);
  
  nextTick(() => {
    if (editorInstance && activeTab.value) {
      editorInstance.setValue(activeTab.value.content);
      editorInstance.focus();
    }
  });
};

// Handle editor actions
const handleExecute = async () => {
  if (!activeTab.value || !currentDatabase.value) {
    notification.showError('Please select a database first');
    return;
  }

  const selection = editorInstance?.getSelection();
  let sqlToExecute = '';

  if (selection && !selection.isEmpty()) {
    // Execute selected text
    sqlToExecute = editorInstance?.getModel()?.getValueInRange(selection) || '';
  } else {
    // Execute all content
    sqlToExecute = activeTab.value.content;
  }

  if (!sqlToExecute.trim()) {
    notification.showWarning('Please enter SQL to execute');
    return;
  }

  // Set executing state
  sqlEditorStore.setTabExecuting(activeTab.value.id, true);

  try {
    const startTime = Date.now();
    
    // Execute SQL
    const response = await executeSql(currentDatabase.value, sqlToExecute);
    
    const duration = Date.now() - startTime;

    if (response.success && response.data) {
      // Set result
      sqlEditorStore.setTabResult(activeTab.value.id, response.data);
      
      // Add to history
      sqlEditorStore.addToHistory({
        id: `history-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        query: sqlToExecute,
        database: currentDatabase.value,
        executedAt: new Date(),
        duration,
        success: true,
      });

      // Show success notification
      if (response.data.resultType === 'Select') {
        notification.showSuccess(`Query executed successfully. ${response.data.rows?.length || 0} rows returned in ${duration}ms`);
      } else if (response.data.resultType === 'Insert' || response.data.resultType === 'Update' || response.data.resultType === 'Delete') {
        notification.showSuccess(`Query executed successfully. ${response.data.affectedRows || 0} rows affected in ${duration}ms`);
      } else {
        notification.showSuccess(`Query executed successfully in ${duration}ms`);
      }
    } else {
      // Handle error
      const errorResult = {
        resultType: 'Error' as const,
        durationMs: duration,
        error: response.message || 'Unknown error occurred',
      };
      
      sqlEditorStore.setTabResult(activeTab.value.id, errorResult);
      
      // Add to history
      sqlEditorStore.addToHistory({
        id: `history-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        query: sqlToExecute,
        database: currentDatabase.value,
        executedAt: new Date(),
        duration,
        success: false,
        error: response.message,
      });

      notification.showError(`Query failed: ${response.message}`);
    }
  } catch (error) {
    console.error('Failed to execute SQL:', error);
    notification.showError(`Failed to execute SQL: ${error}`);
    
    // Set error result
    sqlEditorStore.setTabResult(activeTab.value.id, {
      type: 'error',
      duration: 0,
      error: String(error),
    });
  } finally {
    // Clear executing state
    sqlEditorStore.setTabExecuting(activeTab.value.id, false);
  }
};

const handleClear = () => {
  if (editorInstance) {
    editorInstance.setValue('');
    editorInstance.focus();
  }
};

const handleLoadQuery = (query: string) => {
  if (editorInstance) {
    editorInstance.setValue(query);
    editorInstance.focus();
  }
  showHistory.value = false;
};

// Watch for active tab changes
watch(activeTab, (newTab) => {
  if (newTab && editorInstance) {
    editorInstance.setValue(newTab.content);
  }
});

// Lifecycle
onMounted(async () => {
  // Initialize store
  sqlEditorStore.initialize();
  
  // Create initial tab if none exist
  if (tabs.value.length === 0) {
    sqlEditorStore.createTab('', currentDatabase.value);
  }

  // Initialize Monaco Editor
  await nextTick();
  await initializeEditor();
});

onBeforeUnmount(() => {
  if (editorInstance) {
    editorInstance.dispose();
    editorInstance = null;
  }
});
</script>

<style scoped>
.sql-editor-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
}

.editor-toolbar {
  padding: 12px 16px;
  border-bottom: 1px solid var(--n-border-color);
  background-color: var(--n-color);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.editor-tabs {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.editor-tabs :deep(.n-tabs-pane-wrapper) {
  flex: 1;
  overflow: hidden;
}

.editor-tabs :deep(.n-tab-pane) {
  height: 100%;
  padding: 0;
}

.tab-content {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.monaco-editor-wrapper {
  width: 100%;
  flex: 1;
  min-height: 300px;
}

.result-panel-wrapper {
  width: 100%;
  flex: 1;
  min-height: 200px;
  border-top: 1px solid var(--n-border-color);
  overflow: auto;
}
</style>
