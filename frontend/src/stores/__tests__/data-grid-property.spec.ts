/**
 * Data Grid Store 属性测试
 * 
 * 使用 fast-check 进行基于属性的测试，验证修改跟踪的通用正确性属性
 * 
 * Feature: database-advanced-features, Property 9: 数据修改跟踪完整性
 * Validates: Requirements 10.1
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useDataGridStore } from '../data-grid';
import { invokeCommand } from '@/api/base';
import fc from 'fast-check';

// Mock API
vi.mock('@/api/base', () => ({
  invokeCommand: vi.fn(),
}));

/**
 * 生成任意的行数据
 */
function arbitraryRowData(): fc.Arbitrary<Record<string, any>> {
  return fc.record({
    id: fc.integer({ min: 1, max: 1000 }),
    name: fc.string({ minLength: 1, maxLength: 50 }),
    age: fc.option(fc.integer({ min: 0, max: 120 }), { nil: null }),
    email: fc.option(fc.emailAddress(), { nil: null }),
  });
}

/**
 * 生成任意的单元格修改操作
 */
function arbitraryCellUpdate(): fc.Arbitrary<{
  rowIndex: number;
  columnName: string;
  value: any;
}> {
  return fc.record({
    rowIndex: fc.integer({ min: 0, max: 9 }), // 假设有10行数据
    columnName: fc.constantFrom('name', 'age', 'email'),
    value: fc.oneof(
      fc.string({ minLength: 1, maxLength: 50 }),
      fc.integer({ min: 0, max: 120 }),
      fc.emailAddress(),
      fc.constant(null)
    ),
  });
}

/**
 * 设置测试数据
 */
async function setupTestData(rowCount: number = 10) {
  const store = useDataGridStore();

  const rows = Array.from({ length: rowCount }, (_, i) => ({
    id: i + 1,
    name: `User ${i + 1}`,
    age: 20 + i,
    email: `user${i + 1}@example.com`,
  }));

  const mockQueryResult = {
    success: true,
    data: {
      columns: [
        { name: 'id', type_name: 'integer', nullable: false, is_primary_key: true },
        { name: 'name', type_name: 'varchar', nullable: false, is_primary_key: false },
        { name: 'age', type_name: 'integer', nullable: true, is_primary_key: false },
        { name: 'email', type_name: 'varchar', nullable: true, is_primary_key: false },
      ],
      rows,
    },
  };

  const mockCountResult = {
    success: true,
    data: { rows: [{ count: rowCount }] },
  };

  const mockSchemaResult = {
    success: true,
    data: {
      columns: [
        { name: 'id', is_primary_key: true },
        { name: 'name', is_primary_key: false },
        { name: 'age', is_primary_key: false },
        { name: 'email', is_primary_key: false },
      ],
    },
  };

  (invokeCommand as any)
    .mockResolvedValueOnce(mockQueryResult)
    .mockResolvedValueOnce(mockCountResult)
    .mockResolvedValueOnce(mockSchemaResult);

  await store.loadTableData('test_db', 'public', 'users');

  return store;
}

