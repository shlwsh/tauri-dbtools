# Implementation Plan: Vue3 Frontend Refactor

## Overview

本实施计划将 PostgreSQL Database Tool 前端从 React 迁移到 Vue3，并添加主题系统、配置管理和数据库资源管理器功能。实施将按照以下顺序进行：

1. 项目结构搭建和基础设施
2. 核心组件和布局
3. 配置管理功能
4. 数据库导出/导入功能
5. 数据库资源管理器
6. 主题系统
7. 后端 API 扩展
8. 测试和优化

## Tasks

- [x] 1. 项目初始化和目录结构搭建
  - 创建新的 Vue3 项目结构
  - 安装依赖：Vue3、TypeScript、Vite、Naive UI、Pinia、Vue Router
  - 配置 TypeScript 和 Vite
  - 创建符合设计的目录结构（src/components、src/views、src/api、src/stores 等）
  - 创建 README.md 文档说明目录结构
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 2.1, 2.2, 2.4, 2.5_

- [x] 1.1 配置 ESLint 和 Prettier
  - 设置代码规范和格式化规则
  - _Requirements: 1.6_

- [ ] 2. 类型定义和 API 基础层
  - [x] 2.1 创建 TypeScript 类型定义
    - 在 src/types/ 下创建 common.ts、config.ts、database.ts
    - 定义 ApiResponse、DatabaseConnection、AppConfig、TableInfo、ColumnInfo 等类型
    - _Requirements: 2.3, 2.4_

  - [x] 2.2 实现 API 基础调用层
    - 在 src/api/base.ts 中实现统一的 invokeCommand 函数
    - 实现错误处理和类型转换
    - _Requirements: 2.3, 10.1_

  - [ ] 2.3 编写 API 层单元测试
    - 测试 API 调用和错误处理
    - _Requirements: 2.3_

- [ ] 3. 状态管理（Pinia Stores）
  - [x] 3.1 实现 Theme Store
    - 创建 src/stores/theme.ts
    - 实现主题切换、保存和加载功能
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 11.2_

  - [ ] 3.2 编写 Theme Store 属性测试
    - **Property 2: 主题持久化往返**
    - **Validates: Requirements 3.3, 3.4**

  - [x] 3.3 实现 Config Store
    - 创建 src/stores/config.ts
    - 实现连接配置的 CRUD 操作
    - 实现默认连接管理
    - 实现配置加载和保存
    - _Requirements: 6.2, 6.3, 6.4, 6.5, 6.7, 6.8, 6.9, 11.1, 11.3, 11.4_

  - [ ] 3.4 编写 Config Store 属性测试
    - **Property 3: 配置 CRUD 操作一致性**
    - **Validates: Requirements 6.2, 6.3, 6.4, 6.5**
    - **Property 5: 配置加载完整性**
    - **Validates: Requirements 6.9, 11.4**

  - [x] 3.5 实现 Database Store
    - 创建 src/stores/database.ts
    - 实现当前连接和数据库状态管理
    - _Requirements: 4.2, 4.3, 5.2, 7.2_

  - [ ] 3.6 编写 Store 单元测试
    - 测试状态变化和副作用
    - _Requirements: 3.1, 6.2, 6.3, 6.4_

- [ ] 4. 工具函数和组合式函数
  - [x] 4.1 实现本地存储工具
    - 创建 src/utils/storage.ts
    - 实现配置加载、保存和验证
    - 实现错误处理和默认配置
    - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5_

  - [ ] 4.2 编写存储工具属性测试
    - **Property 16: 配置数据格式正确性**
    - **Validates: Requirements 11.5**

  - [x] 4.3 实现表单验证工具
    - 创建 src/utils/validation.ts
    - 实现连接配置验证规则
    - _Requirements: 10.4_

  - [x] 4.4 实现组合式函数
    - 创建 src/composables/useTheme.ts
    - 创建 src/composables/useConfig.ts
    - 创建 src/composables/useNotification.ts
    - _Requirements: 3.2, 6.8, 10.1, 10.2, 10.3_

