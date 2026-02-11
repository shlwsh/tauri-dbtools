/**
 * Data Grid Type Definitions
 * 
 * This file defines types for the data grid module including:
 * - Grid state and configuration
 * - Cell editing state
 * - Data modifications tracking
 * - Validation results
 * 
 * Validates: Requirements 9.1, 10.1, 11.1, 12.1
 */

import type { ColumnInfo } from './sql-editor';

/**
 * Complete state of the data grid
 */
export interface DataGridState {
  /** Database containing the table */
  database: string;
  /** Schema containing the table */
  schema: string;
  /** Table name */
  table: string;
  /** Column information */
  columns: ColumnInfo[];
  /** Current page of data */
  data: Record<string, any>[];
  /** Total number of rows in the table */
  totalRows: number;
  /** Current page number (0-based) */
  page: number;
  /** Number of rows per page */
  pageSize: number;
  /** Names of primary key columns */
  primaryKeys: string[];
  /** Whether the grid allows editing */
  editable: boolean;
}

/**
 * State of a cell being edited
 */
export interface CellEditState {
  /** Index of the row being edited */
  rowIndex: number;
  /** Name of the column being edited */
  columnName: string;
  /** Original value before editing */
  originalValue: any;
  /** Current value during editing */
  currentValue: any;
  /** Whether the current value is valid */
  isValid: boolean;
  /** Validation error message if invalid */
  validationError?: string;
}

/**
 * Tracks all modifications made in the data grid
 */
export interface DataModifications {
  /** Map of row index to row modifications */
  updated: Map<number, RowModification>;
  /** Array of newly inserted rows */
  inserted: RowData[];
  /** Set of row indexes marked for deletion */
  deleted: Set<number>;
}

/**
 * Modifications made to a single row
 */
export interface RowModification {
  /** Original data before modifications */
  originalData: Record<string, any>;
  /** Map of column name to new value */
  changes: Record<string, any>;
}

/**
 * Represents a row of data
 */
export type RowData = Record<string, any>;

/**
 * Result of validating a cell value
 */
export interface ValidationResult {
  /** Whether the value is valid */
  isValid: boolean;
  /** Error message if invalid */
  error?: string;
}

/**
 * Request to update multiple rows
 */
export interface BatchUpdateRequest {
  /** Database name */
  database: string;
  /** Schema name */
  schema: string;
  /** Table name */
  table: string;
  /** Array of row updates */
  updates: RowUpdate[];
}

/**
 * Update for a single row
 */
export interface RowUpdate {
  /** Primary key values identifying the row */
  primaryKey: Record<string, any>;
  /** Column values to update */
  changes: Record<string, any>;
}

/**
 * Request to insert multiple rows
 */
export interface BatchInsertRequest {
  /** Database name */
  database: string;
  /** Schema name */
  schema: string;
  /** Table name */
  table: string;
  /** Array of rows to insert */
  rows: RowData[];
}

/**
 * Request to delete multiple rows
 */
export interface BatchDeleteRequest {
  /** Database name */
  database: string;
  /** Schema name */
  schema: string;
  /** Table name */
  table: string;
  /** Array of primary key values identifying rows to delete */
  primaryKeys: Record<string, any>[];
}

/**
 * Options for cell editor based on column type
 */
export interface CellEditorOptions {
  /** Column data type */
  dataType: string;
  /** Whether the column allows NULL */
  nullable: boolean;
  /** Maximum length (for VARCHAR/CHAR) */
  maxLength?: number;
  /** Numeric precision */
  precision?: number;
  /** Numeric scale */
  scale?: number;
  /** Default value */
  defaultValue?: any;
}
