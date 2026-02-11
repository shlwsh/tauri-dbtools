/**
 * Table Designer Store Tests
 * 
 * Tests for the table designer store functionality including:
 * - Designer open/close state management
 * - Table design state management (columns, constraints, indexes)
 * - Dirty state tracking
 * - Column management (add, update, delete)
 * - Constraint management (add, delete)
 * - Index management (add, delete)
 * - Change computation
 * 
 * Validates: Requirements 5.1, 5.2, 8.5, 8.6
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useTableDesignerStore } from '../table-designer';
import type { ColumnDefinition, ConstraintDefinition, IndexDefinition } from '@/types/table-designer';

describe('Table Designer Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  describe('Designer State Management', () => {
    it('should initialize with closed state', () => {
      const store = useTableDesignerStore();
      
      expect(store.isOpen).toBe(false);
      expect(store.currentDesign).toBeNull();
      expect(store.originalDesign).toBeNull();
      expect(store.isDirty).toBe(false);
    });

    it('should open designer in create mode with empty design', async () => {
      const store = useTableDesignerStore();
      
      await store.openDesigner('create', {
        database: 'test_db',
        schema: 'public',
      });

      expect(store.isOpen).toBe(true);
      expect(store.mode).toBe('create');
      expect(store.currentDesign).not.toBeNull();
      expect(store.currentDesign?.tableName).toBe('');
      expect(store.currentDesign?.schema).toBe('public');
      expect(store.currentDesign?.columns).toEqual([]);
      expect(store.currentDesign?.constraints).toEqual([]);
      expect(store.currentDesign?.indexes).toEqual([]);
    });

    it('should close designer and reset state', async () => {
      const store = useTableDesignerStore();
      
      await store.openDesigner('create', {
        database: 'test_db',
        schema: 'public',
      });

      store.closeDesigner();

      expect(store.isOpen).toBe(false);
      expect(store.currentDesign).toBeNull();
      expect(store.originalDesign).toBeNull();
      expect(store.currentDatabase).toBe('');
      expect(store.error).toBeNull();
    });
  });

  describe('Dirty State Tracking', () => {
    it('should not be dirty initially', async () => {
      const store = useTableDesignerStore();
      
      await store.openDesigner('create', {
        database: 'test_db',
        schema: 'public',
      });

      expect(store.isDirty).toBe(false);
    });

    it('should be dirty after updating design', async () => {
      const store = useTableDesignerStore();
      
      await store.openDesigner('create', {
        database: 'test_db',
        schema: 'public',
      });

      store.updateDesign({ tableName: 'new_table' });

      expect(store.isDirty).toBe(true);
      expect(store.currentDesign?.isDirty).toBe(true);
    });

    it('should be dirty after adding a column', async () => {
      const store = useTableDesignerStore();
      
      await store.openDesigner('create', {
        database: 'test_db',
        schema: 'public',
      });

      const column: ColumnDefinition = {
        name: 'id',
        type: 'INTEGER',
        nullable: false,
        isPrimaryKey: true,
        isUnique: false,
      };

      store.addColumn(column);

      expect(store.isDirty).toBe(true);
    });
  });

  describe('Column Management', () => {
    it('should add a column to the design', async () => {
      const store = useTableDesignerStore();
      
      await store.openDesigner('create', {
        database: 'test_db',
        schema: 'public',
      });

      const column: ColumnDefinition = {
        name: 'id',
        type: 'INTEGER',
        nullable: false,
        isPrimaryKey: true,
        isUnique: false,
      };

      store.addColumn(column);

      expect(store.currentDesign?.columns).toHaveLength(1);
      expect(store.currentDesign?.columns[0]).toMatchObject(column);
    });

    it('should mark column as new in edit mode', async () => {
      const store = useTableDesignerStore();
      
      // Manually set up edit mode without backend call
      await store.openDesigner('create', {
        database: 'test_db',
        schema: 'public',
      });
      
      // Manually switch to edit mode
      store.mode = 'edit';

      const column: ColumnDefinition = {
        name: 'email',
        type: 'VARCHAR',
        length: 255,
        nullable: false,
        isPrimaryKey: false,
        isUnique: true,
      };

      store.addColumn(column);

      expect(store.currentDesign?.columns[0].isNew).toBe(true);
    });

    it('should update an existing column', async () => {
      const store = useTableDesignerStore();
      
      await store.openDesigner('create', {
        database: 'test_db',
        schema: 'public',
      });

      const column: ColumnDefinition = {
        name: 'id',
        type: 'INTEGER',
        nullable: false,
        isPrimaryKey: true,
        isUnique: false,
      };

      store.addColumn(column);

      const updatedColumn: ColumnDefinition = {
        ...column,
        type: 'BIGINT',
      };

      store.updateColumn(0, updatedColumn);

      expect(store.currentDesign?.columns[0].type).toBe('BIGINT');
    });

    it('should mark column as modified in edit mode', async () => {
      const store = useTableDesignerStore();
      
      // Set up create mode first
      await store.openDesigner('create', {
        database: 'test_db',
        schema: 'public',
      });

      // Add a column
      const column: ColumnDefinition = {
        name: 'id',
        type: 'INTEGER',
        nullable: false,
        isPrimaryKey: true,
        isUnique: false,
      };
      store.addColumn(column);

      // Switch to edit mode
      store.mode = 'edit';

      const updatedColumn: ColumnDefinition = {
        name: 'id',
        type: 'BIGINT',
        nullable: false,
        isPrimaryKey: true,
        isUnique: false,
      };

      store.updateColumn(0, updatedColumn);

      expect(store.currentDesign?.columns[0].isModified).toBe(true);
    });

    it('should delete a column in create mode', async () => {
      const store = useTableDesignerStore();
      
      await store.openDesigner('create', {
        database: 'test_db',
        schema: 'public',
      });

      const column: ColumnDefinition = {
        name: 'id',
        type: 'INTEGER',
        nullable: false,
        isPrimaryKey: true,
        isUnique: false,
      };

      store.addColumn(column);
      expect(store.currentDesign?.columns).toHaveLength(1);

      store.deleteColumn(0);
      expect(store.currentDesign?.columns).toHaveLength(0);
    });

    it('should mark column as deleted in edit mode', async () => {
      const store = useTableDesignerStore();
      
      // Set up create mode and add a column
      await store.openDesigner('create', {
        database: 'test_db',
        schema: 'public',
      });

      const column: ColumnDefinition = {
        name: 'id',
        type: 'INTEGER',
        nullable: false,
        isPrimaryKey: true,
        isUnique: false,
      };
      store.addColumn(column);

      // Switch to edit mode
      store.mode = 'edit';

      store.deleteColumn(0);

      expect(store.currentDesign?.columns).toHaveLength(1);
      expect(store.currentDesign?.columns[0].isDeleted).toBe(true);
    });

    it('should not update column with invalid index', async () => {
      const store = useTableDesignerStore();
      
      await store.openDesigner('create', {
        database: 'test_db',
        schema: 'public',
      });

      const column: ColumnDefinition = {
        name: 'id',
        type: 'INTEGER',
        nullable: false,
        isPrimaryKey: true,
        isUnique: false,
      };

      store.updateColumn(0, column);

      expect(store.currentDesign?.columns).toHaveLength(0);
    });
  });

  describe('Constraint Management', () => {
    it('should add a constraint to the design', async () => {
      const store = useTableDesignerStore();
      
      await store.openDesigner('create', {
        database: 'test_db',
        schema: 'public',
      });

      const constraint: ConstraintDefinition = {
        type: 'primary_key',
        name: 'pk_users',
        columns: ['id'],
      };

      store.addConstraint(constraint);

      expect(store.currentDesign?.constraints).toHaveLength(1);
      expect(store.currentDesign?.constraints[0]).toMatchObject(constraint);
    });

    it('should add a foreign key constraint', async () => {
      const store = useTableDesignerStore();
      
      await store.openDesigner('create', {
        database: 'test_db',
        schema: 'public',
      });

      const constraint: ConstraintDefinition = {
        type: 'foreign_key',
        name: 'fk_user_role',
        columns: ['role_id'],
        referencedTable: 'roles',
        referencedColumns: ['id'],
        onDelete: 'CASCADE',
        onUpdate: 'CASCADE',
      };

      store.addConstraint(constraint);

      expect(store.currentDesign?.constraints[0].type).toBe('foreign_key');
      expect(store.currentDesign?.constraints[0].referencedTable).toBe('roles');
      expect(store.currentDesign?.constraints[0].onDelete).toBe('CASCADE');
    });

    it('should delete a constraint in create mode', async () => {
      const store = useTableDesignerStore();
      
      await store.openDesigner('create', {
        database: 'test_db',
        schema: 'public',
      });

      const constraint: ConstraintDefinition = {
        type: 'unique',
        name: 'uk_email',
        columns: ['email'],
      };

      store.addConstraint(constraint);
      expect(store.currentDesign?.constraints).toHaveLength(1);

      store.deleteConstraint(0);
      expect(store.currentDesign?.constraints).toHaveLength(0);
    });

    it('should mark constraint as deleted in edit mode', async () => {
      const store = useTableDesignerStore();
      
      // Set up create mode and add a constraint
      await store.openDesigner('create', {
        database: 'test_db',
        schema: 'public',
      });

      const constraint: ConstraintDefinition = {
        type: 'unique',
        name: 'uk_email',
        columns: ['email'],
      };
      store.addConstraint(constraint);

      // Switch to edit mode
      store.mode = 'edit';

      store.deleteConstraint(0);

      expect(store.currentDesign?.constraints).toHaveLength(1);
      expect(store.currentDesign?.constraints[0].isDeleted).toBe(true);
    });
  });

  describe('Index Management', () => {
    it('should add an index to the design', async () => {
      const store = useTableDesignerStore();
      
      await store.openDesigner('create', {
        database: 'test_db',
        schema: 'public',
      });

      const index: IndexDefinition = {
        name: 'idx_email',
        columns: ['email'],
        type: 'btree',
        unique: false,
      };

      store.addIndex(index);

      expect(store.currentDesign?.indexes).toHaveLength(1);
      expect(store.currentDesign?.indexes[0]).toMatchObject(index);
    });

    it('should add a unique index', async () => {
      const store = useTableDesignerStore();
      
      await store.openDesigner('create', {
        database: 'test_db',
        schema: 'public',
      });

      const index: IndexDefinition = {
        name: 'idx_username',
        columns: ['username'],
        type: 'btree',
        unique: true,
      };

      store.addIndex(index);

      expect(store.currentDesign?.indexes[0].unique).toBe(true);
    });

    it('should delete an index in create mode', async () => {
      const store = useTableDesignerStore();
      
      await store.openDesigner('create', {
        database: 'test_db',
        schema: 'public',
      });

      const index: IndexDefinition = {
        name: 'idx_email',
        columns: ['email'],
        type: 'btree',
        unique: false,
      };

      store.addIndex(index);
      expect(store.currentDesign?.indexes).toHaveLength(1);

      store.deleteIndex(0);
      expect(store.currentDesign?.indexes).toHaveLength(0);
    });

    it('should mark index as deleted in edit mode', async () => {
      const store = useTableDesignerStore();
      
      // Set up create mode and add an index
      await store.openDesigner('create', {
        database: 'test_db',
        schema: 'public',
      });

      const index: IndexDefinition = {
        name: 'idx_email',
        columns: ['email'],
        type: 'btree',
        unique: false,
      };
      store.addIndex(index);

      // Switch to edit mode
      store.mode = 'edit';

      store.deleteIndex(0);

      expect(store.currentDesign?.indexes).toHaveLength(1);
      expect(store.currentDesign?.indexes[0].isDeleted).toBe(true);
    });
  });
});