- [ ] 5. 核心布局组件
  - [ ] 5.1 实现 AppLayout 组件
    - 创建 src/components/common/AppLayout.vue
    - 集成 Naive UI 的 n-config-provider 和 n-layout
    - 实现主题提供者
    - _Requirements: 8.1, 8.5, 9.1_

  - [ ] 5.2 实现 AppHeader 组件
    - 创建 src/components/common/AppHeader.vue
    - 显示应用标题
    - 添加主题切换按钮
    - 添加服务器状态指示器
    - _Requirements: 3.2, 8.5, 9.3_

  - [ ] 5.3 实现 AppSidebar 组件
    - 创建 src/components/common/AppSidebar.vue
    - 实现导航菜单
    - 实现路由高亮
    - _Requirements: 8.1, 8.2, 8.3, 8.4_

  - [ ] 5.4 编写布局组件单元测试
    - 测试组件渲染和交互
    - _Requirements: 8.1, 8.2, 8.3, 8.4_

  - [ ] 5.5 编写导航属性测试
    - **Property 11: 路由导航正确性**
    - **Validates: Requirements 8.3, 8.4**

- [ ] 6. 路由配置
  - [x] 6.1 实现路由配置
    - 创建 src/router/index.ts
    - 配置所有页面路由（Home、Export、Import、Explorer、Settings）
    - _Requirements: 2.2, 8.1, 8.2, 8.3_

  - [x] 6.2 创建占位页面组件
    - 创建 src/views/Home.vue
    - 创建 src/views/DatabaseExport.vue
    - 创建 src/views/DatabaseImport.vue
    - 创建 src/views/DatabaseExplorer.vue
    - 创建 src/views/Settings.vue
    - _Requirements: 4.1, 5.1, 6.1, 7.1_

- [ ] 7. 数据库相关通用组件
  - [ ] 7.1 实现 ConnectionSelector 组件
    - 创建 src/components/database/ConnectionSelector.vue
    - 显示所有连接配置
    - 标记默认连接
    - 支持连接切换
    - _Requirements: 4.2, 5.2, 6.8, 7.2_

  - [ ] 7.2 实现 DatabaseSelector 组件
    - 创建 src/components/database/DatabaseSelector.vue
    - 根据连接加载数据库列表
    - 支持搜索过滤
    - 显示加载状态
    - _Requirements: 4.3, 5.3, 7.2, 10.2_

  - [ ] 7.3 实现 DatabaseList 组件
    - 创建 src/components/database/DatabaseList.vue
    - 显示数据库列表
    - _Requirements: 4.2, 5.2, 7.2_

  - [ ] 7.4 编写数据库组件单元测试
    - 测试组件渲染和交互
    - _Requirements: 4.2, 4.3, 5.2, 5.3_

- [ ] 8. 配置管理页面
  - [x] 8.1 实现配置管理 API
    - 创建 src/api/config.ts
    - 实现 loadConfig 和 saveConfig 函数
    - _Requirements: 6.5, 6.9, 11.1, 11.3, 11.4_

  - [x] 8.2 实现 Settings 页面
    - 完善 src/views/Settings.vue
    - 显示连接配置列表
    - 实现添加、编辑、删除连接功能
    - 实现设置默认连接功能
    - 实现表单验证
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.6, 6.7, 10.4_

  - [ ] 8.3 编写配置管理单元测试
    - 测试 CRUD 操作
    - 测试表单验证
    - _Requirements: 6.2, 6.3, 6.4, 10.4_

  - [ ] 8.4 编写配置管理属性测试
    - **Property 4: 默认连接传播**
    - **Validates: Requirements 6.8**

- [ ] 9. 数据库导出页面
  - [x] 9.1 实现数据库导出 API
    - 创建 src/api/database.ts
    - 实现 listDatabases 和 exportDatabase 函数
    - _Requirements: 2.3, 4.3, 4.4_

  - [x] 9.2 实现 DatabaseExport 页面
    - 完善 src/views/DatabaseExport.vue
    - 集成 ConnectionSelector 和 DatabaseSelector
    - 实现导出按钮和逻辑
    - 显示导出结果和文件路径
    - 实现加载状态和错误处理
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 10.1, 10.2, 10.3_

  - [ ] 9.3 编写导出页面单元测试
    - 测试页面交互和 API 调用
    - _Requirements: 4.4, 4.5_

  - [ ] 9.4 编写导出操作属性测试
    - **Property 7: 导出操作完整性**
    - **Validates: Requirements 4.4, 4.5**

