/**
 * Validation Utils Unit Tests
 * Using Bun's built-in test runner
 */

import { describe, it, expect } from 'bun:test';
import { connectionValidationRules } from '../validation';

describe('Validation Utils', () => {
  describe('connectionValidationRules', () => {
    it('should have name validation rule', () => {
      expect(connectionValidationRules.name).toBeDefined();
      expect(connectionValidationRules.name.required).toBe(true);
      expect(connectionValidationRules.name.message).toBe('请输入连接名称');
    });

    it('should have host validation rule', () => {
      expect(connectionValidationRules.host).toBeDefined();
      expect(connectionValidationRules.host.required).toBe(true);
      expect(connectionValidationRules.host.message).toBe('请输入主机地址');
    });

    it('should have port validation rule', () => {
      expect(connectionValidationRules.port).toBeDefined();
      expect(connectionValidationRules.port.required).toBe(true);
      expect(connectionValidationRules.port.type).toBe('number');
      expect(connectionValidationRules.port.message).toBe('请输入有效的端口号');
    });

    it('should have username validation rule', () => {
      expect(connectionValidationRules.username).toBeDefined();
      expect(connectionValidationRules.username.required).toBe(true);
      expect(connectionValidationRules.username.message).toBe('请输入用户名');
    });

    it('should have password validation rule', () => {
      expect(connectionValidationRules.password).toBeDefined();
      expect(connectionValidationRules.password.required).toBe(true);
      expect(connectionValidationRules.password.message).toBe('请输入密码');
    });
  });
});
