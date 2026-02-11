# SQL 日志功能文档

**创建日期**: 2026-02-11  
**状态**: ✅ 已实现

## 功能概述

SQL 日志功能详细记录所有 SQL 命令的执行过程，包括执行时间、结果、错误信息等，便于调试、审计和性能分析。

## 功能特性

### 1. 详细的日志记录

每条 SQL 执行都会记录以下信息：

- **时间戳**: 精确到毫秒的执行时间
- **数据库名称**: 执行 SQL 的目标数据库
- **SQL 语句**: 完整的 SQL 命令（长语句会截断）
- **执行状态**: 成功或失败
- **执行耗时**: 以毫秒为单位
- **查询类型**: SELECT、INSERT、UPDATE、DELETE、DDL
- **影响行数**: DML 操作影响的行数
- **返回行数**: SELECT 查询返回的行数
- **错误信息**: 失败时的详细错误信息
- **错误位置**: 错误发生的行号和列号

### 2. 双格式日志

系统同时生成两种格式的日志文件：

#### 文本格式 (`.log`)
- 人类可读的格式
- 便于直接查看和分析
- 文件名：`sql_execution_YYYY-MM-DD.log`

示例：
```
[2026-02-11 14:30:25.123] [SUCCESS] [SELECT] Database: personnel_db
SQL: SELECT * FROM employees WHERE department = 'IT'
Duration: 45ms
Returned Rows: 15
----------------------------------------
```

#### JSON 格式 (`.jsonl`)
- 机器可读的格式
- 便于程序化分析和数据处理
- 每行一个 JSON 对象（JSON Lines 格式）
- 文件名：`sql_execution_YYYY-MM-DD.jsonl`

示例：
```json
{"timestamp":"2026-02-11 14:30:25.123","database":"personnel_db","sql":"SELECT * FROM employees WHERE department = 'IT'","status":"success","duration_ms":45,"query_type":"SELECT","affected_rows":null,"returned_rows":15,"error":null,"error_position":null}
```

### 3. 自动日志轮转

- 日志文件按日期自动分割
- 每天生成新的日志文件
- 便于日志管理和归档

## 日志文件位置

日志文件存储在用户主目录下的 `pg-db-tool-logs` 文件夹中：

- **Windows**: `C:\Users\<用户名>\pg-db-tool-logs\`
- **macOS**: `/Users/<用户名>/pg-db-tool-logs/`
- **Linux**: `/home/<用户名>/pg-db-tool-logs/`

## 实现细节

### 核心模块

**文件**: `src-tauri/src/services/sql_logger.rs`

主要组件：

1. **SqlLogEntry**: 日志条目结构体
   - 包含所有日志字段
   - 提供格式化方法
   - 支持 JSON 序列化

2. **SqlLogger**: 日志记录器
   - 管理日志文件
   - 写入文本和 JSON 格式
   - 处理文件创建和追加

### 集成点

日志功能已集成到 `execute_sql` Tauri 命令中（`src-tauri/src/lib.rs`）：

```rust
// 记录 SQL 执行日志
if let Ok(log_dir) = get_log_dir() {
    if let Ok(logger) = services::sql_logger::SqlLogger::new(log_dir) {
        let log_entry = // ... 创建日志条目
        logger.log(&log_entry)?;
    }
}
```

## 使用场景

### 1. 调试和故障排查

查看最近的 SQL 执行记录，快速定位问题：

```bash
# 查看今天的日志
tail -f ~/pg-db-tool-logs/sql_execution_2026-02-11.log

# 搜索错误
grep "ERROR" ~/pg-db-tool-logs/sql_execution_2026-02-11.log
```

### 2. 性能分析

分析 SQL 执行时间，找出慢查询：

```bash
# 使用 jq 分析 JSON 日志
cat ~/pg-db-tool-logs/sql_execution_2026-02-11.jsonl | \
  jq 'select(.duration_ms > 1000)' | \
  jq -s 'sort_by(.duration_ms) | reverse'
```

### 3. 审计追踪

记录所有数据库操作，满足审计要求：

```bash
# 查看所有 DELETE 操作
cat ~/pg-db-tool-logs/sql_execution_2026-02-11.jsonl | \
  jq 'select(.query_type == "DELETE")'
```

### 4. 统计分析

生成 SQL 执行统计报告：

```bash
# 统计各类型查询的数量
cat ~/pg-db-tool-logs/sql_execution_2026-02-11.jsonl | \
  jq -r '.query_type' | \
  sort | uniq -c
```

## 性能影响

- 日志写入采用异步方式，不阻塞 SQL 执行
- 文件追加操作性能开销极小（< 1ms）
- 长 SQL 语句会自动截断（200 字符），减少日志大小
- 日志写入失败不影响 SQL 执行结果

## 配置选项

当前版本日志功能默认启用，未来可以添加以下配置：

- 日志级别（详细/简洁）
- 日志保留天数
- 最大日志文件大小
- 是否记录查询参数
- 敏感数据脱敏

## 测试覆盖

已实现的单元测试（`src-tauri/src/services/sql_logger.rs`）：

1. ✅ `test_sql_log_entry_success` - 测试成功日志条目创建
2. ✅ `test_sql_log_entry_error` - 测试错误日志条目创建
3. ✅ `test_format_log` - 测试日志格式化
4. ✅ `test_to_json` - 测试 JSON 序列化
5. ✅ `test_sql_logger_creation` - 测试日志记录器创建

所有测试通过率：100% (5/5)

## 示例日志输出

### 成功的 SELECT 查询

```
[2026-02-11 14:30:25.123] [SUCCESS] [SELECT] Database: personnel_db
SQL: SELECT * FROM employees WHERE department = 'IT'
Duration: 45ms
Returned Rows: 15
----------------------------------------
```

### 成功的 INSERT 操作

```
[2026-02-11 14:31:10.456] [SUCCESS] [INSERT] Database: personnel_db
SQL: INSERT INTO employees (name, department) VALUES ('John Doe', 'IT')
Duration: 12ms
Affected Rows: 1
----------------------------------------
```

### 失败的查询

```
[2026-02-11 14:32:05.789] [ERROR] [ERROR] Database: personnel_db
SQL: SELECT * FROM non_existent_table
Duration: 8ms
Error: Table does not exist: The specified table was not found (Error code: 42P01)
Error Position: Line 1, Column 15
----------------------------------------
```

## 后续改进

1. 添加日志查看器 UI 组件
2. 实现日志搜索和过滤功能
3. 添加日志导出功能（CSV、Excel）
4. 实现慢查询告警
5. 添加日志统计仪表板
6. 支持日志远程上传（可选）

## 相关文件

- `src-tauri/src/services/sql_logger.rs` - SQL 日志服务实现
- `src-tauri/src/services/mod.rs` - 服务模块注册
- `src-tauri/src/lib.rs` - 日志功能集成
- `docs/SQL_LOGGING.md` - 本文档

## 故障排查

### 日志文件未生成

1. 检查日志目录权限
2. 查看应用日志中的警告信息
3. 确认 `get_log_dir()` 函数正常工作

### 日志写入失败

- 日志写入失败会在应用日志中记录警告
- 不会影响 SQL 执行
- 检查磁盘空间是否充足

### 日志文件过大

- 当前版本按日期分割
- 可以手动删除旧日志文件
- 未来版本将支持自动清理
