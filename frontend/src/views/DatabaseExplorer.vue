<template>
  <div class="explorer-container">
    <!-- Left Sidebar: Database Tree -->
    <div class="explorer-sidebar">
      <div class="sidebar-header">
        <n-text strong>连接</n-text>
        <n-button text @click="loadDatabases" :loading="loadingDatabases">
          <template #icon>
            <n-icon><RefreshIcon /></n-icon>
          </template>
        </n-button>
      </div>

      <div class="sidebar-content">
        <n-spin :show="loadingDatabases || loadingTables">
          <n-tree
            block-line
            :data="treeData"
            :expanded-keys="expandedKeys"
            :selected-keys="selectedKeys"
            :node-props="nodeProps"
            @update:expanded-keys="handleExpandedKeysChange"
            @update:selected-keys="handleSelectedKeysChange"
          />
        </n-spin>
      </div>
    </div>

    <!-- Right Content: SQL Editor and Results -->
    <div class="explorer-content">
      <n-tabs v-model:value="activeTab" type="card" closable @close="handleTabClose">
        <n-tab-pane
          v-for="tab in tabs"
          :key="tab.key"
          :name="tab.key"
          :tab="tab.label"
          :closable="tab.closable"
        >
          <!-- SQL Editor Tab -->
          <div v-if="tab.type === 'sql'" class="sql-editor-container">
            <div class="editor-toolbar">
              <n-space>
                <n-button size="small" type="primary" @click="executeQuery(tab.key)">
                  <template #icon>
                    <n-icon><PlayIcon /></n-icon>
                  </template>
                  执行
                </n-button>
                <n-button size="small" @click="clearEditor(tab.key)">
                  <template #icon>
                    <n-icon><ClearIcon /></n-icon>
                  </template>
                  清空
                </n-button>
              </n-space>
            </div>

            <div class="editor-area">
              <n-input
                v-model:value="tab.content"
                type="textarea"
                placeholder="输入 SQL 查询..."
                :autosize="{ minRows: 10, maxRows: 20 }"
                :input-props="{ spellcheck: false }"
              />
            </div>

            <div v-if="tab.result" class="result-area">
              <div class="result-header">
                <n-text strong>查询结果</n-text>
                <n-text depth="3" style="font-size: 12px">
                  {{ tab.result.rowCount }} 行，耗时 {{ tab.result.duration }}ms
                </n-text>
              </div>
              <n-data-table
                :columns="tab.result.columns"
                :data="tab.result.data"
                :max-height="300"
                :bordered="true"
                size="small"
                striped
              />
            </div>
          </div>

          <!-- Table Data Tab -->
          <div v-else-if="tab.type === 'table'" class="table-view-container">
            <div class="table-toolbar">
              <n-space justify="space-between">
                <n-space>
                  <n-text strong>{{ tab.tableName }}</n-text>
                  <n-text depth="3" style="font-size: 12px">
                    {{ tab.schema }}
                  </n-text>
                </n-space>
                <n-space>
                  <n-button size="small" @click="refreshTableData(tab.key)">
                    <template #icon>
                      <n-icon><RefreshIcon /></n-icon>
                    </template>
                    刷新
                  </n-button>
                  <n-button size="small" type="primary" @click="addRecord(tab.key)">
                    <template #icon>
                      <n-icon><AddIcon /></n-icon>
                    </template>
                    新增
                  </n-button>
                </n-space>
              </n-space>
            </div>

            <n-spin :show="tab.loading">
              <n-data-table
                :columns="tab.columns"
                :data="tab.data"
                :pagination="tab.pagination"
                :bordered="true"
                size="small"
                striped
                @update:page="(page) => handlePageChange(tab.key, page)"
                @update:page-size="(size) => handlePageSizeChange(tab.key, size)"
              />
            </n-spin>
          </div>
        </n-tab-pane>

        <!-- Add New Query Tab Button -->
        <template #suffix>
          <n-button text @click="addSqlTab">
            <template #icon>
              <n-icon><AddIcon /></n-icon>
            </template>
          </n-button>
        </template>
      </n-tabs>
    </div>

    <!-- Record Editor Modal -->
    <n-modal
      v-model:show="showEditor"
      :title="editingRecord ? '编辑记录' : '新增记录'"
      preset="dialog"
      :positive-text="editingRecord ? '保存' : '创建'"
      negative-text="取消"
      @positive-click="handleSaveRecord"
      style="width: 600px"
    >
      <n-form :model="formData" label-placement="left" label-width="120">
        <n-form-item
          v-for="col in currentColumns"
          :key="col.name"
          :label="col.name"
        >
          <n-input
            v-model:value="formData[col.name]"
            :placeholder="`${col.type}${col.nullable ? ' (可为空)' : ''}`"
            :disabled="editingRecord && col.isPrimaryKey"
          />
          <template #label>
            <n-space size="small" align="center">
              <n-text>{{ col.name }}</n-text>
              <n-tag v-if="col.isPrimaryKey" size="tiny" type="warning">PK</n-tag>
            </n-space>
          </template>
        </n-form-item>
      </n-form>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, h } from 'vue';
