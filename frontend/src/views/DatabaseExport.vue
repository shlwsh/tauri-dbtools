<template>
  <div class="export-view">
    <n-card title="数据库导出">
      <n-space vertical size="large">
        <!-- Step 1: Select Database -->
        <n-form-item label="选择数据库">
          <n-space vertical style="width: 100%">
            <n-button @click="loadDatabases" :loading="loading" type="primary">
              <template #icon>
                <n-icon><RefreshIcon /></n-icon>
              </template>
              {{ databases.length > 0 ? '刷新数据库列表' : '加载数据库列表' }}
            </n-button>

            <n-select
              v-if="databases.length > 0"
              v-model:value="selectedDatabase"
              :options="databaseOptions"
              placeholder="请选择要导出的数据库"
              filterable
            />

            <n-alert v-if="databases.length === 0 && !loading" type="info">
              点击上方按钮加载数据库列表
            </n-alert>
          </n-space>
        </n-form-item>

        <!-- Step 2: Export Button -->
        <n-space>
          <n-button
            type="primary"
            size="large"
            :disabled="!selectedDatabase"
            :loading="exporting"
            @click="handleExport"
          >
            <template #icon>
              <n-icon><ExportIcon /></n-icon>
            </template>
            导出数据库
          </n-button>
        </n-space>

        <!-- Export Result -->
        <n-alert v-if="exportResult" :type="exportResult.success ? 'success' : 'error'">
          <template #header>
            {{ exportResult.success ? '导出成功' : '导出失败' }}
          </template>
          {{ exportResult.message }}
          <div v-if="exportResult.success && exportResult.data" style="margin-top: 8px">
            <n-text depth="3">文件路径：</n-text>
            <n-text code>{{ exportResult.data }}</n-text>
          </div>
        </n-alert>
      </n-space>
    </n-card>

    <!-- Export History (Optional) -->
    <n-card title="最近导出" style="margin-top: 16px" v-if="exportHistory.length > 0">
      <n-list bordered>
        <n-list-item v-for="(item, index) in exportHistory" :key="index">
          <n-thing>
            <template #header>
              <n-text>{{ item.database }}</n-text>
            </template>
            <template #description>
              <n-space vertical size="small">
                <n-text depth="3">{{ item.timestamp }}</n-text>
                <n-text depth="3" v-if="item.filePath">{{ item.filePath }}</n-text>
              </n-space>
            </template>
          </n-thing>
        </n-list-item>
      </n-list>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, h } from 'vue';
import {
  NCard,
  NSpace,
  NFormItem,
  NButton,
  NIcon,
  NSelect,
  NAlert,
  NText,
  NList,
  NListItem,
  NThing,
  type SelectOption,
} from 'naive-ui';
import { listDatabases, exportDatabase } from '@/api/database';
import { useNotification } from '@/composables/useNotification';
import type { ApiResponse } from '@/types/common';

// Icons
const RefreshIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', {
      d: 'M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z',
    }),
  ]);

const ExportIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', { d: 'M19 12v7H5v-7H3v7c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2v-7h-2zm-6 .67l2.59-2.58L17 11.5l-5 5-5-5 1.41-1.41L11 12.67V3h2z' }),
  ]);

const { showSuccess, showError } = useNotification();

const loading = ref(false);
const exporting = ref(false);
const databases = ref<string[]>([]);
const selectedDatabase = ref<string | null>(null);
const exportResult = ref<ApiResponse<string> | null>(null);

interface ExportHistoryItem {
  database: string;
  timestamp: string;
  filePath?: string;
}

const exportHistory = ref<ExportHistoryItem[]>([]);

const databaseOptions = computed<SelectOption[]>(() => {
  return databases.value.map(db => ({
    label: db,
    value: db,
  }));
});

const loadDatabases = async () => {
  loading.value = true;
  exportResult.value = null;

  try {
    const response = await listDatabases('default'); // Using default connection for now
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
    loading.value = false;
  }
};

const handleExport = async () => {
  if (!selectedDatabase.value) {
    showError('请选择要导出的数据库');
    return;
  }

  exporting.value = true;
  exportResult.value = null;

  try {
    const response = await exportDatabase('default', selectedDatabase.value);
    exportResult.value = response;

    if (response.success) {
      showSuccess('数据库导出成功');

      // Add to history
      exportHistory.value.unshift({
        database: selectedDatabase.value,
        timestamp: new Date().toLocaleString('zh-CN'),
        filePath: response.data,
      });

      // Keep only last 5 items
      if (exportHistory.value.length > 5) {
        exportHistory.value = exportHistory.value.slice(0, 5);
      }
    } else {
      showError(response.message || '导出失败');
    }
  } catch (error) {
    showError('导出数据库时发生错误');
    console.error('Export error:', error);
    exportResult.value = {
      success: false,
      message: '导出数据库时发生错误',
    };
  } finally {
    exporting.value = false;
  }
};
</script>

<style scoped>
.export-view {
  padding: 24px;
  max-width: 1200px;
  margin: 0 auto;
}
</style>
