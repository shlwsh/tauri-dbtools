<template>
  <div class="result-panel">
    <div v-if="!result" class="empty-state">
      <n-empty description="No query results yet. Execute a query to see results here." />
    </div>

    <div v-else-if="result.type === 'error'" class="error-result">
      <n-alert
        type="error"
        title="Query Error"
        class="error-alert"
      >
        <div class="error-content">
          <p>{{ result.error }}</p>
          <p v-if="result.errorPosition" class="error-position">
            Error at line {{ result.errorPosition.line }}, column {{ result.errorPosition.column }}
          </p>
        </div>
      </n-alert>
    </div>

    <div v-else-if="result.type === 'select'" class="select-result">
      <!-- Stats and Export -->
      <div class="result-stats">
        <n-space justify="space-between">
          <n-space>
            <n-tag type="success">
              {{ result.rowCount || 0 }} rows returned
            </n-tag>
            <n-tag type="info">
              Execution time: {{ result.duration }}ms
            </n-tag>
          </n-space>

          <!-- Export Button -->
          <n-dropdown
            v-if="result.rows && result.rows.length > 0"
            trigger="click"
            :options="exportOptions"
            @select="handleExportSelect"
          >
            <n-button size="small">
              <template #icon>
                <n-icon><DownloadOutline /></n-icon>
              </template>
              导出
            </n-button>
          </n-dropdown>
        </n-space>
      </div>

      <!-- Data Table -->
      <div v-if="result.rows && result.rows.length > 0" class="result-table-container">
        <n-data-table
          :columns="tableColumns"
          :data="result.rows"
          :pagination="paginationProps"
          :max-height="500"
          :scroll-x="scrollX"
          striped
          size="small"
        />
      </div>

      <div v-else class="empty-result">
        <n-empty description="Query returned 0 rows" />
      </div>
    </div>

    <div v-else-if="result.type === 'dml'" class="dml-result">
      <n-alert
        type="success"
        title="Query Executed Successfully"
      >
        <div class="dml-content">
          <p>{{ result.affectedRows || 0 }} rows affected</p>
          <p class="execution-time">Execution time: {{ result.duration }}ms</p>
        </div>
      </n-alert>
    </div>

    <div v-else-if="result.type === 'ddl'" class="ddl-result">
      <n-alert
        type="success"
        title="DDL Executed Successfully"
      >
        <div class="ddl-content">
          <p>Command completed successfully</p>
          <p class="execution-time">Execution time: {{ result.duration }}ms</p>
        </div>
      </n-alert>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import {
  NEmpty,
  NAlert,
  NSpace,
  NTag,
  NDataTable,
  NButton,
  NDropdown,
  NIcon,
  useMessage,
  type DataTableColumns,
  type PaginationProps,
  type DropdownOption,
} from 'naive-ui';
import { DownloadOutline } from '@vicons/ionicons5';
import type { QueryResult } from '@/types/sql-editor';
import {
  exportQueryResult,
  downloadExportedFile,
  getFormatDisplayName,
  type ExportFormat,
} from '@/services/export-service';

// Props
const props = defineProps<{
  result?: QueryResult;
}>();

// UI
const message = useMessage();

// State
const isExporting = ref(false);

// Computed

/**
 * 导出选项
 */
const exportOptions = computed<DropdownOption[]>(() => [
  {
    label: getFormatDisplayName('csv'),
    key: 'csv',
  },
  {
    label: getFormatDisplayName('json'),
    key: 'json',
  },
  {
    label: getFormatDisplayName('excel'),
    key: 'excel',
  },
]);

// Methods

/**
 * 处理导出格式选择
 */
async function handleExportSelect(key: string) {
  if (!props.result || props.result.type !== 'select' || !props.result.rows || !props.result.columns) {
    message.error('没有可导出的数据');
    return;
  }

  const format = key as ExportFormat;

  try {
    isExporting.value = true;
    message.loading(`正在导出为 ${getFormatDisplayName(format)}...`);

    // 导出数据
    const result = await exportQueryResult(
      props.result.columns,
      props.result.rows,
      {
        format,
        includeHeaders: true,
      }
    );

    // 下载文件
    downloadExportedFile(result);

    message.success(`成功导出 ${props.result.rows.length} 行数据`);
  } catch (err) {
    message.error(err instanceof Error ? err.message : '导出失败');
  } finally {
    isExporting.value = false;
  }
}
const tableColumns = computed<DataTableColumns<Record<string, any>>>(() => {
  if (!props.result || props.result.type !== 'select' || !props.result.columns) {
    return [];
  }

  return props.result.columns.map(col => ({
    key: col.name,
    title: `${col.name} (${col.typeName})`,
    render: (row: Record<string, any>) => {
      const value = row[col.name];
      
      if (value === null || value === undefined) {
        return 'NULL';
      }
      
      if (typeof value === 'string' && value.length > 100) {
        return value.substring(0, 100) + '...';
      }
      
      if (typeof value === 'object') {
        return JSON.stringify(value);
      }
      
      return String(value);
    },
    ellipsis: {
      tooltip: true,
    },
    minWidth: 150,
  }));
});

const scrollX = computed(() => {
  if (!props.result || props.result.type !== 'select' || !props.result.columns) {
    return undefined;
  }
  
  return props.result.columns.length * 150;
});

const paginationProps = computed<PaginationProps>(() => ({
  pageSize: 50,
  showSizePicker: true,
  pageSizes: [10, 20, 50, 100],
  showQuickJumper: true,
}));
</script>

<style scoped>
.result-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 40px;
}

.error-result,
.dml-result,
.ddl-result {
  padding: 16px;
}

.error-alert {
  margin-bottom: 0;
}

.error-content p {
  margin: 4px 0;
}

.error-position {
  font-size: 12px;
  color: var(--n-text-color-3);
  margin-top: 8px;
}

.select-result {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.result-stats {
  padding: 12px 16px;
  border-bottom: 1px solid var(--n-border-color);
  background-color: var(--n-color);
}

.result-table-container {
  flex: 1;
  overflow: auto;
  padding: 16px;
}

.empty-result {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px;
}

.dml-content,
.ddl-content {
  padding: 8px 0;
}

.dml-content p,
.ddl-content p {
  margin: 4px 0;
}

.execution-time {
  font-size: 12px;
  color: var(--n-text-color-3);
  margin-top: 8px;
}

:deep(.null-value) {
  color: var(--n-text-color-3);
  font-style: italic;
}
</style>
