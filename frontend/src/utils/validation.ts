/**
 * Form validation utilities
 */

import type { FormItemRule } from 'naive-ui';

/**
 * Validation rules for database connection configuration
 */
export const connectionValidationRules = {
  name: {
    required: true,
    message: '请输入连接名称',
    trigger: ['blur', 'input'],
  } as FormItemRule,

  host: {
    required: true,
    message: '请输入主机地址',
    trigger: ['blur', 'input'],
  } as FormItemRule,

  port: {
    required: true,
    type: 'number',
    message: '请输入有效的端口号',
    trigger: ['blur', 'change'],
  } as FormItemRule,

  username: {
    required: true,
    message: '请输入用户名',
    trigger: ['blur', 'input'],
  } as FormItemRule,

  password: {
    required: true,
    message: '请输入密码',
    trigger: ['blur', 'input'],
  } as FormItemRule,
};
