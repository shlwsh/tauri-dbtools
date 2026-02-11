# 测试总览

## 项目测试架构

本项目采用全面的测试策略，包括单元测试、集成测试、属性测试和端到端测试。

## 测试目录结构

```
test/
├── frontend/              # 前端测试
│   ├── unit/             # 单元测试
│   ├── integration/      # 集成测试
│   └── e2e/              # 端到端测试
├── backend/              # 后端测试
│   ├── unit/             # 单元测试（位于 src-tauri/src/）
│   ├── integration/      # 集成测试（位于 src-tauri/tests/）
│   └── property/         # 属性测试（位于 src-tauri/tests/）
├── docs/                 # 测试文档
│   ├── test-cases.md     # 测试用例索引
│   ├── coverage.md       # 测试覆盖率报告
│   └── modules/          # 各模块测试文档
│       ├── transaction-manager.md
│       └── table-designer.md
└── README.md             # 本文件
```

## 后端测试

### 单元测试

位于源代码文件中的 `#[cfg(test)]` 模块。

**运行命令**:
```bash
cd src-tauri
cargo test --lib
```

### 集成测试

位于 `src-tauri/tests/test_*.rs` 文件。

**运行命令**:
```bash
cd src-tauri
cargo test --test test_transaction_manager
```

### 属性测试

位于 `src-tauri/tests/property_test_*.rs` 文件。

**运行命令**:
```bash
cd src-tauri
cargo test --test property_test_transaction_manager
```

**详细说明**: 参见 `test/backend/property/README.md`

## 前端测试

### 单元测试

使用 Vitest 框架，测试文件为 `*.spec.ts` 或 `*.test.ts`。

**运行命令**:
```bash
bun test
```

### 集成测试

测试组件间交互和状态管理。

**运行命令**:
```bash
bun test:integration
```

### E2E测试

使用 Playwright 进行端到端测试。

**运行命令**:
```bash
bun test:e2e
```

## 已实现的测试模块

### Transaction Manager（事务管理器）

- **单元测试**: 12个测试用例 ✅
- **集成测试**: 9个测试用例 ✅
- **属性测试**: 4个测试用例 ✅
- **文档**: `test/docs/modules/transaction-manager.md`

**测试覆盖率**:
- 行覆盖率: ~95%
- 分支覆盖率: ~90%
- 函数覆盖率: 100%

### Table Designer（表设计器）

- **前端单元测试**: 已实现 ✅
- **文档**: `test/docs/modules/table-designer.md`

## 测试规范

### 命名规范

- **前端测试文件**: `*.spec.ts` 或 `*.test.ts`
- **后端单元测试**: 在源文件中的 `#[cfg(test)] mod tests`
- **后端集成测试**: `test_*.rs`
- **后端属性测试**: `property_test_*.rs`

### 测试函数命名

- 使用描述性名称，清晰表达测试意图
- 前端: `test_<功能描述>` 或 `should_<预期行为>`
- 后端: `test_<功能描述>`
- 属性测试: `property_<属性名称>`

### 注释规范

所有测试文件应包含：
- 模块概述注释
- 验证的需求编号
- 对于属性测试，必须包含 Feature 和 Property 标签

## 运行所有测试

### 后端所有测试
```bash
cd src-tauri
cargo test
```

### 前端所有测试
```bash
bun test
```

### 完整测试套件
```bash
# 后端测试
cd src-tauri && cargo test

# 前端测试
cd .. && bun test
```

## 持续集成

测试在以下情况自动运行：
- 每次代码提交
- Pull Request创建时
- 合并到主分支前

## 测试文档

详细的测试文档位于 `test/docs/modules/` 目录，每个模块包含：
1. 模块概述
2. 测试用例列表
3. 测试覆盖率
4. 已知问题
5. 待补充测试

## 贡献指南

添加新测试时：
1. 遵循项目测试规范
2. 更新相应的测试文档
3. 确保所有测试通过
4. 更新测试覆盖率报告

## 参考资源

- [Rust测试文档](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Vitest文档](https://vitest.dev/)
- [Proptest文档](https://docs.rs/proptest/)
- [Playwright文档](https://playwright.dev/)
