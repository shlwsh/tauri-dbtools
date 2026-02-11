/**
 * Data Validator Service
 * 
 * 提供数据验证功能，包括：
 * - 数据类型验证（数字、日期、布尔等）
 * - NOT NULL 验证
 * - 长度限制验证
 * - 数值范围验证
 * 
 * Validates: Requirements 11.1, 11.2, 11.3, 11.4, 11.5
 */

import type { ValidationResult, CellEditorOptions } from '@/types/data-grid';
import type { ColumnInfo } from '@/types/sql-editor';

/**
 * 验证单元格值
 * @param value - 要验证的值
 * @param column - 列信息
 * @returns 验证结果
 */
export function validateCellValue(
  value: any,
  column: ColumnInfo
): ValidationResult {
  // 1. NOT NULL 验证
  if (!column.nullable && (value === null || value === undefined || value === '')) {
    return {
      isValid: false,
      error: `${column.name} 不能为空`,
    };
  }

  // 如果值为 NULL 且列允许 NULL，则验证通过
  if (value === null || value === undefined || value === '') {
    return { isValid: true };
  }

  // 2. 数据类型验证
  const typeValidation = validateDataType(value, column.type_name);
  if (!typeValidation.isValid) {
    return typeValidation;
  }

  // 3. 长度限制验证（针对字符串类型）
  if (isStringType(column.type_name)) {
    const lengthValidation = validateLength(value, column);
    if (!lengthValidation.isValid) {
      return lengthValidation;
    }
  }

  // 4. 数值范围验证（针对数值类型）
  if (isNumericType(column.type_name)) {
    const rangeValidation = validateNumericRange(value, column);
    if (!rangeValidation.isValid) {
      return rangeValidation;
    }
  }

  return { isValid: true };
}

/**
 * 验证数据类型
 * @param value - 要验证的值
 * @param dataType - PostgreSQL 数据类型
 * @returns 验证结果
 */
export function validateDataType(
  value: any,
  dataType: string
): ValidationResult {
  const normalizedType = dataType.toLowerCase();

  // 整数类型
  if (
    normalizedType.includes('int') ||
    normalizedType === 'smallint' ||
    normalizedType === 'bigint'
  ) {
    return validateInteger(value);
  }

  // 浮点数类型
  if (
    normalizedType === 'real' ||
    normalizedType === 'double precision' ||
    normalizedType === 'float'
  ) {
    return validateFloat(value);
  }

  // 数值类型（NUMERIC, DECIMAL）
  if (normalizedType.includes('numeric') || normalizedType.includes('decimal')) {
    return validateNumeric(value);
  }

  // 布尔类型
  if (normalizedType === 'boolean' || normalizedType === 'bool') {
    return validateBoolean(value);
  }

  // 日期类型
  if (normalizedType === 'date') {
    return validateDate(value);
  }

  // 时间类型
  if (normalizedType === 'time' || normalizedType === 'time without time zone') {
    return validateTime(value);
  }

  // 时间戳类型
  if (
    normalizedType === 'timestamp' ||
    normalizedType === 'timestamp without time zone' ||
    normalizedType === 'timestamptz' ||
    normalizedType === 'timestamp with time zone'
  ) {
    return validateTimestamp(value);
  }

  // JSON 类型
  if (normalizedType === 'json' || normalizedType === 'jsonb') {
    return validateJSON(value);
  }

  // UUID 类型
  if (normalizedType === 'uuid') {
    return validateUUID(value);
  }

  // 字符串类型（VARCHAR, CHAR, TEXT 等）
  // 默认接受任何值，长度验证在单独的函数中进行
  return { isValid: true };
}

/**
 * 验证整数
 */
function validateInteger(value: any): ValidationResult {
  const num = Number(value);

  if (isNaN(num)) {
    return {
      isValid: false,
      error: '必须是有效的整数',
    };
  }

  if (!Number.isInteger(num)) {
    return {
      isValid: false,
      error: '必须是整数，不能包含小数部分',
    };
  }

  return { isValid: true };
}

/**
 * 验证浮点数
 */
function validateFloat(value: any): ValidationResult {
  const num = Number(value);

  if (isNaN(num)) {
    return {
      isValid: false,
      error: '必须是有效的数字',
    };
  }

  if (!isFinite(num)) {
    return {
      isValid: false,
      error: '数字不能是无穷大',
    };
  }

  return { isValid: true };
}

