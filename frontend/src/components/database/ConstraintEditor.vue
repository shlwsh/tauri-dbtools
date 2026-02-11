<template>
  <div class="constraint-editor">
    <n-space vertical :size="16">
      <!-- Toolbar -->
      <n-space justify="space-between">
        <n-text strong>约束定义</n-text>
        <n-button type="primary" @click="handleAddConstraint">
          <template #icon>
            <n-icon><AddIcon /></n-icon>
          </template>
          添加约束
        </n-button>
      </n-space>

      <!-- Constraints List -->
      <n-data-table
        :columns="tableColumns"
        :data="visibleConstraints"
        :pagination="false"
        :bordered="true"
        size="small"
        class="constraints-table"
      />
    </n-space>

    <!-- Constraint Edit Modal -->
    <n-modal
      v-model:show="showEditModal"
      preset="card"
      title="添加约束"
      style="width: 600px"
      @close="handleCancelEdit"
    >
      <n-form
        ref="formRef"
        :model="editingConstraint"
        :rules="formRules"
        label-placement="left"
        label-width="120"
      >
        <n-form-item label="约束类型" path="type">
          <n-select
            v-model:value="editingConstraint.type"
            :options="constraintTypeOptions"
            placeholder="选择约束类型"
            @update:value="handleTypeChange"
          />
        </n-form-item>

        <n-form-item label="约束名称" path="name">
          <n-input
            v-model:value="editingConstraint.name"
            placeholder="输入约束名称"
          />
        </n-form-item>

        <n-form-item label="列" path="columns">
          <n-select
            v-model:value="editingConstraint.columns"
            :options="columnOptions"
            multiple
            placeholder="选择列"
          />
        </n-form-item>

        <!-- Foreign Key specific fields -->
        <template v-if="editingConstraint.type === 'foreign_key'">
          <n-form-item label="引用表" path="referencedTable">
            <n-input
              v-model:value="editingConstraint.referencedTable"
              placeholder="输入引用表名"
            />
          </n-form-item>

          <n-form-item label="引用列" path="referencedColumns">
            <n-dynamic-tags
              v-model:value="editingConstraint.referencedColumns"
              placeholder="输入引用列名"
            />
          </n-form-item>

          <n-form-item label="ON DELETE" path="onDelete">
            <n-select
              v-model:value="editingConstraint.onDelete"
              :options="referentialActionOptions"
              placeholder="选择ON DELETE操作"
              clearable
            />
          </n-form-item>

          <n-form-item label="ON UPDATE" path="onUpdate">
            <n-select
              v-model:value="editingConstraint.onUpdate"
              :options="referentialActionOptions"
              placeholder="选择ON UPDATE操作"
              clearable
            />
          </n-form-item>
        </template>

        <!-- Check constraint specific field -->
        <n-form-item
          v-if="editingConstraint.type === 'check'"
          label="检查表达式"
          path="checkExpression"
        >
          <n-input
            v-model:value="editingConstraint.checkExpression"
            type="textarea"
            placeholder="输入检查表达式，例如: age >= 18"
            :rows="3"
          />
        </n-form-item>
      </n-form>

      <template #footer>
        <n-space justify="end">
          <n-button @click="handleCancelEdit">取消</n-button>
          <n-button type="primary" @click="handleSaveConstraint">保存</n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, h } from 'vue';
import {
  NSpace,
  NText,
  NButton,
  NIcon,
  NDataTable,
  NModal,
  NForm,
  NFormItem,
  NInput,
  NSelect,
  NDynamicTags,
  NTag,
  useDialog,
  useMessage,
  type DataTableColumns,
  type SelectOption,
} from 'naive-ui';
import { Add as AddIcon, Trash as DeleteIcon } from '@vicons/ionicons5';
import { useTableDesignerStore } from '@/stores/table-designer';
import type { ConstraintDefinition, ReferentialAction } from '@/types/table-designer';

const designerStore = useTableDesignerStore();
const dialog = useDialog();
const message = useMessage();

const showEditModal = ref(false);
const formRef = ref();

const defaultConstraint: ConstraintDefinition = {
  type: 'primary_key',
  name: '',
  columns: [],
};

const editingConstraint = ref<ConstraintDefinition>({ ...defaultConstraint });

const constraintTypeOptions: SelectOption[] = [
  { label: '主键 (Primary Key)', value: 'primary_key' },
  { label: '外键 (Foreign Key)', value: 'foreign_key' },
  { label: '唯一 (Unique)', value: 'unique' },
  { label: '检查 (Check)', value: 'check' },
];

