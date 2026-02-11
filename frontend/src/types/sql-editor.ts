/**
 * SQL Editor Type Definitions
 * 
 * This file defines types for the SQL editor module including:
 * - Editor tabs and their state
 * - Query results and execution metadata
 * - Query history items
 * - Auto-completion items
 * 
 * Validates: Requirements 1.1, 2.1, 4.1
 */

/**
 * Represents a single editor tab in the SQL editor
 */
export interface EditorTab {
  /** Unique identifier for the tab */
  id: string;
  /** Display label for the tab */
  label: string;
  /** SQL content in the editor */
  content: string;
  /** Database this tab is connected to */
  database?: string;
  /** Result of the last query execution */
  result?: QueryResult;
  /** Whether a query is currently executing */
  isExecuting: boolean;
  /** Whether the content has been modified since last save */
  isDirty: boolean;
}

/**
 * Result of a SQL query execution
 */
export interface QueryResult {
  /** Type of query result */
  type: 'select' | 'dml' | 'ddl' | 'error';
  /** Column information for SELECT queries */
  columns?: ColumnInfo[];
  /** Row data for SELECT queries */
  rows?: Record<string, any>[];
  /** Total number of rows returned (for SELECT) */
  rowCount?: number;
  /** Number of rows affected (for INSERT/UPDATE/DELETE) */
  affectedRows?: number;
  /** Query execution duration in milliseconds */
  duration: number;
  /** Error message if query failed */
  error?: string;
  /** Position of error in SQL (if available) */
  errorPosition?: ErrorPosition;
}

/**
 * Information about a database column
 */
export interface ColumnInfo {
  /** Column name */
  name: string;
  /** PostgreSQL type name */
  typeName: string;
  /** Whether the column allows NULL values */
  nullable: boolean;
  /** Whether this column is part of the primary key */
  isPrimaryKey: boolean;
}

/**
 * Position of an error in SQL text
 */
export interface ErrorPosition {
  /** Line number (1-based) */
  line: number;
  /** Column number (1-based) */
  column: number;
}

/**
 * A query stored in the query history
 */
export interface QueryHistoryItem {
  /** Unique identifier for the history item */
  id: string;
  /** The SQL query text */
  query: string;
  /** Database the query was executed against */
  database: string;
  /** Timestamp when the query was executed */
  executedAt: Date;
  /** Query execution duration in milliseconds */
  duration: number;
  /** Whether the query executed successfully */
  success: boolean;
  /** Error message if query failed */
  error?: string;
}

/**
 * An auto-completion suggestion item
 */
export interface AutoCompleteItem {
  /** Display label for the suggestion */
  label: string;
  /** Type of completion item */
  kind: 'keyword' | 'table' | 'column' | 'function';
  /** Additional detail text */
  detail?: string;
  /** Documentation/description for the item */
  documentation?: string;
}
