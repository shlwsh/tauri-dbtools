/**
 * Table Designer Store
 * 
 * Manages the state and actions for the table designer module.
 * Handles table design state, column/constraint/index management,
 * and DDL generation/execution.
 * 
 * Validates: Requirements 5.1, 5.2, 8.5, 8.6
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type {
  TableDesignState,
  ColumnDefinition,
  ConstraintDefinition,
  IndexDefinition,
  DesignerOptions,
  TableChanges,
  ColumnModification,
} from '@/types/table-designer';
import { invoke } from '@tauri-apps/api/tauri';

export const useTableDesignerStore = defineStore('table-designer', () => {
  // State
  const isOpen = ref(false);
  const mode = ref<'create' | 'edit'>('create');
  const currentDesign = ref<TableDesignState | null>(null);
  const originalDesign = ref<TableDesignState | null>(null);
  const currentDatabase = ref<string>('');
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  // Computed
  const isDirty = computed(() => {
    if (!currentDesign.value) return false;
    return currentDesign.value.isDirty;
  });

  const hasChanges = computed(() => {
    if (!currentDesign.value || !originalDesign.value) return false;
    return JSON.stringify(currentDesign.value) !== JSON.stringify(originalDesign.value);
  });

  // Actions

  /**
   * Open the table designer
   * @param designMode - 'create' for new table, 'edit' for existing table
   * @param options - Designer options including database, schema, and table name
   */
  async function openDesigner(
    designMode: 'create' | 'edit',
    options: DesignerOptions
  ): Promise<void> {
    mode.value = designMode;
    currentDatabase.value = options.database;
    isLoading.value = true;
    error.value = null;

    try {
      if (designMode === 'create') {
        // Initialize empty design for new table
        const newDesign: TableDesignState = {
          tableName: '',
          schema: options.schema || 'public',
          columns: [],
          constraints: [],
          indexes: [],
          isDirty: false,
        };
        currentDesign.value = newDesign;
        originalDesign.value = JSON.parse(JSON.stringify(newDesign));
      } else {
        // Load existing table structure
        if (!options.tableName) {
          throw new Error('Table name is required for edit mode');
        }

        const schema = await invoke<any>('get_table_schema', {
          database: options.database,
          schema: options.schema || 'public',
          table: options.tableName,
        });

        const design: TableDesignState = {
          tableName: schema.table_name,
          schema: schema.schema,
          columns: schema.columns.map((col: any) => ({
            name: col.name,
            type: col.data_type,
            length: col.character_maximum_length,
            precision: col.numeric_precision,
            scale: col.numeric_scale,
            nullable: col.is_nullable,
            defaultValue: col.column_default,
            isPrimaryKey: col.is_primary_key,
            isUnique: col.is_unique,
            comment: col.comment,
          })),
          constraints: schema.constraints.map((con: any) => ({
            type: con.constraint_type,
            name: con.constraint_name,
            columns: con.columns,
            referencedTable: con.referenced_table,
            referencedColumns: con.referenced_columns,
            onDelete: con.on_delete,
            onUpdate: con.on_update,
            checkExpression: con.check_clause,
          })),
          indexes: schema.indexes.map((idx: any) => ({
            name: idx.index_name,
            columns: idx.columns,
            type: idx.index_type,
            unique: idx.is_unique,
          })),
          isDirty: false,
        };

        currentDesign.value = design;
        originalDesign.value = JSON.parse(JSON.stringify(design));
      }

      isOpen.value = true;
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to open designer';
      throw err;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Close the table designer
   */
  function closeDesigner(): void {
    isOpen.value = false;
    currentDesign.value = null;
    originalDesign.value = null;
    currentDatabase.value = '';
    error.value = null;
  }

  /**
   * Update the table design
   * @param design - Partial design state to update
   */
  function updateDesign(design: Partial<TableDesignState>): void {
    if (!currentDesign.value) return;

    currentDesign.value = {
      ...currentDesign.value,
      ...design,
      isDirty: true,
    };
  }

  /**
   * Add a new column to the table design
   * @param column - Column definition to add
   */
  function addColumn(column: ColumnDefinition): void {
    if (!currentDesign.value) return;

    currentDesign.value.columns.push({
      ...column,
      isNew: mode.value === 'edit',
    });
    currentDesign.value.isDirty = true;
  }

  /**
   * Update an existing column
   * @param index - Index of the column to update
   * @param column - Updated column definition
   */
  function updateColumn(index: number, column: ColumnDefinition): void {
    if (!currentDesign.value || index < 0 || index >= currentDesign.value.columns.length) {
      return;
    }

    const existingColumn = currentDesign.value.columns[index];
    currentDesign.value.columns[index] = {
      ...column,
      isNew: existingColumn.isNew,
      isModified: mode.value === 'edit' && !existingColumn.isNew,
    };
    currentDesign.value.isDirty = true;
  }

  /**
   * Delete a column from the table design
   * @param index - Index of the column to delete
   */
  function deleteColumn(index: number): void {
    if (!currentDesign.value || index < 0 || index >= currentDesign.value.columns.length) {
      return;
    }

    if (mode.value === 'create' || currentDesign.value.columns[index].isNew) {
      // Remove the column completely if it's new
      currentDesign.value.columns.splice(index, 1);
    } else {
      // Mark for deletion if it's an existing column
      currentDesign.value.columns[index].isDeleted = true;
    }
    currentDesign.value.isDirty = true;
  }

  /**
   * Add a new constraint to the table design
   * @param constraint - Constraint definition to add
   */
  function addConstraint(constraint: ConstraintDefinition): void {
    if (!currentDesign.value) return;

    currentDesign.value.constraints.push({
      ...constraint,
      isNew: mode.value === 'edit',
    });
    currentDesign.value.isDirty = true;
  }

  /**
   * Delete a constraint from the table design
   * @param index - Index of the constraint to delete
   */
  function deleteConstraint(index: number): void {
    if (!currentDesign.value || index < 0 || index >= currentDesign.value.constraints.length) {
      return;
    }

    if (mode.value === 'create' || currentDesign.value.constraints[index].isNew) {
      // Remove the constraint completely if it's new
      currentDesign.value.constraints.splice(index, 1);
    } else {
      // Mark for deletion if it's an existing constraint
      currentDesign.value.constraints[index].isDeleted = true;
    }
    currentDesign.value.isDirty = true;
  }

  /**
   * Add a new index to the table design
   * @param index - Index definition to add
   */
  function addIndex(indexDef: IndexDefinition): void {
    if (!currentDesign.value) return;

    currentDesign.value.indexes.push({
      ...indexDef,
      isNew: mode.value === 'edit',
    });
    currentDesign.value.isDirty = true;
  }

  /**
   * Delete an index from the table design
   * @param index - Index of the index to delete
   */
  function deleteIndex(index: number): void {
    if (!currentDesign.value || index < 0 || index >= currentDesign.value.indexes.length) {
      return;
    }

    if (mode.value === 'create' || currentDesign.value.indexes[index].isNew) {
      // Remove the index completely if it's new
      currentDesign.value.indexes.splice(index, 1);
    } else {
      // Mark for deletion if it's an existing index
      currentDesign.value.indexes[index].isDeleted = true;
    }
    currentDesign.value.isDirty = true;
  }

  /**
   * Generate DDL statements for the current design
   * @returns Generated DDL SQL statements
   */
  async function generateDDL(): Promise<string> {
    if (!currentDesign.value) {
      throw new Error('No design to generate DDL from');
    }

    try {
      if (mode.value === 'create') {
        // Generate CREATE TABLE statement
        const ddl = await invoke<string>('generate_create_table_ddl', {
          design: {
            table_name: currentDesign.value.tableName,
            schema: currentDesign.value.schema,
            columns: currentDesign.value.columns,
            constraints: currentDesign.value.constraints,
            indexes: currentDesign.value.indexes,
          },
        });
        return ddl;
      } else {
        // Generate ALTER TABLE statements
        const changes = computeTableChanges();
        const ddl = await invoke<string>('generate_alter_table_ddl', {
          schema: currentDesign.value.schema,
          tableName: currentDesign.value.tableName,
          changes,
        });
        return ddl;
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to generate DDL';
      throw err;
    }
  }

  /**
   * Compute the changes between original and current design
   * @returns Table changes object
   */
  function computeTableChanges(): TableChanges {
    if (!currentDesign.value || !originalDesign.value) {
      throw new Error('Cannot compute changes without original design');
    }

    const changes: TableChanges = {
      addedColumns: [],
      modifiedColumns: [],
      droppedColumns: [],
      addedConstraints: [],
      droppedConstraints: [],
      addedIndexes: [],
      droppedIndexes: [],
    };

    // Compute column changes
    for (const column of currentDesign.value.columns) {
      if (column.isNew) {
        changes.addedColumns.push(column);
      } else if (column.isModified) {
        const originalColumn = originalDesign.value.columns.find(c => c.name === column.name);
        if (originalColumn) {
          changes.modifiedColumns.push({
            oldName: originalColumn.name,
            newDefinition: column,
          });
        }
      } else if (column.isDeleted) {
        changes.droppedColumns.push(column.name);
      }
    }

    // Find dropped columns (not in current design)
    for (const originalColumn of originalDesign.value.columns) {
      const exists = currentDesign.value.columns.some(c => c.name === originalColumn.name);
      if (!exists) {
        changes.droppedColumns.push(originalColumn.name);
      }
    }

    // Compute constraint changes
    for (const constraint of currentDesign.value.constraints) {
      if (constraint.isNew) {
        changes.addedConstraints.push(constraint);
      } else if (constraint.isDeleted) {
        changes.droppedConstraints.push(constraint.name);
      }
    }

    // Find dropped constraints
    for (const originalConstraint of originalDesign.value.constraints) {
      const exists = currentDesign.value.constraints.some(c => c.name === originalConstraint.name);
      if (!exists) {
        changes.droppedConstraints.push(originalConstraint.name);
      }
    }

    // Compute index changes
    for (const index of currentDesign.value.indexes) {
      if (index.isNew) {
        changes.addedIndexes.push(index);
      } else if (index.isDeleted) {
        changes.droppedIndexes.push(index.name);
      }
    }

    // Find dropped indexes
    for (const originalIndex of originalDesign.value.indexes) {
      const exists = currentDesign.value.indexes.some(i => i.name === originalIndex.name);
      if (!exists) {
        changes.droppedIndexes.push(originalIndex.name);
      }
    }

    return changes;
  }

  /**
   * Apply the current design to the database
   */
  async function applyChanges(): Promise<void> {
    if (!currentDesign.value) {
      throw new Error('No design to apply');
    }

    isLoading.value = true;
    error.value = null;

    try {
      if (mode.value === 'create') {
        // Create new table
        await invoke('create_table', {
          database: currentDatabase.value,
          design: {
            table_name: currentDesign.value.tableName,
            schema: currentDesign.value.schema,
            columns: currentDesign.value.columns,
            constraints: currentDesign.value.constraints,
            indexes: currentDesign.value.indexes,
          },
        });
      } else {
        // Alter existing table
        const changes = computeTableChanges();
        await invoke('alter_table', {
          database: currentDatabase.value,
          schema: currentDesign.value.schema,
          table: currentDesign.value.tableName,
          changes,
        });
      }

      // Success - close the designer
      closeDesigner();
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to apply changes';
      throw err;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Reset the design to its original state
   */
  function resetDesign(): void {
    if (!originalDesign.value) return;
    currentDesign.value = JSON.parse(JSON.stringify(originalDesign.value));
  }

  return {
    // State
    isOpen,
    mode,
    currentDesign,
    originalDesign,
    currentDatabase,
    isLoading,
    error,

    // Computed
    isDirty,
    hasChanges,

    // Actions
    openDesigner,
    closeDesigner,
    updateDesign,
    addColumn,
    updateColumn,
    deleteColumn,
    addConstraint,
    deleteConstraint,
    addIndex,
    deleteIndex,
    generateDDL,
    applyChanges,
    resetDesign,
  };
});