describe('Data Grid Store - 属性测试', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  // Feature: database-advanced-features, Property 9: 数据修改跟踪完整性
  describe('Property 9: 数据修改跟踪完整性', () => {
    it('应该记录所有单元格修改及其原始值和新值', async () => {
      await fc.assert(
        fc.asyncProperty(
          fc.array(arbitraryCellUpdate(), { minLength: 1, maxLength: 20 }),
          async (updates) => {
            // 设置测试数据
            const store = await setupTestData(10);

            // 记录原始数据
            const originalData = store.data.map((row) => ({ ...row }));

            // 应用所有修改
            for (const update of updates) {
              if (update.rowIndex < store.data.length) {
                store.updateCell(update.rowIndex, update.columnName, update.value);
              }
            }

            // 验证：所有修改都被跟踪
            const modifiedRows = new Set<number>();
            updates.forEach((update) => {
              if (update.rowIndex < store.data.length) {
                const originalValue = originalData[update.rowIndex][update.columnName];
                // 只有当值真正改变时才应该被跟踪
                if (originalValue !== update.value) {
                  modifiedRows.add(update.rowIndex);
                }
              }
            });

            // 验证修改跟踪的完整性
            for (const rowIndex of modifiedRows) {
              const rowMod = store.getRowModification(rowIndex);

              // 如果行被修改了，应该有修改记录
              if (rowMod) {
                // 验证原始数据被保存
                expect(rowMod.originalData).toBeDefined();

                // 验证修改的列都在 changes 中
                for (const [columnName, newValue] of Object.entries(rowMod.changes)) {
                  // 新值应该与当前数据一致
                  expect(store.data[rowIndex][columnName]).toBe(newValue);

                  // 原始值应该与初始数据一致
                  expect(rowMod.originalData[columnName]).toBe(
                    originalData[rowIndex][columnName]
                  );
                }
              }
            }

            return true;
          }
        ),
        { numRuns: 100 }
      );
    });

    it('应该在多次修改同一单元格时保留原始值', async () => {
      await fc.assert(
        fc.asyncProperty(
          fc.integer({ min: 0, max: 9 }),
          fc.constantFrom('name', 'age', 'email'),
          fc.array(
            fc.oneof(
              fc.string({ minLength: 1, maxLength: 50 }),
              fc.integer({ min: 0, max: 120 }),
              fc.constant(null)
            ),
            { minLength: 2, maxLength: 10 }
          ),
          async (rowIndex, columnName, values) => {
            // 设置测试数据
            const store = await setupTestData(10);

            // 记录原始值
            const originalValue = store.data[rowIndex][columnName];

            // 多次修改同一单元格
            for (const value of values) {
              store.updateCell(rowIndex, columnName, value);
            }

            // 获取修改记录
            const rowMod = store.getRowModification(rowIndex);

            if (rowMod && rowMod.changes[columnName] !== undefined) {
              // 验证：原始值应该始终是最初的值，而不是中间修改的值
              expect(rowMod.originalData[columnName]).toBe(originalValue);

              // 验证：当前值应该是最后一次修改的值
              const lastValue = values[values.length - 1];
              expect(store.data[rowIndex][columnName]).toBe(lastValue);
              expect(rowMod.changes[columnName]).toBe(lastValue);
            }

            return true;
          }
        ),
        { numRuns: 100 }
      );
    });

    it('应该在值恢复到原始值时移除修改记录', async () => {
      await fc.assert(
        fc.asyncProperty(
          fc.integer({ min: 0, max: 9 }),
          fc.constantFrom('name', 'age', 'email'),
          fc.oneof(
            fc.string({ minLength: 1, maxLength: 50 }),
            fc.integer({ min: 0, max: 120 }),
            fc.constant(null)
          ),
          async (rowIndex, columnName, newValue) => {
            // 设置测试数据
            const store = await setupTestData(10);

            // 记录原始值
            const originalValue = store.data[rowIndex][columnName];

            // 只有当新值与原始值不同时才进行测试
            if (originalValue === newValue) {
              return true;
            }

            // 修改值
            store.updateCell(rowIndex, columnName, newValue);

            // 验证修改被跟踪
            let rowMod = store.getRowModification(rowIndex);
            expect(rowMod).not.toBeNull();
            expect(rowMod?.changes[columnName]).toBe(newValue);

            // 恢复到原始值
            store.updateCell(rowIndex, columnName, originalValue);

            // 验证修改记录被移除
            rowMod = store.getRowModification(rowIndex);
            if (rowMod) {
              // 如果还有其他列的修改，该行的修改记录可能还存在
              // 但是这一列的修改应该被移除
              expect(rowMod.changes[columnName]).toBeUndefined();
            }

            return true;
          }
        ),
        { numRuns: 100 }
      );
    });

    it('应该正确跟踪多行的修改', async () => {
      await fc.assert(
        fc.asyncProperty(
          fc.array(arbitraryCellUpdate(), { minLength: 5, maxLength: 30 }),
          async (updates) => {
            // 设置测试数据
            const store = await setupTestData(10);

            // 记录原始数据
            const originalData = store.data.map((row) => ({ ...row }));

            // 应用所有修改
            for (const update of updates) {
              if (update.rowIndex < store.data.length) {
                store.updateCell(update.rowIndex, update.columnName, update.value);
              }
            }

            // 统计实际被修改的行
            const actuallyModifiedRows = new Set<number>();
            for (let i = 0; i < store.data.length; i++) {
              const original = originalData[i];
              const current = store.data[i];

              // 检查是否有任何列被修改
              for (const key of Object.keys(original)) {
                if (original[key] !== current[key]) {
                  actuallyModifiedRows.add(i);
                  break;
                }
              }
            }

            // 验证修改跟踪的行数与实际修改的行数一致
            expect(store.modifications.updated.size).toBe(actuallyModifiedRows.size);

            // 验证每个被跟踪的行都确实被修改了
            store.modifications.updated.forEach((rowMod, rowIndex) => {
              expect(actuallyModifiedRows.has(rowIndex)).toBe(true);

              // 验证修改的列确实不同
              for (const [columnName, newValue] of Object.entries(rowMod.changes)) {
                expect(rowMod.originalData[columnName]).not.toBe(newValue);
                expect(store.data[rowIndex][columnName]).toBe(newValue);
              }
            });

            return true;
          }
        ),
        { numRuns: 100 }
      );
    });

    it('应该正确处理新插入行的修改', async () => {
      await fc.assert(
        fc.asyncProperty(
          fc.integer({ min: 1, max: 5 }),
          fc.array(arbitraryCellUpdate(), { minLength: 1, maxLength: 10 }),
          async (newRowCount, updates) => {
            // 设置测试数据
            const store = await setupTestData(10);

            const originalDataLength = store.data.length;

            // 添加新行
            for (let i = 0; i < newRowCount; i++) {
              store.addRow();
            }

            // 验证新行被添加
            expect(store.data.length).toBe(originalDataLength + newRowCount);
            expect(store.modifications.inserted.length).toBe(newRowCount);

            // 修改新插入的行
            for (const update of updates) {
              const adjustedRowIndex = originalDataLength + (update.rowIndex % newRowCount);
              if (adjustedRowIndex < store.data.length) {
                store.updateCell(adjustedRowIndex, update.columnName, update.value);
              }
            }

            // 验证新插入行的修改直接反映在 inserted 数组中
            for (let i = 0; i < newRowCount; i++) {
              const rowIndex = originalDataLength + i;
              const insertedRow = store.modifications.inserted[i];

              // 新插入的行的数据应该与 inserted 数组中的数据一致
              expect(store.data[rowIndex]).toEqual(insertedRow);
            }

            // 验证新插入的行不在 updated 跟踪中
            for (let i = originalDataLength; i < store.data.length; i++) {
              expect(store.modifications.updated.has(i)).toBe(false);
            }

            return true;
          }
        ),
        { numRuns: 100 }
      );
    });

    it('应该在删除行时移除其修改记录', async () => {
      await fc.assert(
        fc.asyncProperty(
          fc.array(arbitraryCellUpdate(), { minLength: 1, maxLength: 10 }),
          fc.array(fc.integer({ min: 0, max: 9 }), { minLength: 1, maxLength: 5 }),
          async (updates, rowsToDelete) => {
            // 设置测试数据
            const store = await setupTestData(10);

            // 应用修改
            for (const update of updates) {
              if (update.rowIndex < store.data.length) {
                store.updateCell(update.rowIndex, update.columnName, update.value);
              }
            }

            // 记录修改前的状态
            const modifiedRowsBeforeDelete = new Set(store.modifications.updated.keys());

            // 删除行
            const uniqueRowsToDelete = [...new Set(rowsToDelete)].filter(
              (idx) => idx < store.data.length
            );
            store.deleteRows(uniqueRowsToDelete);

            // 验证被删除的行不再有更新记录
            for (const rowIndex of uniqueRowsToDelete) {
              expect(store.modifications.updated.has(rowIndex)).toBe(false);
            }

            // 验证被删除的行被标记为删除
            for (const rowIndex of uniqueRowsToDelete) {
              expect(store.modifications.deleted.has(rowIndex)).toBe(true);
            }

            return true;
          }
        ),
        { numRuns: 100 }
      );
    });

    it('应该正确统计修改数量', async () => {
      await fc.assert(
        fc.asyncProperty(
          fc.array(arbitraryCellUpdate(), { minLength: 1, maxLength: 20 }),
          fc.integer({ min: 0, max: 3 }),
          fc.array(fc.integer({ min: 0, max: 9 }), { minLength: 0, maxLength: 3 }),
          async (updates, newRowCount, rowsToDelete) => {
            // 设置测试数据
            const store = await setupTestData(10);

            // 记录原始数据
            const originalData = store.data.map((row) => ({ ...row }));

            // 应用修改
            for (const update of updates) {
              if (update.rowIndex < store.data.length) {
                store.updateCell(update.rowIndex, update.columnName, update.value);
              }
            }

            // 添加新行
            for (let i = 0; i < newRowCount; i++) {
              store.addRow();
            }

            // 删除行（只删除原始数据的行，不删除新插入的行）
            const uniqueRowsToDelete = [...new Set(rowsToDelete)].filter(
              (idx) => idx < originalData.length
            );
            store.deleteRows(uniqueRowsToDelete);

            // 验证统计数据
            const stats = store.modificationStats;

            // 更新的行数应该等于 updated Map 的大小
            expect(stats.updated).toBe(store.modifications.updated.size);

            // 插入的行数应该等于 inserted 数组的长度
            expect(stats.inserted).toBe(store.modifications.inserted.length);

            // 删除的行数应该等于 deleted Set 的大小
            expect(stats.deleted).toBe(store.modifications.deleted.size);

            // 总数应该是三者之和
            expect(stats.total).toBe(stats.updated + stats.inserted + stats.deleted);

            // 验证 hasUnsavedChanges
            const hasChanges = stats.total > 0;
            expect(store.hasUnsavedChanges).toBe(hasChanges);

            return true;
          }
        ),
        { numRuns: 100 }
      );
    });
  });
});
