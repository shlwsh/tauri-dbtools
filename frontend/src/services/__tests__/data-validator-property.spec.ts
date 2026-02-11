/**
 * Data Validator Service 属性测试
 * 
 * 使用 fast-check 进行基于属性的测试，验证数据类型验证的通用正确性属性
 * 
 * Feature: database-advanced-features, Property 11: 数据类型验证
 * Validates: Requirements 11.1
 */

import { describe, it, expect } from 'vitest';
import { validateDataType, validateCellValue } from '../data-validator';
import type { ColumnInfo } from '@/types/sql-editor';
import fc from 'fast-check';

describe('Data Validator Service - 属性测试', () => {
  // Feature: database-advanced-features, Property 11: 数据类型验证
  describe('Property 11: 数据类型验证', () => {
    it('应该接受所有有效的整数值', () => {
      fc.assert(
        fc.property(fc.integer(), (value) => {
          const result = validateDataType(value, 'integer');
          return result.isValid === true;
        }),
        { numRuns: 100 }
      );
    });

    it('应该拒绝所有非整数的数值', () => {
      fc.assert(
        fc.property(
          fc.double({ noNaN: true }).filter((n) => !Number.isInteger(n)),
          (value) => {
            const result = validateDataType(value, 'integer');
            return result.isValid === false;
          }
        ),
        { numRuns: 100 }
      );
    });

    it('应该接受所有有效的浮点数', () => {
      fc.assert(
        fc.property(
          fc.double({ noNaN: true, noDefaultInfinity: true }),
          (value) => {
            const result = validateDataType(value, 'real');
            return result.isValid === true;
          }
        ),
        { numRuns: 100 }
      );
    });

    it('应该拒绝无穷大的浮点数', () => {
      expect(validateDataType(Infinity, 'real').isValid).toBe(false);
      expect(validateDataType(-Infinity, 'real').isValid).toBe(false);
    });

    it('应该接受所有有效的布尔值表示', () => {
      const validBooleans = [
        true,
        false,
        'true',
        'false',
        't',
        'f',
        'yes',
        'no',
        'y',
        'n',
        '1',
        '0',
        1,
        0,
      ];

      fc.assert(
        fc.property(fc.constantFrom(...validBooleans), (value) => {
          const result = validateDataType(value, 'boolean');
          return result.isValid === true;
        }),
        { numRuns: 100 }
      );
    });

    it('应该拒绝无效的布尔值', () => {
      fc.assert(
        fc.property(
          fc
            .string()
            .filter(
              (s) =>
                !['true', 'false', 't', 'f', 'yes', 'no', 'y', 'n', '1', '0'].includes(
                  s.toLowerCase()
                )
            ),
          (value) => {
            const result = validateDataType(value, 'boolean');
            return result.isValid === false;
          }
        ),
        { numRuns: 100 }
      );
    });

    it('应该接受有效的日期格式 YYYY-MM-DD', () => {
      fc.assert(
        fc.property(
          fc.date({ min: new Date('1900-01-01'), max: new Date('2100-12-31') }),
          (date) => {
            const dateStr = date.toISOString().split('T')[0]; // YYYY-MM-DD
            const result = validateDataType(dateStr, 'date');
            return result.isValid === true;
          }
        ),
        { numRuns: 100 }
      );
    });

    it('应该拒绝无效的日期格式', () => {
      const invalidDates = [
        '2024/01/15', // 错误的分隔符
        '15-01-2024', // 错误的顺序
        '2024-13-01', // 无效的月份
        '2024-01-32', // 无效的日期
        'not-a-date',
      ];

      invalidDates.forEach((dateStr) => {
        const result = validateDataType(dateStr, 'date');
        expect(result.isValid).toBe(false);
      });
    });

    it('应该接受有效的时间格式', () => {
      fc.assert(
        fc.property(
          fc.integer({ min: 0, max: 23 }),
          fc.integer({ min: 0, max: 59 }),
          fc.integer({ min: 0, max: 59 }),
          (hour, minute, second) => {
            const timeStr = `${String(hour).padStart(2, '0')}:${String(minute).padStart(
              2,
              '0'
            )}:${String(second).padStart(2, '0')}`;
            const result = validateDataType(timeStr, 'time');
            return result.isValid === true;
          }
        ),
        { numRuns: 100 }
      );
    });

    it('应该接受有效的 JSON 字符串', () => {
      fc.assert(
        fc.property(fc.jsonValue(), (value) => {
          const jsonStr = JSON.stringify(value);
          const result = validateDataType(jsonStr, 'json');
          return result.isValid === true;
        }),
        { numRuns: 100 }
      );
    });

    it('应该接受有效的 JSON 对象', () => {
      fc.assert(
        fc.property(fc.object(), (value) => {
          const result = validateDataType(value, 'json');
          return result.isValid === true;
        }),
        { numRuns: 100 }
      );
    });

    it('应该拒绝无效的 JSON 字符串', () => {
      const invalidJSON = [
        '{invalid json}',
        '{key: value}', // 缺少引号
        "{'key': 'value'}", // 单引号
        '{',
        '}',
        'undefined',
      ];

      invalidJSON.forEach((jsonStr) => {
        const result = validateDataType(jsonStr, 'json');
        expect(result.isValid).toBe(false);
      });
    });

    it('应该接受有效的 UUID 格式', () => {
      // UUID v4 格式生成器
      const uuidArbitrary = fc
        .tuple(
          fc.hexaString({ minLength: 8, maxLength: 8 }),
          fc.hexaString({ minLength: 4, maxLength: 4 }),
          fc.hexaString({ minLength: 4, maxLength: 4 }),
          fc.hexaString({ minLength: 4, maxLength: 4 }),
          fc.hexaString({ minLength: 12, maxLength: 12 })
        )
        .map(([a, b, c, d, e]) => {
          // 确保符合 UUID v4 格式
          const cFixed = '4' + c.substring(1); // 版本位
          const dFixed = ['8', '9', 'a', 'b'][Math.floor(Math.random() * 4)] + d.substring(1); // 变体位
          return `${a}-${b}-${cFixed}-${dFixed}-${e}`.toLowerCase();
        });

      fc.assert(
        fc.property(uuidArbitrary, (uuid) => {
          const result = validateDataType(uuid, 'uuid');
          return result.isValid === true;
        }),
        { numRuns: 100 }
      );
    });

    it('应该拒绝无效的 UUID 格式', () => {
      fc.assert(
        fc.property(
          fc
            .string()
            .filter(
              (s) =>
                !/^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(
                  s
                )
            ),
          (value) => {
            const result = validateDataType(value, 'uuid');
            return result.isValid === false;
          }
        ),
        { numRuns: 100 }
      );
    });

    it('应该正确验证 SMALLINT 范围', () => {
      // 范围内的值应该通过
      fc.assert(
        fc.property(fc.integer({ min: -32768, max: 32767 }), (value) => {
          const column: ColumnInfo = {
            name: 'test',
            type_name: 'smallint',
            nullable: false,
            is_primary_key: false,
          };
          const result = validateCellValue(value, column);
          return result.isValid === true;
        }),
        { numRuns: 100 }
      );

      // 范围外的值应该失败
      const outOfRange = [-32769, 32768, -100000, 100000];
      outOfRange.forEach((value) => {
        const column: ColumnInfo = {
          name: 'test',
          type_name: 'smallint',
          nullable: false,
          is_primary_key: false,
        };
        const result = validateCellValue(value, column);
        expect(result.isValid).toBe(false);
      });
    });

    it('应该正确验证 VARCHAR 长度限制', () => {
      fc.assert(
        fc.property(
          fc.string({ minLength: 1, maxLength: 50 }), // 从 1 开始，避免空字符串
          (value) => {
            const column: ColumnInfo = {
              name: 'test',
              type_name: 'varchar(50)',
              nullable: false,
              is_primary_key: false,
            };
            const result = validateCellValue(value, column);
            return result.isValid === true;
          }
        ),
        { numRuns: 100 }
      );

      // 超长字符串应该失败
      fc.assert(
        fc.property(
          fc.string({ minLength: 51, maxLength: 100 }),
          (value) => {
            const column: ColumnInfo = {
              name: 'test',
              type_name: 'varchar(50)',
              nullable: false,
              is_primary_key: false,
            };
            const result = validateCellValue(value, column);
            return result.isValid === false;
          }
        ),
        { numRuns: 100 }
      );
    });

    it('应该正确验证 NUMERIC 精度和小数位数', () => {
      // 生成符合 numeric(10,2) 的数字
      fc.assert(
        fc.property(
          fc.integer({ min: -99999999, max: 99999999 }),
          fc.integer({ min: 0, max: 99 }),
          (intPart, decPart) => {
            const value = `${intPart}.${String(decPart).padStart(2, '0')}`;
            const column: ColumnInfo = {
              name: 'test',
              type_name: 'numeric(10,2)',
              nullable: false,
              is_primary_key: false,
            };
            const result = validateCellValue(value, column);
            return result.isValid === true;
          }
        ),
        { numRuns: 100 }
      );

      // 超过小数位数的应该失败
      const column: ColumnInfo = {
        name: 'test',
        type_name: 'numeric(10,2)',
        nullable: false,
        is_primary_key: false,
      };
      expect(validateCellValue('123.456', column).isValid).toBe(false);
      expect(validateCellValue('123.4567', column).isValid).toBe(false);
    });

    it('应该正确处理 NOT NULL 约束', () => {
      const column: ColumnInfo = {
        name: 'test',
        type_name: 'varchar(50)',
        nullable: false,
        is_primary_key: false,
      };

      // NULL 值应该失败
      expect(validateCellValue(null, column).isValid).toBe(false);
      expect(validateCellValue(undefined, column).isValid).toBe(false);
      expect(validateCellValue('', column).isValid).toBe(false);

      // 非 NULL 值应该通过
      fc.assert(
        fc.property(fc.string({ minLength: 1, maxLength: 50 }), (value) => {
          const result = validateCellValue(value, column);
          return result.isValid === true;
        }),
        { numRuns: 100 }
      );
    });

    it('应该正确处理可空列', () => {
      const column: ColumnInfo = {
        name: 'test',
        type_name: 'varchar(50)',
        nullable: true,
        is_primary_key: false,
      };

      // NULL 值应该通过
      expect(validateCellValue(null, column).isValid).toBe(true);
      expect(validateCellValue(undefined, column).isValid).toBe(true);
      expect(validateCellValue('', column).isValid).toBe(true);

      // 非 NULL 值也应该通过
      fc.assert(
        fc.property(fc.string({ minLength: 1, maxLength: 50 }), (value) => {
          const result = validateCellValue(value, column);
          return result.isValid === true;
        }),
        { numRuns: 100 }
      );
    });
  });
});
