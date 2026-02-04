# 数据库导出导入功能测试指南

## 当前状态
- ✅ 纯 Rust 实现，无需外部工具
- ✅ 配置文件支持
- ✅ 详细日志记录
- ⚠️ 需要测试验证

## 配置

配置文件：`config.json`
```json
{
  "database": {
    "host": "localhost",
    "port": "5432",
    "user": "postgres",
    "password": "postgres",
    "default_database": "personnel_db"
  }
}
```

## 测试步骤

### 1. 导出测试
1. 启动应用：`bun run dev`
2. 在界面中选择数据库 `personnel_db`
3. 点击"Export Database"
4. 查看终端日志，应该看到：
   ```
   [INFO] ========== Starting database export (Pure Rust) ==========
   [INFO] Database: personnel_db
   [INFO] Exporting table: xxx
   ...
   [INFO] ========== Export completed ==========
   ```
5. 检查导出文件：`~/pg-db-tool-exports/personnel_db_YYYYMMDD_HHMMSS.sql.gz`

### 2. 导入测试
1. 在界面中输入新数据库名：`p14`
2. 选择刚才导出的文件
3. 点击"Import Database"
4. 查看终端日志，注意：
   - 成功执行的语句数
   - 失败的语句数和错误信息

### 3. 验证结果
使用 psql 验证：
```bash
# 连接到新数据库
psql -h localhost -U postgres -d p14

# 检查表
\dt

# 检查数据
SELECT COUNT(*) FROM <table_name>;
```

## 已知问题和解决方案

### 问题1：部分 INSERT 语句失败
**原因**：数据类型转换不完整
**解决方案**：需要改进 `format_sql_value` 函数，支持更多数据类型

### 问题2：CREATE TABLE 语句不完整
**原因**：缺少约束、索引、序列等
**解决方案**：使用更完整的 DDL 生成逻辑

### 问题3：大数据库导出慢
**原因**：逐行处理
**解决方案**：使用批量处理或 COPY 命令

## 调试技巧

### 查看详细日志
日志文件位置：`~/pg-db-tool-logs/pg-db-tool_YYYYMMDD.log`

### 查看导出的 SQL
```powershell
# 解压并查看
$reader = [System.IO.File]::OpenRead("path/to/file.sql.gz")
$gzip = New-Object System.IO.Compression.GZipStream($reader, [System.IO.Compression.CompressionMode]::Decompress)
$streamReader = New-Object System.IO.StreamReader($gzip)
$content = $streamReader.ReadToEnd()
$streamReader.Close()
Write-Host $content
```

### 手动测试 SQL
将导出的 SQL 解压后，手动执行：
```bash
gunzip personnel_db_20260204_142012.sql.gz
psql -h localhost -U postgres -d p14 -f personnel_db_20260204_142012.sql
```

## 需要改进的地方

1. **数据类型支持**
   - [ ] TIMESTAMP
   - [ ] DATE
   - [ ] JSON/JSONB
   - [ ] ARRAY
   - [ ] BYTEA

2. **DDL 完整性**
   - [ ] 主键
   - [ ] 外键
   - [ ] 索引
   - [ ] 序列
   - [ ] 触发器

3. **性能优化**
   - [ ] 批量插入
   - [ ] 使用 COPY 命令
   - [ ] 并行处理

4. **错误处理**
   - [ ] 更详细的错误信息
   - [ ] 失败重试机制
   - [ ] 部分成功处理

## 下一步

请按照测试步骤操作，并将以下信息反馈：
1. 导出是否成功？
2. 导入是否成功？
3. 终端日志中的错误信息
4. 数据是否完整？

根据反馈，我将进一步优化代码。
