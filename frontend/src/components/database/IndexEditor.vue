<template>
  <div class="index-editor">
    <n-space vertical :size="16">
      <!-- Toolbar -->
      <n-space justify="space-between">
        <n-text strong>索引定义</n-text>
        <n-button type="primary" @click="handleAddIndex">
          <template #icon>
            <n-icon><AddIcon /></n-icon>
          </template>
          添加索引
        </n-button>
      </n-space>

      <!-- Indexes List -->
      <n-data-table
        :columns="tableColumns"
        :data="visibleIndexes"
        :pagination="false"
        :bordered="true"
        :single-line="false"
        size="small"
        class="indexes-table"
      />
    </n-space>

    <!-- Index Edit Modal -->
    <n-modal
      v-model:show="showEditModal"
      preset="card"
      :title="editingIndex === -1 ? '添加索引' : '编辑索引'"
      style="width: 600px"
      @close="handleCancelEdit"
    >
      <n-form
        ref="formRef"
        :model="editingIndexDef"
        :rules="formRules"
        label-placement="left"
        label-width="120"
      >
        <n-form-item label="索引名" path="name">
          <n-input
            v-model:value="editingIndexDef.name"
            placeholder="输入索引名"
          />
        </n-form-item>

        <n-form-item label="索引列" path="columns">
          <n-select
            v-model:value="editingIndexDef.columns"
            :options="columnOptions"
            placeholder="选择索引列"
            multiple
            filterable
          />
        </n-form-item>

        <n-form-item label="索引类型" path="type">
          <n-select
            v-model:value="editingIndexDef.type"
            :options="indexTypeOptions"
            placeholder="选择索引类型"
          />
        </n-form-item>

        <n-form-item label="唯一索引" path="unique">
          <n-checkbox v-model:checked="editingIndexDef.unique">
            唯一索引
          </n-checkbox>
        </n-form-item>
      </n-form>

      <template #footer>
        <n-space justify="end">
          <n-button @click="handleCancelEdit">取消</n-button>
          <n-button type="primary" @click="handleSaveIndex">保存</n-button>
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
  NCheckbox,
  NTag,
  useDialog,
  useMessage,
  type DataTableColumns,
} from 'naive-ui';
import { Add as AddIcon, Create as EditIcon, Trash as DeleteIcon } from '@vicons/ionicons5';
import { useTableDesignerStore } from '@/stores/table-designer';
import type { IndexDefinition } from '@/types/table-designer';

const designerStore = useTableDesignerStore();
const dialog = useDialog();
const message = useMessage();

const showEditModal = ref(false);
const editingIndex = ref(-1);
const formRef = ref();

const defaultIndexDef: IndexDefinition = {
  name: '',
  columns: [],
  type: 'btree',
  unique: false,
};

const editingIndexDef = ref<IndexDefinition>({ ...defaultIndexDef });

const indexTypeOptions = [
  { label: 'B-tree', value: 'btree' },
  { label: 'Hash', value: 'hash' },
  { label: 'GiST', value: 'gist' },
  { label: 'GIN', value: 'gin' },
];

const formRules = {
  name: {
    required: true,
    message: '请输入索引名',
    trigger: 'blur',
  },
  columns: {
    required: true,
    type: 'array',
    min: 1,
    message: '请至少选择一个列',
    trigger: 'change',
  },
  type: {
    required: true,
    message: '请选择索引类型',
    trigger: 'change',
  },
};

const columnOptions = computed(() => {
  if (!designerStore.currentDesign) return [];
  return designerStore.currentDesign.columns
    .filter(col => !col.isDeleted)
    .map(col => ({
      label: col.name,
      value: col.name,
    }));
});

const visibleIndexes = computed(() => {
  if (!designerStore.currentDesign) return [];
  return designerStore.currentDesign.indexes.filter(idx => !idx.isDeleted);
});

const tableColumns: DataTableColumns<IndexDefinition> = [
  {
    title: '索引名',
    key: 'name',
    width: 200,
  },
  {
    title: '索引列',
    key: 'columns',
    width: 250,
    render: (row) => {
      return row.columns.join(', ');
    },
  },
  {
    title: '索引类型',
    key: 'type',
    width: 120,
    render: (row) => {
      const typeLabels: Record<string, string> = {
        btree: 'B-tree',
        hash: 'Hash',
        gist: 'GiST',
        gin: 'GIN',
      };
      return typeLabels[row.type] || row.type;
    },
  },
  {
    title: '属性',
    key: 'properties',
    width: 150,
    render: (row) => {
      const tags = [];
      if (row.unique) {
        tags.push(h(NTag, { type: 'info', size: 'small' }, { default: () => 'UNIQUE' }));
      }
      if (row.isNew) {
        tags.push(h(NTag, { type: 'warning', size: 'small' }, { default: () => '新增' }));
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
                onClick: () => handleEditIndex(index),
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
                onClick: () => handleDeleteIndex(index),
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

function handleAddIndex() {
  editingIndex.value = -1;
  editingIndexDef.value = { ...defaultIndexDef };
  showEditModal.value = true;
}

function handleEditIndex(index: number) {
  if (!designerStore.currentDesign) return;
  editingIndex.value = index;
  editingIndexDef.value = { ...designerStore.currentDesign.indexes[index] };
  showEditModal.value = true;
}

function handleDeleteIndex(index: number) {
  if (!designerStore.currentDesign) return;
  
  const indexDef = designerStore.currentDesign.indexes[index];
  
  dialog.warning({
    title: '确认删除',
    content: `确定要删除索引 "${indexDef.name}" 吗？`,
    positiveText: '确定',
    negativeText: '取消',
    onPositiveClick: () => {
      designerStore.deleteIndex(index);
      message.success('索引已删除');
    },
  });
}

async function handleSaveIndex() {
  if (!formRef.value) return;

  try {
    await formRef.value.validate();

    if (editingIndex.value === -1) {
      // Add new index
      designerStore.addIndex({ ...editingIndexDef.value });
      message.success('索引已添加');
    } else {
      // Update existing index - delete old and add new
      designerStore.deleteIndex(editingIndex.value);
      designerStore.addIndex({ ...editingIndexDef.value });
      message.success('索引已更新');
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
.index-editor {
  padding: 16px;
}

.indexes-table {
  width: 100%;
}
</style>
