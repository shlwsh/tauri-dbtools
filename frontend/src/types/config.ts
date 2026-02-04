/**
 * Configuration type definitions
 */

export interface DatabaseConnection {
  id: string;
  name: string;
  host: string;
  port: number;
  username: string;
  password: string;
  isDefault: boolean;
}

export interface AppConfig {
  connections: DatabaseConnection[];
  theme: 'light' | 'dark';
  defaultConnectionId?: string;
}