import {
  NText,
  NButton,
  NIcon,
  NSpin,
  NTree,
  NTabs,
  NTabPane,
  NSpace,
  NInput,
  NDataTable,
  NModal,
  NForm,
  NFormItem,
  NTag,
  NPopconfirm,
  type TreeOption,
  type DataTableColumns,
} from 'naive-ui';
import { listDatabases } from '@/api/database';
import { listTables, getTableData, createRecord, updateRecord, deleteRecord } from '@/api/explorer';
import { useNotification } from '@/composables/useNotification';
import type { TableInfo, ColumnInfo } from '@/types/database';

// Icons
const RefreshIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', {
      d: 'M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z',
    }),
  ]);

const DatabaseIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', {
      d: 'M12 3C7.58 3 4 4.79 4 7s3.58 4 8 4 8-1.79 8-4-3.58-4-8-4zM4 9v3c0 2.21 3.58 4 8 4s8-1.79 8-4V9c0 2.21-3.58 4-8 4s-8-1.79-8-4zm0 5v3c0 2.21 3.58 4 8 4s8-1.79 8-4v-3c0 2.21-3.58 4-8 4s-8-1.79-8-4z',
    }),
  ]);

const TableIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', {
      d: 'M10 10.02h5V21h-5zM17 21h3c1.1 0 2-.9 2-2v-9h-5v11zm3-18H5c-1.1 0-2 .9-2 2v3h19V5c0-1.1-.9-2-2-2zM3 19c0 1.1.9 2 2 2h3V10.02H3V19z',
    }),
  ]);

const FolderIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', {
      d: 'M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z',
    }),
  ]);

const PlayIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', { d: 'M8 5v14l11-7z' }),
  ]);

const ClearIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', {
      d: 'M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z',
    }),
  ]);

const AddIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', { d: 'M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z' }),
  ]);

const EditIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', {
      d: 'M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z',
    }),
  ]);

const DeleteIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', {
      d: 'M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z',
    }),
  ]);

const { showSuccess, showError } = useNotification();

// State
const loadingDatabases = ref(false);
const loadingTables = ref(false);
const databases = ref<string[]>([]);
const databaseTables = ref<Map<string, TableInfo[]>>(new Map());
const expandedKeys = ref<string[]>([]);
const selectedKeys = ref<string[]>([]);

// Tabs
interface Tab {
  key: string;
  label: string;
  type: 'sql' | 'table';
  closable: boolean;
  content?: string;
  result?: any;
  tableName?: string;
  database?: string;
  schema?: string;
  columns?: DataTableColumns;
  data?: any[];
  loading?: boolean;
  pagination?: any;
  currentPage?: number;
  pageSize?: number;
  totalRows?: number;
}

const tabs = ref<Tab[]>([
  {
    key: 'welcome',
    label: '欢迎',
    type: 'sql',
    closable: false,
    content: '-- 欢迎使用 PostgreSQL 数据库工具\n-- 点击左侧数据库树展开查看表\n-- 双击表名打开表数据\n-- 点击 + 按钮创建新的 SQL 查询',
  },
]);

