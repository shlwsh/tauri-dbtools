/**
 * Transaction Manager Service
 * 
 * 此模块提供数据库事务管理功能，包括：
 * - 批量更新行（batch_update_rows）
 * - 批量插入行（batch_insert_rows）
 * - 批量删除行（batch_delete_rows）
 * - 所有操作在单个事务中执行
 * - 失败时自动回滚
 * 
 * Validates: Requirements 10.2, 10.3, 16.1, 16.2
 */

use crate::models::data::{RowUpdate, BatchOperationResponse};
use std::collections::HashMap;
use tokio_postgres::Client;

/// 批量更新多行数据
/// 
/// 在单个事务中执行多个UPDATE操作。如果任何操作失败，所有更改将被回滚。
/// 
/// # Arguments
/// * `client` - PostgreSQL客户端连接
/// * `schema` - 模式名称
/// * `table` - 表名称
/// * `updates` - 要更新的行数组，每个包含主键和更改的字段
/// 
/// # Returns
/// * `BatchOperationResponse` - 包含成功状态、受影响行数或错误信息
/// 
/// # Example
/// ```rust
/// let updates = vec![
///     RowUpdate {
///         primary_key: HashMap::from([("id".to_string(), json!(1))]),
///         changes: HashMap::from([("name".to_string(), json!("Alice"))]),
///     },
/// ];
/// let result = batch_update_rows(&client, "public", "users", updates).await;
/// ```
pub async fn batch_update_rows(
    client: &Client,
    schema: &str,
    table: &str,
    updates: Vec<RowUpdate>,
) -> BatchOperationResponse {
    if updates.is_empty() {
        return BatchOperationResponse::error("没有要更新的行".to_string());
    }

    log::info!("========== 批量更新行 ==========");
    log::info!("表: {}.{}, 更新数量: {}", schema, table, updates.len());

    // 开始事务
    match client.query("BEGIN", &[]).await {
        Ok(_) => {
            log::info!("事务已开始");
        }
        Err(e) => {
            let error_msg = format!("无法开始事务: {}", e);
            log::error!("{}", error_msg);
            return BatchOperationResponse::error(error_msg);
        }
    };

    let mut total_affected = 0u64;

    // 执行每个更新操作
    for (index, update) in updates.iter().enumerate() {
        log::debug!("执行更新 {}/{}", index + 1, updates.len());

        // 构建UPDATE语句
        let sql = match build_update_statement(schema, table, update) {
            Ok(sql) => sql,
            Err(e) => {
                // 回滚事务
                let _ = client.query("ROLLBACK", &[]).await;
                log::error!("构建UPDATE语句失败: {}", e);
                return BatchOperationResponse::error(format!("构建UPDATE语句失败: {}", e));
            }
        };

        log::debug!("SQL: {}", sql);

        // 执行UPDATE
        match client.execute(&sql, &[]).await {
            Ok(affected) => {
                total_affected += affected;
                log::debug!("更新 {} 成功，影响 {} 行", index + 1, affected);
            }
            Err(e) => {
                // 回滚事务
                let _ = client.query("ROLLBACK", &[]).await;
                let error_msg = format!("更新操作 {} 失败: {}. 所有更改已回滚", index + 1, e);
                log::error!("{}", error_msg);
                return BatchOperationResponse::error(error_msg);
            }
        }
    }

    // 提交事务
    match client.query("COMMIT", &[]).await {
        Ok(_) => {
            log::info!("事务已提交，总共影响 {} 行", total_affected);
            BatchOperationResponse::success(total_affected)
        }
        Err(e) => {
            // 尝试回滚
            let _ = client.query("ROLLBACK", &[]).await;
            let error_msg = format!("提交事务失败: {}. 所有更改已回滚", e);
            log::error!("{}", error_msg);
            BatchOperationResponse::error(error_msg)
        }
    }
}

