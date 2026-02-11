/**
 * Mock implementation of Tauri API for testing
 */

export const invoke = async (cmd: string, args?: any): Promise<any> => {
  // Return empty/default responses for testing
  return {};
};

export const convertFileSrc = (filePath: string): string => {
  return `asset://localhost/${filePath}`;
};