const activeTab = ref('welcome');
let tabCounter = 0;

// Editor
const showEditor = ref(false);
const editingRecord = ref<Record<string, any> | null>(null);
const formData = ref<Record<string, any>>({});
const currentColumns = ref<ColumnInfo[]>([]);
const currentTabKey = ref<string>('');

// Tree Data
const treeData = computed<TreeOption[]>(() => {
  return databases.value.map(db => {
    const tables = databaseTables.value.get(db) || [];
    return {
      key: `db-${db}`,
      label: db,
      prefix: () => h(NIcon, null, { default: () => h(DatabaseIcon) }),
      children: [
        {
          key: `${db}-schemas`,
          label: 'Schemas',
          prefix: () => h(NIcon, null, { default: () => h(FolderIcon) }),
          children: groupTablesBySchema(db, tables),
        },
      ],
    };
  });
});

function groupTablesBySchema(database: string, tables: TableInfo[]): TreeOption[] {
  const schemaMap = new Map<string, TableInfo[]>();
  
  tables.forEach(table => {
    const schema = table.schema || 'public';
    if (!schemaMap.has(schema)) {
      schemaMap.set(schema, []);
    }
    schemaMap.get(schema)!.push(table);
  });

  return Array.from(schemaMap.entries()).map(([schema, schemaTables]) => ({
    key: `${database}-schema-${schema}`,
    label: schema,
    prefix: () => h(NIcon, null, { default: () => h(FolderIcon) }),
    children: [
      {
        key: `${database}-${schema}-tables`,
        label: 'Tables',
        prefix: () => h(NIcon, null, { default: () => h(FolderIcon) }),
        children: schemaTables.map(table => ({
          key: `table-${database}-${schema}-${table.name}`,
          label: `${table.name} ${table.rowCount !== undefined ? `(${table.rowCount})` : ''}`,
          prefix: () => h(NIcon, null, { default: () => h(TableIcon) }),
          database,
          schema,
          tableName: table.name,
        })),
      },
    ],
  }));
}

// Methods
const loadDatabases = async () => {
  loadingDatabases.value = true;
  try {
    const response = await listDatabases('default');
    if (response.success && response.data) {
      databases.value = response.data;
      showSuccess(`已加载 ${response.data.length} 个数据库`);
    } else {
      showError(response.message || '加载数据库列表失败');
    }
  } catch (error) {
    showError('加载数据库列表时发生错误');
    console.error('Load databases error:', error);
  } finally {
    loadingDatabases.value = false;
  }
};

const loadTables = async (database: string) => {
  if (databaseTables.value.has(database)) {
    return; // Already loaded
  }

  loadingTables.value = true;
  try {
    const response = await listTables('default', database);
    if (response.success && response.data) {
      databaseTables.value.set(database, response.data);
    } else {
      showError(response.message || '加载表列表失败');
    }
  } catch (error) {
    showError('加载表列表时发生错误');
    console.error('Load tables error:', error);
  } finally {
    loadingTables.value = false;
  }
};

const handleExpandedKeysChange = async (keys: string[]) => {
  expandedKeys.value = keys;
  
  // Load tables when database is expanded
  for (const key of keys) {
    if (key.startsWith('db-')) {
      const database = key.substring(3);
      await loadTables(database);
    }
  }
};

const handleSelectedKeysChange = (keys: string[]) => {
  selectedKeys.value = keys;
};

const nodeProps = ({ option }: { option: TreeOption }) => {
  return {
    onDblclick: () => {
      if (option.key.toString().startsWith('table-')) {
        openTableTab(option.database as string, option.schema as string, option.tableName as string);
      }
    },
  };
};

const openTableTab = async (database: string, schema: string, tableName: string) => {
  const tabKey = `table-${database}-${schema}-${tableName}`;
  
  // Check if tab already exists
  const existingTab = tabs.value.find(t => t.key === tabKey);
  if (existingTab) {
    activeTab.value = tabKey;
    return;
  }

  // Create new tab
  const newTab: Tab = {
    key: tabKey,
    label: tableName,
    type: 'table',
    closable: true,
    tableName,
    database,
    schema,
    columns: [],
    data: [],
    loading: true,
    currentPage: 1,
    pageSize: 20,
    totalRows: 0,
    pagination: {
      page: 1,
      pageSize: 20,
      showSizePicker: true,
      pageSizes: [10, 20, 50, 100],
    },
  };

  tabs.value.push(newTab);
  activeTab.value = tabKey;

  // Load table data
  await loadTableDataForTab(tabKey);
};

