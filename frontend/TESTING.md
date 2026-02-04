# 测试文档

## 测试概述

本项目使用 Bun 的内置测试运行器和 fast-check 进行属性测试，确保代码质量和正确性。

## 测试统计

- **总测试数**: 56
- **通过**: 56
- **失败**: 0
- **测试覆盖率**: ~60% (目标: >80%)

## 测试类型

### 1. 单元测试

单元测试验证特定功能、边缘情况和错误条件。

#### Store 测试
- **Theme Store** (5 tests)
  - 主题初始化
  - 主题切换
  - 主题设置
  - 主题保存到 localStorage
  - 主题从 localStorage 加载

- **Config Store** (8 tests)
  - 配置初始化
  - 添加连接
  - 更新连接
  - 删除连接
  - 设置默认连接
  - 获取默认连接
  - 通过 ID 获取连接
  - 删除时清除默认连接

- **Database Store** (6 tests)
  - 状态初始化
  - 设置当前连接
  - 切换连接时清除数据库
  - 设置当前数据库
  - 设置数据库列表
  - 设置加载状态

#### Utils 测试
- **Storage Utils** (13 tests)
  - 获取默认配置
  - 配置验证（7个测试）
  - 从 localStorage 加载配置（4个测试）
  - 保存配置到 localStorage（2个测试）

- **Format Utils** (12 tests)
  - 文件大小格式化（6个测试）
  - 日期格式化（2个测试）
  - 字符串截断（4个测试）

- **Validation Utils** (5 tests)
  - 连接配置验证规则（5个测试）

### 2. 属性测试 (Property-Based Tests)

属性测试验证跨所有输入的通用属性，使用 fast-check 生成随机测试数据。

#### 已实现的属性测试 (6/22)

1. **Property 2: 主题持久化往返**
   - 验证需求: 3.3, 3.4
   - 测试: 任何主题选择保存后再加载应得到相同设置

2. **Property 3: 配置 CRUD 操作一致性**
   - 验证需求: 6.2, 6.3, 6.4, 6.5
   - 测试: 配置的创建、更新、删除操作正确反映变化

3. **Property 5: 配置加载完整性**
   - 验证需求: 6.9, 11.4
   - 测试: 保存的配置与加载的配置完全一致

4. **Property 12: 错误处理一致性**
   - 验证需求: 10.1, 10.5
   - 测试: 无效配置被正确处理

5. **Property 15: 表单验证反馈**
   - 验证需求: 10.4
   - 测试: 无效输入被正确验证

6. **Property 16: 配置数据格式正确性**
   - 验证需求: 11.5
   - 测试: 保存的配置是有效的 JSON 格式

#### 待实现的属性测试 (16/22)

以下属性测试需要在后续完善：

- Property 1: API 兼容性
- Property 4: 默认连接传播
- Property 6: 数据库列表响应性
- Property 7: 导出操作完整性
- Property 8: 导入操作完整性
- Property 9: 表数据 CRUD 往返
- Property 10: 分页数据一致性
- Property 11: 路由导航正确性
- Property 13: 加载状态可见性
- Property 14: 成功反馈一致性
- Property 17-22: 后端 API 相关属性

## 运行测试

### 运行所有测试
```bash
bun test
```

### 监视模式
```bash
bun test --watch
```

### 测试覆盖率
```bash
bun test --coverage
```

## 测试环境配置

### 测试设置文件
- `test/setup.ts` - 全局测试设置
  - JSDOM 环境配置
  - localStorage mock
  - document mock
  - 其他 DOM API mocks

### Bun 配置
- `bunfig.toml` - Bun 测试配置
  - 预加载测试设置文件

## 测试最佳实践

### 单元测试
1. 每个测试应该独立且可重复
2. 使用描述性的测试名称
3. 测试一个功能点
4. 使用 beforeEach 清理状态

### 属性测试
1. 每个属性测试至少运行 100 次迭代
2. 使用 fast-check 生成随机测试数据
3. 每个测试必须引用设计文档中的属性
4. 使用标签格式: `// Feature: vue3-frontend-refactor, Property X: ...`

### 测试覆盖率目标
- 单元测试覆盖率: > 80%
- 关键路径覆盖率: 100%
- 所有设计文档中的属性都应该有对应的测试

## 测试文件结构

```
frontend/
├── src/
│   ├── __tests__/
│   │   └── properties.spec.ts      # 属性测试
│   ├── stores/
│   │   └── __tests__/
│   │       ├── theme.spec.ts       # Theme Store 测试
│   │       ├── config.spec.ts      # Config Store 测试
│   │       └── database.spec.ts    # Database Store 测试
│   └── utils/
│       └── __tests__/
│           ├── storage.spec.ts     # Storage Utils 测试
│           ├── format.spec.ts      # Format Utils 测试
│           └── validation.spec.ts  # Validation Utils 测试
├── test/
│   └── setup.ts                    # 测试环境设置
└── bunfig.toml                     # Bun 测试配置
```

## 下一步

### 短期目标
1. 添加 API 层测试
2. 添加 Composables 测试
3. 添加 Vue 组件测试（使用 @vue/test-utils）
4. 完善剩余的属性测试

### 中期目标
1. 提高测试覆盖率到 >80%
2. 添加集成测试
3. 添加端到端测试
4. 配置 CI/CD 测试流程

### 长期目标
1. 性能测试
2. 可访问性测试
3. 视觉回归测试
4. 负载测试

## 故障排除

### 常见问题

#### 1. localStorage is not defined
**解决方案**: 确保 `test/setup.ts` 被正确加载，检查 `bunfig.toml` 配置。

#### 2. document is not defined
**解决方案**: 确保 JSDOM 已安装并在 `test/setup.ts` 中正确配置。

#### 3. 测试超时
**解决方案**: 增加测试超时时间或优化测试代码。

#### 4. 属性测试失败
**解决方案**: 检查反例（counterexample），修复代码或调整测试策略。

## 参考资料

- [Bun Test Runner](https://bun.sh/docs/cli/test)
- [fast-check Documentation](https://fast-check.dev/)
- [Vue Test Utils](https://test-utils.vuejs.org/)
- [JSDOM](https://github.com/jsdom/jsdom)
