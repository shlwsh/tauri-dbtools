<template>
  <div class="column-editor">
    <n-space vertical :size="16">
      <!-- Toolbar -->
      <n-space justify="space-between">
        <n-text strong>列定义</n-text>
        <n-button type="primary" @click="handleAddColumn">
          <template #icon>
            <n-icon><AddIcon /></n-icon>
          </template>
          添加列
        </n-button>
      </n-space>

      <!-- Columns List -->
      <n-data-table
        :columns="tableColumns"
        :data="visibleColumns"
        :pagination="false"
        :bordered="true"
        :single-line="false"
        size="small"
        class="columns-table"
      />
    </n-space>

    <!-- Column Edit Modal -->
    <n-modal
      v-model:show="showEditModal"
      preset="card"
      :title="editingIndex === -1 ? '添加列' : '编辑列'"
      style="width: 600px"
      @close="handleCancelEdit"
    >
      <n-form
        ref="formRef"
        :model="editingColumn"
        :rules="formRules"
        label-placement="left"
        label-width="120"
      >
        <n-form-item label="列名" path="name">
          <n-input
            v-model:value="editingColumn.name"
            placeholder="输入列名"
          />
        </n-form-item>

        <n-form-item label="数据类型" path="type">
          <n-select
            v-model:value="editingColumn.type"
            :options="dataTypeOptions"
            placeholder="选择数据类型"
            @update:value="handleTypeChange"
          />
        </n-form-item>

        <!-- Length for VARCHAR/CHAR -->
        <n-form-item
          v-if="needsLength"
          label="长度"
          path="length"
        >
          <n-input-number
            v-model:value="editingColumn.length"
            :min="1"
            :max="10485760"
            placeholder="输入长度"
            style="width: 100%"
          />
        </n-form-item>

        <!-- Precision and Scale for NUMERIC/DECIMAL -->
        <n-form-item
          v-if="needsPrecision"
          label="精度"
          path="precision"
        >
          <n-input-number
            v-model:value="editingColumn.precision"
            :min="1"
            :max="1000"
            placeholder="输入精度"
            style="width: 100%"
          />
        </n-form-item>

        <n-form-item
          v-if="needsPrecision"
          label="小数位数"
          path="scale"
        >
          <n-input-number
            v-model:value="editingColumn.scale"
            :min="0"
            :max="editingColumn.precision || 0"
            placeholder="输入小数位数"
            style="width: 100%"
          />
        </n-form-item>

        <n-form-item label="可空" path="nullable">
          <n-checkbox v-model:checked="editingColumn.nullable">
            允许NULL值
          </n-checkbox>
        </n-form-item>

        <n-form-item label="默认值" path="defaultValue">
          <n-input
            v-model:value="editingColumn.defaultValue"
            placeholder="输入默认值表达式"
          />
        </n-form-item>

        <n-form-item label="主键" path="isPrimaryKey">
          <n-checkbox v-model:checked="editingColumn.isPrimaryKey">
            设为主键
          </n-checkbox>
        </n-form-item>

        <n-form-item label="唯一约束" path="isUnique">
          <n-checkbox v-model:checked="editingColumn.isUnique">
            唯一约束
          </n-checkbox>
        </n-form-item>

        <n-form-item label="注释" path="comment">
          <n-input
            v-model:value="editingColumn.comment"
            type="textarea"
            placeholder="输入列注释"
            :rows="2"
          />
        </n-form-item>
      </n-form>

      <template #footer>
        <n-space justify="end">
          <n-button @click="handleCancelEdit">取消</n-button>
          <n-button type="primary" @click="handleSaveColumn">保存</n-button>
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
  NInputNumber,
  NSelect,
  NCheckbox,
  NTag,
  useDialog,
  useMessage,
  type DataTableColumns,
} from 'naive-ui';
import { Add as AddIcon, Create as EditIcon, Trash as DeleteIcon } from '@vicons/ionicons5';
import { useTableDesignerStore } from '@/stores/table-designer';
import type { ColumnDefinition, PostgreSQLType } from '@/types/table-designer';

const designerStore = useTableDesignerStore();
const dialog = useDialog();
const message = useMessage();

const showEditModal = ref(false);
const editingIndex = ref(-1);
const formRef = ref();

const defaultColumn: ColumnDefinition = {
  name: '',
  type: 'VARCHAR',
  nullable: true,
  isPrimaryKey: false,
  isUnique: false,
};

const editingColumn = ref<ColumnDefinition>({ ...defaultColumn });