/// 批量插入多行数据
/// 
/// 在单个事务中执行多个INSERT操作。如果任何操作失败，所有更改将被回滚。
/// 
/// # Arguments
/// * `client` - PostgreSQL客户端连接
/// * `schema` - 模式名称
/// * `table` - 表名称
/// * `rows` - 要插入的行数组，每个是列名到值的映射
/// 
/// # Returns
/// * `BatchOperationResponse` - 包含成功状态、受影响行数或错误信息
/// 
/// # Example
/// ```rust
/// let rows = vec![
///     HashMap::from([
///         ("id".to_string(), json!(1)),
///         ("name".to_string(), json!("Alice")),
///     ]),
/// ];
/// let result = batch_insert_rows(&client, "public", "users", rows).await;
/// ```
pub async fn batch_insert_rows(
    client: &Client,
    schema: &str,
    table: &str,
    rows: Vec<HashMap<String, serde_json::Value>>,
) -> BatchOperationResponse {
    if rows.is_empty() {
        return BatchOperationResponse::error("没有要插入的行".to_string());
    }

    log::info!("========== 批量插入行 ==========");
    log::info!("表: {}.{}, 插入数量: {}", schema, table, rows.len());

    // 开始事务
    match client.query("BEGIN", &[]).await {
        Ok(_) => {
            log::info!("事务已开始");
        }
        Err(e) => {
            let error_msg = format!("无法开始事务: {}", e);
            log::error!("{}", error_msg);
            return BatchOperationResponse::error(error_msg);
        }
    };

    let mut total_affected = 0u64;

    // 执行每个插入操作
    for (index, row) in rows.iter().enumerate() {
        log::debug!("执行插入 {}/{}", index + 1, rows.len());

        // 构建INSERT语句
        let sql = match build_insert_statement(schema, table, row) {
            Ok(sql) => sql,
            Err(e) => {
                // 回滚事务
                let _ = client.query("ROLLBACK", &[]).await;
                log::error!("构建INSERT语句失败: {}", e);
                return BatchOperationResponse::error(format!("构建INSERT语句失败: {}", e));
            }
        };

        log::debug!("SQL: {}", sql);

        // 执行INSERT
        match client.execute(&sql, &[]).await {
            Ok(affected) => {
                total_affected += affected;
                log::debug!("插入 {} 成功，影响 {} 行", index + 1, affected);
            }
            Err(e) => {
                // 回滚事务
                let _ = client.query("ROLLBACK", &[]).await;
                let error_msg = format!("插入操作 {} 失败: {}. 所有更改已回滚", index + 1, e);
                log::error!("{}", error_msg);
                return BatchOperationResponse::error(error_msg);
            }
        }
    }

    // 提交事务
    match client.query("COMMIT", &[]).await {
        Ok(_) => {
            log::info!("事务已提交，总共影响 {} 行", total_affected);
            BatchOperationResponse::success(total_affected)
        }
        Err(e) => {
            // 尝试回滚
            let _ = client.query("ROLLBACK", &[]).await;
            let error_msg = format!("提交事务失败: {}. 所有更改已回滚", e);
            log::error!("{}", error_msg);
            BatchOperationResponse::error(error_msg)
        }
    }
}

