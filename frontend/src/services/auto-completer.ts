/**
 * SQL Auto-Completer Service
 * 
 * Provides auto-completion suggestions for SQL keywords and database objects.
 * 
 * Validates: Requirements 1.3, 1.4
 */

import type { AutoCompleteItem } from '@/types/sql-editor';
import { getDatabaseObjects } from '@/api/database';

/**
 * SQL Keywords for auto-completion
 */
const SQL_KEYWORDS = [
  // DML
  'SELECT', 'FROM', 'WHERE', 'INSERT', 'INTO', 'VALUES', 'UPDATE', 'SET',
  'DELETE', 'RETURNING',
  
  // DDL
  'CREATE', 'ALTER', 'DROP', 'TRUNCATE', 'RENAME',
  'TABLE', 'INDEX', 'VIEW', 'SEQUENCE', 'SCHEMA', 'DATABASE',
  'CONSTRAINT', 'TRIGGER', 'FUNCTION', 'PROCEDURE',
  
  // Constraints
  'PRIMARY', 'FOREIGN', 'KEY', 'REFERENCES', 'UNIQUE', 'CHECK',
  'NOT', 'NULL', 'DEFAULT',
  
  // Joins
  'JOIN', 'INNER', 'LEFT', 'RIGHT', 'FULL', 'OUTER', 'CROSS',
  'ON', 'USING',
  
  // Clauses
  'GROUP', 'BY', 'HAVING', 'ORDER', 'ASC', 'DESC',
  'LIMIT', 'OFFSET', 'FETCH', 'FIRST', 'NEXT', 'ROWS', 'ONLY',
  
  // Set operations
  'UNION', 'INTERSECT', 'EXCEPT', 'ALL', 'DISTINCT',
  
  // Logical operators
  'AND', 'OR', 'NOT', 'IN', 'BETWEEN', 'LIKE', 'ILIKE', 'IS',
  'EXISTS', 'ANY', 'SOME',
  
  // Case
  'CASE', 'WHEN', 'THEN', 'ELSE', 'END',
  
  // Transactions
  'BEGIN', 'COMMIT', 'ROLLBACK', 'SAVEPOINT', 'TRANSACTION',
  'START', 'WORK',
  
  // Other
  'AS', 'WITH', 'RECURSIVE', 'CAST', 'COALESCE', 'NULLIF',
  'CASCADE', 'RESTRICT', 'NO', 'ACTION', 'SET NULL', 'SET DEFAULT',
];

/**
 * SQL Built-in Functions
 */
const SQL_FUNCTIONS = [
  // Aggregate functions
  'COUNT', 'SUM', 'AVG', 'MIN', 'MAX', 'ARRAY_AGG', 'STRING_AGG',
  'JSON_AGG', 'JSONB_AGG', 'BOOL_AND', 'BOOL_OR',
  
  // String functions
  'CONCAT', 'CONCAT_WS', 'SUBSTRING', 'SUBSTR', 'UPPER', 'LOWER',
  'TRIM', 'LTRIM', 'RTRIM', 'LENGTH', 'CHAR_LENGTH', 'POSITION',
  'REPLACE', 'SPLIT_PART', 'REGEXP_REPLACE', 'REGEXP_MATCH',
  
  // Date/Time functions
  'NOW', 'CURRENT_DATE', 'CURRENT_TIME', 'CURRENT_TIMESTAMP',
  'EXTRACT', 'DATE_PART', 'DATE_TRUNC', 'AGE', 'TO_CHAR',
  'TO_DATE', 'TO_TIMESTAMP', 'INTERVAL',
  
  // Math functions
  'ABS', 'CEIL', 'CEILING', 'FLOOR', 'ROUND', 'TRUNC', 'MOD',
  'POWER', 'SQRT', 'EXP', 'LN', 'LOG', 'RANDOM',
  
  // Window functions
  'ROW_NUMBER', 'RANK', 'DENSE_RANK', 'PERCENT_RANK', 'CUME_DIST',
  'NTILE', 'LAG', 'LEAD', 'FIRST_VALUE', 'LAST_VALUE', 'NTH_VALUE',
  
  // JSON functions
  'JSON_BUILD_OBJECT', 'JSON_BUILD_ARRAY', 'JSON_OBJECT',
  'JSONB_BUILD_OBJECT', 'JSONB_BUILD_ARRAY', 'JSON_EXTRACT_PATH',
  
  // Type conversion
  'CAST', 'CONVERT', 'TO_NUMBER', 'TO_CHAR', 'TO_DATE',
  
  // Other
  'COALESCE', 'NULLIF', 'GREATEST', 'LEAST', 'GENERATE_SERIES',
];

/**
 * PostgreSQL Data Types
 */
const SQL_TYPES = [
  // Numeric types
  'INTEGER', 'INT', 'BIGINT', 'SMALLINT', 'SERIAL', 'BIGSERIAL', 'SMALLSERIAL',
  'DECIMAL', 'NUMERIC', 'REAL', 'DOUBLE PRECISION', 'MONEY',
  
  // Character types
  'VARCHAR', 'CHAR', 'CHARACTER', 'CHARACTER VARYING', 'TEXT',
  
  // Binary types
  'BYTEA',
  
  // Date/Time types
  'DATE', 'TIME', 'TIMESTAMP', 'TIMESTAMPTZ', 'TIMESTAMP WITH TIME ZONE',
  'TIMESTAMP WITHOUT TIME ZONE', 'INTERVAL',
  
  // Boolean
  'BOOLEAN', 'BOOL',
  
  // Geometric types
  'POINT', 'LINE', 'LSEG', 'BOX', 'PATH', 'POLYGON', 'CIRCLE',
  
  // Network types
  'INET', 'CIDR', 'MACADDR', 'MACADDR8',
  
  // JSON types
  'JSON', 'JSONB',
  
  // UUID
  'UUID',
  
  // Array
  'ARRAY',
  
  // Other
  'XML', 'HSTORE', 'ENUM',
];

