/**
 * Mock implementation of Tauri FS Plugin for testing
 */

export const writeTextFile = async (path: string, contents: string): Promise<void> => {
  // Mock file write operation
};

export const readTextFile = async (path: string): Promise<string> => {
  return 'mock file contents';
};

export const exists = async (path: string): Promise<boolean> => {
  return true;
};

export const createDir = async (path: string, options?: any): Promise<void> => {
  // Mock directory creation
};

export const readDir = async (path: string, options?: any): Promise<any[]> => {
  return [];
};

export const removeFile = async (path: string): Promise<void> => {
  // Mock file removal
};