/// 批量删除多行数据
/// 
/// 在单个事务中执行多个DELETE操作。如果任何操作失败，所有更改将被回滚。
/// 
/// # Arguments
/// * `client` - PostgreSQL客户端连接
/// * `schema` - 模式名称
/// * `table` - 表名称
/// * `primary_keys` - 要删除的行的主键数组，每个是列名到值的映射
/// 
/// # Returns
/// * `BatchOperationResponse` - 包含成功状态、受影响行数或错误信息
/// 
/// # Example
/// ```rust
/// let primary_keys = vec![
///     HashMap::from([("id".to_string(), json!(1))]),
///     HashMap::from([("id".to_string(), json!(2))]),
/// ];
/// let result = batch_delete_rows(&client, "public", "users", primary_keys).await;
/// ```
pub async fn batch_delete_rows(
    client: &Client,
    schema: &str,
    table: &str,
    primary_keys: Vec<HashMap<String, serde_json::Value>>,
) -> BatchOperationResponse {
    if primary_keys.is_empty() {
        return BatchOperationResponse::error("没有要删除的行".to_string());
    }

    log::info!("========== 批量删除行 ==========");
    log::info!("表: {}.{}, 删除数量: {}", schema, table, primary_keys.len());

    // 开始事务
    match client.query("BEGIN", &[]).await {
        Ok(_) => {
            log::info!("事务已开始");
        }
        Err(e) => {
            let error_msg = format!("无法开始事务: {}", e);
            log::error!("{}", error_msg);
            return BatchOperationResponse::error(error_msg);
        }
    };

    let mut total_affected = 0u64;

    // 执行每个删除操作
    for (index, pk) in primary_keys.iter().enumerate() {
        log::debug!("执行删除 {}/{}", index + 1, primary_keys.len());

        // 构建DELETE语句
        let sql = match build_delete_statement(schema, table, pk) {
            Ok(sql) => sql,
            Err(e) => {
                // 回滚事务
                let _ = client.query("ROLLBACK", &[]).await;
                log::error!("构建DELETE语句失败: {}", e);
                return BatchOperationResponse::error(format!("构建DELETE语句失败: {}", e));
            }
        };

        log::debug!("SQL: {}", sql);

        // 执行DELETE
        match client.execute(&sql, &[]).await {
            Ok(affected) => {
                total_affected += affected;
                log::debug!("删除 {} 成功，影响 {} 行", index + 1, affected);
            }
            Err(e) => {
                // 回滚事务
                let _ = client.query("ROLLBACK", &[]).await;
                let error_msg = format!("删除操作 {} 失败: {}. 所有更改已回滚", index + 1, e);
                log::error!("{}", error_msg);
                return BatchOperationResponse::error(error_msg);
            }
        }
    }

    // 提交事务
    match client.query("COMMIT", &[]).await {
        Ok(_) => {
            log::info!("事务已提交，总共影响 {} 行", total_affected);
            BatchOperationResponse::success(total_affected)
        }
        Err(e) => {
            // 尝试回滚
            let _ = client.query("ROLLBACK", &[]).await;
            let error_msg = format!("提交事务失败: {}. 所有更改已回滚", e);
            log::error!("{}", error_msg);
            BatchOperationResponse::error(error_msg)
        }
    }
}

/// 构建UPDATE语句
/// 
/// 根据RowUpdate生成SQL UPDATE语句
fn build_update_statement(
    schema: &str,
    table: &str,
    update: &RowUpdate,
) -> Result<String, String> {
    if update.changes.is_empty() {
        return Err("没有要更新的字段".to_string());
    }

    if update.primary_key.is_empty() {
        return Err("主键不能为空".to_string());
    }

    // 构建SET子句
    let set_clauses: Vec<String> = update
        .changes
        .iter()
        .map(|(col, val)| format!("{} = {}", col, format_value(val)))
        .collect();

    // 构建WHERE子句
    let where_clauses: Vec<String> = update
        .primary_key
        .iter()
        .map(|(col, val)| format!("{} = {}", col, format_value(val)))
        .collect();

    Ok(format!(
        "UPDATE {}.{} SET {} WHERE {}",
        schema,
        table,
        set_clauses.join(", "),
        where_clauses.join(" AND ")
    ))
}

/// 构建INSERT语句
/// 
/// 根据行数据生成SQL INSERT语句
fn build_insert_statement(
    schema: &str,
    table: &str,
    row: &HashMap<String, serde_json::Value>,
) -> Result<String, String> {
    if row.is_empty() {
        return Err("没有要插入的数据".to_string());
    }

    let columns: Vec<String> = row.keys().cloned().collect();
    let values: Vec<String> = row.values().map(format_value).collect();

    Ok(format!(
        "INSERT INTO {}.{} ({}) VALUES ({})",
        schema,
        table,
        columns.join(", "),
        values.join(", ")
    ))
}

/// 构建DELETE语句
/// 
/// 根据主键生成SQL DELETE语句
fn build_delete_statement(
    schema: &str,
    table: &str,
    primary_key: &HashMap<String, serde_json::Value>,
) -> Result<String, String> {
    if primary_key.is_empty() {
        return Err("主键不能为空".to_string());
    }

    // 构建WHERE子句
    let where_clauses: Vec<String> = primary_key
        .iter()
        .map(|(col, val)| format!("{} = {}", col, format_value(val)))
        .collect();

    Ok(format!(
        "DELETE FROM {}.{} WHERE {}",
        schema,
        table,
        where_clauses.join(" AND ")
    ))
}