const dataTypeOptions = [
  { label: 'INTEGER', value: 'INTEGER' },
  { label: 'BIGINT', value: 'BIGINT' },
  { label: 'SMALLINT', value: 'SMALLINT' },
  { label: 'DECIMAL', value: 'DECIMAL' },
  { label: 'NUMERIC', value: 'NUMERIC' },
  { label: 'REAL', value: 'REAL' },
  { label: 'DOUBLE PRECISION', value: 'DOUBLE PRECISION' },
  { label: 'VARCHAR', value: 'VARCHAR' },
  { label: 'CHAR', value: 'CHAR' },
  { label: 'TEXT', value: 'TEXT' },
  { label: 'BOOLEAN', value: 'BOOLEAN' },
  { label: 'DATE', value: 'DATE' },
  { label: 'TIME', value: 'TIME' },
  { label: 'TIMESTAMP', value: 'TIMESTAMP' },
  { label: 'TIMESTAMPTZ', value: 'TIMESTAMPTZ' },
  { label: 'JSON', value: 'JSON' },
  { label: 'JSONB', value: 'JSONB' },
  { label: 'UUID', value: 'UUID' },
  { label: 'BYTEA', value: 'BYTEA' },
];

const formRules = {
  name: {
    required: true,
    message: '请输入列名',
    trigger: 'blur',
  },
  type: {
    required: true,
    message: '请选择数据类型',
    trigger: 'change',
  },
};

const needsLength = computed(() => {
  return editingColumn.value.type === 'VARCHAR' || editingColumn.value.type === 'CHAR';
});

const needsPrecision = computed(() => {
  return editingColumn.value.type === 'DECIMAL' || editingColumn.value.type === 'NUMERIC';
});

const visibleColumns = computed(() => {
  if (!designerStore.currentDesign) return [];
  return designerStore.currentDesign.columns.filter(col => !col.isDeleted);
});

const tableColumns: DataTableColumns<ColumnDefinition> = [
  {
    title: '列名',
    key: 'name',
    width: 150,
  },
  {
    title: '数据类型',
    key: 'type',
    width: 150,
    render: (row) => {
      let typeStr = row.type;
      if (row.length) {
        typeStr += `(${row.length})`;
      } else if (row.precision) {
        typeStr += row.scale ? `(${row.precision},${row.scale})` : `(${row.precision})`;
      }
      return typeStr;
    },
  },
  {
    title: '可空',
    key: 'nullable',
    width: 80,
    render: (row) => {
      return row.nullable ? '是' : '否';
    },
  },
  {
    title: '默认值',
    key: 'defaultValue',
    width: 120,
    ellipsis: {
      tooltip: true,
    },
  },
  {
    title: '约束',
    key: 'constraints',
    width: 150,
    render: (row) => {
      const tags = [];
      if (row.isPrimaryKey) {
        tags.push(h(NTag, { type: 'success', size: 'small' }, { default: () => 'PK' }));
      }
      if (row.isUnique) {
        tags.push(h(NTag, { type: 'info', size: 'small' }, { default: () => 'UNIQUE' }));
      }
      if (row.isNew) {
        tags.push(h(NTag, { type: 'warning', size: 'small' }, { default: () => '新增' }));
      }
      if (row.isModified) {
        tags.push(h(NTag, { type: 'warning', size: 'small' }, { default: () => '已修改' }));
      }
      return h(NSpace, { size: 4 }, { default: () => tags });
    },
  },
  {
    title: '操作',
    key: 'actions',
    width: 120,
    render: (row, index) => {
      return h(
        NSpace,
        { size: 4 },
        {
          default: () => [
            h(
              NButton,
              {
                size: 'small',
                onClick: () => handleEditColumn(index),
              },
              {
                icon: () => h(NIcon, null, { default: () => h(EditIcon) }),
              }
            ),
            h(
              NButton,
              {
                size: 'small',
                type: 'error',
                onClick: () => handleDeleteColumn(index),
              },
              {
                icon: () => h(NIcon, null, { default: () => h(DeleteIcon) }),
              }
            ),
          ],
        }
      );
    },
  },
];

function handleAddColumn() {
  editingIndex.value = -1;
  editingColumn.value = { ...defaultColumn };
  showEditModal.value = true;
}

function handleEditColumn(index: number) {
  if (!designerStore.currentDesign) return;
  editingIndex.value = index;
  editingColumn.value = { ...designerStore.currentDesign.columns[index] };
  showEditModal.value = true;
}

function handleDeleteColumn(index: number) {
  if (!designerStore.currentDesign) return;
  
  const column = designerStore.currentDesign.columns[index];
  
  dialog.warning({
    title: '确认删除',
    content: `确定要删除列 "${column.name}" 吗？`,
    positiveText: '确定',
    negativeText: '取消',
    onPositiveClick: () => {
      designerStore.deleteColumn(index);
      message.success('列已删除');
    },
  });
}

function handleTypeChange() {
  // Clear length/precision/scale when type changes
  if (!needsLength.value) {
    editingColumn.value.length = undefined;
  }
  if (!needsPrecision.value) {
    editingColumn.value.precision = undefined;
    editingColumn.value.scale = undefined;
  }
}

async function handleSaveColumn() {
  if (!formRef.value) return;

  try {
    await formRef.value.validate();

    if (editingIndex.value === -1) {
      // Add new column
      designerStore.addColumn({ ...editingColumn.value });
      message.success('列已添加');
    } else {
      // Update existing column
      designerStore.updateColumn(editingIndex.value, { ...editingColumn.value });
      message.success('列已更新');
    }

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
.column-editor {
  padding: 16px;
}

.columns-table {
  width: 100%;
}
</style>
