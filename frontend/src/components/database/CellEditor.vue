<!--
  CellEditor Component
  
  单元格编辑器子组件，提供：
  - 双击进入编辑模式
  - 根据数据类型显示不同输入控件
  - Enter/Tab 保存、Escape 取消
  - 实时验证和错误显示
  
  Validates: Requirements 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7, 9.8
-->

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';
import {
  NInput,
  NInputNumber,
  NCheckbox,
  NDatePicker,
  NTimePicker,
  NSelect,
  NTooltip,
} from 'naive-ui';
import { validateCellValue } from '@/services/data-validator';
import type { ColumnInfo } from '@/types/sql-editor';
import type { ValidationResult } from '@/types/data-grid';

// Props
interface Props {
  /** 列信息 */
  column: ColumnInfo;
  /** 当前值 */
  value: any;
  /** 是否处于编辑模式 */
  editing: boolean;
  /** 行索引 */
  rowIndex: number;
}

const props = defineProps<Props>();

// Emits
interface Emits {
  /** 值变化 */
  (e: 'update:value', value: any): void;
  /** 进入编辑模式 */
  (e: 'start-edit'): void;
  /** 保存编辑 */
  (e: 'save', value: any): void;
  /** 取消编辑 */
  (e: 'cancel'): void;
}

const emit = defineEmits<Emits>();

// State
const editValue = ref<any>(props.value);
const inputRef = ref<any>(null);
const validation = ref<ValidationResult>({ isValid: true });

// Computed

/**
 * 数据类型（标准化）
 */
const dataType = computed(() => {
  return props.column.type_name.toLowerCase();
});

/**
 * 是否为整数类型
 */
const isIntegerType = computed(() => {
  return (
    dataType.value.includes('int') ||
    dataType.value === 'smallint' ||
    dataType.value === 'bigint'
  );
});

/**
 * 是否为浮点数类型
 */
const isFloatType = computed(() => {
  return (
    dataType.value === 'real' ||
    dataType.value === 'double precision' ||
    dataType.value === 'float' ||
    dataType.value.includes('numeric') ||
    dataType.value.includes('decimal')
  );
});

/**
 * 是否为布尔类型
 */
const isBooleanType = computed(() => {
  return dataType.value === 'boolean' || dataType.value === 'bool';
});

/**
 * 是否为日期类型
 */
const isDateType = computed(() => {
  return dataType.value === 'date';
});

/**
 * 是否为时间类型
 */
const isTimeType = computed(() => {
  return (
    dataType.value === 'time' ||
    dataType.value === 'time without time zone'
  );
});

/**
 * 是否为时间戳类型
 */
const isTimestampType = computed(() => {
  return (
    dataType.value === 'timestamp' ||
    dataType.value === 'timestamp without time zone' ||
    dataType.value === 'timestamptz' ||
    dataType.value === 'timestamp with time zone'
  );
});

/**
 * 是否为 JSON 类型
 */
const isJSONType = computed(() => {
  return dataType.value === 'json' || dataType.value === 'jsonb';
});

/**
 * 显示值（用于非编辑模式）
 */
const displayValue = computed(() => {
  if (props.value === null || props.value === undefined) {
    return 'NULL';
  }

  if (isBooleanType.value) {
    return props.value ? '✓' : '✗';
  }

  if (isDateType.value || isTimestampType.value) {
    return formatDate(props.value);
  }

  if (isJSONType.value && typeof props.value === 'object') {
    return JSON.stringify(props.value);
  }

  return String(props.value);
});

/**
 * 是否有验证错误
 */
const hasError = computed(() => {
  return !validation.value.isValid;
});

/**
 * 错误消息
 */
const errorMessage = computed(() => {
  return validation.value.error || '';
});

// Methods

/**
 * 格式化日期
 */
function formatDate(value: any): string {
  if (!value) return '';
  
  try {
    const date = new Date(value);
    if (isNaN(date.getTime())) return String(value);
    
    if (isDateType.value) {
      return date.toISOString().split('T')[0];
    }
    
    return date.toLocaleString();
  } catch {
    return String(value);
  }
}

/**
 * 验证当前值
 */
function validateValue(value: any): void {
  validation.value = validateCellValue(value, props.column);
}

/**
 * 处理值变化
 */
function handleValueChange(value: any): void {
  editValue.value = value;
  validateValue(value);
}

/**
 * 保存编辑
 */
function handleSave(): void {
  // 验证值
  validateValue(editValue.value);
  
  if (!validation.value.isValid) {
    return;
  }

  emit('save', editValue.value);
}

/**
 * 取消编辑
 */
function handleCancel(): void {
  editValue.value = props.value;
  validation.value = { isValid: true };
  emit('cancel');
}

/**
 * 处理键盘事件
 */
function handleKeyDown(event: KeyboardEvent): void {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault();
    handleSave();
  } else if (event.key === 'Escape') {
    event.preventDefault();
    handleCancel();
  } else if (event.key === 'Tab') {
    event.preventDefault();
    handleSave();
    // TODO: 移动到下一个单元格
  }
}

/**
 * 处理双击（进入编辑模式）
 */
