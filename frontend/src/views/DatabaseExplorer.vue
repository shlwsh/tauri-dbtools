<template>
  <div class="explorer-view">
    <n-space vertical :size="16">
      <!-- Database Selection -->
      <n-card title="选择数据库">
        <n-space vertical>
          <n-button @click="loadDatabases" :loading="loadingDatabases" type="primary">
            <template #icon>
              <n-icon><RefreshIcon /></n-icon>
            </template>
            {{ databases.length > 0 ? '刷新数据库列表' : '加载数据库列表' }}
          </n-button>

          <n-select
            v-if="databases.length > 0"
            v-model:value="selectedDatabase"
            :options="databaseOptions"
            placeholder="请选择数据库"
            filterable
            @update:value="handleDatabaseChange"
          />
        </n-space>
      </n-card>

      <!-- Tables and Data -->
      <n-grid v-if="selectedDatabase" :cols="4" :x-gap="16">
        <!-- Table List -->
        <n-gi :span="1">
          <n-card title="表列表" :segmented="{ content: true }">
            <n-spin :show="loadingTables">
              <n-empty v-if="tables.length === 0" description="暂无表" />
              <n-list v-else clickable hoverable>
                <n-list-item
                  v-for="table in tables"
                  :key="`${table.schema}.${table.name}`"
                  @click="handleTableSelect(table)"
                  :class="{ 'selected-table': selectedTable?.name === table.name }"
                >
                  <n-thing>
                    <template #header>
                      <n-text>{{ table.name }}</n-text>
                    </template>
                    <template #description>
                      <n-space size="small">
                        <n-text depth="3" style="font-size: 12px">{{ table.schema }}</n-text>
                        <n-text v-if="table.rowCount !== undefined" depth="3" style="font-size: 12px">
                          {{ table.rowCount }} 行
                        </n-text>
                      </n-space>
                    </template>
                  </n-thing>
                </n-list-item>
              </n-list>
            </n-spin>
          </n-card>
        </n-gi>

        <!-- Data Table -->
        <n-gi :span="3">
          <n-card v-if="selectedTable" :title="`表: ${selectedTable.name}`">
            <template #header-extra>
              <n-space>
                <n-button size="small" @click="handleRefreshData">
                  <template #icon>
                    <n-icon><RefreshIcon /></n-icon>
                  </template>
                  刷新
                </n-button>
                <n-button size="small" type="primary" @click="handleAddRecord">
                  <template #icon>
                    <n-icon><AddIcon /></n-icon>
                  </template>
                  新增
                </n-button>
              </n-space>
            </template>

            <n-spin :show="loadingData">
              <n-data-table
                :columns="tableColumns"
                :data="tableData"
                :pagination="pagination"
                :bordered="true"
                :single-line="false"
                size="small"
                @update:page="handlePageChange"
              />
            </n-spin>
          </n-card>

          <n-empty v-else description="请选择一个表" />
        </n-gi>
      </n-grid>
    </n-space>

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
      <n-form ref="formRef" :model="formData" label-placement="left" label-width="120">
        <n-form-item
          v-for="col in columns"
          :key="col.name"
          :label="col.name"
          :path="col.name"
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
  NCard,
  NSpace,
  NButton,
  NIcon,
  NSelect,
  NGrid,
  NGi,
  NList,
  NListItem,
  NThing,
  NText,
  NEmpty,
  NSpin,
  NDataTable,
  NModal,
  NForm,
  NFormItem,
  NInput,
  NTag,
  NPopconfirm,
  type DataTableColumns,
  type SelectOption,
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
const loadingData = ref(false);
const databases = ref<string[]>([]);
const selectedDatabase = ref<string | null>(null);
const tables = ref<TableInfo[]>([]);
const selectedTable = ref<TableInfo | null>(null);
const columns = ref<ColumnInfo[]>([]);
const tableData = ref<Record<string, any>[]>([]);
const totalRows = ref(0);
const currentPage = ref(1);
const pageSize = ref(20);

const showEditor = ref(false);
const editingRecord = ref<Record<string, any> | null>(null);
const formData = ref<Record<string, any>>({});
const formRef = ref();

// Computed
const databaseOptions = computed<SelectOption[]>(() => {
  return databases.value.map(db => ({
    label: db,
    value: db,
  }));
});

const pagination = computed(() => ({
  page: currentPage.value,
  pageSize: pageSize.value,
  pageCount: Math.ceil(totalRows.value / pageSize.value),
  showSizePicker: true,
  pageSizes: [10, 20, 50, 100],
  onChange: (page: number) => {
    currentPage.value = page;
    loadTableData();
  },
  onUpdatePageSize: (size: number) => {
    pageSize.value = size;
    currentPage.value = 1;
    loadTableData();
  },
}));