/**
 * 验证数值（NUMERIC/DECIMAL）
 */
function validateNumeric(value: any): ValidationResult {
  const num = Number(value);

  if (isNaN(num)) {
    return {
      isValid: false,
      error: '必须是有效的数字',
    };
  }

  if (!isFinite(num)) {
    return {
      isValid: false,
      error: '数字不能是无穷大',
    };
  }

  return { isValid: true };
}

/**
 * 验证布尔值
 */
function validateBoolean(value: any): ValidationResult {
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

  const normalizedValue =
    typeof value === 'string' ? value.toLowerCase() : value;

  if (!validBooleans.includes(normalizedValue)) {
    return {
      isValid: false,
      error: '必须是有效的布尔值 (true/false, yes/no, 1/0)',
    };
  }

  return { isValid: true };
}

/**
 * 验证日期
 */
function validateDate(value: any): ValidationResult {
  // 尝试解析日期
  const date = new Date(value);

  if (isNaN(date.getTime())) {
    return {
      isValid: false,
      error: '必须是有效的日期格式 (YYYY-MM-DD)',
    };
  }

  // 检查日期格式（如果是字符串）
  if (typeof value === 'string') {
    // 支持 YYYY-MM-DD 格式
    const datePattern = /^\d{4}-\d{2}-\d{2}$/;
    if (!datePattern.test(value)) {
      return {
        isValid: false,
        error: '日期格式必须是 YYYY-MM-DD',
      };
    }
  }

  return { isValid: true };
}

/**
 * 验证时间
 */
function validateTime(value: any): ValidationResult {
  if (typeof value !== 'string') {
    return {
      isValid: false,
      error: '时间必须是字符串格式',
    };
  }

  // 支持 HH:MM:SS 或 HH:MM 格式
  const timePattern = /^([01]\d|2[0-3]):([0-5]\d)(:([0-5]\d))?$/;
  if (!timePattern.test(value)) {
    return {
      isValid: false,
      error: '时间格式必须是 HH:MM:SS 或 HH:MM',
    };
  }

  return { isValid: true };
}

/**
 * 验证时间戳
 */
function validateTimestamp(value: any): ValidationResult {
  // 尝试解析时间戳
  const date = new Date(value);

  if (isNaN(date.getTime())) {
    return {
      isValid: false,
      error: '必须是有效的时间戳格式',
    };
  }

  return { isValid: true };
}

/**
 * 验证 JSON
 */
function validateJSON(value: any): ValidationResult {
  // 如果已经是对象，直接通过
  if (typeof value === 'object' && value !== null) {
    return { isValid: true };
  }

  // 如果是字符串，尝试解析
  if (typeof value === 'string') {
    try {
      JSON.parse(value);
      return { isValid: true };
    } catch (e) {
      return {
        isValid: false,
        error: '必须是有效的 JSON 格式',
      };
    }
  }

  return {
    isValid: false,
    error: '必须是有效的 JSON',
  };
}

/**
 * 验证 UUID
 */
function validateUUID(value: any): ValidationResult {
  if (typeof value !== 'string') {
    return {
      isValid: false,
      error: 'UUID 必须是字符串格式',
    };
  }

  const uuidPattern =
    /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

  if (!uuidPattern.test(value)) {
    return {
      isValid: false,
      error: '必须是有效的 UUID 格式',
    };
  }

  return { isValid: true };
}

/**
 * 验证字符串长度
 */
function validateLength(
  value: any,
  column: ColumnInfo
): ValidationResult {
  const str = String(value);
  const dataType = column.type_name.toLowerCase();

  // VARCHAR 和 CHAR 类型需要检查长度
  if (dataType.includes('varchar') || dataType.includes('char')) {
    // 从类型名称中提取长度限制
    // 例如: "character varying(255)" 或 "varchar(100)"
    const lengthMatch = column.type_name.match(/\((\d+)\)/);
    if (lengthMatch) {
      const maxLength = parseInt(lengthMatch[1], 10);
      if (str.length > maxLength) {
        return {
          isValid: false,
          error: `长度不能超过 ${maxLength} 个字符（当前: ${str.length}）`,
        };
      }
    }
  }

  return { isValid: true };
}

/**
 * 验证数值范围
 */