function handleDblClick(): void {
  if (!props.editing) {
    emit('start-edit');
  }
}

/**
 * 聚焦输入框
 */
async function focusInput(): Promise<void> {
  await nextTick();
  
  if (inputRef.value) {
    if (inputRef.value.$el) {
      // NaiveUI 组件
      const input = inputRef.value.$el.querySelector('input, textarea');
      if (input) {
        input.focus();
        if (input.select) {
          input.select();
        }
      }
    } else if (inputRef.value.focus) {
      // 原生元素
      inputRef.value.focus();
    }
  }
}

// Watchers

watch(
  () => props.editing,
  (editing) => {
    if (editing) {
      editValue.value = props.value;
      validation.value = { isValid: true };
      focusInput();
    }
  }
);

watch(
  () => props.value,
  (newValue) => {
    if (!props.editing) {
      editValue.value = newValue;
    }
  }
);

// Lifecycle

onMounted(() => {
  if (props.editing) {
    focusInput();
  }
});
</script>

<template>
  <div class="cell-editor" @dblclick="handleDblClick">
    <!-- 编辑模式 -->
    <div v-if="editing" class="cell-editor-input">
      <!-- 布尔类型 -->
      <NCheckbox
        v-if="isBooleanType"
        ref="inputRef"
        :checked="editValue"
        @update:checked="handleValueChange"
        @keydown="handleKeyDown"
      />

      <!-- 整数类型 -->
      <NInputNumber
        v-else-if="isIntegerType"
        ref="inputRef"
        :value="editValue"
        :status="hasError ? 'error' : undefined"
        :precision="0"
        :show-button="false"
        size="small"
        @update:value="handleValueChange"
        @keydown="handleKeyDown"
      />

      <!-- 浮点数类型 -->
      <NInputNumber
        v-else-if="isFloatType"
        ref="inputRef"
        :value="editValue"
        :status="hasError ? 'error' : undefined"
        :show-button="false"
        size="small"
        @update:value="handleValueChange"
        @keydown="handleKeyDown"
      />

      <!-- 日期类型 -->
      <NDatePicker
        v-else-if="isDateType"
        ref="inputRef"
        :value="editValue ? new Date(editValue).getTime() : null"
        :status="hasError ? 'error' : undefined"
        type="date"
        size="small"
        @update:value="(val) => handleValueChange(val ? new Date(val).toISOString().split('T')[0] : null)"
        @keydown="handleKeyDown"
      />

      <!-- 时间戳类型 -->
      <NDatePicker
        v-else-if="isTimestampType"
        ref="inputRef"
        :value="editValue ? new Date(editValue).getTime() : null"
        :status="hasError ? 'error' : undefined"
        type="datetime"
        size="small"
        @update:value="(val) => handleValueChange(val ? new Date(val).toISOString() : null)"
        @keydown="handleKeyDown"
      />

      <!-- 时间类型 -->
      <NInput
        v-else-if="isTimeType"
        ref="inputRef"
        :value="editValue"
        :status="hasError ? 'error' : undefined"
        placeholder="HH:MM:SS"
        size="small"
        @update:value="handleValueChange"
        @keydown="handleKeyDown"
      />

      <!-- JSON 类型 -->
      <NInput
        v-else-if="isJSONType"
        ref="inputRef"
        :value="typeof editValue === 'object' ? JSON.stringify(editValue) : editValue"
        :status="hasError ? 'error' : undefined"
        type="textarea"
        :autosize="{ minRows: 2, maxRows: 6 }"
        size="small"
        @update:value="handleValueChange"
        @keydown="handleKeyDown"
      />

      <!-- 默认文本类型 -->
      <NInput
        v-else
        ref="inputRef"
        :value="editValue"
        :status="hasError ? 'error' : undefined"
        size="small"
        @update:value="handleValueChange"
        @keydown="handleKeyDown"
      />

      <!-- 验证错误提示 -->
      <div v-if="hasError" class="cell-editor-error">
        {{ errorMessage }}
      </div>
    </div>

    <!-- 显示模式 -->
    <NTooltip v-else :disabled="!hasError">
      <template #trigger>
        <div
          class="cell-editor-display"
          :class="{
            'cell-null': value === null || value === undefined,
            'cell-error': hasError,
          }"
        >
          {{ displayValue }}
        </div>
      </template>
      {{ errorMessage }}
    </NTooltip>
  </div>
</template>

<style scoped>
.cell-editor {
  width: 100%;
  height: 100%;
  position: relative;
}

.cell-editor-input {
  width: 100%;
  position: relative;
}

.cell-editor-display {
  padding: 4px 8px;
  cursor: pointer;
  user-select: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.cell-editor-display:hover {
  background-color: var(--n-color-hover);
}

.cell-null {
  color: var(--n-text-color-3);
  font-style: italic;
}

.cell-error {
  border: 1px solid var(--n-color-error);
  background-color: var(--n-color-error-hover);
}

.cell-editor-error {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  z-index: 1000;
  padding: 4px 8px;
  background-color: var(--n-color-error);
  color: white;
  font-size: 12px;
  border-radius: 0 0 4px 4px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}
</style>
