/**
 * Data Grid Store
 * 
 * 管理数据网格的状态和操作，包括：
 * - 当前表状态管理
 * - 数据加载和分页
 * - 修改跟踪（updated、inserted、deleted）
 * - 批量保存和回滚
 * 
 * Validates: Requirements 9.1, 10.1
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type {
  DataGridState,
  DataModifications,
  RowModification,
  RowData,
  CellEditState,
  BatchUpdateRequest,
  BatchInsertRequest,
  BatchDeleteRequest,
  RowUpdate,
} from '@/types/data-grid';
import type { ColumnInfo } from '@/types/sql-editor';
import { invokeCommand } from '@/api/base';
import type { ApiResponse } from '@/types/common';

export const useDataGridStore = defineStore('data-grid', () => {
  // State
  const currentTable = ref<{
    database: string;
    schema: string;
    table: string;
  } | null>(null);

  const columns = ref<ColumnInfo[]>([]);
  const data = ref<RowData[]>([]);
  const totalRows = ref(0);
  const page = ref(0);
  const pageSize = ref(100);
  const primaryKeys = ref<string[]>([]);
  const editable = ref(false);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  // 修改跟踪
  const modifications = ref<DataModifications>({
    updated: new Map<number, RowModification>(),
    inserted: [],
    deleted: new Set<number>(),
  });

  // 当前正在编辑的单元格
  const editingCell = ref<CellEditState | null>(null);

  // Computed

  /**
   * 是否有未保存的修改
   */
  const hasUnsavedChanges = computed(() => {
    return (
      modifications.value.updated.size > 0 ||
      modifications.value.inserted.length > 0 ||
      modifications.value.deleted.size > 0
    );
  });

  /**
   * 获取修改的行数统计
   */
  const modificationStats = computed(() => {
    return {
      updated: modifications.value.updated.size,
      inserted: modifications.value.inserted.length,
      deleted: modifications.value.deleted.size,
      total:
        modifications.value.updated.size +
        modifications.value.inserted.length +
        modifications.value.deleted.size,
    };
  });

  /**
   * 检查某一行是否被修改
   */
  const isRowModified = computed(() => {
    return (rowIndex: number): boolean => {
      return (
        modifications.value.updated.has(rowIndex) ||
        modifications.value.deleted.has(rowIndex)
      );
    };
  });

  /**
   * 检查某一行是否被标记为删除
   */
  const isRowDeleted = computed(() => {
    return (rowIndex: number): boolean => {
      return modifications.value.deleted.has(rowIndex);
    };
  });

  /**
   * 检查某一行是否是新插入的
   */
  const isRowInserted = computed(() => {
    return (rowIndex: number): boolean => {
      // 新插入的行在 data 数组的末尾
      const insertedStartIndex = data.value.length - modifications.value.inserted.length;
      return rowIndex >= insertedStartIndex;
    };
  });

  /**
   * 获取当前表的完整名称
   */
  const fullTableName = computed(() => {
    if (!currentTable.value) return '';
    return `${currentTable.value.schema}.${currentTable.value.table}`;
  });

  /**
   * 是否可以编辑（有主键且表已加载）
   */
  const canEdit = computed(() => {
    return currentTable.value !== null && primaryKeys.value.length > 0;
  });

  // Actions

  /**
   * 加载表数据
   * @param database - 数据库名称
   * @param schema - 模式名称
   * @param table - 表名称
   */
  async function loadTableData(
    database: string,
    schema: string,
    table: string
  ): Promise<void> {
    isLoading.value = true;
    error.value = null;

    try {
      // 构建查询语句
      const sql = `SELECT * FROM ${schema}.${table} LIMIT ${pageSize.value} OFFSET ${
        page.value * pageSize.value
      }`;

      // 执行查询获取数据
      const dataResponse = await invokeCommand<any>('execute_sql', {
        database,
        sql,
      });

      if (!dataResponse.success || !dataResponse.data) {
        throw new Error(dataResponse.message || '加载数据失败');
      }

      const queryResult = dataResponse.data;

      // 获取总行数
      const countSql = `SELECT COUNT(*) as count FROM ${schema}.${table}`;
      const countResponse = await invokeCommand<any>('execute_sql', {
        database,
        sql: countSql,
      });

      if (!countResponse.success || !countResponse.data) {
        throw new Error(countResponse.message || '获取行数失败');
      }

      const countResult = countResponse.data;
      const count = countResult.rows?.[0]?.count || 0;

      // 获取表结构信息（包括主键）
      const schemaResponse = await invokeCommand<any>('get_table_schema', {
        database,
        schema,
        table,
      });

      if (!schemaResponse.success || !schemaResponse.data) {
        throw new Error(schemaResponse.message || '获取表结构失败');
      }

      const tableSchema = schemaResponse.data;

      // 更新状态
      currentTable.value = { database, schema, table };
      columns.value = queryResult.columns || [];
      data.value = queryResult.rows || [];
      totalRows.value = parseInt(count);

      // 提取主键列
      primaryKeys.value = tableSchema.columns
        .filter((col: any) => col.is_primary_key)
        .map((col: any) => col.name);

      // 设置是否可编辑
      editable.value = primaryKeys.value.length > 0;

      // 清除修改跟踪
      clearModifications();
    } catch (err) {
      error.value = err instanceof Error ? err.message : '加载表数据失败';
      throw err;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * 设置当前页
   * @param newPage - 新的页码（0-based）
   */
  async function setPage(newPage: number): Promise<void> {
    if (newPage < 0 || !currentTable.value) return;

    const maxPage = Math.ceil(totalRows.value / pageSize.value) - 1;
    if (newPage > maxPage) return;

    page.value = newPage;

    // 重新加载数据
    await loadTableData(
      currentTable.value.database,
      currentTable.value.schema,
      currentTable.value.table
    );
  }

  /**
   * 设置每页行数
   * @param newPageSize - 新的每页行数
   */
  async function setPageSize(newPageSize: number): Promise<void> {
    if (newPageSize <= 0 || !currentTable.value) return;

    pageSize.value = newPageSize;
    page.value = 0; // 重置到第一页

    // 重新加载数据
    await loadTableData(
      currentTable.value.database,
      currentTable.value.schema,
      currentTable.value.table
    );
  }

  /**
   * 更新单元格值
   * @param rowIndex - 行索引
   * @param columnName - 列名
   * @param value - 新值
   */
  function updateCell(rowIndex: number, columnName: string, value: any): void {
    if (!canEdit.value || rowIndex < 0 || rowIndex >= data.value.length) {
      return;
    }

    // 检查是否是新插入的行
    const insertedStartIndex = data.value.length - modifications.value.inserted.length;
    if (rowIndex >= insertedStartIndex) {
      // 更新插入行的数据
      const insertedIndex = rowIndex - insertedStartIndex;
      modifications.value.inserted[insertedIndex][columnName] = value;
      data.value[rowIndex][columnName] = value;
      return;
    }

    // 检查是否已被标记为删除
    if (modifications.value.deleted.has(rowIndex)) {
      return;
    }

    // 获取或创建行修改记录
    let rowMod = modifications.value.updated.get(rowIndex);
    if (!rowMod) {
      rowMod = {
        originalData: { ...data.value[rowIndex] },
        changes: {},
      };
      modifications.value.updated.set(rowIndex, rowMod);
    }

    // 记录修改
    rowMod.changes[columnName] = value;

    // 更新显示的数据
    data.value[rowIndex][columnName] = value;

    // 如果修改后的值与原始值相同，移除该列的修改记录
    if (rowMod.originalData[columnName] === value) {
      delete rowMod.changes[columnName];

      // 如果没有任何修改了，移除整个行修改记录
      if (Object.keys(rowMod.changes).length === 0) {
        modifications.value.updated.delete(rowIndex);
      }
    }
  }

  /**
   * 添加新行
   */
  function addRow(): void {
    if (!canEdit.value) return;

    // 创建新行，使用列的默认值
    const newRow: RowData = {};
    columns.value.forEach((col) => {
      newRow[col.name] = null; // 默认为 NULL
    });

    // 添加到插入列表
    modifications.value.inserted.push(newRow);

    // 添加到显示数据
    data.value.push(newRow);
  }

  /**
   * 删除行
   * @param rowIndexes - 要删除的行索引数组
   */
  function deleteRows(rowIndexes: number[]): void {
    if (!canEdit.value) return;

    const insertedStartIndex = data.value.length - modifications.value.inserted.length;

    rowIndexes.forEach((rowIndex) => {
      if (rowIndex < 0 || rowIndex >= data.value.length) return;

      // 检查是否是新插入的行
      if (rowIndex >= insertedStartIndex) {
        // 从插入列表中移除
        const insertedIndex = rowIndex - insertedStartIndex;
        modifications.value.inserted.splice(insertedIndex, 1);
        // 从显示数据中移除
        data.value.splice(rowIndex, 1);
      } else {
        // 标记为删除
        modifications.value.deleted.add(rowIndex);

        // 如果该行有更新记录，移除它
        modifications.value.updated.delete(rowIndex);
      }
    });
  }

  /**
   * 保存所有修改
   */
  async function saveChanges(): Promise<void> {
    if (!currentTable.value || !hasUnsavedChanges.value) {
      return;
    }

    isLoading.value = true;
    error.value = null;

    try {
      const { database, schema, table } = currentTable.value;

      // 1. 处理更新
      if (modifications.value.updated.size > 0) {
        const updates: RowUpdate[] = [];

        modifications.value.updated.forEach((rowMod, rowIndex) => {
          const row = data.value[rowIndex];
          const primaryKey: Record<string, any> = {};

          // 提取主键值
          primaryKeys.value.forEach((pkCol) => {
            primaryKey[pkCol] = rowMod.originalData[pkCol];
          });

          updates.push({
            primaryKey,
            changes: rowMod.changes,
          });
        });

        const updateResponse = await invokeCommand<void>('batch_update_rows', {
          database,
          schema,
          table,
          updates,
        });

        if (!updateResponse.success) {
          throw new Error(updateResponse.message || '批量更新失败');
        }
      }

      // 2. 处理插入
      if (modifications.value.inserted.length > 0) {
        const insertResponse = await invokeCommand<void>('batch_insert_rows', {
          database,
          schema,
          table,
          rows: modifications.value.inserted,
        });

        if (!insertResponse.success) {
          throw new Error(insertResponse.message || '批量插入失败');
        }
      }

      // 3. 处理删除
      if (modifications.value.deleted.size > 0) {
        const primaryKeysToDelete: Record<string, any>[] = [];

        modifications.value.deleted.forEach((rowIndex) => {
          const row = data.value[rowIndex];
          const primaryKey: Record<string, any> = {};

          primaryKeys.value.forEach((pkCol) => {
            primaryKey[pkCol] = row[pkCol];
          });

          primaryKeysToDelete.push(primaryKey);
        });

        const deleteResponse = await invokeCommand<void>('batch_delete_rows', {
          database,
          schema,
          table,
          primaryKeys: primaryKeysToDelete,
        });

        if (!deleteResponse.success) {
          throw new Error(deleteResponse.message || '批量删除失败');
        }
      }

      // 保存成功，重新加载数据
      await loadTableData(database, schema, table);
    } catch (err) {
      error.value = err instanceof Error ? err.message : '保存修改失败';
      throw err;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * 放弃所有修改
   */
  async function discardChanges(): Promise<void> {
    if (!currentTable.value) return;

    // 重新加载数据以恢复原始状态
    await loadTableData(
      currentTable.value.database,
      currentTable.value.schema,
      currentTable.value.table
    );
  }

  /**
   * 清除修改跟踪
   */
  function clearModifications(): void {
    modifications.value = {
      updated: new Map<number, RowModification>(),
      inserted: [],
      deleted: new Set<number>(),
    };
  }

  /**
   * 刷新当前表数据
   */
  async function refresh(): Promise<void> {
    if (!currentTable.value) return;

    await loadTableData(
      currentTable.value.database,
      currentTable.value.schema,
      currentTable.value.table
    );
  }

  /**
   * 设置正在编辑的单元格
   * @param cellState - 单元格编辑状态
   */
  function setEditingCell(cellState: CellEditState | null): void {
    editingCell.value = cellState;
  }

  /**
   * 获取行的修改信息
   * @param rowIndex - 行索引
   * @returns 行修改信息，如果没有修改则返回 null
   */
  function getRowModification(rowIndex: number): RowModification | null {
    return modifications.value.updated.get(rowIndex) || null;
  }

  /**
   * 关闭当前表
   */
  function closeTable(): void {
    currentTable.value = null;
    columns.value = [];
    data.value = [];
    totalRows.value = 0;
    page.value = 0;
    primaryKeys.value = [];
    editable.value = false;
    error.value = null;
    clearModifications();
  }

  /**
   * 获取数据网格的完整状态
   */
  function getState(): DataGridState | null {
    if (!currentTable.value) return null;

    return {
      database: currentTable.value.database,
      schema: currentTable.value.schema,
      table: currentTable.value.table,
      columns: columns.value,
      data: data.value,
      totalRows: totalRows.value,
      page: page.value,
      pageSize: pageSize.value,
      primaryKeys: primaryKeys.value,
      editable: editable.value,
    };
  }

  return {
    // State
    currentTable,
    columns,
    data,
    totalRows,
    page,
    pageSize,
    primaryKeys,
    editable,
    isLoading,
    error,
    modifications,
    editingCell,

    // Computed
    hasUnsavedChanges,
    modificationStats,
    isRowModified,
    isRowDeleted,
    isRowInserted,
    fullTableName,
    canEdit,

    // Actions
    loadTableData,
    setPage,
    setPageSize,
    updateCell,
    addRow,
    deleteRows,
    saveChanges,
    discardChanges,
    clearModifications,
    refresh,
    setEditingCell,
    getRowModification,
    closeTable,
    getState,
  };
});