const referentialActionOptions: SelectOption[] = [
  { label: 'CASCADE', value: 'CASCADE' },
  { label: 'SET NULL', value: 'SET NULL' },
  { label: 'RESTRICT', value: 'RESTRICT' },
  { label: 'NO ACTION', value: 'NO ACTION' },
];

const columnOptions = computed(() => {
  if (!designerStore.currentDesign) return [];
  return designerStore.currentDesign.columns
    .filter(col => !col.isDeleted)
    .map(col => ({
      label: col.name,
      value: col.name,
    }));
});

const formRules = computed(() => ({
  type: {
    required: true,
    message: '请选择约束类型',
    trigger: 'change',
  },
  name: {
    required: true,
    message: '请输入约束名称',
    trigger: 'blur',
  },
  columns: {
    required: true,
    type: 'array',
    min: 1,
    message: '请至少选择一列',
    trigger: 'change',
  },
  referencedTable: {
    required: editingConstraint.value.type === 'foreign_key',
    message: '请输入引用表名',
    trigger: 'blur',
  },
  referencedColumns: {
    required: editingConstraint.value.type === 'foreign_key',
    type: 'array',
    min: 1,
    message: '请至少输入一个引用列',
    trigger: 'change',
  },
  checkExpression: {
    required: editingConstraint.value.type === 'check',
    message: '请输入检查表达式',
    trigger: 'blur',
  },
}));

const visibleConstraints = computed(() => {
  if (!designerStore.currentDesign) return [];
  return designerStore.currentDesign.constraints.filter(con => !con.isDeleted);
});

const tableColumns: DataTableColumns<ConstraintDefinition> = [
  {
    title: '约束名称',
    key: 'name',
    width: 200,
  },
  {
    title: '类型',
    key: 'type',
    width: 120,
    render: (row) => {
      const typeMap: Record<string, { label: string; type: any }> = {
        primary_key: { label: '主键', type: 'success' },
        foreign_key: { label: '外键', type: 'info' },
        unique: { label: '唯一', type: 'warning' },
        check: { label: '检查', type: 'default' },
      };
      const info = typeMap[row.type] || { label: row.type, type: 'default' };
      return h(NTag, { type: info.type, size: 'small' }, { default: () => info.label });
    },
  },
  {
    title: '列',
    key: 'columns',
    width: 200,
    render: (row) => {
      return row.columns.join(', ');
    },
  },
  {
    title: '详情',
    key: 'details',
    ellipsis: {
      tooltip: true,
    },
    render: (row) => {
      if (row.type === 'foreign_key') {
        return `引用: ${row.referencedTable}(${row.referencedColumns?.join(', ')})`;
      } else if (row.type === 'check') {
        return row.checkExpression || '';
      }
      return '';
    },
  },
  {
    title: '操作',
    key: 'actions',
    width: 80,
    render: (row, index) => {
      return h(
        NButton,
        {
          size: 'small',
          type: 'error',
          onClick: () => handleDeleteConstraint(index),
        },
        {
          icon: () => h(NIcon, null, { default: () => h(DeleteIcon) }),
        }
      );
    },
  },
];

function handleAddConstraint() {
  editingConstraint.value = { ...defaultConstraint };
  showEditModal.value = true;
}

function handleDeleteConstraint(index: number) {
  if (!designerStore.currentDesign) return;
  
  const constraint = designerStore.currentDesign.constraints[index];
  
  dialog.warning({
    title: '确认删除',
    content: `确定要删除约束 "${constraint.name}" 吗？`,
    positiveText: '确定',
    negativeText: '取消',
    onPositiveClick: () => {
      designerStore.deleteConstraint(index);
      message.success('约束已删除');
    },
  });
}

function handleTypeChange() {
  // Clear type-specific fields when type changes
  if (editingConstraint.value.type !== 'foreign_key') {
    editingConstraint.value.referencedTable = undefined;
    editingConstraint.value.referencedColumns = undefined;
    editingConstraint.value.onDelete = undefined;
    editingConstraint.value.onUpdate = undefined;
  }
  if (editingConstraint.value.type !== 'check') {
    editingConstraint.value.checkExpression = undefined;
  }
}

async function handleSaveConstraint() {
  if (!formRef.value) return;

  try {
    await formRef.value.validate();

    designerStore.addConstraint({ ...editingConstraint.value });
    message.success('约束已添加');
    showEditModal.value = false;
  } catch (error) {
    // Validation failed
  }
}

function handleCancelEdit() {
  showEditModal.value = false;
}
</script>

<style scoped>
.constraint-editor {
  padding: 16px;
}

.constraints-table {
  width: 100%;
}
</style>
