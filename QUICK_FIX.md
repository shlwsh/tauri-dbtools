# 快速修复建议

基于之前的日志 "Failed to execute statement: db error"，主要问题是：

## 1. 数据类型问题
当前代码只处理了基本类型，需要添加：
- TIMESTAMP/DATE → 需要引号
- BOOLEAN → true/false 不需要引号  
- NULL 值处理

## 2. SQL 语法问题
- 字符串中的特殊字符未转义
- 列名可能是保留字，需要用引号包裹

## 3. 建议的修复

### 修复导出函数中的数据格式化：

```rust
// 在 lib.rs 中，替换数据导出部分
for row in rows {
    let mut values = Vec::new();
    for i in 0..col_names.len() {
        let value = if row.try_get::<_, Option<bool>>(i).is_ok() {
            // 布尔值
            match row.try_get::<_, Option<bool>>(i).unwrap() {
                Some(v) => v.to_string(),
                None => "NULL".to_string(),
            }
        } else if row.try_get::<_, Option<i32>>(i).is_ok() {
            // 整数
            match row.try_get::<_, Option<i32>>(i).unwrap() {
                Some(v) => v.to_string(),
                None => "NULL".to_string(),
            }
        } else {
            // 字符串或其他类型
            match row.try_get::<_, Option<String>>(i) {
                Ok(Some(v)) => format!("'{}'", v.replace("'", "''")),
                Ok(None) => "NULL".to_string(),
                Err(_) => "NULL".to_string(),
            }
        };
        values.push(value);
    }
    
    writeln!(
        writer,
        "INSERT INTO \"{}\" ({}) VALUES ({});",
        table_name,
        col_names.iter().map(|c| format!("\"{}\"", c)).collect::<Vec<_>>().join(", "),
        values.join(", ")
    )?;
}
```

### 修复导入函数中的事务处理：

```rust
// 使用事务确保原子性
target_client.execute("BEGIN", &[]).await?;

for line in reader.lines() {
    // ... SQL 解析逻辑 ...
    
    if let Err(e) = target_client.execute(&sql_buffer, &[]).await {
        log::warn!("Failed: {} - SQL: {}", e, sql_buffer.trim());
        // 继续执行，不回滚
    }
}

target_client.execute("COMMIT", &[]).await?;
```

## 4. 测试命令

```powershell
# 1. 导出
# 在应用中导出 personnel_db

# 2. 查看导出文件
.\debug_export.ps1 -ExportFile "C:\Users\Administrator\pg-db-tool-exports\personnel_db_xxx.sql.gz"

# 3. 导入
# 在应用中导入到 p14

# 4. 验证
psql -h localhost -U postgres -d p14 -c "\dt"
psql -h localhost -U postgres -d personnel_db -c "SELECT COUNT(*) FROM <table>" 
psql -h localhost -U postgres -d p14 -c "SELECT COUNT(*) FROM <table>"
```

## 5. 如果仍有问题

请提供：
1. 导出文件的前100行（使用 debug_export.ps1）
2. 完整的错误日志
3. 失败的具体 SQL 语句

我将根据这些信息进一步优化代码。