/// 格式化JSON值为SQL字符串
/// 
/// 将serde_json::Value转换为适合SQL语句的字符串表示
fn format_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "NULL".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => {
            // 转义单引号
            let escaped = s.replace("'", "''");
            format!("'{}'", escaped)
        }
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            // 对于复杂类型，转换为JSON字符串
            let json_str = value.to_string().replace("'", "''");
            format!("'{}'", json_str)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_format_value_null() {
        assert_eq!(format_value(&json!(null)), "NULL");
    }

    #[test]
    fn test_format_value_bool() {
        assert_eq!(format_value(&json!(true)), "true");
        assert_eq!(format_value(&json!(false)), "false");
    }

    #[test]
    fn test_format_value_number() {
        assert_eq!(format_value(&json!(42)), "42");
        assert_eq!(format_value(&json!(3.14)), "3.14");
    }

    #[test]
    fn test_format_value_string() {
        assert_eq!(format_value(&json!("hello")), "'hello'");
        assert_eq!(format_value(&json!("O'Brien")), "'O''Brien'");
    }

    #[test]
    fn test_build_update_statement() {
        let mut primary_key = HashMap::new();
        primary_key.insert("id".to_string(), json!(1));

        let mut changes = HashMap::new();
        changes.insert("name".to_string(), json!("Alice"));
        changes.insert("age".to_string(), json!(30));

        let update = RowUpdate {
            primary_key,
            changes,
        };

        let sql = build_update_statement("public", "users", &update).unwrap();
        
        // 由于HashMap的顺序不确定，我们检查SQL包含所有必要部分
        assert!(sql.starts_with("UPDATE public.users SET "));
        assert!(sql.contains("name = 'Alice'"));
        assert!(sql.contains("age = 30"));
        assert!(sql.contains("WHERE id = 1"));
    }

    #[test]
    fn test_build_update_statement_empty_changes() {
        let mut primary_key = HashMap::new();
        primary_key.insert("id".to_string(), json!(1));

        let update = RowUpdate {
            primary_key,
            changes: HashMap::new(),
        };

        let result = build_update_statement("public", "users", &update);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "没有要更新的字段");
    }

    #[test]
    fn test_build_update_statement_empty_primary_key() {
        let mut changes = HashMap::new();
        changes.insert("name".to_string(), json!("Alice"));

        let update = RowUpdate {
            primary_key: HashMap::new(),
            changes,
        };

        let result = build_update_statement("public", "users", &update);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "主键不能为空");
    }

    #[test]
    fn test_build_insert_statement() {
        let mut row = HashMap::new();
        row.insert("id".to_string(), json!(1));
        row.insert("name".to_string(), json!("Alice"));
        row.insert("age".to_string(), json!(30));

        let sql = build_insert_statement("public", "users", &row).unwrap();
        
        assert!(sql.starts_with("INSERT INTO public.users ("));
        assert!(sql.contains("id"));
        assert!(sql.contains("name"));
        assert!(sql.contains("age"));
        assert!(sql.contains("VALUES ("));
        assert!(sql.contains("1"));
        assert!(sql.contains("'Alice'"));
        assert!(sql.contains("30"));
    }

    #[test]
    fn test_build_insert_statement_empty_row() {
        let row = HashMap::new();
        let result = build_insert_statement("public", "users", &row);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "没有要插入的数据");
    }

    #[test]
    fn test_build_delete_statement() {
        let mut primary_key = HashMap::new();
        primary_key.insert("id".to_string(), json!(1));

        let sql = build_delete_statement("public", "users", &primary_key).unwrap();
        assert_eq!(sql, "DELETE FROM public.users WHERE id = 1");
    }

    #[test]
    fn test_build_delete_statement_composite_key() {
        let mut primary_key = HashMap::new();
        primary_key.insert("user_id".to_string(), json!(1));
        primary_key.insert("role_id".to_string(), json!(2));

        let sql = build_delete_statement("public", "user_roles", &primary_key).unwrap();
        
        // 由于HashMap的顺序不确定，我们检查SQL包含所有必要部分
        assert!(sql.starts_with("DELETE FROM public.user_roles WHERE "));
        assert!(sql.contains("user_id = 1"));
        assert!(sql.contains("role_id = 2"));
        assert!(sql.contains(" AND "));
    }

    #[test]
    fn test_build_delete_statement_empty_primary_key() {
        let primary_key = HashMap::new();
        let result = build_delete_statement("public", "users", &primary_key);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "主键不能为空");
    }
}
