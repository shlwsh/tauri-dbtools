/**
 * Export Service Property Tests
 * 
 * 属性测试：验证查询结果导出格式的正确性
 * 
 * Feature: database-advanced-features, Property 13: 查询结果导出格式正确性
 * Validates: Requirements 18.3, 18.4, 18.5
 */

import { describe, it, expect } from 'vitest';
import fc from 'fast-check';
import {
  exportToCSV,
  exportToJSON,
  exportQueryResult,
  type ExportOptions,
} from '../export-service';
import type { ColumnInfo } from '@/types/sql-editor';

/**
 * 生成测试用的列信息
 */
function arbitraryColumn(): fc.Arbitrary<ColumnInfo> {
  return fc.record({
    name: fc.stringMatching(/^[a-z][a-z0-9_]{0,9}$/),
    type_name: fc.constantFrom('integer', 'varchar', 'boolean', 'date'),
    nullable: fc.boolean(),
    is_primary_key: fc.boolean(),
  });
}

/**
 * 生成测试用的行数据
 */
function arbitraryRow(columns: ColumnInfo[]): fc.Arbitrary<Record<string, any>> {
  const record: Record<string, fc.Arbitrary<any>> = {};
  
  columns.forEach(col => {
    record[col.name] = fc.oneof(
      fc.integer(),
      fc.string({ maxLength: 20 }),
      fc.boolean(),
      fc.constant(null)
    );
  });
  
  return fc.record(record);
}

