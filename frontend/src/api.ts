import { invoke } from '@tauri-apps/api/core';

export interface ApiResponse<T> {
  success: boolean;
  message: string;
  data?: T;
}

export async function exportDatabase(database: string): Promise<ApiResponse<string>> {
  return await invoke('export_database', { database });
}

export async function importDatabase(filePath: string, database: string): Promise<ApiResponse<void>> {
  return await invoke('import_database', { 
    filePath: filePath,
    database: database 
  });
}

export async function listDatabases(): Promise<ApiResponse<string[]>> {
  return await invoke('list_databases');
}

export async function checkHealth(): Promise<ApiResponse<void>> {
  return await invoke('check_health');
}

export async function getExportDirPath(): Promise<string> {
  return await invoke('get_export_dir_path');
}

export async function getLogDirPath(): Promise<string> {
  return await invoke('get_log_dir_path');
}

export async function testPgTools(): Promise<ApiResponse<string>> {
  return await invoke('test_pg_tools');
}
