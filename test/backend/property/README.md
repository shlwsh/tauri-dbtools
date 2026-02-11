# 后端属性测试

## 概述

属性测试（Property-Based Testing）使用 `proptest` 库验证代码的通用属性，通过生成大量随机输入来发现边界情况和潜在bug。

## 测试文件位置

根据Rust项目标准，所有属性测试文件位于：

```
src-tauri/tests/property_test_*.rs
```

## 已实现的属性测试

### Transaction Manager 属性测试

**文件**: `src-tauri/tests/property_test_transaction_manager.rs`

**测试用例**:
1. `property_batch_update_atomicity` - 批量更新事务原子性（失败回滚）
2. `property_batch_insert_atomicity` - 批量插入事务原子性（重复主键回滚）
3. `property_batch_delete_atomicity` - 批量删除事务原子性（外键约束回滚）
4. `property_batch_update_success_commits_all` - 批量更新成功提交所有更改

**验证属性**: Property 10 - 事务原子性

**运行次数**: 每个测试100次迭代

**详细文档**: 参见 `test/docs/modules/transaction-manager.md`

## 运行属性测试

### 运行所有属性测试
```bash
cd src-tauri
cargo test --test property_test_transaction_manager
```

### 运行特定属性测试
```bash
cd src-tauri
cargo test property_batch_update_atomicity
```

### 查看详细输出
```bash
cd src-tauri
cargo test --test property_test_transaction_manager -- --nocapture
```

## 属性测试配置

- **测试库**: proptest 1.4
- **迭代次数**: 100次（可通过 `ProptestConfig::with_cases()` 配置）
- **收缩策略**: 自动收缩失败用例以找到最小反例
- **失败持久化**: 失败用例保存在 `.proptest-regressions` 文件中

## 编写新的属性测试

1. 在 `src-tauri/tests/` 目录下创建 `property_test_<module>.rs` 文件
2. 使用 `proptest!` 宏定义属性测试
3. 使用策略（Strategy）生成测试数据
4. 验证属性在所有生成的输入上都成立
5. 添加注释标签引用设计文档中的属性

### 示例模板

```rust
use proptest::prelude::*;

// Feature: <feature-name>, Property <N>: <property-description>
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn property_<test_name>(
        input in <strategy>
    ) {
        // 测试逻辑
        prop_assert!(condition, "error message");
    }
}
```

## 参考资源

- [proptest 文档](https://docs.rs/proptest/)
- [Property-Based Testing 介绍](https://hypothesis.works/articles/what-is-property-based-testing/)
- 项目设计文档: `.kiro/specs/database-advanced-features/design.md`
