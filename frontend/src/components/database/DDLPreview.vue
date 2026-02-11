<template>
  <div class="ddl-preview">
    <n-space vertical :size="16">
      <!-- Toolbar -->
      <n-space justify="space-between" align="center">
        <n-text strong>DDL 预览</n-text>
        <n-space>
          <n-button
            :loading="isGenerating"
            @click="handleGenerateDDL"
          >
            <template #icon>
              <n-icon><RefreshIcon /></n-icon>
            </template>
            刷新 DDL
          </n-button>
          <n-button
            type="primary"
            :disabled="!generatedDDL || isApplying"
            :loading="isApplying"
            @click="handleApply"
          >
            <template #icon>
              <n-icon><CheckmarkIcon /></n-icon>
            </template>
            应用
          </n-button>
          <n-button
            :disabled="!generatedDDL"
            @click="handleSaveAsScript"
          >
            <template #icon>
              <n-icon><SaveIcon /></n-icon>
            </template>
            保存为脚本
          </n-button>
        </n-space>
      </n-space>

      <!-- Status Messages -->
      <n-alert
        v-if="statusMessage"
        :type="statusType"
        closable
        @close="statusMessage = ''"
      >
        {{ statusMessage }}
      </n-alert>

      <!-- Monaco Editor for DDL Display -->
      <div
        ref="editorContainer"
        class="ddl-editor-container"
      />

      <!-- DDL Info -->
      <n-space v-if="generatedDDL" align="center">
        <n-text depth="3">
          <n-icon><DocumentTextIcon /></n-icon>
          {{ ddlLineCount }} 行
        </n-text>
        <n-text depth="3">
          <n-icon><TimeIcon /></n-icon>
          生成时间: {{ generationTime }}ms
        </n-text>
      </n-space>
    </n-space>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed, watch, nextTick } from 'vue';
import {
  NSpace,
  NText,
  NButton,
  NIcon,
  NAlert,
  useMessage,
  useDialog,
} from 'naive-ui';
import {
  Refresh as RefreshIcon,
  Checkmark as CheckmarkIcon,
  Save as SaveIcon,
  DocumentText as DocumentTextIcon,
  Time as TimeIcon,
} from '@vicons/ionicons5';
import * as monaco from 'monaco-editor';
import { useTableDesignerStore } from '@/stores/table-designer';
import { save } from '@tauri-apps/plugin-dialog';
import { writeTextFile } from '@tauri-apps/plugin-fs';

const designerStore = useTableDesignerStore();
const message = useMessage();
const dialog = useDialog();

const editorContainer = ref<HTMLElement>();
let editorInstance: monaco.editor.IStandaloneCodeEditor | null = null;

const isGenerating = ref(false);
const isApplying = ref(false);
const generatedDDL = ref('');
const generationTime = ref(0);
const statusMessage = ref('');
const statusType = ref<'success' | 'info' | 'warning' | 'error'>('info');

const ddlLineCount = computed(() => {
  if (!generatedDDL.value) return 0;
  return generatedDDL.value.split('\n').length;
});

/**
 * 初始化 Monaco Editor
 */
const initializeEditor = async () => {
  if (!editorContainer.value) return;

  // 创建只读的 SQL 编辑器
  editorInstance = monaco.editor.create(editorContainer.value, {
    value: '',
    language: 'sql',
    theme: 'vs-dark',
    readOnly: true,
    automaticLayout: true,
    minimap: { enabled: false },
    scrollBeyondLastLine: false,
    fontSize: 14,
    lineNumbers: 'on',
    renderWhitespace: 'selection',
    folding: true,
    wordWrap: 'on',
  });
};

/**
 * 生成 DDL
 */
const handleGenerateDDL = async () => {
  if (!designerStore.currentDesign) {
    message.warning('没有可用的表设计');
    return;
  }

  // 验证表设计
  if (!designerStore.currentDesign.tableName.trim()) {
    message.warning('请输入表名');
    return;
  }

  if (designerStore.currentDesign.columns.length === 0) {
    message.warning('请至少添加一列');
    return;
  }

  isGenerating.value = true;
  statusMessage.value = '';

  try {
    const startTime = performance.now();
    const ddl = await designerStore.generateDDL();
    const endTime = performance.now();

    generatedDDL.value = ddl;
    generationTime.value = Math.round(endTime - startTime);

    // 更新编辑器内容
    if (editorInstance) {
      editorInstance.setValue(ddl);
    }

    statusMessage.value = 'DDL 生成成功';
    statusType.value = 'success';
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : 'DDL 生成失败';
    statusMessage.value = errorMsg;
    statusType.value = 'error';
    message.error(errorMsg);
  } finally {
    isGenerating.value = false;
  }
};

/**
 * 应用 DDL 到数据库
 */
const handleApply = async () => {
  if (!generatedDDL.value) {
    message.warning('请先生成 DDL');
    return;
  }

  dialog.warning({
    title: '确认应用',
    content: `确定要将此 DDL 应用到数据库吗？此操作将${
      designerStore.mode === 'create' ? '创建新表' : '修改现有表结构'
    }。`,
    positiveText: '确定',
    negativeText: '取消',
    onPositiveClick: async () => {
      isApplying.value = true;
      statusMessage.value = '';

      try {
        await designerStore.applyChanges();
        statusMessage.value = designerStore.mode === 'create' 
          ? '表创建成功' 
          : '表修改成功';
        statusType.value = 'success';
        message.success(statusMessage.value);

        // 成功后关闭设计器
        setTimeout(() => {
          designerStore.closeDesigner();
        }, 1000);
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : '应用 DDL 失败';
        statusMessage.value = errorMsg;
        statusType.value = 'error';
        message.error(errorMsg);
      } finally {
        isApplying.value = false;
      }
    },
  });
};

/**
 * 保存 DDL 为脚本文件
 */
const handleSaveAsScript = async () => {
  if (!generatedDDL.value) {
    message.warning('请先生成 DDL');
    return;
  }

  try {
    // 打开保存对话框
    const filePath = await save({
      defaultPath: `${designerStore.currentDesign?.tableName || 'table'}.sql`,
      filters: [
        {
          name: 'SQL Script',
          extensions: ['sql'],
        },
      ],
    });

    if (filePath) {
      // 写入文件
      await writeTextFile(filePath, generatedDDL.value);
      message.success('DDL 脚本已保存');
    }
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : '保存脚本失败';
    message.error(errorMsg);
  }
};

/**
 * 监听设计变化，自动重新生成 DDL
 */
watch(
  () => designerStore.currentDesign,
  async () => {
    if (designerStore.currentDesign && designerStore.isDirty) {
      // 设计发生变化时，清空之前的 DDL
      generatedDDL.value = '';
      if (editorInstance) {
        editorInstance.setValue('');
      }
      statusMessage.value = '表设计已更改，请重新生成 DDL';
      statusType.value = 'info';
    }
  },
  { deep: true }
);

onMounted(async () => {
  await nextTick();
  await initializeEditor();
  
  // 自动生成初始 DDL
  if (designerStore.currentDesign) {
    await handleGenerateDDL();
  }
});

onBeforeUnmount(() => {
  if (editorInstance) {
    editorInstance.dispose();
    editorInstance = null;
  }
});
</script>

<style scoped>
.ddl-preview {
  padding: 16px;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.ddl-editor-container {
  flex: 1;
  min-height: 400px;
  border: 1px solid var(--n-border-color);
  border-radius: 4px;
  overflow: hidden;
}
</style>
