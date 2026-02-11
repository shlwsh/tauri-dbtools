/**
 * Database Explorer API
 */

import type { ApiResponse } from '@/types/common';
import type { TableInfo, TableData } from '@/types/database';
import { invokeCommand } from './base';

/**
 * List all tables in a database
 */
export async function listTables(
  connectionId: string,
  database: string
): Promise<ApiResponse<TableInfo[]>> {
  // For now, use the default connection from backend
  // TODO: Update backend to support connection parameter
  return await invokeCommand<TableInfo[]>('list_tables', { database });
}

/**
 * Get table data with pagination
 */
export async function getTableData(
  connectionId: string,
  database: string,
  table: string,
  page: number,
  pageSize: number
): Promise<ApiResponse<TableData>> {
  // For now, use the default connection from backend
  // TODO: Update backend to support connection parameter
  return await invokeCommand<TableData>('get_table_data', {
    database,
    table,
    page,
    pageSize,
  });
}

/**
 * Create a new record in a table
 */
export async function createRecord(
  connectionId: string,
  database: string,
  table: string,
  data: Record<string, any>
): Promise<ApiResponse<void>> {
  // For now, use the default connection from backend
  // TODO: Update backend to support connection parameter
  return await invokeCommand<void>('create_record', {
    database,
    table,
    data,
  });
}

/**
 * Update an existing record in a table
 */
export async function updateRecord(
  connectionId: string,
  database: string,
  table: string,
  primaryKey: Record<string, any>,
  data: Record<string, any>
): Promise<ApiResponse<void>> {
  // For now, use the default connection from backend
  // TODO: Update backend to support connection parameter
  return await invokeCommand<void>('update_record', {
    database,
    table,
    primaryKey,
    data,
  });
}

/**
 * Delete a record from a table
 */
export async function deleteRecord(
  connectionId: string,
  database: string,
  table: string,
  primaryKey: Record<string, any>
): Promise<ApiResponse<void>> {
  // For now, use the default connection from backend
  // TODO: Update backend to support connection parameter
  return await invokeCommand<void>('delete_record', {
    database,
    table,
    primaryKey,
  });
}
