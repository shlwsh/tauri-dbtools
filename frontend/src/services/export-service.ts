/**
 * Export Service
 * 
 * 提供查询结果导出功能，支持多种格式：
 * - CSV 格式
 * - JSON 格式
 * - Excel 格式（使用 xlsx 库）
 * 
 * Validates: Requirements 18.2, 18.3, 18.4, 18.5
 */

import type { ColumnInfo } from '@/types/sql-editor';

/**
 * 导出格式
 */
export type ExportFormat = 'csv' | 'json' | 'excel';

/**
 * 导出选项
 */
export interface ExportOptions {
  /** 导出格式 */
  format: ExportFormat;
  /** 是否包含列标题 */
  includeHeaders?: boolean;
  /** CSV 分隔符 */
  csvDelimiter?: string;
  /** JSON 缩进 */
  jsonIndent?: number;
  /** Excel 工作表名称 */
  sheetName?: string;
}

/**
 * 导出结果
 */
export interface ExportResult {
  /** 导出的数据（Blob 或字符串） */
  data: Blob | string;
  /** 建议的文件名 */
  filename: string;
  /** MIME 类型 */
  mimeType: string;
}

/**
 * 导出查询结果为 CSV 格式
 * @param columns - 列信息
 * @param rows - 数据行
 * @param options - 导出选项
 * @returns CSV 字符串
 */
export function exportToCSV(
  columns: ColumnInfo[],
  rows: Record<string, any>[],
  options: ExportOptions = { format: 'csv' }
): string {
  const delimiter = options.csvDelimiter || ',';
  const lines: string[] = [];

  // 添加列标题
  if (options.includeHeaders !== false) {
    const headers = columns.map(col => escapeCSVValue(col.name));
    lines.push(headers.join(delimiter));
  }

  // 添加数据行
  rows.forEach(row => {
    const values = columns.map(col => {
      const value = row[col.name];
      return escapeCSVValue(formatValueForCSV(value));
    });
    lines.push(values.join(delimiter));
  });

  return lines.join('\n');
}

/**
 * 导出查询结果为 JSON 格式
 * @param columns - 列信息
 * @param rows - 数据行
 * @param options - 导出选项
 * @returns JSON 字符串
 */
export function exportToJSON(
  columns: ColumnInfo[],
  rows: Record<string, any>[],
  options: ExportOptions = { format: 'json' }
): string {
  const indent = options.jsonIndent !== undefined ? options.jsonIndent : 2;

  // 转换数据（处理特殊值）
  const data = rows.map(row => {
    const converted: Record<string, any> = {};
    columns.forEach(col => {
      converted[col.name] = formatValueForJSON(row[col.name]);
    });
    return converted;
  });

  return JSON.stringify(data, null, indent);
}

/**
 * 导出查询结果为 Excel 格式
 * @param columns - 列信息
 * @param rows - 数据行
 * @param options - 导出选项
 * @returns Excel Blob
 */
export async function exportToExcel(
  columns: ColumnInfo[],
  rows: Record<string, any>[],
  options: ExportOptions = { format: 'excel' }
): Promise<Blob> {
  // 动态导入 xlsx 库（减少初始加载大小）
  const XLSX = await import('xlsx');

  const sheetName = options.sheetName || 'Sheet1';

  // 准备数据
  const data: any[][] = [];

  // 添加列标题
  if (options.includeHeaders !== false) {
    data.push(columns.map(col => col.name));
  }

  // 添加数据行
  rows.forEach(row => {
    const rowData = columns.map(col => formatValueForExcel(row[col.name]));
    data.push(rowData);
  });

  // 创建工作表
  const worksheet = XLSX.utils.aoa_to_sheet(data);

  // 设置列宽（自动调整）
  const colWidths = columns.map(col => ({
    wch: Math.max(col.name.length, 10),
  }));
  worksheet['!cols'] = colWidths;

  // 创建工作簿
  const workbook = XLSX.utils.book_new();
  XLSX.utils.book_append_sheet(workbook, worksheet, sheetName);

  // 生成 Excel 文件
  const excelBuffer = XLSX.write(workbook, {
    bookType: 'xlsx',
    type: 'array',
  });

  return new Blob([excelBuffer], {
    type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
  });
}

