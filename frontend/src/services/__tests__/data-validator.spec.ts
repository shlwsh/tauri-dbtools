/**
 * Data Validator Service 单元测试
 * 
 * 测试数据验证服务的核心功能：
 * - 数据类型验证
 * - NOT NULL 验证
 * - 长度限制验证
 * - 数值范围验证
 */

import { describe, it, expect } from 'vitest';
import {
  validateCellValue,
  validateDataType,
  validateMultipleCells,
  allValidationsPassed,
  getValidationErrors,
} from '../data-validator';
import type { ColumnInfo } from '@/types/sql-editor';

describe('Data Validator Service', () => {
  describe('validateDataType', () => {
    describe('整数类型', () => {
      it('应该接受有效的整数', () => {
        expect(validateDataType(42, 'integer').isValid).toBe(true);
        expect(validateDataType('42', 'integer').isValid).toBe(true);
        expect(validateDataType(-100, 'integer').isValid).toBe(true);
        expect(validateDataType(0, 'integer').isValid).toBe(true);
      });

      it('应该拒绝非整数值', () => {
        const result = validateDataType(42.5, 'integer');
        expect(result.isValid).toBe(false);
        expect(result.error).toContain('整数');
      });

      it('应该拒绝无效的数字字符串', () => {
        const result = validateDataType('abc', 'integer');
        expect(result.isValid).toBe(false);
        expect(result.error).toBeDefined();
      });
    });

    describe('浮点数类型', () => {
      it('应该接受有效的浮点数', () => {
        expect(validateDataType(42.5, 'real').isValid).toBe(true);
        expect(validateDataType('3.14', 'double precision').isValid).toBe(true);
        expect(validateDataType(-0.001, 'float').isValid).toBe(true);
      });

      it('应该拒绝无穷大', () => {
        const result = validateDataType(Infinity, 'real');
        expect(result.isValid).toBe(false);
        expect(result.error).toContain('无穷大');
      });
    });

    describe('布尔类型', () => {
      it('应该接受有效的布尔值', () => {
        expect(validateDataType(true, 'boolean').isValid).toBe(true);
        expect(validateDataType(false, 'boolean').isValid).toBe(true);
        expect(validateDataType('true', 'boolean').isValid).toBe(true);
        expect(validateDataType('false', 'boolean').isValid).toBe(true);
        expect(validateDataType('yes', 'boolean').isValid).toBe(true);
        expect(validateDataType('no', 'boolean').isValid).toBe(true);
        expect(validateDataType(1, 'boolean').isValid).toBe(true);
        expect(validateDataType(0, 'boolean').isValid).toBe(true);
      });

      it('应该拒绝无效的布尔值', () => {
        const result = validateDataType('invalid', 'boolean');
        expect(result.isValid).toBe(false);
        expect(result.error).toContain('布尔值');
      });
    });

    describe('日期类型', () => {
      it('应该接受有效的日期', () => {
        expect(validateDataType('2024-01-15', 'date').isValid).toBe(true);
        expect(validateDataType('2024-12-31', 'date').isValid).toBe(true);
      });

      it('应该拒绝无效的日期格式', () => {
        const result = validateDataType('15/01/2024', 'date');
        expect(result.isValid).toBe(false);
        expect(result.error).toContain('YYYY-MM-DD');
      });

      it('应该拒绝无效的日期', () => {
        const result = validateDataType('invalid-date', 'date');
        expect(result.isValid).toBe(false);
      });
    });

    describe('时间类型', () => {
      it('应该接受有效的时间', () => {
        expect(validateDataType('12:30:45', 'time').isValid).toBe(true);
        expect(validateDataType('00:00:00', 'time').isValid).toBe(true);
        expect(validateDataType('23:59:59', 'time').isValid).toBe(true);
        expect(validateDataType('12:30', 'time').isValid).toBe(true);
      });

      it('应该拒绝无效的时间格式', () => {
        const result = validateDataType('25:00:00', 'time');
        expect(result.isValid).toBe(false);
        expect(result.error).toContain('HH:MM:SS');
      });
    });

    describe('时间戳类型', () => {
      it('应该接受有效的时间戳', () => {
        expect(validateDataType('2024-01-15 12:30:45', 'timestamp').isValid).toBe(true);
        expect(validateDataType('2024-01-15T12:30:45Z', 'timestamptz').isValid).toBe(true);
        expect(validateDataType(new Date().toISOString(), 'timestamp').isValid).toBe(true);
      });

      it('应该拒绝无效的时间戳', () => {
        const result = validateDataType('invalid-timestamp', 'timestamp');
        expect(result.isValid).toBe(false);
      });
    });

    describe('JSON 类型', () => {
      it('应该接受有效的 JSON', () => {
        expect(validateDataType('{"key": "value"}', 'json').isValid).toBe(true);
        expect(validateDataType({ key: 'value' }, 'json').isValid).toBe(true);
        expect(validateDataType('[1, 2, 3]', 'jsonb').isValid).toBe(true);
      });

      it('应该拒绝无效的 JSON 字符串', () => {
        const result = validateDataType('{invalid json}', 'json');
        expect(result.isValid).toBe(false);
        expect(result.error).toContain('JSON');
      });
    });

    describe('UUID 类型', () => {
      it('应该接受有效的 UUID', () => {
        expect(
          validateDataType('550e8400-e29b-41d4-a716-446655440000', 'uuid').isValid
        ).toBe(true);
        expect(
          validateDataType('123e4567-e89b-12d3-a456-426614174000', 'uuid').isValid
        ).toBe(true);
      });

      it('应该拒绝无效的 UUID', () => {
        const result = validateDataType('invalid-uuid', 'uuid');
        expect(result.isValid).toBe(false);
        expect(result.error).toContain('UUID');
      });
    });
  });

  describe('validateCellValue', () => {
    describe('NOT NULL 验证', () => {
      it('应该拒绝 NOT NULL 列的空值', () => {
        const column: ColumnInfo = {
          name: 'username',
          type_name: 'varchar',
          nullable: false,
          is_primary_key: false,
        };

        expect(validateCellValue(null, column).isValid).toBe(false);
        expect(validateCellValue(undefined, column).isValid).toBe(false);
        expect(validateCellValue('', column).isValid).toBe(false);
      });

      it('应该接受可空列的空值', () => {
        const column: ColumnInfo = {
          name: 'description',
          type_name: 'text',
          nullable: true,
          is_primary_key: false,
        };

        expect(validateCellValue(null, column).isValid).toBe(true);
        expect(validateCellValue(undefined, column).isValid).toBe(true);
        expect(validateCellValue('', column).isValid).toBe(true);
      });
    });

    describe('长度限制验证', () => {
      it('应该拒绝超过长度限制的字符串', () => {
        const column: ColumnInfo = {
          name: 'username',
          type_name: 'character varying(10)',
          nullable: false,
          is_primary_key: false,
        };

        const result = validateCellValue('this_is_too_long', column);
        expect(result.isValid).toBe(false);
        expect(result.error).toContain('长度');
        expect(result.error).toContain('10');
      });

      it('应该接受符合长度限制的字符串', () => {
        const column: ColumnInfo = {
          name: 'username',
          type_name: 'varchar(50)',
          nullable: false,
          is_primary_key: false,
        };

        expect(validateCellValue('john_doe', column).isValid).toBe(true);
      });
    });

    describe('数值范围验证', () => {
      it('应该拒绝超出 SMALLINT 范围的值', () => {
        const column: ColumnInfo = {
          name: 'age',
          type_name: 'smallint',
          nullable: false,
          is_primary_key: false,
        };

        expect(validateCellValue(40000, column).isValid).toBe(false);
        expect(validateCellValue(-40000, column).isValid).toBe(false);
      });

      it('应该接受 SMALLINT 范围内的值', () => {
        const column: ColumnInfo = {
          name: 'age',
          type_name: 'smallint',
          nullable: false,
          is_primary_key: false,
        };

        expect(validateCellValue(100, column).isValid).toBe(true);
        expect(validateCellValue(-100, column).isValid).toBe(true);
      });

      it('应该验证 NUMERIC 的精度和小数位数', () => {
        const column: ColumnInfo = {
          name: 'price',
          type_name: 'numeric(10,2)',
          nullable: false,
          is_primary_key: false,
        };

        // 超过小数位数 - 使用字符串以避免浮点数精度问题
        const result1 = validateCellValue('123.456', column);
        expect(result1.isValid).toBe(false);
        expect(result1.error).toContain('小数位数');

        // 符合要求
        expect(validateCellValue('123.45', column).isValid).toBe(true);
        expect(validateCellValue(123.45, column).isValid).toBe(true);
      });
    });

    describe('组合验证', () => {
      it('应该同时验证类型和长度', () => {
        const column: ColumnInfo = {
          name: 'code',
          type_name: 'varchar(5)',
          nullable: false,
          is_primary_key: false,
        };

        // 空值
        expect(validateCellValue('', column).isValid).toBe(false);

        // 超长
        expect(validateCellValue('TOOLONG', column).isValid).toBe(false);

        // 正确
        expect(validateCellValue('ABC12', column).isValid).toBe(true);
      });
    });
  });

  describe('validateMultipleCells', () => {
    it('应该验证多个单元格', () => {
      const columns: ColumnInfo[] = [
        {
          name: 'id',
          type_name: 'integer',
          nullable: false,
          is_primary_key: true,
        },
        {
          name: 'username',
          type_name: 'varchar(50)',
          nullable: false,
          is_primary_key: false,
        },
        {
          name: 'age',
          type_name: 'integer',
          nullable: true,
          is_primary_key: false,
        },
      ];

      const values = {
        id: 1,
        username: 'john_doe',
        age: 25,
      };

      const results = validateMultipleCells(values, columns);

      expect(results.id.isValid).toBe(true);
      expect(results.username.isValid).toBe(true);
      expect(results.age.isValid).toBe(true);
    });

    it('应该检测多个验证错误', () => {
      const columns: ColumnInfo[] = [
        {
          name: 'id',
          type_name: 'integer',
          nullable: false,
          is_primary_key: true,
        },
        {
          name: 'username',
          type_name: 'varchar(5)',
          nullable: false,
          is_primary_key: false,
        },
      ];

      const values = {
        id: 'invalid',
        username: 'this_is_too_long',
      };

      const results = validateMultipleCells(values, columns);

      expect(results.id.isValid).toBe(false);
      expect(results.username.isValid).toBe(false);
    });
  });

  describe('allValidationsPassed', () => {
    it('应该在所有验证通过时返回 true', () => {
      const results = {
        field1: { isValid: true },
        field2: { isValid: true },
        field3: { isValid: true },
      };

      expect(allValidationsPassed(results)).toBe(true);
    });

    it('应该在有验证失败时返回 false', () => {
      const results = {
        field1: { isValid: true },
        field2: { isValid: false, error: 'Error' },
        field3: { isValid: true },
      };

      expect(allValidationsPassed(results)).toBe(false);
    });
  });

  describe('getValidationErrors', () => {
    it('应该返回所有错误消息', () => {
      const results = {
        field1: { isValid: true },
        field2: { isValid: false, error: 'Error 2' },
        field3: { isValid: false, error: 'Error 3' },
      };

      const errors = getValidationErrors(results);

      expect(errors).toHaveLength(2);
      expect(errors[0]).toContain('field2');
      expect(errors[0]).toContain('Error 2');
      expect(errors[1]).toContain('field3');
      expect(errors[1]).toContain('Error 3');
    });

    it('应该在没有错误时返回空数组', () => {
      const results = {
        field1: { isValid: true },
        field2: { isValid: true },
      };

      const errors = getValidationErrors(results);

      expect(errors).toHaveLength(0);
    });
  });
});
