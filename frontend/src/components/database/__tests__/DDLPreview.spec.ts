/**
 * DDLPreview 组件单元测试
 * 
 * 测试 DDL 预览组件的功能：
 * - 组件渲染
 * - 按钮显示
 * - 编辑器容器
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import DDLPreview from '../DDLPreview.vue';
import { useTableDesignerStore } from '@/stores/table-designer';

// Mock Naive UI composables
vi.mock('naive-ui', async () => {
  const actual = await vi.importActual('naive-ui');
  return {
    ...actual,
    useDialog: () => ({
      warning: vi.fn((options) => {
        if (options.onPositiveClick) {
          options.onPositiveClick();
        }
      }),
    }),
    useMessage: () => ({
      success: vi.fn(),
      error: vi.fn(),
      warning: vi.fn(),
      info: vi.fn(),
    }),
  };
});

// Mock Monaco Editor
vi.mock('monaco-editor', () => ({
  editor: {
    create: vi.fn(() => ({
      setValue: vi.fn(),
      getValue: vi.fn(),
      dispose: vi.fn(),
    })),
  },
}));

describe('DDLPreview', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('应该正确渲染组件', () => {
    const wrapper = mount(DDLPreview);
    expect(wrapper.exists()).toBe(true);
  });

  it('应该有编辑器容器', () => {
    const wrapper = mount(DDLPreview);
    const editorContainer = wrapper.find('.ddl-editor-container');
    expect(editorContainer.exists()).toBe(true);
  });

  it('应该显示工具栏按钮', () => {
    const wrapper = mount(DDLPreview);
    // 组件应该包含按钮
    expect(wrapper.html()).toContain('button');
  });
});