/**
 * Auto-Completer class
 */
export class AutoCompleter {
  private databaseObjectsCache: Map<string, AutoCompleteItem[]> = new Map();
  private cacheExpiry: Map<string, number> = new Map();
  private readonly CACHE_TTL = 60000; // 1 minute

  /**
   * Get keyword suggestions matching the prefix
   * @param prefix - The prefix to match
   * @returns Array of matching keyword suggestions
   */
  getKeywordSuggestions(prefix: string): string[] {
    if (!prefix) {
      return [];
    }

    const lowerPrefix = prefix.toLowerCase();
    const keywords = [...SQL_KEYWORDS, ...SQL_FUNCTIONS, ...SQL_TYPES];
    
    return keywords.filter(keyword =>
      keyword.toLowerCase().startsWith(lowerPrefix)
    );
  }

  /**
   * Get auto-complete items for keywords
   * @param prefix - The prefix to match
   * @returns Array of auto-complete items
   */
  getKeywordCompletionItems(prefix: string): AutoCompleteItem[] {
    const keywords = this.getKeywordSuggestions(prefix);
    
    return keywords.map(keyword => {
      let kind: AutoCompleteItem['kind'] = 'keyword';
      let detail = 'SQL Keyword';
      
      if (SQL_FUNCTIONS.includes(keyword)) {
        kind = 'function';
        detail = 'SQL Function';
      } else if (SQL_TYPES.includes(keyword)) {
        kind = 'keyword';
        detail = 'Data Type';
      }
      
      return {
        label: keyword,
        kind,
        detail,
      };
    });
  }

  /**
   * Get database object suggestions (tables, columns)
   * @param database - The database name
   * @param prefix - The prefix to match
   * @param objectType - Type of object ('tables' or 'columns')
   * @returns Array of matching database objects
   */
  async getDatabaseObjectSuggestions(
    database: string,
    prefix: string,
    objectType: 'tables' | 'columns' = 'tables'
  ): Promise<string[]> {
    if (!database || !prefix) {
      return [];
    }

    try {
      // Check cache
      const cacheKey = `${database}:${objectType}`;
      const cached = this.databaseObjectsCache.get(cacheKey);
      const cacheTime = this.cacheExpiry.get(cacheKey);
      
      let objects: string[];
      
      if (cached && cacheTime && Date.now() - cacheTime < this.CACHE_TTL) {
        // Use cached data
        objects = cached.map(item => item.label);
      } else {
        // Fetch from backend
        const response = await getDatabaseObjects(database, objectType);
        
        if (!response.success || !response.data) {
          console.error('Failed to fetch database objects:', response.message);
          return [];
        }
        
        objects = response.data;
        
        // Update cache
        const items = objects.map(obj => ({
          label: obj,
          kind: objectType === 'tables' ? 'table' as const : 'column' as const,
          detail: objectType === 'tables' ? 'Table' : 'Column',
        }));
        
        this.databaseObjectsCache.set(cacheKey, items);
        this.cacheExpiry.set(cacheKey, Date.now());
      }
      
      // Filter by prefix
      const lowerPrefix = prefix.toLowerCase();
      return objects.filter(obj =>
        obj.toLowerCase().startsWith(lowerPrefix)
      );
    } catch (error) {
      console.error('Failed to fetch database objects:', error);
      return [];
    }
  }

  /**
   * Get auto-complete items for database objects
   * @param database - The database name
   * @param prefix - The prefix to match
   * @param objectType - Type of object ('tables' or 'columns')
   * @returns Array of auto-complete items
   */
  async getDatabaseObjectCompletionItems(
    database: string,
    prefix: string,
    objectType: 'tables' | 'columns' = 'tables'
  ): Promise<AutoCompleteItem[]> {
    const objects = await this.getDatabaseObjectSuggestions(database, prefix, objectType);
    
    return objects.map(obj => ({
      label: obj,
      kind: objectType === 'tables' ? 'table' : 'column',
      detail: objectType === 'tables' ? 'Table' : 'Column',
    }));
  }

  /**
   * Get all completion suggestions
   * @param database - The database name
   * @param prefix - The prefix to match
   * @returns Array of all auto-complete items
   */
  async getAllCompletionItems(
    database: string,
    prefix: string
  ): Promise<AutoCompleteItem[]> {
    const keywordItems = this.getKeywordCompletionItems(prefix);
    
    if (!database) {
      return keywordItems;
    }
    
    // Fetch database objects in parallel
    const [tableItems, columnItems] = await Promise.all([
      this.getDatabaseObjectCompletionItems(database, prefix, 'tables'),
      this.getDatabaseObjectCompletionItems(database, prefix, 'columns'),
    ]);
    
    // Combine and return (database objects first, then keywords)
    return [...tableItems, ...columnItems, ...keywordItems];
  }

  /**
   * Clear the cache for a specific database
   * @param database - The database name
   */
  clearCache(database?: string): void {
    if (database) {
      this.databaseObjectsCache.delete(`${database}:tables`);
      this.databaseObjectsCache.delete(`${database}:columns`);
      this.cacheExpiry.delete(`${database}:tables`);
      this.cacheExpiry.delete(`${database}:columns`);
    } else {
      this.databaseObjectsCache.clear();
      this.cacheExpiry.clear();
    }
  }
}

// Export singleton instance
export const autoCompleter = new AutoCompleter();