/**
 * 导出查询结果
 * @param columns - 列信息
 * @param rows - 数据行
 * @param options - 导出选项
 * @returns 导出结果
 */
export async function exportQueryResult(
  columns: ColumnInfo[],
  rows: Record<string, any>[],
  options: ExportOptions
): Promise<ExportResult> {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, -5);

  switch (options.format) {
    case 'csv': {
      const csv = exportToCSV(columns, rows, options);
      return {
        data: csv,
        filename: `query_result_${timestamp}.csv`,
        mimeType: 'text/csv',
      };
    }

    case 'json': {
      const json = exportToJSON(columns, rows, options);
      return {
        data: json,
        filename: `query_result_${timestamp}.json`,
        mimeType: 'application/json',
      };
    }

    case 'excel': {
      const blob = await exportToExcel(columns, rows, options);
      return {
        data: blob,
        filename: `query_result_${timestamp}.xlsx`,
        mimeType:
          'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
      };
    }

    default:
      throw new Error(`不支持的导出格式: ${options.format}`);
  }
}

/**
 * 转义 CSV 值
 * @param value - 要转义的值
 * @returns 转义后的值
 */
function escapeCSVValue(value: string): string {
  // 如果值包含逗号、引号或换行符，需要用引号包裹
  if (
    value.includes(',') ||
    value.includes('"') ||
    value.includes('\n') ||
    value.includes('\r')
  ) {
    // 引号需要转义为两个引号
    return `"${value.replace(/"/g, '""')}"`;
  }
  return value;
}

/**
 * 格式化值用于 CSV
 * @param value - 原始值
 * @returns 格式化后的字符串
 */
function formatValueForCSV(value: any): string {
  if (value === null || value === undefined) {
    return '';
  }

  if (typeof value === 'boolean') {
    return value ? 'true' : 'false';
  }

  if (typeof value === 'object') {
    // 日期对象
    if (value instanceof Date) {
      return value.toISOString();
    }
    // 其他对象转为 JSON
    return JSON.stringify(value);
  }

  return String(value);
}

/**
 * 格式化值用于 JSON
 * @param value - 原始值
 * @returns 格式化后的值
 */
function formatValueForJSON(value: any): any {
  if (value === null || value === undefined) {
    return null;
  }

  // 日期对象转为 ISO 字符串
  if (value instanceof Date) {
    return value.toISOString();
  }

  // Buffer 转为 base64
  if (value instanceof Uint8Array || value instanceof ArrayBuffer) {
    return btoa(String.fromCharCode(...new Uint8Array(value)));
  }

  return value;
}

/**
 * 格式化值用于 Excel
 * @param value - 原始值
 * @returns 格式化后的值
 */
function formatValueForExcel(value: any): any {
  if (value === null || value === undefined) {
    return '';
  }

  if (typeof value === 'boolean') {
    return value;
  }

  if (typeof value === 'number') {
    return value;
  }

  // 日期对象
  if (value instanceof Date) {
    return value;
  }

  // 对象转为 JSON 字符串
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }

  return String(value);
}

/**
 * 下载导出的文件
 * @param result - 导出结果
 */
export function downloadExportedFile(result: ExportResult): void {
  let blob: Blob;

  if (result.data instanceof Blob) {
    blob = result.data;
  } else {
    blob = new Blob([result.data], { type: result.mimeType });
  }

  // 创建下载链接
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = result.filename;

  // 触发下载
  document.body.appendChild(link);
  link.click();

  // 清理
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

/**
 * 获取导出格式的显示名称
 * @param format - 导出格式
 * @returns 显示名称
 */
export function getFormatDisplayName(format: ExportFormat): string {
  switch (format) {
    case 'csv':
      return 'CSV';
    case 'json':
      return 'JSON';
    case 'excel':
      return 'Excel (XLSX)';
    default:
      return format;
  }
}

/**
 * 获取导出格式的文件扩展名
 * @param format - 导出格式
 * @returns 文件扩展名
 */
export function getFormatExtension(format: ExportFormat): string {
  switch (format) {
    case 'csv':
      return '.csv';
    case 'json':
      return '.json';
    case 'excel':
      return '.xlsx';
    default:
      return '';
  }
}
