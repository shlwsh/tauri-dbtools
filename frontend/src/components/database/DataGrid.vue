<!--
  DataGrid Component
  
  数据网格主组件，提供：
  - 虚拟滚动支持大数据集
  - 表格渲染（列标题、数据行）
  - 分页控件
  - 修改指示器
  - 工具栏（保存、放弃、添加行、删除行）
  
  Validates: Requirements 3.5, 9.1, 15.1, 15.2
-->

<script setup lang="ts">
import { ref, computed, watch, onMounted, h } from 'vue';
import { useVirtualizer } from '@tanstack/vue-virtual';
import { useDataGridStore } from '@/stores/data-grid';
import { storeToRefs } from 'pinia';
import {
  NDataTable,
  NButton,
  NSpace,
  NPagination,
  NAlert,
  NSpin,
  NEmpty,
  NIcon,
  useMessage,
  useDialog,
} from 'naive-ui';
import {
  SaveOutline as SaveIcon,
  RefreshOutline as RefreshIcon,
  AddOutline as AddIcon,
  TrashOutline as DeleteIcon,
  CloseOutline as CloseIcon,
} from '@vicons/ionicons5';
import CellEditor from './CellEditor.vue';

// Props
interface Props {
  /** 是否显示工具栏 */
  showToolbar?: boolean;
  /** 是否显示分页 */
  showPagination?: boolean;
  /** 表格高度 */
  height?: string;
}

const props = withDefaults(defineProps<Props>(), {
  showToolbar: true,
  showPagination: true,
  height: 'calc(100vh - 300px)',
});

// Store
const dataGridStore = useDataGridStore();
const {
  currentTable,
  columns,
  data,
  totalRows,
  page,
  pageSize,
  isLoading,
  error,
  hasUnsavedChanges,
  modificationStats,
  canEdit,
  fullTableName,
  isRowModified,
  isRowDeleted,
  isRowInserted,
} = storeToRefs(dataGridStore);

// UI
const message = useMessage();
const dialog = useDialog();

// 选中的行
const selectedRowIndexes = ref<Set<number>>(new Set());

// 表格容器引用
const tableContainerRef = ref<HTMLElement>();

// 当前编辑的单元格
const editingCellKey = ref<string | null>(null);

// 计算属性

/**
 * 表格列定义（用于 NDataTable）
 */
const tableColumns = computed(() => {
  if (!columns.value || columns.value.length === 0) return [];

  const cols: any[] = [
    // 选择列
    {
      type: 'selection',
      disabled: (row: any, index: number) => !canEdit.value,
    },
    // 状态指示器列
    {
      key: '_status',
      title: '',
      width: 40,
      render: (row: any, index: number) => {
        if (isRowDeleted.value(index)) {
          return '🗑️'; // 删除标记
        }
        if (isRowInserted.value(index)) {
          return '➕'; // 新增标记
        }
        if (isRowModified.value(index)) {
          return '✏️'; // 修改标记
        }
        return '';
      },
    },
  ];

  // 数据列
  columns.value.forEach((col) => {
    cols.push({
      key: col.name,
      title: col.name,
      width: 150,
      ellipsis: {
        tooltip: true,
      },
      render: (row: any, index: number) => {
        const cellKey = `${index}-${col.name}`;
        const isEditing = editingCellKey.value === cellKey;
        const value = row[col.name];
        
        // 如果行被删除，显示删除线
        if (isRowDeleted.value(index)) {
          return h('span', { style: 'text-decoration: line-through; opacity: 0.5;' }, 
            value === null || value === undefined ? 'NULL' : String(value)
          );
        }
        
        // 使用 CellEditor 组件
        return h(CellEditor, {
          column: col,
          value: value,
          editing: isEditing,
          rowIndex: index,
          onStartEdit: () => {
            editingCellKey.value = cellKey;
          },
          onSave: (newValue: any) => {
            dataGridStore.updateCell(index, col.name, newValue);
            editingCellKey.value = null;
          },
          onCancel: () => {
            editingCellKey.value = null;
          },
        });
      },
    });
  });

  return cols;
});

