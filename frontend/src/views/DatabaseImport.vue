<template>
  <div class="import-view">
    <n-card title="数据库导入">
      <n-space vertical size="large">
        <!-- Step 1: Select Backup File -->
        <n-form-item label="选择备份文件">
          <n-space vertical style="width: 100%">
            <n-button @click="selectFile" type="primary">
              <template #icon>
                <n-icon><FolderIcon /></n-icon>
              </template>
              选择备份文件
            </n-button>

            <n-alert v-if="selectedFile" type="info">
              <template #header>已选择文件</template>
              <n-text code>{{ selectedFile }}</n-text>
            </n-alert>

            <n-text depth="3">支持的文件格式：.backup, .sql, .gz</n-text>
          </n-space>
        </n-form-item>

        <!-- Step 2: Enter Database Name -->
        <n-form-item label="目标数据库名称" required>
          <n-input
            v-model:value="databaseName"
            placeholder="请输入目标数据库名称"
            :disabled="!selectedFile"
          />
        </n-form-item>

        <!-- Step 3: Import Button -->
        <n-space>
          <n-button
            type="primary"
            size="large"
            :disabled="!selectedFile || !databaseName"
            :loading="importing"
            @click="handleImport"
          >
            <template #icon>
              <n-icon><ImportIcon /></n-icon>
            </template>
            导入数据库
          </n-button>
        </n-space>

        <!-- Import Result -->
        <n-alert v-if="importResult" :type="importResult.success ? 'success' : 'error'">
          <template #header>
            {{ importResult.success ? '导入成功' : '导入失败' }}
          </template>
          {{ importResult.message }}
        </n-alert>

        <!-- Warning -->
        <n-alert type="warning" title="注意">
          <ul style="margin: 0; padding-left: 20px">
            <li>如果目标数据库已存在，将会被删除并重新创建</li>
            <li>导入过程可能需要几分钟，请耐心等待</li>
            <li>请确保备份文件完整且未损坏</li>
          </ul>
        </n-alert>
      </n-space>
    </n-card>

    <!-- Import History (Optional) -->
    <n-card title="最近导入" style="margin-top: 16px" v-if="importHistory.length > 0">
      <n-list bordered>
        <n-list-item v-for="(item, index) in importHistory" :key="index">
          <n-thing>
            <template #header>
              <n-text>{{ item.database }}</n-text>
            </template>
            <template #description>
              <n-space vertical size="small">
                <n-text depth="3">{{ item.timestamp }}</n-text>
                <n-text depth="3">{{ item.fileName }}</n-text>
              </n-space>
            </template>
            <template #header-extra>
              <n-tag :type="item.success ? 'success' : 'error'" size="small">
                {{ item.success ? '成功' : '失败' }}
              </n-tag>
            </template>
          </n-thing>
        </n-list-item>
      </n-list>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, h } from 'vue';
import {
  NCard,
  NSpace,
  NFormItem,
  NButton,
  NIcon,
  NInput,
  NAlert,
  NText,
  NList,
  NListItem,
  NThing,
  NTag,
} from 'naive-ui';
import { open } from '@tauri-apps/plugin-dialog';
import { importDatabase } from '@/api/database';
import { useNotification } from '@/composables/useNotification';
import type { ApiResponse } from '@/types/common';

// Icons
const FolderIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', {
      d: 'M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z',
    }),
  ]);

const ImportIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', { d: 'M19 12v7H5v-7H3v7c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2v-7h-2zm-6 .67l2.59-2.58L17 11.5l-5 5-5-5 1.41-1.41L11 12.67V3h2z' }),
  ]);

const { showSuccess, showError, showWarning } = useNotification();

const selectedFile = ref<string | null>(null);
const databaseName = ref('');
const importing = ref(false);
const importResult = ref<ApiResponse<void> | null>(null);

interface ImportHistoryItem {
  database: string;
  fileName: string;
  timestamp: string;
  success: boolean;
}

const importHistory = ref<ImportHistoryItem[]>([]);

const selectFile = async () => {
  try {
    const file = await open({
      multiple: false,
      filters: [
        {
          name: 'Database Backup',
          extensions: ['backup', 'sql', 'gz'],
        },
      ],
    });

    if (file) {
      selectedFile.value = file as string;
      
      // Auto-fill database name from filename
      const fileName = (file as string).split(/[/\\]/).pop() || '';
      const nameWithoutExt = fileName.replace(/\.(backup|sql|gz)$/, '');
      const dbName = nameWithoutExt.split('_')[0]; // Extract database name before timestamp
      if (dbName && !databaseName.value) {
        databaseName.value = dbName;
      }
    }
  } catch (error) {
    showError('选择文件时发生错误');
    console.error('File selection error:', error);
  }
};

const handleImport = async () => {
  if (!selectedFile.value || !databaseName.value) {
    showError('请选择备份文件并输入数据库名称');
    return;
  }

  // Confirm before import
  if (!confirm(`确定要导入到数据库 "${databaseName.value}" 吗？\n\n如果该数据库已存在，将会被删除并重新创建。`)) {
    return;
  }

  importing.value = true;
  importResult.value = null;

  try {
    const response = await importDatabase('default', selectedFile.value, databaseName.value);
    importResult.value = response;

    if (response.success) {
      showSuccess('数据库导入成功');

      // Add to history
      const fileName = selectedFile.value.split(/[/\\]/).pop() || selectedFile.value;
      importHistory.value.unshift({
        database: databaseName.value,
        fileName,
        timestamp: new Date().toLocaleString('zh-CN'),
        success: true,
      });

      // Keep only last 5 items
      if (importHistory.value.length > 5) {
        importHistory.value = importHistory.value.slice(0, 5);
      }

      // Reset form
      selectedFile.value = null;
      databaseName.value = '';
    } else {
      showError(response.message || '导入失败');

      // Add failed import to history
      const fileName = selectedFile.value.split(/[/\\]/).pop() || selectedFile.value;
      importHistory.value.unshift({
        database: databaseName.value,
        fileName,
        timestamp: new Date().toLocaleString('zh-CN'),
        success: false,
      });
    }
  } catch (error) {
    showError('导入数据库时发生错误');
    console.error('Import error:', error);
    importResult.value = {
      success: false,
      message: '导入数据库时发生错误',
    };
  } finally {
    importing.value = false;
  }
};
</script>

<style scoped>
.import-view {
  padding: 24px;
  max-width: 1200px;
  margin: 0 auto;
}
</style>
