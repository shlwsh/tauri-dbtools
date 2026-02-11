/**
 * Table Designer Type Definitions
 * 
 * This file defines types for the table designer module including:
 * - Table design state and structure
 * - Column definitions and properties
 * - Constraint definitions (primary key, foreign key, unique, check)
 * - Index definitions
 * 
 * Validates: Requirements 5.1, 6.1, 7.1, 8.1
 */

/**
 * Complete state of a table design
 */
export interface TableDesignState {
  /** Name of the table */
  tableName: string;
  /** Schema the table belongs to */
  schema: string;
  /** List of column definitions */
  columns: ColumnDefinition[];
  /** List of table constraints */
  constraints: ConstraintDefinition[];
  /** List of indexes */
  indexes: IndexDefinition[];
  /** Whether the design has unsaved changes */
  isDirty: boolean;
}

/**
 * Definition of a table column
 */
export interface ColumnDefinition {
  /** Column name */
  name: string;
  /** PostgreSQL data type */
  type: PostgreSQLType;
  /** Length for VARCHAR/CHAR types */
  length?: number;
  /** Precision for NUMERIC/DECIMAL types */
  precision?: number;
  /** Scale for NUMERIC/DECIMAL types */
  scale?: number;
  /** Whether the column allows NULL values */
  nullable: boolean;
  /** Default value expression */
  defaultValue?: string;
  /** Whether this column is part of the primary key */
  isPrimaryKey: boolean;
  /** Whether this column has a unique constraint */
  isUnique: boolean;
  /** Column comment/description */
  comment?: string;
  /** Flag indicating this is a newly added column (for ALTER TABLE) */
  isNew?: boolean;
  /** Flag indicating this column has been modified (for ALTER TABLE) */
  isModified?: boolean;
  /** Flag indicating this column should be deleted (for ALTER TABLE) */
  isDeleted?: boolean;
}

/**
 * PostgreSQL data types supported by the table designer
 */
export type PostgreSQLType =
  // Integer types
  | 'INTEGER'
  | 'BIGINT'
  | 'SMALLINT'
  // Numeric types
  | 'DECIMAL'
  | 'NUMERIC'
  | 'REAL'
  | 'DOUBLE PRECISION'
  // Character types
  | 'VARCHAR'
  | 'CHAR'
  | 'TEXT'
  // Boolean type
  | 'BOOLEAN'
  // Date/Time types
  | 'DATE'
  | 'TIME'
  | 'TIMESTAMP'
  | 'TIMESTAMPTZ'
  // JSON types
  | 'JSON'
  | 'JSONB'
  // Other types
  | 'UUID'
  | 'BYTEA';

/**
 * Definition of a table constraint
 */
export interface ConstraintDefinition {
  /** Type of constraint */
  type: 'primary_key' | 'foreign_key' | 'unique' | 'check';
  /** Constraint name */
  name: string;
  /** Columns involved in the constraint */
  columns: string[];
  /** Referenced table (for foreign key) */
  referencedTable?: string;
  /** Referenced columns (for foreign key) */
  referencedColumns?: string[];
  /** ON DELETE action (for foreign key) */
  onDelete?: ReferentialAction;
  /** ON UPDATE action (for foreign key) */
  onUpdate?: ReferentialAction;
  /** Check expression (for check constraint) */
  checkExpression?: string;
  /** Flag indicating this is a newly added constraint (for ALTER TABLE) */
  isNew?: boolean;
  /** Flag indicating this constraint should be deleted (for ALTER TABLE) */
  isDeleted?: boolean;
}

/**
 * Referential actions for foreign key constraints
 */
export type ReferentialAction = 'CASCADE' | 'SET NULL' | 'RESTRICT' | 'NO ACTION';

/**
 * Definition of a table index
 */
export interface IndexDefinition {
  /** Index name */
  name: string;
  /** Columns included in the index */
  columns: string[];
  /** Index type */
  type: 'btree' | 'hash' | 'gist' | 'gin';
  /** Whether this is a unique index */
  unique: boolean;
  /** Flag indicating this is a newly added index (for ALTER TABLE) */
  isNew?: boolean;
  /** Flag indicating this index should be deleted (for ALTER TABLE) */
  isDeleted?: boolean;
}

/**
 * Options for opening the table designer
 */
export interface DesignerOptions {
  /** Database to design the table in */
  database: string;
  /** Schema to design the table in */
  schema?: string;
  /** Table name (for edit mode) */
  tableName?: string;
}

/**
 * Changes to be applied to an existing table
 */
export interface TableChanges {
  /** Columns to be added */
  addedColumns: ColumnDefinition[];
  /** Columns to be modified */
  modifiedColumns: ColumnModification[];
  /** Column names to be dropped */
  droppedColumns: string[];
  /** Constraints to be added */
  addedConstraints: ConstraintDefinition[];
  /** Constraint names to be dropped */
  droppedConstraints: string[];
  /** Indexes to be added */
  addedIndexes: IndexDefinition[];
  /** Index names to be dropped */
  droppedIndexes: string[];
}

/**
 * Modification to an existing column
 */
export interface ColumnModification {
  /** Original column name */
  oldName: string;
  /** New column definition */
  newDefinition: ColumnDefinition;
}
