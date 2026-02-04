/**
 * Database type definitions
 */

export interface DatabaseInfo {
  name: string;
  size?: string;
  tables?: number;
}

export interface TableInfo {
  name: string;
  schema: string;
  rowCount?: number;
}

export interface ColumnInfo {
  name: string;
  type: string;
  nullable: boolean;
  isPrimaryKey: boolean;
}

export interface TableData {
  columns: ColumnInfo[];
  rows: Record<string, any>[];
  totalRows: number;
  page: number;
  pageSize: number;
}
