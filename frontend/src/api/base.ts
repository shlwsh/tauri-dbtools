/**
 * Base API layer for Tauri command invocation
 */

import { invoke } from '@tauri-apps/api/core';
import type { ApiResponse } from '@/types/common';

/**
 * Invoke a Tauri command with unified error handling
 */
export async function invokeCommand<T>(
  command: string,
  args?: Record<string, any>
): Promise<ApiResponse<T>> {
  try {
    const response = await invoke<ApiResponse<T>>(command, args);
    return response;
  } catch (error) {
    console.error(`API call failed: ${command}`, error);

    // Convert to unified error response
    return {
      success: false,
      message: error instanceof Error ? error.message : 'Unknown error',
      data: undefined,
    };
  }
}