const tableColumns = computed<DataTableColumns>(() => {
  if (columns.value.length === 0) return [];

  const cols: DataTableColumns = columns.value.map(col => ({
    title: col.name,
    key: col.name,
    ellipsis: {
      tooltip: true,
    },
    render: (row: any) => {
      const value = row[col.name];
      return h('span', {}, value !== null && value !== undefined ? String(value) : 'NULL');
    },
  }));

  // Add actions column
  cols.push({
    title: '操作',
    key: 'actions',
    width: 150,
    render: (row: any) => {
      return h(
        NSpace,
        { size: 'small' },
        {
          default: () => [
            h(
              NButton,
              {
                size: 'small',
                onClick: () => handleEditRecord(row),
              },
              {
                icon: () => h(NIcon, null, { default: () => h(EditIcon) }),
                default: () => '编辑',
              }
            ),
            h(
              NPopconfirm,
              {
                onPositiveClick: () => handleDeleteRecord(row),
              },
              {
                trigger: () =>
                  h(
                    NButton,
                    {
                      size: 'small',
                      type: 'error',
                    },
                    {
                      icon: () => h(NIcon, null, { default: () => h(DeleteIcon) }),
                      default: () => '删除',
                    }
                  ),
                default: () => '确定要删除这条记录吗？',
              }
            ),
          ],
        }
      );
    },
  });

  return cols;
});

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

const handleDatabaseChange = async (database: string) => {
  selectedTable.value = null;
  tables.value = [];
  tableData.value = [];
  await loadTables();
};

const loadTables = async () => {
  if (!selectedDatabase.value) return;

  loadingTables.value = true;
  try {
    const response = await listTables('default', selectedDatabase.value);
    if (response.success && response.data) {
      tables.value = response.data;
      showSuccess(`找到 ${response.data.length} 个表`);
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

const handleTableSelect = (table: TableInfo) => {
  selectedTable.value = table;
  currentPage.value = 1;
  loadTableData();
};

const loadTableData = async () => {
  if (!selectedDatabase.value || !selectedTable.value) return;

  loadingData.value = true;
  try {
    const response = await getTableData(
      'default',
      selectedDatabase.value,
      selectedTable.value.name,
      currentPage.value,
      pageSize.value
    );

    if (response.success && response.data) {
      columns.value = response.data.columns;
      tableData.value = response.data.rows;
      totalRows.value = response.data.totalRows;
    } else {
      showError(response.message || '加载表数据失败');
    }
  } catch (error) {
    showError('加载表数据时发生错误');
    console.error('Load table data error:', error);
  } finally {
    loadingData.value = false;
  }
};

const handleRefreshData = () => {
  loadTableData();
};

const handleAddRecord = () => {
  editingRecord.value = null;
  formData.value = {};
  columns.value.forEach(col => {
    formData.value[col.name] = '';
  });
  showEditor.value = true;
};

const handleEditRecord = (record: Record<string, any>) => {
  editingRecord.value = record;
  formData.value = { ...record };
  showEditor.value = true;
};

const handleSaveRecord = async () => {
  if (!selectedDatabase.value || !selectedTable.value) return;

  try {
    if (editingRecord.value) {
      // Update existing record
      const primaryKeys = columns.value.filter(col => col.isPrimaryKey);
      const pkData: Record<string, any> = {};
      primaryKeys.forEach(pk => {
        pkData[pk.name] = editingRecord.value![pk.name];
      });

      const response = await updateRecord(
        'default',
        selectedDatabase.value,
        selectedTable.value.name,
        pkData,
        formData.value
      );

      if (response.success) {
        showSuccess('记录更新成功');
        showEditor.value = false;
        loadTableData();
      } else {
        showError(response.message || '更新失败');
      }
    } else {
      // Create new record
      const response = await createRecord(
        'default',
        selectedDatabase.value,
        selectedTable.value.name,
        formData.value
      );

      if (response.success) {
        showSuccess('记录创建成功');
        showEditor.value = false;
        loadTableData();
      } else {
        showError(response.message || '创建失败');
      }
    }
  } catch (error) {
    showError('保存记录时发生错误');
    console.error('Save record error:', error);
  }
};

const handleDeleteRecord = async (record: Record<string, any>) => {
  if (!selectedDatabase.value || !selectedTable.value) return;

  try {
    const primaryKeys = columns.value.filter(col => col.isPrimaryKey);
    const pkData: Record<string, any> = {};
    primaryKeys.forEach(pk => {
      pkData[pk.name] = record[pk.name];
    });

    const response = await deleteRecord(
      'default',
      selectedDatabase.value,
      selectedTable.value.name,
      pkData
    );

    if (response.success) {
      showSuccess('记录删除成功');
      loadTableData();
    } else {
      showError(response.message || '删除失败');
    }
  } catch (error) {
    showError('删除记录时发生错误');
    console.error('Delete record error:', error);
  }
};

const handlePageChange = (page: number) => {
  currentPage.value = page;
  loadTableData();
};
</script>

<style scoped>
.explorer-view {
  padding: 24px;
  max-width: 1600px;
  margin: 0 auto;
}

.selected-table {
  background-color: var(--n-item-color-active);
}
</style>
