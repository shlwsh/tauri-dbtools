# Table Designer 模块测试文档

## 模块概述

Table Designer 模块提供可视化的数据库表结构设计功能，包括：
- 列定义管理
- 约束管理（主键、外键、唯一、检查）
- 索引管理
- DDL 预览和生成
- 表结构应用到数据库

## 测试用例列表

### 8.5 DDLPreview 组件

#### TC-TD-001: DDLPreview 组件渲染
- **用例编号**: TC-TD-001
- **用例描述**: 验证 DDLPreview 组件能够正确渲染
- **测试类型**: 单元测试
- **对应测试文件**: `frontend/src/components/database/__tests__/DDLPreview.spec.ts`
- **测试状态**: ⚠️ 部分通过（组件构建成功，但单元测试因 Naive UI mock 问题失败）
- **已知问题**: Naive UI 组件在测试环境中的 stub 配置问题

#### TC-TD-002: DDL 生成功能
- **用例编号**: TC-TD-002
- **用例描述**: 验证 DDL 生成功能能够正确调用后端服务并显示生成的 SQL
- **测试类型**: 集成测试
- **对应测试文件**: 待实现
- **测试状态**: ⏳ 待实现
- **依赖**: 后端 DDL 生成服务（已实现）

#### TC-TD-003: DDL 应用功能
- **用例编号**: TC-TD-003
- **用例描述**: 验证应用按钮能够将生成的 DDL 执行到数据库
- **测试类型**: 集成测试
- **对应测试文件**: 待实现
- **测试状态**: ⏳ 待实现
- **依赖**: 后端 create_table/alter_table 命令

#### TC-TD-004: DDL 保存为脚本
- **用例编号**: TC-TD-004
- **用例描述**: 验证保存为脚本功能能够将 DDL 保存到文件
- **测试类型**: 单元测试
- **对应测试文件**: 待实现
- **测试状态**: ⏳ 待实现

#### TC-TD-005: Monaco Editor 集成
- **用例编号**: TC-TD-005
- **用例描述**: 验证 Monaco Editor 正确集成并显示 SQL 语法高亮
- **测试类型**: 单元测试
- **对应测试文件**: 待实现
- **测试状态**: ⏳ 待实现

#### TC-TD-006: 设计变化时 DDL 更新
- **用例编号**: TC-TD-006
- **用例描述**: 验证当表设计发生变化时，DDL 预览能够正确更新
- **测试类型**: 单元测试
- **对应测试文件**: 待实现
- **测试状态**: ⏳ 待实现

## 组件功能验证

### ✅ 已实现功能

1. **DDL 预览界面**
   - Monaco Editor 集成
   - 只读 SQL 编辑器
   - 语法高亮（SQL）
   - 深色主题

2. **工具栏按钮**
   - 刷新 DDL 按钮
   - 应用按钮
   - 保存为脚本按钮

3. **DDL 生成**
   - 调用 Table Designer Store 的 generateDDL 方法
   - 支持 CREATE TABLE 模式
   - 支持 ALTER TABLE 模式
   - 显示生成时间

4. **状态显示**
   - 成功/错误/警告消息
   - 加载状态指示器
   - DDL 行数统计
   - 生成时间显示

5. **DDL 应用**
   - 确认对话框
   - 调用 applyChanges 方法
   - 成功后自动关闭设计器
   - 错误处理

6. **保存为脚本**
   - 文件保存对话框
   - 默认文件名（表名.sql）
   - 文件写入功能

7. **自动更新**
   - 监听设计变化
   - 设计变化时清空旧 DDL
   - 提示用户重新生成

8. **验证**
   - 表名验证
   - 列数量验证
   - 生成前验证

## 测试覆盖率

### 组件测试覆盖
- **组件渲染**: ✅ 通过（构建验证）
- **按钮显示**: ✅ 通过（构建验证）
- **编辑器容器**: ✅ 通过（构建验证）
- **DDL 生成逻辑**: ⏳ 待测试
- **应用逻辑**: ⏳ 待测试
- **保存逻辑**: ⏳ 待测试

### 集成测试覆盖
- **与 Store 集成**: ⏳ 待测试
- **与后端 API 集成**: ⏳ 待测试
- **与 Monaco Editor 集成**: ⏳ 待测试

## 已知问题

### 1. 单元测试 Mock 问题
- **问题描述**: Naive UI 组件在测试环境中的 stub 配置导致 WeakMap 错误
- **影响范围**: 单元测试无法运行
- **解决方案**: 
  - 选项 1: 使用 shallowMount 代替 mount
  - 选项 2: 改进 Naive UI mock 配置
  - 选项 3: 使用 E2E 测试代替单元测试
- **优先级**: 中
- **状态**: 待解决

### 2. Monaco Editor Mock
- **问题描述**: Monaco Editor 在测试环境中需要完整的 DOM 环境
- **影响范围**: 编辑器相关功能测试
- **解决方案**: 已添加基本 mock，但可能需要更完善的实现
- **优先级**: 低
- **状态**: 已缓解

## 待补充测试

### 高优先级
1. **集成测试**: DDL 生成和应用的完整流程测试
2. **E2E 测试**: 用户从打开设计器到应用 DDL 的完整流程

### 中优先级
1. **单元测试**: 各个功能函数的独立测试
2. **错误处理测试**: 各种错误场景的测试

### 低优先级
1. **性能测试**: 大型表结构的 DDL 生成性能
2. **UI 测试**: 按钮状态、加载指示器等 UI 元素测试

## 测试执行记录

### 2024-01-XX - 初始实现
- ✅ 组件创建完成
- ✅ 构建验证通过
- ⚠️ 单元测试因 Naive UI mock 问题失败
- 📝 创建测试文档

## 相关文档

- 设计文档: `.kiro/specs/database-advanced-features/design.md`
- 需求文档: `.kiro/specs/database-advanced-features/requirements.md`
- 任务列表: `.kiro/specs/database-advanced-features/tasks.md`
- Store 测试: `frontend/src/stores/__tests__/table-designer.spec.ts`
- 组件实现: `frontend/src/components/database/DDLPreview.vue`

## 测试命令

```bash
# 运行所有测试
bun test

# 运行 DDLPreview 组件测试
bun test DDLPreview.spec.ts

# 运行 Table Designer 相关测试
bun test table-designer

# 构建验证
bun run build
```

## 备注

- DDLPreview 组件依赖 Table Designer Store 的 generateDDL 和 applyChanges 方法
- 后端 DDL 生成服务已实现并通过属性测试
- Monaco Editor 已在 SQLEditor 组件中成功使用，DDLPreview 采用相同的集成方式
- 组件已在 TableDesigner 主组件中引入并使用
