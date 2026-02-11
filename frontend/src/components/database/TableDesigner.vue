<template>
  <n-modal
    v-model:show="designerStore.isOpen"
    :mask-closable="false"
    preset="card"
    :title="modalTitle"
    class="table-designer-modal"
    style="width: 90%; max-width: 1200px"
    @close="handleClose"
  >
    <template #header-extra>
      <n-space>
        <n-button
          v-if="designerStore.hasChanges"
          quaternary
          @click="handleReset"
        >
          重置
        </n-button>
      </n-space>
    </template>

    <n-spin :show="designerStore.isLoading">
      <div v-if="designerStore.currentDesign" class="table-designer-content">
        <!-- Table Name and Schema -->
        <n-form
          ref="formRef"
          :model="designerStore.currentDesign"
          label-placement="left"
          label-width="100"
          class="table-info-form"
        >
          <n-form-item label="表名" path="tableName" required>
            <n-input
              v-model:value="designerStore.currentDesign.tableName"
              placeholder="输入表名"
              :disabled="designerStore.mode === 'edit'"
              @update:value="handleDesignChange"
            />
          </n-form-item>
          <n-form-item label="Schema" path="schema" required>
            <n-input
              v-model:value="designerStore.currentDesign.schema"
              placeholder="输入schema名称"
              :disabled="designerStore.mode === 'edit'"
              @update:value="handleDesignChange"
            />
          </n-form-item>
        </n-form>

        <!-- Tabs for Columns, Constraints, and Indexes -->
        <n-tabs
          v-model:value="activeTab"
          type="line"
          animated
          class="designer-tabs"
        >
          <n-tab-pane name="columns" tab="列">
            <ColumnEditor />
          </n-tab-pane>
          <n-tab-pane name="constraints" tab="约束">
            <ConstraintEditor />
          </n-tab-pane>
          <n-tab-pane name="indexes" tab="索引">
            <IndexEditor />
          </n-tab-pane>
          <n-tab-pane name="preview" tab="预览SQL">
            <DDLPreview />
          </n-tab-pane>
        </n-tabs>
      </div>

      <!-- Error Display -->
      <n-alert
        v-if="designerStore.error"
        type="error"
        :title="'错误'"
        closable
        @close="designerStore.error = null"
        class="error-alert"
      >
        {{ designerStore.error }}
      </n-alert>
    </n-spin>

    <template #footer>
      <n-space justify="end">
        <n-button @click="handleClose">
          取消
        </n-button>
        <n-button
          type="primary"
          :disabled="!canApply"
          :loading="designerStore.isLoading"
          @click="handleApply"
        >
          应用
        </n-button>
      </n-space>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import {
  NModal,
  NButton,
  NSpace,
  NForm,
  NFormItem,
  NInput,
  NTabs,
  NTabPane,
  NSpin,
  NAlert,
  useDialog,
  useMessage,
} from 'naive-ui';
import { useTableDesignerStore } from '@/stores/table-designer';
import ColumnEditor from './ColumnEditor.vue';
import ConstraintEditor from './ConstraintEditor.vue';
import IndexEditor from './IndexEditor.vue';
import DDLPreview from './DDLPreview.vue';

const designerStore = useTableDesignerStore();
const dialog = useDialog();
const message = useMessage();

const activeTab = ref('columns');
const formRef = ref();

const modalTitle = computed(() => {
  if (designerStore.mode === 'create') {
    return '创建新表';
  }
  return `设计表: ${designerStore.currentDesign?.tableName || ''}`;
});

const canApply = computed(() => {
  if (!designerStore.currentDesign) return false;
  if (!designerStore.currentDesign.tableName.trim()) return false;
  if (!designerStore.currentDesign.schema.trim()) return false;
  if (designerStore.currentDesign.columns.length === 0) return false;
  return designerStore.isDirty;
});

function handleDesignChange() {
  if (designerStore.currentDesign) {
    designerStore.currentDesign.isDirty = true;
  }
}

function handleClose() {
  if (designerStore.hasChanges) {
    dialog.warning({
      title: '确认关闭',
      content: '您有未保存的更改，确定要关闭吗？',
      positiveText: '确定',
      negativeText: '取消',
      onPositiveClick: () => {
        designerStore.closeDesigner();
      },
    });
  } else {
    designerStore.closeDesigner();
  }
}

function handleReset() {
  dialog.warning({
    title: '确认重置',
    content: '确定要重置所有更改吗？',
    positiveText: '确定',
    negativeText: '取消',
    onPositiveClick: () => {
      designerStore.resetDesign();
      message.success('已重置');
    },
  });
}

async function handleApply() {
  try {
    await designerStore.applyChanges();
    message.success(
      designerStore.mode === 'create' ? '表创建成功' : '表修改成功'
    );
  } catch (error) {
    message.error(
      error instanceof Error ? error.message : '应用更改失败'
    );
  }
}
</script>

<style scoped>
.table-designer-modal {
  min-height: 600px;
}

.table-designer-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.table-info-form {
  padding: 16px;
  background-color: var(--n-color-target);
  border-radius: 4px;
}

.designer-tabs {
  flex: 1;
  min-height: 400px;
}

.error-alert {
  margin-top: 16px;
}
</style>