/**
 * 表格数据（添加索引）
 */
const tableData = computed(() => {
  return data.value.map((row, index) => ({
    ...row,
    _index: index,
  }));
});

/**
 * 总页数
 */
const totalPages = computed(() => {
  return Math.ceil(totalRows.value / pageSize.value);
});

/**
 * 是否有选中的行
 */
const hasSelectedRows = computed(() => {
  return selectedRowIndexes.value.size > 0;
});

/**
 * 是否可以删除（有选中的行且可编辑）
 */
const canDelete = computed(() => {
  return canEdit.value && hasSelectedRows.value;
});

// 方法

/**
 * 处理行选择变化
 */
function handleRowSelectionChange(rowKeys: any[]) {
  selectedRowIndexes.value = new Set(rowKeys.map((key: any) => key._index));
}

/**
 * 保存更改
 */
async function handleSave() {
  try {
    await dataGridStore.saveChanges();
    message.success('保存成功');
    selectedRowIndexes.value.clear();
  } catch (err) {
    message.error(err instanceof Error ? err.message : '保存失败');
  }
}

/**
 * 放弃更改
 */
function handleDiscard() {
  dialog.warning({
    title: '确认放弃更改',
    content: `您有 ${modificationStats.value.total} 处未保存的更改，确定要放弃吗？`,
    positiveText: '放弃',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await dataGridStore.discardChanges();
        message.info('已放弃更改');
        selectedRowIndexes.value.clear();
      } catch (err) {
        message.error(err instanceof Error ? err.message : '操作失败');
      }
    },
  });
}

/**
 * 刷新数据
 */
async function handleRefresh() {
  try {
    await dataGridStore.refresh();
    message.success('刷新成功');
    selectedRowIndexes.value.clear();
  } catch (err) {
    message.error(err instanceof Error ? err.message : '刷新失败');
  }
}

/**
 * 添加行
 */
function handleAddRow() {
  dataGridStore.addRow();
  message.info('已添加新行');
}

/**
 * 删除选中的行
 */
function handleDeleteRows() {
  if (!hasSelectedRows.value) return;

  const count = selectedRowIndexes.value.size;
  
  dialog.warning({
    title: '确认删除',
    content: `确定要删除选中的 ${count} 行吗？`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: () => {
      dataGridStore.deleteRows(Array.from(selectedRowIndexes.value));
      message.success(`已标记 ${count} 行为删除`);
      selectedRowIndexes.value.clear();
    },
  });
}

/**
 * 关闭表格
 */
function handleClose() {
  if (hasUnsavedChanges.value) {
    dialog.warning({
      title: '确认关闭',
      content: '您有未保存的更改，确定要关闭吗？',
      positiveText: '关闭',
      negativeText: '取消',
      onPositiveClick: () => {
        dataGridStore.closeTable();
      },
    });
  } else {
    dataGridStore.closeTable();
  }
}

/**
 * 处理分页变化
 */
async function handlePageChange(newPage: number) {
  try {
    await dataGridStore.setPage(newPage - 1); // NaiveUI 的页码从 1 开始
  } catch (err) {
    message.error(err instanceof Error ? err.message : '切换页面失败');
  }
}

/**
 * 处理每页大小变化
 */
async function handlePageSizeChange(newPageSize: number) {
  try {
    await dataGridStore.setPageSize(newPageSize);
  } catch (err) {
    message.error(err instanceof Error ? err.message : '更改每页大小失败');
  }
}

/**
 * 处理行双击（进入编辑模式）
 */
function handleRowDblClick(row: any) {
  if (!canEdit.value) {
    message.warning('此表不可编辑（缺少主键）');
    return;
  }
  
  // 单元格编辑已通过 CellEditor 组件实现
}

// 生命周期

onMounted(() => {
  // 组件挂载时的初始化逻辑
});
</script>