const loadTableDataForTab = async (tabKey: string) => {
  const tab = tabs.value.find(t => t.key === tabKey);
  if (!tab || tab.type !== 'table') return;

  tab.loading = true;
  try {
    const response = await getTableData(
      'default',
      tab.database!,
      tab.tableName!,
      tab.currentPage || 1,
      tab.pageSize || 20
    );

    if (response.success && response.data) {
      const columns: DataTableColumns = response.data.columns.map(col => ({
        title: col.name,
        key: col.name,
        ellipsis: { tooltip: true },
        render: (row: any) => {
          const value = row[col.name];
          return h('span', {}, value !== null && value !== undefined ? String(value) : 'NULL');
        },
      }));

      // Add actions column
      columns.push({
        title: '操作',
        key: 'actions',
        width: 150,
        fixed: 'right',
        render: (row: any) => {
          return h(NSpace, { size: 'small' }, {
            default: () => [
              h(NButton, {
                size: 'small',
                onClick: () => editRecord(tabKey, row, response.data!.columns),
              }, {
                icon: () => h(NIcon, null, { default: () => h(EditIcon) }),
                default: () => '编辑',
              }),
              h(NPopconfirm, {
                onPositiveClick: () => deleteRecordFromTab(tabKey, row, response.data!.columns),
              }, {
                trigger: () => h(NButton, {
                  size: 'small',
                  type: 'error',
                }, {
                  icon: () => h(NIcon, null, { default: () => h(DeleteIcon) }),
                  default: () => '删除',
                }),
                default: () => '确定要删除这条记录吗？',
              }),
            ],
          });
        },
      });

      tab.columns = columns;
      tab.data = response.data.rows;
      tab.totalRows = response.data.totalRows;
      tab.pagination = {
        page: tab.currentPage || 1,
        pageSize: tab.pageSize || 20,
        pageCount: Math.ceil(response.data.totalRows / (tab.pageSize || 20)),
        showSizePicker: true,
        pageSizes: [10, 20, 50, 100],
      };
    } else {
      showError(response.message || '加载表数据失败');
    }
  } catch (error) {
    showError('加载表数据时发生错误');
    console.error('Load table data error:', error);
  } finally {
    tab.loading = false;
  }
};

const handlePageChange = (tabKey: string, page: number) => {
  const tab = tabs.value.find(t => t.key === tabKey);
  if (tab) {
    tab.currentPage = page;
    loadTableDataForTab(tabKey);
  }
};

const handlePageSizeChange = (tabKey: string, pageSize: number) => {
  const tab = tabs.value.find(t => t.key === tabKey);
  if (tab) {
    tab.pageSize = pageSize;
    tab.currentPage = 1;
    loadTableDataForTab(tabKey);
  }
};

const refreshTableData = (tabKey: string) => {
  loadTableDataForTab(tabKey);
};

const addSqlTab = () => {
  tabCounter++;
  const newTab: Tab = {
    key: `sql-${tabCounter}`,
    label: `查询 ${tabCounter}`,
    type: 'sql',
    closable: true,
    content: '',
  };
  tabs.value.push(newTab);
  activeTab.value = newTab.key;
};

const handleTabClose = (name: string) => {
  const index = tabs.value.findIndex(t => t.key === name);
  if (index !== -1) {
    tabs.value.splice(index, 1);
    if (activeTab.value === name && tabs.value.length > 0) {
      activeTab.value = tabs.value[tabs.value.length - 1].key;
    }
  }
};

const executeQuery = async (tabKey: string) => {
  const tab = tabs.value.find(t => t.key === tabKey);
  if (!tab || !tab.content) {
    showError('请输入 SQL 查询');
    return;
  }

  showError('SQL 查询执行功能待实现');
};

