/**
 * INSERT Statement Generator Property Tests
 * 
 * 属性测试：验证 INSERT 语句生成的正确性
 * 
 * Feature: database-advanced-features, Property 12: INSERT语句生成正确性
 * Validates: Requirements 12.4
 */

import { describe, it, expect } from 'vitest';
import fc from 'fast-check';

/**
 * 生成 INSERT 语句
 * @param schema - 模式名称
 * @param table - 表名称
 * @param rows - 要插入的行数据
 * @returns INSERT 语句
 */
function generateInsertStatement(
  schema: string,
  table: string,
  rows: Record<string, any>[]
): string {
  if (rows.length === 0) {
    throw new Error('至少需要一行数据');
  }

  // 获取列名（从第一行）
  const columns = Object.keys(rows[0]);
  
  if (columns.length === 0) {
    throw new Error('至少需要一列');
  }

  // 构建 INSERT 语句
  const columnList = columns.map(col => `"${col}"`).join(', ');
  
  // 构建 VALUES 子句
  const valuesList = rows.map(row => {
    const values = columns.map(col => {
      const value = row[col];
      
      // NULL 值
      if (value === null || value === undefined) {
        return 'NULL';
      }
      
      // 数字类型
      if (typeof value === 'number') {
        return String(value);
      }
      
      // 布尔类型
      if (typeof value === 'boolean') {
        return value ? 'TRUE' : 'FALSE';
      }
      
      // 字符串类型（需要转义单引号）
      if (typeof value === 'string') {
        return `'${value.replace(/'/g, "''")}'`;
      }
      
      // 对象类型（JSON）
      if (typeof value === 'object') {
        return `'${JSON.stringify(value).replace(/'/g, "''")}'`;
      }
      
      return 'NULL';
    });
    
    return `(${values.join(', ')})`;
  });

  return `INSERT INTO ${schema}.${table} (${columnList}) VALUES ${valuesList.join(', ')};`;
}

/**
 * 解析 INSERT 语句（简单解析器用于验证）
 */