- [ ] 10. 数据库导入页面
  - [x] 10.1 实现数据库导入 API
    - 在 src/api/database.ts 中添加 importDatabase 函数
    - _Requirements: 2.3, 5.5_

  - [x] 10.2 实现 DatabaseImport 页面
    - 完善 src/views/DatabaseImport.vue
    - 集成 ConnectionSelector
    - 实现文件选择功能（使用 @tauri-apps/plugin-dialog）
    - 实现数据库名称输入
    - 实现导入按钮和逻辑
    - 显示导入结果
    - 实现加载状态和错误处理
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 10.1, 10.2, 10.3_

  - [ ] 10.3 编写导入页面单元测试
    - 测试页面交互和 API 调用
    - _Requirements: 5.5, 5.6_

  - [ ] 10.4 编写导入操作属性测试
    - **Property 8: 导入操作完整性**
    - **Validates: Requirements 5.5, 5.6**

- [ ] 11. 数据库资源管理器 - 后端 API 扩展
  - [x] 11.1 扩展 Rust 后端类型定义
    - 在 src-tauri/src/lib.rs 中添加 TableInfo、ColumnInfo、TableData 结构体
    - _Requirements: 12.1, 12.2, 12.3_

  - [x] 11.2 实现列出表的 API
    - 添加 list_tables Tauri 命令
    - 使用 psql 查询表列表
    - _Requirements: 12.2_

  - [ ] 11.3 编写列出表 API 属性测试
    - **Property 18: 后端 API 表列表正确性**
    - **Validates: Requirements 12.2**

  - [x] 11.4 实现查询表数据的 API
    - 添加 get_table_data Tauri 命令
    - 实现分页查询
    - 返回列信息和行数据
    - _Requirements: 12.3_

  - [ ] 11.5 编写查询表数据 API 属性测试
    - **Property 19: 后端 API 分页查询正确性**
    - **Validates: Requirements 12.3**

  - [x] 11.6 实现 CRUD 操作 API
    - 添加 create_record Tauri 命令
    - 添加 update_record Tauri 命令
    - 添加 delete_record Tauri 命令
    - 实现 SQL 生成和执行
    - _Requirements: 12.4, 12.5, 12.6_

  - [ ] 11.7 编写 CRUD API 属性测试
    - **Property 20: 后端 API CRUD 操作正确性**
    - **Validates: Requirements 12.4, 12.5, 12.6**

  - [ ] 11.8 更新后端以支持连接配置参数
    - 修改所有数据库操作命令以接受 DatabaseConnection 参数
    - 更新 list_databases、export_database、import_database 命令
    - _Requirements: 12.7_

  - [ ] 11.9 编写连接配置支持属性测试
    - **Property 21: 后端 API 连接配置支持**
    - **Validates: Requirements 12.7**

  - [ ] 11.10 实现后端错误处理
    - 统一错误响应格式
    - 实现详细的错误消息
    - _Requirements: 12.8_

  - [ ] 11.11 编写后端错误处理属性测试
    - **Property 22: 后端 API 错误响应格式**
    - **Validates: Requirements 12.8**

- [ ] 12. 数据库资源管理器 - 前端实现
  - [x] 12.1 实现资源管理器 API
    - 创建 src/api/explorer.ts
    - 实现 listTables、getTableData、createRecord、updateRecord、deleteRecord 函数
    - _Requirements: 7.2, 7.3, 7.4, 7.6, 7.8, 7.9, 7.10_

  - [x] 12.2 实现 TableList 组件
    - 创建 src/components/explorer/TableList.vue
    - 显示表列表
    - 支持表选择
    - _Requirements: 7.3_

  - [x] 12.3 实现 DataTable 组件
    - 创建 src/components/explorer/DataTable.vue
    - 以表格形式显示数据
    - 实现分页控件
    - 实现行选择
    - 添加 CRUD 操作按钮
    - _Requirements: 7.4, 7.5, 7.6, 7.7, 7.8, 7.9_

  - [x] 12.4 实现 RecordEditor 组件
    - 创建 src/components/explorer/RecordEditor.vue
    - 实现表单编辑器
    - 根据列类型显示不同输入控件
    - 实现表单验证
    - _Requirements: 7.6, 7.8, 10.4_

  - [x] 12.5 实现 RecordViewer 组件
    - 创建 src/components/explorer/RecordViewer.vue
    - 显示记录详情
    - _Requirements: 7.7_

  - [x] 12.6 实现 DatabaseExplorer 页面
    - 完善 src/views/DatabaseExplorer.vue
    - 集成 ConnectionSelector、DatabaseSelector、TableList、DataTable
    - 实现 CRUD 操作流程
    - 实现数据刷新
    - 实现加载状态和错误处理
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 7.8, 7.9, 7.10, 7.11, 10.1, 10.2, 10.3_

  - [ ] 12.7 编写资源管理器组件单元测试
    - 测试组件渲染和交互
    - _Requirements: 7.2, 7.3, 7.4, 7.5_

  - [ ] 12.8 编写资源管理器 CRUD 属性测试
    - **Property 9: 表数据 CRUD 往返**
    - **Validates: Requirements 7.6, 7.8, 7.9, 7.11**
    - **Property 10: 分页数据一致性**
    - **Validates: Requirements 7.5**

