/**
 * Database API
 */

import type { ApiResponse } from '@/types/common';
import type { QueryResult } from '@/types/sql-editor';
import { invokeCommand } from './base';

/**
 * List all databases for a connection
 */
export async function listDatabases(connectionId: string): Promise<ApiResponse<string[]>> {
  // For now, use the default connection from backend
  // TODO: Update backend to support connection parameter
  return await invokeCommand<string[]>('list_databases');
}

/**
 * Export a database
 */
export async function exportDatabase(
  connectionId: string,
  database: string
): Promise<ApiResponse<string>> {
  // For now, use the default connection from backend
  // TODO: Update backend to support connection parameter
  return await invokeCommand<string>('export_database', { database });
}

/**
 * Import a database
 */
export async function importDatabase(
  connectionId: string,
  filePath: string,
  database: string
): Promise<ApiResponse<void>> {
  // For now, use the default connection from backend
  // TODO: Update backend to support connection parameter
  return await invokeCommand<void>('import_database', {
    filePath,
    database,
  });
}

/**
 * Execute SQL query
 */
export async function executeSql(
  database: string,
  sql: string
): Promise<ApiResponse<QueryResult>> {
  return await invokeCommand<QueryResult>('execute_sql', {
    database,
    sql,
  });
}

/**
 * Get database objects for auto-completion
 */
export async function getDatabaseObjects(
  database: string,
  objectType: 'tables' | 'columns'
): Promise<ApiResponse<string[]>> {
  return await invokeCommand<string[]>('get_database_objects', {
    database,
    objectType,
  });
}