function parseInsertStatement(sql: string): {
  schema: string;
  table: string;
  columns: string[];
  rowCount: number;
} | null {
  // 匹配 INSERT INTO schema.table (col1, col2) VALUES ...
  const match = sql.match(/INSERT INTO (\w+)\.(\w+) \(([^)]+)\) VALUES (.+);/);
  
  if (!match) {
    return null;
  }

  const schema = match[1];
  const table = match[2];
  const columnsStr = match[3];
  const valuesStr = match[4];

  // 解析列名
  const columns = columnsStr
    .split(',')
    .map(col => col.trim().replace(/"/g, ''));

  // 计算行数（通过统计 VALUES 中的括号对）
  const rowCount = (valuesStr.match(/\(/g) || []).length;

  return { schema, table, columns, rowCount };
}

describe('INSERT Statement Generator - Property Tests', () => {
  // Feature: database-advanced-features, Property 12: INSERT语句生成正确性
  it('Property 12.1: 生成的 INSERT 语句应该包含所有提供的列', () => {
    fc.assert(
      fc.property(
        fc.record({
          schema: fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
          table: fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
          columns: fc.array(
            fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
            { minLength: 1, maxLength: 10 }
          ).map(cols => [...new Set(cols)]), // 去重
          values: fc.array(
            fc.oneof(
              fc.integer(),
              fc.string({ maxLength: 20 }),
              fc.boolean(),
              fc.constant(null)
            ),
            { minLength: 1, maxLength: 10 }
          ),
        }),
        ({ schema, table, columns, values }) => {
          // 构建行数据
          const row: Record<string, any> = {};
          columns.forEach((col, idx) => {
            row[col] = values[idx % values.length];
          });

          // 生成 INSERT 语句
          const sql = generateInsertStatement(schema, table, [row]);

          // 解析语句
          const parsed = parseInsertStatement(sql);

          // 验证：所有列都应该出现在语句中
          expect(parsed).not.toBeNull();
          expect(parsed!.columns).toHaveLength(columns.length);
          
          columns.forEach(col => {
            expect(parsed!.columns).toContain(col);
          });

          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  // Feature: database-advanced-features, Property 12: INSERT语句生成正确性
  it('Property 12.2: 生成的 INSERT 语句应该包含正确的行数', () => {
    fc.assert(
      fc.property(
        fc.record({
          schema: fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
          table: fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
          rowCount: fc.integer({ min: 1, max: 20 }),
        }),
        ({ schema, table, rowCount }) => {
          // 生成多行数据
          const rows: Record<string, any>[] = [];
          for (let i = 0; i < rowCount; i++) {
            rows.push({
              id: i,
              name: `row_${i}`,
            });
          }

          // 生成 INSERT 语句
          const sql = generateInsertStatement(schema, table, rows);

          // 解析语句
          const parsed = parseInsertStatement(sql);

          // 验证：行数应该匹配
          expect(parsed).not.toBeNull();
          expect(parsed!.rowCount).toBe(rowCount);

          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  // Feature: database-advanced-features, Property 12: INSERT语句生成正确性
  it('Property 12.3: NULL 值应该正确表示为 NULL', () => {
    fc.assert(
      fc.property(
        fc.record({
          schema: fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
          table: fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
          nullableColumns: fc.array(
            fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
            { minLength: 1, maxLength: 5 }
          ).map(cols => [...new Set(cols)]),
        }),
        ({ schema, table, nullableColumns }) => {
          // 构建包含 NULL 值的行
          const row: Record<string, any> = {};
          nullableColumns.forEach(col => {
            row[col] = null;
          });

          // 生成 INSERT 语句
          const sql = generateInsertStatement(schema, table, [row]);

          // 验证：NULL 值应该表示为 NULL（不是字符串 'null'）
          const nullCount = (sql.match(/NULL/g) || []).length;
          expect(nullCount).toBe(nullableColumns.length);

          // 不应该包含字符串 'null'
          expect(sql).not.toContain("'null'");

          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  // Feature: database-advanced-features, Property 12: INSERT语句生成正确性
  it('Property 12.4: 字符串值应该正确转义单引号', () => {
    fc.assert(
      fc.property(
        fc.record({
          schema: fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
          table: fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
          stringWithQuotes: fc.string({ maxLength: 20 }).filter(s => s.includes("'")),
        }),
        ({ schema, table, stringWithQuotes }) => {
          // 构建包含单引号的字符串
          const row = {
            text_column: stringWithQuotes,
          };

          // 生成 INSERT 语句
          const sql = generateInsertStatement(schema, table, [row]);

          // 验证：单引号应该被转义为两个单引号
          const expectedEscaped = stringWithQuotes.replace(/'/g, "''");
          expect(sql).toContain(expectedEscaped);

          // 验证：语句应该是有效的（引号应该成对）
          const singleQuotes = sql.match(/'/g) || [];
          expect(singleQuotes.length % 2).toBe(0); // 引号数量应该是偶数

          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  // Feature: database-advanced-features, Property 12: INSERT语句生成正确性
  it('Property 12.5: 数字值应该不带引号', () => {
    fc.assert(
      fc.property(
        fc.record({
          schema: fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
          table: fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
          intValue: fc.integer(),
          floatValue: fc.float(),
        }),
        ({ schema, table, intValue, floatValue }) => {
          // 构建包含数字的行
          const row = {
            int_col: intValue,
            float_col: floatValue,
          };

          // 生成 INSERT 语句
          const sql = generateInsertStatement(schema, table, [row]);

          // 验证：数字应该不带引号
          // 提取 VALUES 部分
          const valuesMatch = sql.match(/VALUES \(([^)]+)\)/);
          expect(valuesMatch).not.toBeNull();

          const valuesStr = valuesMatch![1];
          const values = valuesStr.split(',').map(v => v.trim());

          // 第一个值（整数）不应该有引号
          expect(values[0]).toBe(String(intValue));
          expect(values[0]).not.toMatch(/^'/);

          // 第二个值（浮点数）不应该有引号
          expect(values[1]).toBe(String(floatValue));
          expect(values[1]).not.toMatch(/^'/);

          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  // Feature: database-advanced-features, Property 12: INSERT语句生成正确性
  it('Property 12.6: 布尔值应该表示为 TRUE/FALSE', () => {
    fc.assert(
      fc.property(
        fc.record({
          schema: fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
          table: fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
          boolValue: fc.boolean(),
        }),
        ({ schema, table, boolValue }) => {
          // 构建包含布尔值的行
          const row = {
            bool_col: boolValue,
          };

          // 生成 INSERT 语句
          const sql = generateInsertStatement(schema, table, [row]);

          // 验证：布尔值应该表示为 TRUE 或 FALSE
          if (boolValue) {
            expect(sql).toContain('TRUE');
          } else {
            expect(sql).toContain('FALSE');
          }

          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  // Feature: database-advanced-features, Property 12: INSERT语句生成正确性
  it('Property 12.7: 生成的语句应该以分号结尾', () => {
    fc.assert(
      fc.property(
        fc.record({
          schema: fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
          table: fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
        }),
        ({ schema, table }) => {
          const row = { id: 1 };

          // 生成 INSERT 语句
          const sql = generateInsertStatement(schema, table, [row]);

          // 验证：应该以分号结尾
          expect(sql).toMatch(/;$/);

          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  // Feature: database-advanced-features, Property 12: INSERT语句生成正确性
  it('Property 12.8: 空行数组应该抛出错误', () => {
    fc.assert(
      fc.property(
        fc.record({
          schema: fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
          table: fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
        }),
        ({ schema, table }) => {
          // 验证：空行数组应该抛出错误
          expect(() => {
            generateInsertStatement(schema, table, []);
          }).toThrow('至少需要一行数据');

          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  // Feature: database-advanced-features, Property 12: INSERT语句生成正确性
  it('Property 12.9: 多行插入应该使用单个 INSERT 语句', () => {
    fc.assert(
      fc.property(
        fc.record({
          schema: fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
          table: fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
          rowCount: fc.integer({ min: 2, max: 10 }),
        }),
        ({ schema, table, rowCount }) => {
          // 生成多行数据
          const rows: Record<string, any>[] = [];
          for (let i = 0; i < rowCount; i++) {
            rows.push({ id: i });
          }

          // 生成 INSERT 语句
          const sql = generateInsertStatement(schema, table, rows);

          // 验证：应该只有一个 INSERT INTO
          const insertCount = (sql.match(/INSERT INTO/g) || []).length;
          expect(insertCount).toBe(1);

          // 验证：应该有多个 VALUES 子句（用逗号分隔）
          const valuesMatch = sql.match(/VALUES (.+);/);
          expect(valuesMatch).not.toBeNull();

          const valuesStr = valuesMatch![1];
          const valueGroups = valuesStr.split(/\),\s*\(/);
          expect(valueGroups.length).toBe(rowCount);

          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  // Feature: database-advanced-features, Property 12: INSERT语句生成正确性
  it('Property 12.10: 所有行应该有相同的列', () => {
    fc.assert(
      fc.property(
        fc.record({
          schema: fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
          table: fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
          columns: fc.array(
            fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
            { minLength: 2, maxLength: 5 }
          ).map(cols => [...new Set(cols)]),
          rowCount: fc.integer({ min: 2, max: 5 }),
        }),
        ({ schema, table, columns, rowCount }) => {
          // 生成多行数据，每行都有相同的列
          const rows: Record<string, any>[] = [];
          for (let i = 0; i < rowCount; i++) {
            const row: Record<string, any> = {};
            columns.forEach((col, idx) => {
              row[col] = i * 10 + idx;
            });
            rows.push(row);
          }

          // 生成 INSERT 语句
          const sql = generateInsertStatement(schema, table, rows);

          // 解析语句
          const parsed = parseInsertStatement(sql);

          // 验证：列数应该匹配
          expect(parsed).not.toBeNull();
          expect(parsed!.columns).toHaveLength(columns.length);

          return true;
        }
      ),
      { numRuns: 100 }
    );
  });
});