const clearEditor = (tabKey: string) => {
  const tab = tabs.value.find(t => t.key === tabKey);
  if (tab) {
    tab.content = '';
    tab.result = undefined;
  }
};

const addRecord = (tabKey: string) => {
  const tab = tabs.value.find(t => t.key === tabKey);
  if (!tab || tab.type !== 'table') return;

  // Get columns from the first data row or from stored columns
  const response = tab.data && tab.data.length > 0 ? Object.keys(tab.data[0]) : [];
  
  // We need to fetch column info
  loadTableDataForTab(tabKey).then(() => {
    const tableTab = tabs.value.find(t => t.key === tabKey);
    if (tableTab && tableTab.columns) {
      // Extract column info from columns
      currentColumns.value = [];
      formData.value = {};
      editingRecord.value = null;
      currentTabKey.value = tabKey;
      showEditor.value = true;
    }
  });
};

const editRecord = (tabKey: string, record: Record<string, any>, columns: ColumnInfo[]) => {
  currentColumns.value = columns;
  editingRecord.value = record;
  formData.value = { ...record };
  currentTabKey.value = tabKey;
  showEditor.value = true;
};

const handleSaveRecord = async () => {
  const tab = tabs.value.find(t => t.key === currentTabKey.value);
  if (!tab || tab.type !== 'table') return;

  try {
    if (editingRecord.value) {
      // Update
      const primaryKeys = currentColumns.value.filter(col => col.isPrimaryKey);
      const pkData: Record<string, any> = {};
      primaryKeys.forEach(pk => {
        pkData[pk.name] = editingRecord.value![pk.name];
      });

      const response = await updateRecord(
        'default',
        tab.database!,
        tab.tableName!,
        pkData,
        formData.value
      );

      if (response.success) {
        showSuccess('记录更新成功');
        showEditor.value = false;
        refreshTableData(currentTabKey.value);
      } else {
        showError(response.message || '更新失败');
      }
    } else {
      // Create
      const response = await createRecord(
        'default',
        tab.database!,
        tab.tableName!,
        formData.value
      );

      if (response.success) {
        showSuccess('记录创建成功');
        showEditor.value = false;
        refreshTableData(currentTabKey.value);
      } else {
        showError(response.message || '创建失败');
      }
    }
  } catch (error) {
    showError('保存记录时发生错误');
    console.error('Save record error:', error);
  }
};

const deleteRecordFromTab = async (tabKey: string, record: Record<string, any>, columns: ColumnInfo[]) => {
  const tab = tabs.value.find(t => t.key === tabKey);
  if (!tab || tab.type !== 'table') return;

  try {
    const primaryKeys = columns.filter(col => col.isPrimaryKey);
    const pkData: Record<string, any> = {};
    primaryKeys.forEach(pk => {
      pkData[pk.name] = record[pk.name];
    });

    const response = await deleteRecord(
      'default',
      tab.database!,
      tab.tableName!,
      pkData
    );

    if (response.success) {
      showSuccess('记录删除成功');
      refreshTableData(tabKey);
    } else {
      showError(response.message || '删除失败');
    }
  } catch (error) {
    showError('删除记录时发生错误');
    console.error('Delete record error:', error);
  }
};

// Load databases on mount
loadDatabases();
</script>

<style scoped>
.explorer-container {
  display: flex;
  height: calc(100vh - 60px);
  overflow: hidden;
}

.explorer-sidebar {
  width: 280px;
  border-right: 1px solid var(--n-border-color);
  display: flex;
  flex-direction: column;
  background: var(--n-color);
}

.sidebar-header {
  padding: 12px 16px;
  border-bottom: 1px solid var(--n-border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.sidebar-content {
  flex: 1;
  overflow: auto;
  padding: 8px;
}

.explorer-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.sql-editor-container,
.table-view-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 16px;
}

.editor-toolbar,
.table-toolbar {
  margin-bottom: 16px;
}

.editor-area {
  flex: 1;
  margin-bottom: 16px;
}

.result-area {
  border-top: 1px solid var(--n-border-color);
  padding-top: 16px;
}

.result-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}
</style>
