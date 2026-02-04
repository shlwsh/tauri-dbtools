/**
 * Common type definitions
 */

export interface ApiResponse<T> {
  success: boolean;
  message: string;
  data?: T;
}
