/**
 * Data Grid Store 单元测试
 * 
 * 测试 Data Grid Store 的核心功能：
 * - 数据加载和分页
 * - 修改跟踪（updated、inserted、deleted）
 * - 批量保存和回滚
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useDataGridStore } from '../data-grid';
import { invokeCommand } from '@/api/base';

// Mock API
vi.mock('@/api/base', () => ({
  invokeCommand: vi.fn(),
}));

describe('Data Grid Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  describe('初始状态', () => {
    it('应该有正确的初始状态', () => {
      const store = useDataGridStore();

      expect(store.currentTable).toBeNull();
      expect(store.columns).toEqual([]);
      expect(store.data).toEqual([]);
      expect(store.totalRows).toBe(0);
      expect(store.page).toBe(0);
      expect(store.pageSize).toBe(100);
      expect(store.primaryKeys).toEqual([]);
      expect(store.editable).toBe(false);
      expect(store.hasUnsavedChanges).toBe(false);
    });
  });

  describe('loadTableData', () => {
    it('应该成功加载表数据', async () => {
      const store = useDataGridStore();

      const mockQueryResult = {
        success: true,
        data: {
          columns: [
            { name: 'id', type_name: 'integer', nullable: false, is_primary_key: true },
            { name: 'name', type_name: 'varchar', nullable: true, is_primary_key: false },
          ],
          rows: [
            { id: 1, name: 'Alice' },
            { id: 2, name: 'Bob' },
          ],
        },
      };

      const mockCountResult = {
        success: true,
        data: {
          rows: [{ count: 2 }],
        },
      };

      const mockSchemaResult = {
        success: true,
        data: {
          columns: [
            { name: 'id', is_primary_key: true },
            { name: 'name', is_primary_key: false },
          ],
        },
      };

      (invokeCommand as any)
        .mockResolvedValueOnce(mockQueryResult)
        .mockResolvedValueOnce(mockCountResult)
        .mockResolvedValueOnce(mockSchemaResult);

      await store.loadTableData('test_db', 'public', 'users');

      expect(store.currentTable).toEqual({
        database: 'test_db',
        schema: 'public',
        table: 'users',
      });
      expect(store.columns).toHaveLength(2);
      expect(store.data).toHaveLength(2);
      expect(store.totalRows).toBe(2);
      expect(store.primaryKeys).toEqual(['id']);
      expect(store.editable).toBe(true);
    });

    it('应该处理加载失败', async () => {
      const store = useDataGridStore();

      (invokeCommand as any).mockResolvedValueOnce({
        success: false,
        message: '数据库连接失败',
      });

      await expect(
        store.loadTableData('test_db', 'public', 'users')
      ).rejects.toThrow('数据库连接失败');

      expect(store.error).toBe('数据库连接失败');
    });
  });

  describe('修改跟踪', () => {
    beforeEach(async () => {
      const store = useDataGridStore();

      // 设置初始数据
      const mockQueryResult = {
        success: true,
        data: {
          columns: [
            { name: 'id', type_name: 'integer', nullable: false, is_primary_key: true },
            { name: 'name', type_name: 'varchar', nullable: true, is_primary_key: false },
          ],
          rows: [
            { id: 1, name: 'Alice' },
            { id: 2, name: 'Bob' },
          ],
        },
      };

      const mockCountResult = {
        success: true,
        data: { rows: [{ count: 2 }] },
      };

      const mockSchemaResult = {
        success: true,
        data: {
          columns: [
            { name: 'id', is_primary_key: true },
            { name: 'name', is_primary_key: false },
          ],
        },
      };

      (invokeCommand as any)
        .mockResolvedValueOnce(mockQueryResult)
        .mockResolvedValueOnce(mockCountResult)
        .mockResolvedValueOnce(mockSchemaResult);

      await store.loadTableData('test_db', 'public', 'users');
    });

    it('应该跟踪单元格更新', () => {
      const store = useDataGridStore();

      store.updateCell(0, 'name', 'Alice Updated');

      expect(store.hasUnsavedChanges).toBe(true);
      expect(store.modifications.updated.size).toBe(1);
      expect(store.data[0].name).toBe('Alice Updated');

      const rowMod = store.getRowModification(0);
      expect(rowMod).not.toBeNull();
      expect(rowMod?.originalData.name).toBe('Alice');
      expect(rowMod?.changes.name).toBe('Alice Updated');
    });

    it('应该在值恢复到原始值时移除修改记录', () => {
      const store = useDataGridStore();

      // 修改值
      store.updateCell(0, 'name', 'Alice Updated');
      expect(store.hasUnsavedChanges).toBe(true);

      // 恢复到原始值
      store.updateCell(0, 'name', 'Alice');
      expect(store.hasUnsavedChanges).toBe(false);
      expect(store.modifications.updated.size).toBe(0);
    });

    it('应该跟踪新插入的行', () => {
      const store = useDataGridStore();

      store.addRow();

      expect(store.hasUnsavedChanges).toBe(true);
      expect(store.modifications.inserted).toHaveLength(1);
      expect(store.data).toHaveLength(3);
      expect(store.isRowInserted(2)).toBe(true);
    });

    it('应该允许编辑新插入的行', () => {
      const store = useDataGridStore();

      store.addRow();
      store.updateCell(2, 'name', 'Charlie');

      expect(store.modifications.inserted[0].name).toBe('Charlie');
      expect(store.data[2].name).toBe('Charlie');
    });

    it('应该跟踪删除的行', () => {
      const store = useDataGridStore();

      store.deleteRows([0]);

      expect(store.hasUnsavedChanges).toBe(true);
      expect(store.modifications.deleted.size).toBe(1);
      expect(store.isRowDeleted(0)).toBe(true);
    });

    it('应该直接移除新插入的行而不是标记为删除', () => {
      const store = useDataGridStore();

      store.addRow();
      expect(store.data).toHaveLength(3);

      store.deleteRows([2]);
      expect(store.data).toHaveLength(2);
      expect(store.modifications.inserted).toHaveLength(0);
      expect(store.modifications.deleted.size).toBe(0);
    });

    it('应该在删除行时移除其更新记录', () => {
      const store = useDataGridStore();

      store.updateCell(0, 'name', 'Alice Updated');
      expect(store.modifications.updated.size).toBe(1);

      store.deleteRows([0]);
      expect(store.modifications.updated.size).toBe(0);
      expect(store.modifications.deleted.size).toBe(1);
    });
  });

  describe('修改统计', () => {
    beforeEach(async () => {
      const store = useDataGridStore();

      const mockQueryResult = {
        success: true,
        data: {
          columns: [
            { name: 'id', type_name: 'integer', nullable: false, is_primary_key: true },
            { name: 'name', type_name: 'varchar', nullable: true, is_primary_key: false },
          ],
          rows: [
            { id: 1, name: 'Alice' },
            { id: 2, name: 'Bob' },
          ],
        },
      };

      const mockCountResult = {
        success: true,
        data: { rows: [{ count: 2 }] },
      };

      const mockSchemaResult = {
        success: true,
        data: {
          columns: [
            { name: 'id', is_primary_key: true },
            { name: 'name', is_primary_key: false },
          ],
        },
      };

      (invokeCommand as any)
        .mockResolvedValueOnce(mockQueryResult)
        .mockResolvedValueOnce(mockCountResult)
        .mockResolvedValueOnce(mockSchemaResult);

      await store.loadTableData('test_db', 'public', 'users');
    });

    it('应该正确统计修改数量', () => {
      const store = useDataGridStore();

      store.updateCell(0, 'name', 'Alice Updated');
      store.addRow();
      store.deleteRows([1]);

      const stats = store.modificationStats;
      expect(stats.updated).toBe(1);
      expect(stats.inserted).toBe(1);
      expect(stats.deleted).toBe(1);
      expect(stats.total).toBe(3);
    });
  });

  describe('saveChanges', () => {
    beforeEach(async () => {
      const store = useDataGridStore();

      const mockQueryResult = {
        success: true,
        data: {
          columns: [
            { name: 'id', type_name: 'integer', nullable: false, is_primary_key: true },
            { name: 'name', type_name: 'varchar', nullable: true, is_primary_key: false },
          ],
          rows: [
            { id: 1, name: 'Alice' },
            { id: 2, name: 'Bob' },
          ],
        },
      };

      const mockCountResult = {
        success: true,
        data: { rows: [{ count: 2 }] },
      };

      const mockSchemaResult = {
        success: true,
        data: {
          columns: [
            { name: 'id', is_primary_key: true },
            { name: 'name', is_primary_key: false },
          ],
        },
      };

      (invokeCommand as any)
        .mockResolvedValueOnce(mockQueryResult)
        .mockResolvedValueOnce(mockCountResult)
        .mockResolvedValueOnce(mockSchemaResult);

      await store.loadTableData('test_db', 'public', 'users');
    });

    it('应该成功保存更新', async () => {
      const store = useDataGridStore();

      store.updateCell(0, 'name', 'Alice Updated');

      // Mock 批量更新成功
      (invokeCommand as any).mockResolvedValueOnce({
        success: true,
      });

      // Mock 重新加载数据
      (invokeCommand as any)
        .mockResolvedValueOnce({
          success: true,
          data: {
            columns: store.columns,
            rows: [
              { id: 1, name: 'Alice Updated' },
              { id: 2, name: 'Bob' },
            ],
          },
        })
        .mockResolvedValueOnce({
          success: true,
          data: { rows: [{ count: 2 }] },
        })
        .mockResolvedValueOnce({
          success: true,
          data: {
            columns: [
              { name: 'id', is_primary_key: true },
              { name: 'name', is_primary_key: false },
            ],
          },
        });

      await store.saveChanges();

      expect(invokeCommand).toHaveBeenCalledWith('batch_update_rows', {
        database: 'test_db',
        schema: 'public',
        table: 'users',
        updates: [
          {
            primaryKey: { id: 1 },
            changes: { name: 'Alice Updated' },
          },
        ],
      });

      expect(store.hasUnsavedChanges).toBe(false);
    });

    it('应该成功保存插入', async () => {
      const store = useDataGridStore();

      store.addRow();
      store.updateCell(2, 'id', 3);
      store.updateCell(2, 'name', 'Charlie');

      // Mock 批量插入成功
      (invokeCommand as any).mockResolvedValueOnce({
        success: true,
      });

      // Mock 重新加载数据
      (invokeCommand as any)
        .mockResolvedValueOnce({
          success: true,
          data: {
            columns: store.columns,
            rows: [
              { id: 1, name: 'Alice' },
              { id: 2, name: 'Bob' },
              { id: 3, name: 'Charlie' },
            ],
          },
        })
        .mockResolvedValueOnce({
          success: true,
          data: { rows: [{ count: 3 }] },
        })
        .mockResolvedValueOnce({
          success: true,
          data: {
            columns: [
              { name: 'id', is_primary_key: true },
              { name: 'name', is_primary_key: false },
            ],
          },
        });

      await store.saveChanges();

      expect(invokeCommand).toHaveBeenCalledWith('batch_insert_rows', {
        database: 'test_db',
        schema: 'public',
        table: 'users',
        rows: [{ id: 3, name: 'Charlie' }],
      });
    });

    it('应该成功保存删除', async () => {
      const store = useDataGridStore();

      store.deleteRows([0]);

      // Mock 批量删除成功
      (invokeCommand as any).mockResolvedValueOnce({
        success: true,
      });

      // Mock 重新加载数据
      (invokeCommand as any)
        .mockResolvedValueOnce({
          success: true,
          data: {
            columns: store.columns,
            rows: [{ id: 2, name: 'Bob' }],
          },
        })
        .mockResolvedValueOnce({
          success: true,
          data: { rows: [{ count: 1 }] },
        })
        .mockResolvedValueOnce({
          success: true,
          data: {
            columns: [
              { name: 'id', is_primary_key: true },
              { name: 'name', is_primary_key: false },
            ],
          },
        });

      await store.saveChanges();

      expect(invokeCommand).toHaveBeenCalledWith('batch_delete_rows', {
        database: 'test_db',
        schema: 'public',
        table: 'users',
        primaryKeys: [{ id: 1 }],
      });
    });

    it('应该处理保存失败', async () => {
      const store = useDataGridStore();

      store.updateCell(0, 'name', 'Alice Updated');

      (invokeCommand as any).mockResolvedValueOnce({
        success: false,
        message: '更新失败',
      });

      await expect(store.saveChanges()).rejects.toThrow('更新失败');
      expect(store.error).toBe('更新失败');
    });
  });

  describe('discardChanges', () => {
    it('应该放弃所有修改并重新加载数据', async () => {
      const store = useDataGridStore();

      // 初始加载
      const mockQueryResult = {
        success: true,
        data: {
          columns: [
            { name: 'id', type_name: 'integer', nullable: false, is_primary_key: true },
            { name: 'name', type_name: 'varchar', nullable: true, is_primary_key: false },
          ],
          rows: [
            { id: 1, name: 'Alice' },
            { id: 2, name: 'Bob' },
          ],
        },
      };

      const mockCountResult = {
        success: true,
        data: { rows: [{ count: 2 }] },
      };

      const mockSchemaResult = {
        success: true,
        data: {
          columns: [
            { name: 'id', is_primary_key: true },
            { name: 'name', is_primary_key: false },
          ],
        },
      };

      (invokeCommand as any)
        .mockResolvedValueOnce(JSON.parse(JSON.stringify(mockQueryResult)))
        .mockResolvedValueOnce(JSON.parse(JSON.stringify(mockCountResult)))
        .mockResolvedValueOnce(JSON.parse(JSON.stringify(mockSchemaResult)));

      await store.loadTableData('test_db', 'public', 'users');

      // 进行修改
      store.updateCell(0, 'name', 'Alice Updated');
      store.addRow();
      store.deleteRows([1]);

      expect(store.hasUnsavedChanges).toBe(true);

      // Mock 重新加载 - 使用新的深拷贝数据
      vi.clearAllMocks();
      (invokeCommand as any)
        .mockResolvedValueOnce(JSON.parse(JSON.stringify(mockQueryResult)))
        .mockResolvedValueOnce(JSON.parse(JSON.stringify(mockCountResult)))
        .mockResolvedValueOnce(JSON.parse(JSON.stringify(mockSchemaResult)));

      // 放弃修改
      await store.discardChanges();

      expect(store.hasUnsavedChanges).toBe(false);
      expect(store.data[0].name).toBe('Alice');
      expect(store.data).toHaveLength(2);
    });
  });

  describe('分页', () => {
    it('应该正确设置页码', async () => {
      const store = useDataGridStore();

      // 初始加载
      const mockQueryResult = {
        success: true,
        data: {
          columns: [{ name: 'id', type_name: 'integer', nullable: false, is_primary_key: true }],
          rows: [{ id: 1 }],
        },
      };

      const mockCountResult = {
        success: true,
        data: { rows: [{ count: 200 }] },
      };

      const mockSchemaResult = {
        success: true,
        data: {
          columns: [{ name: 'id', is_primary_key: true }],
        },
      };

      (invokeCommand as any)
        .mockResolvedValueOnce(mockQueryResult)
        .mockResolvedValueOnce(mockCountResult)
        .mockResolvedValueOnce(mockSchemaResult);

      await store.loadTableData('test_db', 'public', 'users');

      expect(store.page).toBe(0);

      // Mock 第二页数据
      (invokeCommand as any)
        .mockResolvedValueOnce(mockQueryResult)
        .mockResolvedValueOnce(mockCountResult)
        .mockResolvedValueOnce(mockSchemaResult);

      await store.setPage(1);

      expect(store.page).toBe(1);
    });

    it('应该正确设置每页行数', async () => {
      const store = useDataGridStore();

      // 初始加载
      const mockQueryResult = {
        success: true,
        data: {
          columns: [{ name: 'id', type_name: 'integer', nullable: false, is_primary_key: true }],
          rows: [{ id: 1 }],
        },
      };

      const mockCountResult = {
        success: true,
        data: { rows: [{ count: 200 }] },
      };

      const mockSchemaResult = {
        success: true,
        data: {
          columns: [{ name: 'id', is_primary_key: true }],
        },
      };

      (invokeCommand as any)
        .mockResolvedValueOnce(mockQueryResult)
        .mockResolvedValueOnce(mockCountResult)
        .mockResolvedValueOnce(mockSchemaResult);

      await store.loadTableData('test_db', 'public', 'users');

      expect(store.pageSize).toBe(100);

      // Mock 新页面大小数据
      (invokeCommand as any)
        .mockResolvedValueOnce(mockQueryResult)
        .mockResolvedValueOnce(mockCountResult)
        .mockResolvedValueOnce(mockSchemaResult);

      await store.setPageSize(50);

      expect(store.pageSize).toBe(50);
      expect(store.page).toBe(0); // 应该重置到第一页
    });
  });

  describe('canEdit', () => {
    it('当表有主键时应该可以编辑', async () => {
      const store = useDataGridStore();

      const mockQueryResult = {
        success: true,
        data: {
          columns: [
            { name: 'id', type_name: 'integer', nullable: false, is_primary_key: true },
          ],
          rows: [{ id: 1 }],
        },
      };

      const mockCountResult = {
        success: true,
        data: { rows: [{ count: 1 }] },
      };

      const mockSchemaResult = {
        success: true,
        data: {
          columns: [{ name: 'id', is_primary_key: true }],
        },
      };

      (invokeCommand as any)
        .mockResolvedValueOnce(mockQueryResult)
        .mockResolvedValueOnce(mockCountResult)
        .mockResolvedValueOnce(mockSchemaResult);

      await store.loadTableData('test_db', 'public', 'users');

      expect(store.canEdit).toBe(true);
    });

    it('当表没有主键时不应该可以编辑', async () => {
      const store = useDataGridStore();

      const mockQueryResult = {
        success: true,
        data: {
          columns: [
            { name: 'name', type_name: 'varchar', nullable: true, is_primary_key: false },
          ],
          rows: [{ name: 'Alice' }],
        },
      };

      const mockCountResult = {
        success: true,
        data: { rows: [{ count: 1 }] },
      };

      const mockSchemaResult = {
        success: true,
        data: {
          columns: [{ name: 'name', is_primary_key: false }],
        },
      };

      (invokeCommand as any)
        .mockResolvedValueOnce(mockQueryResult)
        .mockResolvedValueOnce(mockCountResult)
        .mockResolvedValueOnce(mockSchemaResult);

      await store.loadTableData('test_db', 'public', 'users');

      expect(store.canEdit).toBe(false);
      expect(store.editable).toBe(false);
    });
  });
});

