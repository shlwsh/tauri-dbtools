/**
 * Mock implementation of Tauri Dialog Plugin for testing
 */

export const save = async (options?: any): Promise<string | null> => {
  // Return a mock file path for testing
  return '/mock/path/to/file.sql';
};

export const open = async (options?: any): Promise<string | string[] | null> => {
  return '/mock/path/to/file';
};

export const message = async (message: string, options?: any): Promise<void> => {
  // Mock message dialog
};

export const ask = async (message: string, options?: any): Promise<boolean> => {
  return true;
};

export const confirm = async (message: string, options?: any): Promise<boolean> => {
  return true;
};
