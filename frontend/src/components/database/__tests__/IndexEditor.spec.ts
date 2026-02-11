/**
 * Unit tests for IndexEditor component
 * 
 * Tests the index management functionality including:
 * - Index list display
 * - Adding new indexes
 * - Editing existing indexes
 * - Deleting indexes
 * - Index property editing (columns, type, uniqueness)
 * 
 * Validates: Requirements 6.6, 6.7, 6.8, 6.9
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { h } from 'vue';
import IndexEditor from '../IndexEditor.vue';
import { useTableDesignerStore } from '@/stores/table-designer';
import type { TableDesignState, IndexDefinition } from '@/types/table-designer';

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

describe('IndexEditor', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  const createMockDesign = (): TableDesignState => ({
    tableName: 'test_table',
    schema: 'public',
    columns: [
      {
        name: 'id',
        type: 'INTEGER',
        nullable: false,
        isPrimaryKey: true,
        isUnique: false,
      },
      {
        name: 'name',
        type: 'VARCHAR',
        length: 255,
        nullable: false,
        isPrimaryKey: false,
        isUnique: false,
      },
      {
        name: 'email',
        type: 'VARCHAR',
        length: 255,
        nullable: false,
        isPrimaryKey: false,
        isUnique: true,
      },
    ],
    constraints: [],
    indexes: [
      {
        name: 'idx_name',
        columns: ['name'],
        type: 'btree',
        unique: false,
      },
      {
        name: 'idx_email_unique',
        columns: ['email'],
        type: 'btree',
        unique: true,
      },
    ],
    isDirty: false,
  });

  describe('Index List Display', () => {
    it('should display list of indexes', () => {
      const store = useTableDesignerStore();
      store.currentDesign = createMockDesign();

      const wrapper = mount(IndexEditor, {
        global: {
          stubs: {
            NSpace: { template: '<div><slot /></div>' },
            NText: { template: '<span><slot /></span>' },
            NButton: { template: '<button><slot /></button>' },
            NIcon: { template: '<i><slot /></i>' },
            NDataTable: {
              template: '<div class="n-data-table"><slot /></div>',
              props: ['data', 'columns'],
            },
            NModal: { template: '<div><slot /></div>' },
            NForm: { template: '<form><slot /></form>' },
            NFormItem: { template: '<div><slot /></div>' },
            NInput: { template: '<input />' },
            NSelect: { template: '<select />' },
            NCheckbox: { template: '<input type="checkbox" />' },
          },
        },
      });

      expect(wrapper.find('.index-editor').exists()).toBe(true);
    });

    it('should show only non-deleted indexes', () => {
      const store = useTableDesignerStore();
      const design = createMockDesign();
      design.indexes[0].isDeleted = true;
      store.currentDesign = design;

      const wrapper = mount(IndexEditor, {
        global: {
          stubs: {
            NSpace: { template: '<div><slot /></div>' },
            NText: { template: '<span><slot /></span>' },
            NButton: { template: '<button><slot /></button>' },
            NIcon: { template: '<i><slot /></i>' },
            NDataTable: {
              template: '<div class="n-data-table"></div>',
              props: ['data', 'columns'],
            },
            NModal: { template: '<div><slot /></div>' },
          },
        },
      });

      const vm = wrapper.vm as any;
      // Verify that only non-deleted indexes are visible
      expect(vm.visibleIndexes).toHaveLength(1);
      expect(vm.visibleIndexes[0].name).toBe('idx_email_unique');
    });

    it('should display index properties correctly', () => {
      const store = useTableDesignerStore();
      store.currentDesign = createMockDesign();

      const wrapper = mount(IndexEditor, {
        global: {
          stubs: {
            NSpace: { template: '<div><slot /></div>' },
            NText: { template: '<span><slot /></span>' },
            NButton: { template: '<button><slot /></button>' },
            NIcon: { template: '<i><slot /></i>' },
            NDataTable: {
              template: '<div class="n-data-table"></div>',
              props: ['data', 'columns'],
            },
            NModal: { template: '<div><slot /></div>' },
          },
        },
      });

      const vm = wrapper.vm as any;
      expect(vm.visibleIndexes).toHaveLength(2);
      expect(vm.visibleIndexes[0].name).toBe('idx_name');
      expect(vm.visibleIndexes[0].columns).toEqual(['name']);
      expect(vm.visibleIndexes[0].type).toBe('btree');
      expect(vm.visibleIndexes[0].unique).toBe(false);
    });
  });

  describe('Adding Indexes', () => {
    it('should open modal when add button is clicked', async () => {
      const store = useTableDesignerStore();
      store.currentDesign = createMockDesign();

      const wrapper = mount(IndexEditor, {
        global: {
          stubs: {
            NSpace: { template: '<div><slot /></div>' },
            NText: { template: '<span><slot /></span>' },
            NButton: {
              template: '<button @click="$attrs.onClick"><slot /></button>',
            },
            NIcon: { template: '<i><slot /></i>' },
            NDataTable: { template: '<div></div>' },
            NModal: {
              template: '<div v-if="show" class="modal"><slot /></div>',
              props: ['show'],
            },
            NForm: { template: '<form><slot /></form>' },
            NFormItem: { template: '<div><slot /></div>' },
            NInput: { template: '<input />' },
            NSelect: { template: '<select />' },
            NCheckbox: { template: '<input type="checkbox" />' },
          },
        },
      });

      const vm = wrapper.vm as any;
      expect(vm.showEditModal).toBe(false);

      await wrapper.findAll('button')[0].trigger('click');
      expect(vm.showEditModal).toBe(true);
    });

    it('should add new index to store', async () => {
      const store = useTableDesignerStore();
      store.currentDesign = createMockDesign();
      const addIndexSpy = vi.spyOn(store, 'addIndex');

      const wrapper = mount(IndexEditor, {
        global: {
          stubs: {
            NSpace: { template: '<div><slot /></div>' },
            NText: { template: '<span><slot /></span>' },
            NButton: { template: '<button><slot /></button>' },
            NIcon: { template: '<i><slot /></i>' },
            NDataTable: { template: '<div></div>' },
            NModal: { template: '<div><slot /></div>' },
            NForm: { template: '<form><slot /></form>' },
            NFormItem: { template: '<div><slot /></div>' },
            NInput: { template: '<input />' },
            NSelect: { template: '<select />' },
            NCheckbox: { template: '<input type="checkbox" />' },
          },
        },
      });

      const vm = wrapper.vm as any;
      vm.editingIndexDef = {
        name: 'idx_new',
        columns: ['name', 'email'],
        type: 'btree',
        unique: false,
      };
      vm.editingIndex = -1;

      // Mock form validation
      vm.formRef = {
        validate: vi.fn().mockResolvedValue(true),
      };

      await vm.handleSaveIndex();

      expect(addIndexSpy).toHaveBeenCalledWith({
        name: 'idx_new',
        columns: ['name', 'email'],
        type: 'btree',
        unique: false,
      });
    });
  });

  describe('Editing Indexes', () => {
    it('should load index data when editing', async () => {
      const store = useTableDesignerStore();
      store.currentDesign = createMockDesign();

      const wrapper = mount(IndexEditor, {
        global: {
          stubs: {
            NSpace: { template: '<div><slot /></div>' },
            NText: { template: '<span><slot /></span>' },
            NButton: { template: '<button><slot /></button>' },
            NIcon: { template: '<i><slot /></i>' },
            NDataTable: { template: '<div></div>' },
            NModal: { template: '<div><slot /></div>' },
            NForm: { template: '<form><slot /></form>' },
            NFormItem: { template: '<div><slot /></div>' },
            NInput: { template: '<input />' },
            NSelect: { template: '<select />' },
            NCheckbox: { template: '<input type="checkbox" />' },
          },
        },
      });

      const vm = wrapper.vm as any;
      vm.handleEditIndex(0);

      expect(vm.editingIndex).toBe(0);
      expect(vm.editingIndexDef.name).toBe('idx_name');
      expect(vm.editingIndexDef.columns).toEqual(['name']);
      expect(vm.editingIndexDef.type).toBe('btree');
      expect(vm.showEditModal).toBe(true);
    });

    it('should update index in store', async () => {
      const store = useTableDesignerStore();
      store.currentDesign = createMockDesign();
      const deleteIndexSpy = vi.spyOn(store, 'deleteIndex');
      const addIndexSpy = vi.spyOn(store, 'addIndex');

      const wrapper = mount(IndexEditor, {
        global: {
          stubs: {
            NSpace: { template: '<div><slot /></div>' },
            NText: { template: '<span><slot /></span>' },
            NButton: { template: '<button><slot /></button>' },
            NIcon: { template: '<i><slot /></i>' },
            NDataTable: { template: '<div></div>' },
            NModal: { template: '<div><slot /></div>' },
            NForm: { template: '<form><slot /></form>' },
            NFormItem: { template: '<div><slot /></div>' },
            NInput: { template: '<input />' },
            NSelect: { template: '<select />' },
            NCheckbox: { template: '<input type="checkbox" />' },
          },
        },
      });

      const vm = wrapper.vm as any;
      vm.editingIndexDef = {
        name: 'idx_name_updated',
        columns: ['name', 'email'],
        type: 'hash',
        unique: true,
      };
      vm.editingIndex = 0;

      // Mock form validation
      vm.formRef = {
        validate: vi.fn().mockResolvedValue(true),
      };

      await vm.handleSaveIndex();

      expect(deleteIndexSpy).toHaveBeenCalledWith(0);
      expect(addIndexSpy).toHaveBeenCalledWith({
        name: 'idx_name_updated',
        columns: ['name', 'email'],
        type: 'hash',
        unique: true,
      });
    });
  });

  describe('Deleting Indexes', () => {
    it('should call deleteIndex on store when confirmed', async () => {
      const store = useTableDesignerStore();
      store.currentDesign = createMockDesign();
      const deleteIndexSpy = vi.spyOn(store, 'deleteIndex');

      const wrapper = mount(IndexEditor, {
        global: {
          stubs: {
            NSpace: { template: '<div><slot /></div>' },
            NText: { template: '<span><slot /></span>' },
            NButton: { template: '<button><slot /></button>' },
            NIcon: { template: '<i><slot /></i>' },
            NDataTable: { template: '<div></div>' },
            NModal: { template: '<div><slot /></div>' },
            NForm: { template: '<form><slot /></form>' },
            NFormItem: { template: '<div><slot /></div>' },
            NInput: { template: '<input />' },
            NSelect: { template: '<select />' },
            NCheckbox: { template: '<input type="checkbox" />' },
          },
        },
      });

      const vm = wrapper.vm as any;
      vm.handleDeleteIndex(0);

      expect(deleteIndexSpy).toHaveBeenCalledWith(0);
    });
  });

  describe('Index Property Editing', () => {
    it('should support selecting multiple columns', () => {
      const store = useTableDesignerStore();
      store.currentDesign = createMockDesign();

      const wrapper = mount(IndexEditor, {
        global: {
          stubs: {
            NSpace: { template: '<div><slot /></div>' },
            NText: { template: '<span><slot /></span>' },
            NButton: { template: '<button><slot /></button>' },
            NIcon: { template: '<i><slot /></i>' },
            NDataTable: { template: '<div></div>' },
            NModal: { template: '<div><slot /></div>' },
            NForm: { template: '<form><slot /></form>' },
            NFormItem: { template: '<div><slot /></div>' },
            NInput: { template: '<input />' },
            NSelect: { template: '<select />' },
            NCheckbox: { template: '<input type="checkbox" />' },
          },
        },
      });

      const vm = wrapper.vm as any;
      expect(vm.columnOptions).toHaveLength(3);
      expect(vm.columnOptions.map((opt: any) => opt.value)).toEqual(['id', 'name', 'email']);
    });

    it('should support all index types', () => {
      const store = useTableDesignerStore();
      store.currentDesign = createMockDesign();

      const wrapper = mount(IndexEditor, {
        global: {
          stubs: {
            NSpace: { template: '<div><slot /></div>' },
            NText: { template: '<span><slot /></span>' },
            NButton: { template: '<button><slot /></button>' },
            NIcon: { template: '<i><slot /></i>' },
            NDataTable: { template: '<div></div>' },
            NModal: { template: '<div><slot /></div>' },
            NForm: { template: '<form><slot /></form>' },
            NFormItem: { template: '<div><slot /></div>' },
            NInput: { template: '<input />' },
            NSelect: { template: '<select />' },
            NCheckbox: { template: '<input type="checkbox" />' },
          },
        },
      });

      const vm = wrapper.vm as any;
      expect(vm.indexTypeOptions).toHaveLength(4);
      expect(vm.indexTypeOptions.map((opt: any) => opt.value)).toEqual([
        'btree',
        'hash',
        'gist',
        'gin',
      ]);
    });

    it('should support unique index flag', async () => {
      const store = useTableDesignerStore();
      store.currentDesign = createMockDesign();
      const addIndexSpy = vi.spyOn(store, 'addIndex');

      const wrapper = mount(IndexEditor, {
        global: {
          stubs: {
            NSpace: { template: '<div><slot /></div>' },
            NText: { template: '<span><slot /></span>' },
            NButton: { template: '<button><slot /></button>' },
            NIcon: { template: '<i><slot /></i>' },
            NDataTable: { template: '<div></div>' },
            NModal: { template: '<div><slot /></div>' },
            NForm: { template: '<form><slot /></form>' },
            NFormItem: { template: '<div><slot /></div>' },
            NInput: { template: '<input />' },
            NSelect: { template: '<select />' },
            NCheckbox: { template: '<input type="checkbox" />' },
          },
        },
      });

      const vm = wrapper.vm as any;
      vm.editingIndexDef = {
        name: 'idx_unique',
        columns: ['email'],
        type: 'btree',
        unique: true,
      };
      vm.editingIndex = -1;

      // Mock form validation
      vm.formRef = {
        validate: vi.fn().mockResolvedValue(true),
      };

      await vm.handleSaveIndex();

      expect(addIndexSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          unique: true,
        })
      );
    });
  });

  describe('Form Validation', () => {
    it('should require index name', () => {
      const store = useTableDesignerStore();
      store.currentDesign = createMockDesign();

      const wrapper = mount(IndexEditor, {
        global: {
          stubs: {
            NSpace: { template: '<div><slot /></div>' },
            NText: { template: '<span><slot /></span>' },
            NButton: { template: '<button><slot /></button>' },
            NIcon: { template: '<i><slot /></i>' },
            NDataTable: { template: '<div></div>' },
            NModal: { template: '<div><slot /></div>' },
            NForm: { template: '<form><slot /></form>' },
            NFormItem: { template: '<div><slot /></div>' },
            NInput: { template: '<input />' },
            NSelect: { template: '<select />' },
            NCheckbox: { template: '<input type="checkbox" />' },
          },
        },
      });

      const vm = wrapper.vm as any;
      expect(vm.formRules.name.required).toBe(true);
    });

    it('should require at least one column', () => {
      const store = useTableDesignerStore();
      store.currentDesign = createMockDesign();

      const wrapper = mount(IndexEditor, {
        global: {
          stubs: {
            NSpace: { template: '<div><slot /></div>' },
            NText: { template: '<span><slot /></span>' },
            NButton: { template: '<button><slot /></button>' },
            NIcon: { template: '<i><slot /></i>' },
            NDataTable: { template: '<div></div>' },
            NModal: { template: '<div><slot /></div>' },
            NForm: { template: '<form><slot /></form>' },
            NFormItem: { template: '<div><slot /></div>' },
            NInput: { template: '<input />' },
            NSelect: { template: '<select />' },
            NCheckbox: { template: '<input type="checkbox" />' },
          },
        },
      });

      const vm = wrapper.vm as any;
      expect(vm.formRules.columns.required).toBe(true);
      expect(vm.formRules.columns.min).toBe(1);
    });

    it('should require index type', () => {
      const store = useTableDesignerStore();
      store.currentDesign = createMockDesign();

      const wrapper = mount(IndexEditor, {
        global: {
          stubs: {
            NSpace: { template: '<div><slot /></div>' },
            NText: { template: '<span><slot /></span>' },
            NButton: { template: '<button><slot /></button>' },
            NIcon: { template: '<i><slot /></i>' },
            NDataTable: { template: '<div></div>' },
            NModal: { template: '<div><slot /></div>' },
            NForm: { template: '<form><slot /></form>' },
            NFormItem: { template: '<div><slot /></div>' },
            NInput: { template: '<input />' },
            NSelect: { template: '<select />' },
            NCheckbox: { template: '<input type="checkbox" />' },
          },
        },
      });

      const vm = wrapper.vm as any;
      expect(vm.formRules.type.required).toBe(true);
    });
  });
});
