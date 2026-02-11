<template>
  <div class="test-container">
    <n-card title="SQL 执行测试工具">
      <n-space vertical :size="20">
        <n-alert type="info" title="测试说明">
          此页面用于测试前端 API 调用后端 execute_sql 命令是否正常工作
        </n-alert>

        <n-form-item label="数据库名称">
          <n-space vertical style="width: 100%">
            <n-select
              v-model:value="database"
              :options="availableDatabases.map(db => ({ label: db, value: db }))"
              placeholder="选择数据库"
              filterable
            />
            <n-input
              v-model:value="database"
              placeholder="或手动输入数据库名称"
            />
          </n-space>
        </n-form-item>

        <n-form-item label="SQL 语句">
          <n-input
            v-model:value="sql"
            type="textarea"
            placeholder="输入 SQL 语句，例如：SELECT * FROM employees LIMIT 5;"
            :rows="5"
          />
        </n-form-item>

        <n-space>
          <n-button
            type="primary"
            :loading="loading"
            @click="handleExecuteSql"
          >
            执行 SQL
          </n-button>
          <n-button
            :loading="loadingDatabases"
            @click="handleListDatabases"
          >
            列出数据库
          </n-button>
          <n-button
            @click="handleTestTauriInvoke"
          >
            测试 Tauri Invoke
          </n-button>
        </n-space>

        <n-divider />

        <n-card title="执行结果" size="small">
          <n-code
            v-if="result"
            :code="result"
            language="json"
            :word-wrap="true"
          />
          <n-empty v-else description="暂无结果" />
        </n-card>

        <n-card v-if="error" title="错误信息" size="small">
          <n-alert type="error" :title="error" />
        </n-card>
      </n-space>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import {
  NCard,
  NSpace,
  NAlert,
  NFormItem,
  NInput,
  NButton,
  NDivider,
  NCode,
  NEmpty,
  NSelect,
} from 'naive-ui';
import { executeSql, listDatabases } from '@/api/database';
import { useNotification } from '@/composables/useNotification';

const notification = useNotification();

// 表单数据
const database = ref('personnel_db');
const sql = ref('SELECT * FROM employees LIMIT 5;');

// 可用数据库列表
const availableDatabases = ref<string[]>([]);

// 状态
const loading = ref(false);
const loadingDatabases = ref(false);
const result = ref('');
const error = ref('');

// 加载数据库列表
const loadDatabaseList = async () => {
  try {
    const response = await listDatabases('default');
    if (response.success && response.data) {
      availableDatabases.value = response.data;
      console.log('可用数据库:', availableDatabases.value);
    }
  } catch (err) {
    console.error('加载数据库列表失败:', err);
  }
};

// 组件挂载时加载数据库列表
loadDatabaseList();

// 执行 SQL（通过前端 API）
const handleExecuteSql = async () => {
  if (!database.value || !sql.value) {
    notification.showWarning('请输入数据库名称和 SQL 语句');
    return;
  }

  loading.value = true;
  error.value = '';
  result.value = '';

  try {
    console.log('=== 测试 executeSql API ===');
    console.log('数据库:', database.value);
    console.log('SQL:', sql.value);

    const response = await executeSql(database.value, sql.value);

    console.log('API 响应:', response);

    result.value = JSON.stringify(response, null, 2);

    if (response.success) {
      notification.showSuccess('SQL 执行成功');
    } else {
      notification.showError(`SQL 执行失败: ${response.message}`);
      error.value = response.message;
    }
  } catch (err: any) {
    console.error('执行失败:', err);
    error.value = err.message || String(err);
    result.value = JSON.stringify({ error: err.message || String(err) }, null, 2);
    notification.showError(`执行失败: ${err.message || String(err)}`);
  } finally {
    loading.value = false;
  }
};

// 列出数据库
const handleListDatabases = async () => {
  loadingDatabases.value = true;
  error.value = '';
  result.value = '';

  try {
    console.log('=== 测试 listDatabases API ===');

    const response = await listDatabases('default');

    console.log('数据库列表:', response);

    result.value = JSON.stringify(response, null, 2);

    if (response.success) {
      notification.showSuccess('获取数据库列表成功');
    } else {
      notification.showError(`获取失败: ${response.message}`);
      error.value = response.message;
    }
  } catch (err: any) {
    console.error('获取失败:', err);
    error.value = err.message || String(err);
    result.value = JSON.stringify({ error: err.message || String(err) }, null, 2);
    notification.showError(`获取失败: ${err.message || String(err)}`);
  } finally {
    loadingDatabases.value = false;
  }
};

// 直接测试 Tauri invoke
const handleTestTauriInvoke = async () => {
  error.value = '';
  result.value = '';

  try {
    console.log('=== 直接测试 Tauri invoke ===');
    console.log('调用参数:', { database: database.value, sql: sql.value });

    const response = await invoke('execute_sql', {
      database: database.value,
      sql: sql.value,
    });

    console.log('Tauri invoke 响应:', response);

    result.value = JSON.stringify(response, null, 2);
    notification.showSuccess('Tauri invoke 调用成功');
  } catch (err: any) {
    console.error('Tauri invoke 失败:', err);
    error.value = err.message || String(err);
    result.value = JSON.stringify({ error: err.message || String(err) }, null, 2);
    notification.showError(`Tauri invoke 失败: ${err.message || String(err)}`);
  }
};
</script>

<style scoped>
.test-container {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}
</style>
