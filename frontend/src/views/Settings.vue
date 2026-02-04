<template>
  <div class="settings-view">
    <n-card title="数据库连接配置">
      <template #header-extra>
        <n-button type="primary" @click="showAddDialog">
          <template #icon>
            <n-icon><AddIcon /></n-icon>
          </template>
          添加连接
        </n-button>
      </template>

      <n-empty v-if="connections.length === 0" description="暂无连接配置">
        <template #extra>
          <n-button size="small" @click="showAddDialog">创建第一个连接</n-button>
        </template>
      </n-empty>

      <n-list v-else bordered>
        <n-list-item v-for="conn in connections" :key="conn.id">
          <template #prefix>
            <n-icon size="24"><DatabaseIcon /></n-icon>
          </template>

          <n-thing :title="conn.name">
            <template #description>
              <n-space vertical size="small">
                <n-text depth="3">{{ conn.host }}:{{ conn.port }}</n-text>
                <n-text depth="3">用户名: {{ conn.username }}</n-text>
                <n-tag v-if="conn.isDefault" type="success" size="small">默认连接</n-tag>
              </n-space>
            </template>
          </n-thing>

          <template #suffix>
            <n-space>
              <n-button
                v-if="!conn.isDefault"
                size="small"
                @click="handleSetDefault(conn.id)"
              >
                设为默认
              </n-button>
              <n-button size="small" @click="handleEdit(conn)">编辑</n-button>
              <n-popconfirm @positive-click="handleDelete(conn.id)">
                <template #trigger>
                  <n-button size="small" type="error">删除</n-button>
                </template>
                确定要删除连接 "{{ conn.name }}" 吗？
              </n-popconfirm>
            </n-space>
          </template>
        </n-list-item>
      </n-list>
    </n-card>

    <!-- Add/Edit Dialog -->
    <n-modal
      v-model:show="showDialog"
      :title="editingConnection ? '编辑连接' : '添加连接'"
      preset="dialog"
      :positive-text="editingConnection ? '保存' : '添加'"
      negative-text="取消"
      @positive-click="handleSave"
    >
      <n-form
        ref="formRef"
        :model="formData"
        :rules="rules"
        label-placement="left"
        label-width="80"
      >
        <n-form-item label="名称" path="name">
          <n-input v-model:value="formData.name" placeholder="例如：本地开发" />
        </n-form-item>

        <n-form-item label="主机" path="host">
          <n-input v-model:value="formData.host" placeholder="例如：localhost" />
        </n-form-item>

        <n-form-item label="端口" path="port">
          <n-input-number
            v-model:value="formData.port"
            :min="1"
            :max="65535"
            placeholder="5432"
            style="width: 100%"
          />
        </n-form-item>

        <n-form-item label="用户名" path="username">
          <n-input v-model:value="formData.username" placeholder="例如：postgres" />
        </n-form-item>

        <n-form-item label="密码" path="password">
          <n-input
            v-model:value="formData.password"
            type="password"
            show-password-on="click"
            placeholder="数据库密码"
          />
        </n-form-item>

        <n-form-item label="默认连接">
          <n-switch v-model:value="formData.isDefault" />
        </n-form-item>
      </n-form>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, h } from 'vue';
import {
  NCard,
  NButton,
  NIcon,
  NList,
  NListItem,
  NThing,
  NSpace,
  NText,
  NTag,
  NEmpty,
  NModal,
  NForm,
  NFormItem,
  NInput,
  NInputNumber,
  NSwitch,
  NPopconfirm,
  type FormInst,
  type FormRules,
} from 'naive-ui';
import { useConfig } from '@/composables/useConfig';
import { useNotification } from '@/composables/useNotification';
import type { DatabaseConnection } from '@/types/config';
import { connectionValidationRules } from '@/utils/validation';

// Icons
const AddIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', { d: 'M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z' }),
  ]);

const DatabaseIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', {
      d: 'M12 3C7.58 3 4 4.79 4 7s3.58 4 8 4 8-1.79 8-4-3.58-4-8-4zM4 9v3c0 2.21 3.58 4 8 4s8-1.79 8-4V9c0 2.21-3.58 4-8 4s-8-1.79-8-4zm0 5v3c0 2.21 3.58 4 8 4s8-1.79 8-4v-3c0 2.21-3.58 4-8 4s-8-1.79-8-4z',
    }),
  ]);

const { connections, addConnection, updateConnection, deleteConnection, setDefaultConnection } =
  useConfig();
const { showSuccess, showError } = useNotification();

const showDialog = ref(false);
const editingConnection = ref<DatabaseConnection | null>(null);
const formRef = ref<FormInst | null>(null);

const formData = ref({
  name: '',
  host: '',
  port: 5432,
  username: '',
  password: '',
  isDefault: false,
});

const rules: FormRules = {
  name: connectionValidationRules.name,
  host: connectionValidationRules.host,
  port: connectionValidationRules.port,
  username: connectionValidationRules.username,
  password: connectionValidationRules.password,
};

const showAddDialog = () => {
  editingConnection.value = null;
  formData.value = {
    name: '',
    host: '',
    port: 5432,
    username: '',
    password: '',
    isDefault: false,
  };
  showDialog.value = true;
};

const handleEdit = (connection: DatabaseConnection) => {
  editingConnection.value = connection;
  formData.value = {
    name: connection.name,
    host: connection.host,
    port: connection.port,
    username: connection.username,
    password: connection.password,
    isDefault: connection.isDefault,
  };
  showDialog.value = true;
};

const handleSave = async () => {
  try {
    await formRef.value?.validate();

    if (editingConnection.value) {
      // Update existing connection
      updateConnection(editingConnection.value.id, formData.value);
      showSuccess('连接配置已更新');
    } else {
      // Add new connection
      const newConnection: DatabaseConnection = {
        id: crypto.randomUUID(),
        ...formData.value,
      };
      addConnection(newConnection);
      showSuccess('连接配置已添加');
    }

    showDialog.value = false;
  } catch (error) {
    console.error('Form validation failed:', error);
  }
};

const handleSetDefault = (id: string) => {
  setDefaultConnection(id);
  showSuccess('已设置为默认连接');
};

const handleDelete = (id: string) => {
  deleteConnection(id);
  showSuccess('连接配置已删除');
};
</script>

<style scoped>
.settings-view {
  padding: 24px;
  max-width: 1200px;
  margin: 0 auto;
}
</style>