describe('Export Service - Property Tests', () => {
  // Feature: database-advanced-features, Property 13: 查询结果导出格式正确性
  it('Property 13.1: CSV 导出应该包含所有列', () => {
    fc.assert(
      fc.property(
        fc.array(arbitraryColumn(), { minLength: 1, maxLength: 5 })
          .map(cols => {
            // 确保列名唯一
            const uniqueCols = cols.filter((col, idx, arr) => 
              arr.findIndex(c => c.name === col.name) === idx
            );
            return uniqueCols.length > 0 ? uniqueCols : [cols[0]];
          }),
        (columns) => {
          const rows = [
            Object.fromEntries(columns.map(col => [col.name, 'test'])),
          ];

          const csv = exportToCSV(columns, rows, { format: 'csv' });
          const lines = csv.split('\n');
          const headers = lines[0].split(',');

          // 验证：所有列都应该出现在 CSV 标题中
          expect(headers).toHaveLength(columns.length);
          columns.forEach(col => {
            expect(headers).toContain(col.name);
          });

          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  // Feature: database-advanced-features, Property 13: 查询结果导出格式正确性
  it('Property 13.2: CSV 导出应该包含所有行', () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 1, max: 20 }),
        (rowCount) => {
          const columns: ColumnInfo[] = [
            { name: 'id', type_name: 'integer', nullable: false, is_primary_key: true },
          ];

          const rows = Array.from({ length: rowCount }, (_, i) => ({
            id: i,
          }));

          const csv = exportToCSV(columns, rows, { format: 'csv' });
          const lines = csv.split('\n').filter(line => line.trim());

          // 验证：行数应该匹配（标题 + 数据行）
          expect(lines).toHaveLength(rowCount + 1);

          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  // Feature: database-advanced-features, Property 13: 查询结果导出格式正确性
  it('Property 13.3: CSV 值包含逗号时应该用引号包裹', () => {
    fc.assert(
      fc.property(
        fc.string({ maxLength: 20 }).filter(s => s.includes(',')),
        (valueWithComma) => {
          const columns: ColumnInfo[] = [
            { name: 'text', type_name: 'varchar', nullable: true, is_primary_key: false },
          ];

          const rows = [{ text: valueWithComma }];

          const csv = exportToCSV(columns, rows, { format: 'csv' });
          const lines = csv.split('\n');
          const dataLine = lines[1];

          // 验证：包含逗号的值应该用引号包裹
          expect(dataLine).toMatch(/^".*"$/);

          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  // Feature: database-advanced-features, Property 13: 查询结果导出格式正确性
  it('Property 13.4: CSV 值包含引号时应该转义', () => {
    fc.assert(
      fc.property(
        fc.string({ maxLength: 20 }).filter(s => s.includes('"')),
        (valueWithQuote) => {
          const columns: ColumnInfo[] = [
            { name: 'text', type_name: 'varchar', nullable: true, is_primary_key: false },
          ];

          const rows = [{ text: valueWithQuote }];

          const csv = exportToCSV(columns, rows, { format: 'csv' });
          const lines = csv.split('\n');
          const dataLine = lines[1];

          // 验证：引号应该被转义为两个引号
          const expectedEscaped = valueWithQuote.replace(/"/g, '""');
          expect(dataLine).toContain(expectedEscaped);

          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  // Feature: database-advanced-features, Property 13: 查询结果导出格式正确性
  it('Property 13.5: JSON 导出应该是有效的 JSON', () => {
    fc.assert(
      fc.property(
        fc.array(arbitraryColumn(), { minLength: 1, maxLength: 5 })
          .map(cols => {
            const uniqueCols = cols.filter((col, idx, arr) => 
              arr.findIndex(c => c.name === col.name) === idx
            );
            return uniqueCols.length > 0 ? uniqueCols : [cols[0]];
          })
          .chain(columns => 
            fc.tuple(
              fc.constant(columns),
              fc.array(arbitraryRow(columns), { minLength: 1, maxLength: 10 })
            )
          ),
        ([columns, rows]) => {
          const json = exportToJSON(columns, rows, { format: 'json' });

          // 验证：应该是有效的 JSON
          expect(() => JSON.parse(json)).not.toThrow();

          const parsed = JSON.parse(json);
          expect(Array.isArray(parsed)).toBe(true);
          expect(parsed).toHaveLength(rows.length);

          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  // Feature: database-advanced-features, Property 13: 查询结果导出格式正确性
  it('Property 13.6: JSON 导出应该保留所有列', () => {
    fc.assert(
      fc.property(
        fc.array(arbitraryColumn(), { minLength: 1, maxLength: 5 })
          .map(cols => {
            const uniqueCols = cols.filter((col, idx, arr) => 
              arr.findIndex(c => c.name === col.name) === idx
            );
            return uniqueCols.length > 0 ? uniqueCols : [cols[0]];
          })
          .chain(columns => 
            fc.tuple(
              fc.constant(columns),
              fc.array(arbitraryRow(columns), { minLength: 1, maxLength: 5 })
            )
          ),
        ([columns, rows]) => {
          const json = exportToJSON(columns, rows, { format: 'json' });
          const parsed = JSON.parse(json);

          // 验证：每个对象都应该有所有列
          parsed.forEach((obj: any) => {
            columns.forEach(col => {
              expect(obj).toHaveProperty(col.name);
            });
          });

          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  // Feature: database-advanced-features, Property 13: 查询结果导出格式正确性
  it('Property 13.7: NULL 值在 CSV 中应该表示为空字符串', () => {
    fc.assert(
      fc.property(
        fc.constant(null),
        () => {
          const columns: ColumnInfo[] = [
            { name: 'nullable_col', type_name: 'varchar', nullable: true, is_primary_key: false },
          ];

          const rows = [{ nullable_col: null }];

          const csv = exportToCSV(columns, rows, { format: 'csv' });
          const lines = csv.split('\n');
          const dataLine = lines[1];

          // 验证：NULL 应该表示为空（不是字符串 "null"）
          expect(dataLine).toBe('');

          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  // Feature: database-advanced-features, Property 13: 查询结果导出格式正确性
  it('Property 13.8: NULL 值在 JSON 中应该表示为 null', () => {
    fc.assert(
      fc.property(
        fc.constant(null),
        () => {
          const columns: ColumnInfo[] = [
            { name: 'nullable_col', type_name: 'varchar', nullable: true, is_primary_key: false },
          ];

          const rows = [{ nullable_col: null }];

          const json = exportToJSON(columns, rows, { format: 'json' });
          const parsed = JSON.parse(json);

          // 验证：NULL 应该表示为 JSON null
          expect(parsed[0].nullable_col).toBeNull();

          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  // Feature: database-advanced-features, Property 13: 查询结果导出格式正确性
  it('Property 13.9: 布尔值在 CSV 中应该表示为 true/false', () => {
    fc.assert(
      fc.property(
        fc.boolean(),
        (boolValue) => {
          const columns: ColumnInfo[] = [
            { name: 'bool_col', type_name: 'boolean', nullable: false, is_primary_key: false },
          ];

          const rows = [{ bool_col: boolValue }];

          const csv = exportToCSV(columns, rows, { format: 'csv' });
          const lines = csv.split('\n');
          const dataLine = lines[1];

          // 验证：布尔值应该表示为 true 或 false
          expect(dataLine).toBe(boolValue ? 'true' : 'false');

          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  // Feature: database-advanced-features, Property 13: 查询结果导出格式正确性
  it('Property 13.10: exportQueryResult 应该返回正确的文件名和 MIME 类型', async () => {
    await fc.assert(
      fc.asyncProperty(
        fc.constantFrom('csv', 'json'),
        async (format) => {
          const columns: ColumnInfo[] = [
            { name: 'id', type_name: 'integer', nullable: false, is_primary_key: true },
          ];

          const rows = [{ id: 1 }];

          const result = await exportQueryResult(columns, rows, {
            format: format as 'csv' | 'json',
          });

          // 验证：文件名应该包含正确的扩展名
          if (format === 'csv') {
            expect(result.filename).toMatch(/\.csv$/);
            expect(result.mimeType).toBe('text/csv');
          } else if (format === 'json') {
            expect(result.filename).toMatch(/\.json$/);
            expect(result.mimeType).toBe('application/json');
          }

          return true;
        }
      ),
      { numRuns: 100 }
    );
  });
});