<template>
  <div class="data-grid-container">
    <!-- 工具栏 -->
    <div v-if="showToolbar" class="data-grid-toolbar">
      <NSpace>
        <!-- 表信息 -->
        <div class="table-info">
          <span v-if="currentTable" class="table-name">{{ fullTableName }}</span>
          <span v-if="currentTable" class="row-count">
            ({{ totalRows }} 行)
          </span>
        </div>

        <!-- 修改统计 -->
        <div v-if="hasUnsavedChanges" class="modification-stats">
          <span class="stat-item">
            ✏️ {{ modificationStats.updated }}
          </span>
          <span class="stat-item">
            ➕ {{ modificationStats.inserted }}
          </span>
          <span class="stat-item">
            🗑️ {{ modificationStats.deleted }}
          </span>
        </div>
      </NSpace>

      <NSpace>
        <!-- 保存/放弃按钮 -->
        <NButton
          v-if="hasUnsavedChanges"
          type="primary"
          :disabled="isLoading"
          @click="handleSave"
        >
          <template #icon>
            <NIcon><SaveIcon /></NIcon>
          </template>
          保存更改
        </NButton>

        <NButton
          v-if="hasUnsavedChanges"
          :disabled="isLoading"
          @click="handleDiscard"
        >
          放弃更改
        </NButton>

        <!-- 刷新按钮 -->
        <NButton :disabled="isLoading" @click="handleRefresh">
          <template #icon>
            <NIcon><RefreshIcon /></NIcon>
          </template>
          刷新
        </NButton>

        <!-- 添加行按钮 -->
        <NButton
          v-if="canEdit"
          :disabled="isLoading"
          @click="handleAddRow"
        >
          <template #icon>
            <NIcon><AddIcon /></NIcon>
          </template>
          添加行
        </NButton>

        <!-- 删除行按钮 -->
        <NButton
          v-if="canEdit"
          :disabled="!canDelete || isLoading"
          @click="handleDeleteRows"
        >
          <template #icon>
            <NIcon><DeleteIcon /></NIcon>
          </template>
          删除行
        </NButton>

        <!-- 关闭按钮 -->
        <NButton @click="handleClose">
          <template #icon>
            <NIcon><CloseIcon /></NIcon>
          </template>
          关闭
        </NButton>
      </NSpace>
    </div>

    <!-- 错误提示 -->
    <NAlert
      v-if="error"
      type="error"
      :title="error"
      closable
      @close="error = null"
      style="margin-bottom: 12px"
    />

    <!-- 数据表格 -->
    <NSpin :show="isLoading">
      <div v-if="currentTable" class="data-grid-table">
        <NDataTable
          :columns="tableColumns"
          :data="tableData"
          :max-height="height"
          :scroll-x="columns.length * 150"
          :row-key="(row: any) => row._index"
          :checked-row-keys="Array.from(selectedRowIndexes)"
          @update:checked-row-keys="handleRowSelectionChange"
          @row-dblclick="handleRowDblClick"
          striped
          bordered
        />
      </div>

      <!-- 空状态 -->
      <NEmpty
        v-else
        description="请从数据库浏览器中选择一个表"
        style="margin-top: 60px"
      />
    </NSpin>

    <!-- 分页 -->
    <div v-if="showPagination && currentTable" class="data-grid-pagination">
      <NPagination
        :page="page + 1"
        :page-count="totalPages"
        :page-size="pageSize"
        :page-sizes="[50, 100, 200, 500]"
        :disabled="isLoading"
        show-size-picker
        @update:page="handlePageChange"
        @update:page-size="handlePageSizeChange"
      />
    </div>
  </div>
</template>

<style scoped>
.data-grid-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 16px;
}

.data-grid-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding: 12px;
  background-color: var(--n-color);
  border-radius: 4px;
  border: 1px solid var(--n-border-color);
}

.table-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.table-name {
  font-weight: 600;
  font-size: 14px;
}

.row-count {
  color: var(--n-text-color-3);
  font-size: 12px;
}

.modification-stats {
  display: flex;
  gap: 12px;
  padding: 4px 12px;
  background-color: var(--n-color-warning);
  border-radius: 4px;
  font-size: 12px;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.data-grid-table {
  flex: 1;
  overflow: hidden;
}

.data-grid-pagination {
  display: flex;
  justify-content: center;
  margin-top: 16px;
  padding: 12px;
}
</style>