- [ ] 13. 主题系统完善
  - [x] 13.1 创建主题样式文件
    - 创建 src/assets/styles/variables.css
    - 创建 src/assets/styles/themes.css
    - 创建 src/assets/styles/global.css
    - 定义亮色和暗色主题的 CSS 变量
    - _Requirements: 3.1, 3.5, 9.2_

  - [x] 13.2 集成主题到所有组件
    - 确保所有组件使用主题变量
    - 测试主题切换效果
    - _Requirements: 3.2, 3.5_

  - [ ] 13.3 编写主题系统单元测试
    - 测试主题切换和持久化
    - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [ ] 14. 首页实现
  - [x] 14.1 实现 Home 页面
    - 完善 src/views/Home.vue
    - 显示应用介绍
    - 显示快速操作链接
    - 显示服务器状态
    - _Requirements: 8.1, 9.2, 9.3_

- [ ] 15. 错误处理和用户反馈完善
  - [ ] 15.1 实现全局错误处理
    - 在 main.ts 中配置全局错误处理器
    - _Requirements: 10.1_

  - [ ] 15.2 实现通知系统
    - 完善 useNotification 组合式函数
    - 统一成功、错误、加载提示
    - _Requirements: 10.1, 10.2, 10.3, 10.5_

  - [ ] 15.3 编写错误处理属性测试
    - **Property 12: 错误处理一致性**
    - **Validates: Requirements 10.1, 10.5**
    - **Property 13: 加载状态可见性**
    - **Validates: Requirements 10.2**
    - **Property 14: 成功反馈一致性**
    - **Validates: Requirements 10.3**
    - **Property 15: 表单验证反馈**
    - **Validates: Requirements 10.4**

- [ ] 16. 应用入口和配置
  - [x] 16.1 实现应用入口
    - 完善 src/main.ts
    - 初始化 Vue 应用
    - 注册 Pinia、Router、Naive UI
    - 加载初始配置和主题
    - _Requirements: 2.1, 2.2, 3.4, 6.9, 11.4_

  - [x] 16.2 实现根组件
    - 完善 src/App.vue
    - 集成 AppLayout
    - _Requirements: 8.5, 9.2_

  - [ ] 16.3 更新 index.html
    - 设置页面标题和元数据
    - _Requirements: 9.2_

- [ ] 17. 构建配置和优化
  - [ ] 17.1 配置 Vite 构建
    - 更新 vite.config.ts
    - 配置路径别名
    - 配置构建优化
    - _Requirements: 2.5_

  - [ ] 17.2 更新 Tauri 配置
    - 更新 src-tauri/tauri.conf.json
    - 配置前端构建命令
    - _Requirements: 2.3_

  - [ ] 17.3 更新 package.json
    - 添加所有依赖
    - 配置脚本命令
    - _Requirements: 2.1, 2.5_

- [ ] 18. 文档和清理
  - [ ] 18.1 更新 README 文档
    - 说明项目结构
    - 说明开发和构建流程
    - 说明技术栈
    - _Requirements: 1.7_

  - [ ] 18.2 清理旧的 React 代码
    - 备份旧代码（如果需要）
    - 删除不再使用的文件
    - _Requirements: 2.1_

- [ ] 19. 检查点 - 确保所有功能正常工作
  - 测试所有页面导航
  - 测试配置管理功能
  - 测试数据库导出/导入功能
  - 测试数据库资源管理器
  - 测试主题切换
  - 测试错误处理
  - 确保所有测试通过，询问用户是否有问题

- [ ] 20. 集成测试和端到端测试
  - 编写端到端测试覆盖主要用户流程
  - 配置 CI/CD 流程
  - _Requirements: 所有需求_

## Notes

- 每个任务都引用了相关的需求编号，便于追溯
- 属性测试任务明确标注了对应的设计文档属性编号
- 建议按顺序执行任务，因为后续任务依赖前面任务的成果
- 检查点任务用于验证阶段性成果，确保质量
- 所有测试任务都是必需的，以确保代码质量和正确性