function validateNumericRange(
  value: any,
  column: ColumnInfo
): ValidationResult {
  const num = Number(value);
  const dataType = column.type_name.toLowerCase();

  // SMALLINT: -32768 to 32767
  if (dataType === 'smallint') {
    if (num < -32768 || num > 32767) {
      return {
        isValid: false,
        error: 'SMALLINT 范围必须在 -32768 到 32767 之间',
      };
    }
  }

  // INTEGER: -2147483648 to 2147483647
  if (dataType === 'integer' || dataType === 'int') {
    if (num < -2147483648 || num > 2147483647) {
      return {
        isValid: false,
        error: 'INTEGER 范围必须在 -2147483648 到 2147483647 之间',
      };
    }
  }

  // BIGINT: -9223372036854775808 to 9223372036854775807
  if (dataType === 'bigint') {
    if (num < -9223372036854775808 || num > 9223372036854775807) {
      return {
        isValid: false,
        error: 'BIGINT 范围超出限制',
      };
    }
  }

  // NUMERIC/DECIMAL 精度和小数位数验证
  if (dataType.includes('numeric') || dataType.includes('decimal')) {
    // 从类型名称中提取精度和小数位数
    // 例如: "numeric(10,2)"
    const precisionMatch = column.type_name.match(/\((\d+)(?:,(\d+))?\)/);
    if (precisionMatch) {
      const precision = parseInt(precisionMatch[1], 10);
      const scale = precisionMatch[2] ? parseInt(precisionMatch[2], 10) : 0;

      // 使用原始值的字符串表示进行精确检查
      let numStr = typeof value === 'string' ? value : value.toString();
      
      // 移除前导零和负号进行检查
      const isNegative = numStr.startsWith('-');
      numStr = numStr.replace(/^-/, '').replace(/^0+(?=\d)/, '') || '0';
      
      const parts = numStr.split('.');
      const integerPart = parts[0];
      const decimalPart = parts[1] || '';
      
      // 检查小数位数
      if (decimalPart.length > scale) {
        return {
          isValid: false,
          error: `小数位数不能超过 ${scale} 位（当前: ${decimalPart.length}）`,
        };
      }

      // 检查总位数（整数部分 + 小数部分）
      const totalDigits = integerPart.length + decimalPart.length;
      
      if (totalDigits > precision) {
        return {
          isValid: false,
          error: `数字总位数不能超过 ${precision} 位（当前: ${totalDigits}）`,
        };
      }
    }
  }

  return { isValid: true };
}

/**
 * 判断是否为字符串类型
 */
function isStringType(dataType: string): boolean {
  const normalizedType = dataType.toLowerCase();
  return (
    normalizedType.includes('char') ||
    normalizedType === 'text' ||
    normalizedType === 'varchar' ||
    normalizedType === 'character varying'
  );
}

/**
 * 判断是否为数值类型
 */
function isNumericType(dataType: string): boolean {
  const normalizedType = dataType.toLowerCase();
  return (
    normalizedType.includes('int') ||
    normalizedType === 'smallint' ||
    normalizedType === 'bigint' ||
    normalizedType.includes('numeric') ||
    normalizedType.includes('decimal') ||
    normalizedType === 'real' ||
    normalizedType === 'double precision' ||
    normalizedType === 'float'
  );
}

/**
 * 批量验证多个单元格值
 * @param values - 列名到值的映射
 * @param columns - 列信息数组
 * @returns 列名到验证结果的映射
 */
export function validateMultipleCells(
  values: Record<string, any>,
  columns: ColumnInfo[]
): Record<string, ValidationResult> {
  const results: Record<string, ValidationResult> = {};

  for (const [columnName, value] of Object.entries(values)) {
    const column = columns.find((col) => col.name === columnName);
    if (column) {
      results[columnName] = validateCellValue(value, column);
    }
  }

  return results;
}

/**
 * 检查是否所有验证都通过
 * @param results - 验证结果映射
 * @returns 是否全部通过
 */
export function allValidationsPassed(
  results: Record<string, ValidationResult>
): boolean {
  return Object.values(results).every((result) => result.isValid);
}

/**
 * 获取所有验证错误消息
 * @param results - 验证结果映射
 * @returns 错误消息数组
 */
export function getValidationErrors(
  results: Record<string, ValidationResult>
): string[] {
  return Object.entries(results)
    .filter(([_, result]) => !result.isValid)
    .map(([columnName, result]) => `${columnName}: ${result.error}`);
}
