/**
 * Integration tests for Transaction Manager
 * 
 * 这些测试验证事务管理器在实际数据库操作中的正确性，包括：
 * - 批量更新操作
 * - 批量插入操作
 * - 批量删除操作
 * - 事务原子性（失败时回滚）
 * 
 * Validates: Requirements 10.2, 10.3, 16.1, 16.2
 */

use pg_db_tool::services::transaction_manager;
use pg_db_tool::models::data::RowUpdate;
use std::collections::HashMap;
use serde_json::json;

/// 获取测试数据库连接
async fn get_test_client() -> Result<tokio_postgres::Client, tokio_postgres::Error> {
    let host = std::env::var("PG_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = std::env::var("PG_PORT").unwrap_or_else(|_| "5432".to_string());
    let user = std::env::var("PG_USER").unwrap_or_else(|_| "postgres".to_string());
    let password = std::env::var("PG_PASSWORD").unwrap_or_else(|_| "postgres".to_string());
    let database = std::env::var("PG_DATABASE").unwrap_or_else(|_| "personnel_db".to_string());

    let connection_string = format!(
        "host={} port={} user={} password={} dbname={}",
        host, port, user, password, database
    );

    let (client, connection) = tokio_postgres::connect(&connection_string, tokio_postgres::NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    Ok(client)
}

#[tokio::test]
async fn test_batch_update_rows_success() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("无法连接到测试数据库: {}. 跳过测试", e);
            return;
        }
    };

    // 创建测试表
    let _ = client.execute("DROP TABLE IF EXISTS test_batch_update", &[]).await;
    let _ = client.execute(
        "CREATE TABLE test_batch_update (id INTEGER PRIMARY KEY, name VARCHAR(100), value INTEGER)",
        &[],
    ).await;

    // 插入测试数据
    let _ = client.execute(
        "INSERT INTO test_batch_update (id, name, value) VALUES (1, 'Alice', 100), (2, 'Bob', 200), (3, 'Charlie', 300)",
        &[],
    ).await;

    // 准备批量更新
    let updates = vec![
        RowUpdate {
            primary_key: HashMap::from([("id".to_string(), json!(1))]),
            changes: HashMap::from([("name".to_string(), json!("Alice Updated"))]),
        },
        RowUpdate {
            primary_key: HashMap::from([("id".to_string(), json!(2))]),
            changes: HashMap::from([("value".to_string(), json!(250))]),
        },
    ];

    // 执行批量更新
    let result = transaction_manager::batch_update_rows(&client, "public", "test_batch_update", updates).await;

    // 验证结果
    assert!(result.success, "批量更新应该成功");
    assert_eq!(result.rows_affected, 2, "应该影响2行");
    assert!(result.error.is_none(), "不应该有错误");

    // 验证数据已更新
    let rows = client.query("SELECT id, name, value FROM test_batch_update ORDER BY id", &[]).await.unwrap();
    assert_eq!(rows.len(), 3);
    
    let name1: String = rows[0].get(1);
    assert_eq!(name1, "Alice Updated");
    
    let value2: i32 = rows[1].get(2);
    assert_eq!(value2, 250);

    // 清理
    let _ = client.execute("DROP TABLE test_batch_update", &[]).await;
}

#[tokio::test]
async fn test_batch_update_rows_rollback_on_error() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("无法连接到测试数据库: {}. 跳过测试", e);
            return;
        }
    };

    // 创建测试表
    let _ = client.execute("DROP TABLE IF EXISTS test_batch_update_rollback", &[]).await;
    let _ = client.execute(
        "CREATE TABLE test_batch_update_rollback (id INTEGER PRIMARY KEY, name VARCHAR(100), value INTEGER)",
        &[],
    ).await;

    // 插入测试数据
    let _ = client.execute(
        "INSERT INTO test_batch_update_rollback (id, name, value) VALUES (1, 'Alice', 100), (2, 'Bob', 200)",
        &[],
    ).await;

    // 准备批量更新，第二个更新会失败（不存在的列）
    let updates = vec![
        RowUpdate {
            primary_key: HashMap::from([("id".to_string(), json!(1))]),
            changes: HashMap::from([("name".to_string(), json!("Alice Updated"))]),
        },
        RowUpdate {
            primary_key: HashMap::from([("id".to_string(), json!(2))]),
            changes: HashMap::from([("nonexistent_column".to_string(), json!(999))]),
        },
    ];

    // 执行批量更新
    let result = transaction_manager::batch_update_rows(&client, "public", "test_batch_update_rollback", updates).await;

    // 验证结果
    assert!(!result.success, "批量更新应该失败");
    assert!(result.error.is_some(), "应该有错误信息");

    // 验证数据未被修改（事务已回滚）
    let rows = client.query("SELECT id, name FROM test_batch_update_rollback ORDER BY id", &[]).await.unwrap();
    
    let name1: String = rows[0].get(1);
    assert_eq!(name1, "Alice", "第一行不应该被更新（事务已回滚）");

    // 清理
    let _ = client.execute("DROP TABLE test_batch_update_rollback", &[]).await;
}

#[tokio::test]
async fn test_batch_insert_rows_success() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("无法连接到测试数据库: {}. 跳过测试", e);
            return;
        }
    };

    // 创建测试表
    let _ = client.execute("DROP TABLE IF EXISTS test_batch_insert", &[]).await;
    let _ = client.execute(
        "CREATE TABLE test_batch_insert (id INTEGER PRIMARY KEY, name VARCHAR(100), value INTEGER)",
        &[],
    ).await;

    // 准备批量插入
    let rows = vec![
        HashMap::from([
            ("id".to_string(), json!(1)),
            ("name".to_string(), json!("Alice")),
            ("value".to_string(), json!(100)),
        ]),
        HashMap::from([
            ("id".to_string(), json!(2)),
            ("name".to_string(), json!("Bob")),
            ("value".to_string(), json!(200)),
        ]),
        HashMap::from([
            ("id".to_string(), json!(3)),
            ("name".to_string(), json!("Charlie")),
            ("value".to_string(), json!(300)),
        ]),
    ];

    // 执行批量插入
    let result = transaction_manager::batch_insert_rows(&client, "public", "test_batch_insert", rows).await;

    // 验证结果
    assert!(result.success, "批量插入应该成功");
    assert_eq!(result.rows_affected, 3, "应该影响3行");
    assert!(result.error.is_none(), "不应该有错误");

    // 验证数据已插入
    let rows = client.query("SELECT COUNT(*) FROM test_batch_insert", &[]).await.unwrap();
    let count: i64 = rows[0].get(0);
    assert_eq!(count, 3);

    // 清理
    let _ = client.execute("DROP TABLE test_batch_insert", &[]).await;
}

#[tokio::test]
async fn test_batch_insert_rows_rollback_on_duplicate() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("无法连接到测试数据库: {}. 跳过测试", e);
            return;
        }
    };

    // 创建测试表
    let _ = client.execute("DROP TABLE IF EXISTS test_batch_insert_rollback", &[]).await;
    let _ = client.execute(
        "CREATE TABLE test_batch_insert_rollback (id INTEGER PRIMARY KEY, name VARCHAR(100))",
        &[],
    ).await;

    // 准备批量插入，第二个插入会失败（重复的主键）
    let rows = vec![
        HashMap::from([
            ("id".to_string(), json!(1)),
            ("name".to_string(), json!("Alice")),
        ]),
        HashMap::from([
            ("id".to_string(), json!(1)),  // 重复的ID
            ("name".to_string(), json!("Bob")),
        ]),
    ];

    // 执行批量插入
    let result = transaction_manager::batch_insert_rows(&client, "public", "test_batch_insert_rollback", rows).await;

    // 验证结果
    assert!(!result.success, "批量插入应该失败");
    assert!(result.error.is_some(), "应该有错误信息");

    // 验证没有数据被插入（事务已回滚）
    let rows = client.query("SELECT COUNT(*) FROM test_batch_insert_rollback", &[]).await.unwrap();
    let count: i64 = rows[0].get(0);
    assert_eq!(count, 0, "不应该有任何数据被插入（事务已回滚）");

    // 清理
    let _ = client.execute("DROP TABLE test_batch_insert_rollback", &[]).await;
}

#[tokio::test]
async fn test_batch_delete_rows_success() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("无法连接到测试数据库: {}. 跳过测试", e);
            return;
        }
    };

    // 创建测试表
    let _ = client.execute("DROP TABLE IF EXISTS test_batch_delete", &[]).await;
    let _ = client.execute(
        "CREATE TABLE test_batch_delete (id INTEGER PRIMARY KEY, name VARCHAR(100))",
        &[],
    ).await;

    // 插入测试数据
    let _ = client.execute(
        "INSERT INTO test_batch_delete (id, name) VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie'), (4, 'David')",
        &[],
    ).await;

    // 准备批量删除
    let primary_keys = vec![
        HashMap::from([("id".to_string(), json!(1))]),
        HashMap::from([("id".to_string(), json!(3))]),
    ];

    // 执行批量删除
    let result = transaction_manager::batch_delete_rows(&client, "public", "test_batch_delete", primary_keys).await;

    // 验证结果
    assert!(result.success, "批量删除应该成功");
    assert_eq!(result.rows_affected, 2, "应该影响2行");
    assert!(result.error.is_none(), "不应该有错误");

    // 验证数据已删除
    let rows = client.query("SELECT COUNT(*) FROM test_batch_delete", &[]).await.unwrap();
    let count: i64 = rows[0].get(0);
    assert_eq!(count, 2, "应该剩余2行");

    // 验证正确的行被删除
    let rows = client.query("SELECT id FROM test_batch_delete ORDER BY id", &[]).await.unwrap();
    let id1: i32 = rows[0].get(0);
    let id2: i32 = rows[1].get(0);
    assert_eq!(id1, 2);
    assert_eq!(id2, 4);

    // 清理
    let _ = client.execute("DROP TABLE test_batch_delete", &[]).await;
}

#[tokio::test]
async fn test_batch_delete_rows_with_composite_key() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("无法连接到测试数据库: {}. 跳过测试", e);
            return;
        }
    };

    // 创建测试表（复合主键）
    let _ = client.execute("DROP TABLE IF EXISTS test_batch_delete_composite", &[]).await;
    let _ = client.execute(
        "CREATE TABLE test_batch_delete_composite (user_id INTEGER, role_id INTEGER, PRIMARY KEY (user_id, role_id))",
        &[],
    ).await;

    // 插入测试数据
    let _ = client.execute(
        "INSERT INTO test_batch_delete_composite (user_id, role_id) VALUES (1, 1), (1, 2), (2, 1), (2, 2)",
        &[],
    ).await;

    // 准备批量删除（复合主键）
    let primary_keys = vec![
        HashMap::from([
            ("user_id".to_string(), json!(1)),
            ("role_id".to_string(), json!(1)),
        ]),
        HashMap::from([
            ("user_id".to_string(), json!(2)),
            ("role_id".to_string(), json!(2)),
        ]),
    ];

    // 执行批量删除
    let result = transaction_manager::batch_delete_rows(&client, "public", "test_batch_delete_composite", primary_keys).await;

    // 验证结果
    assert!(result.success, "批量删除应该成功");
    assert_eq!(result.rows_affected, 2, "应该影响2行");

    // 验证数据已删除
    let rows = client.query("SELECT COUNT(*) FROM test_batch_delete_composite", &[]).await.unwrap();
    let count: i64 = rows[0].get(0);
    assert_eq!(count, 2, "应该剩余2行");

    // 清理
    let _ = client.execute("DROP TABLE test_batch_delete_composite", &[]).await;
}

#[tokio::test]
async fn test_empty_updates() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("无法连接到测试数据库: {}. 跳过测试", e);
            return;
        }
    };

    // 测试空的更新列表
    let result = transaction_manager::batch_update_rows(&client, "public", "test_table", vec![]).await;
    assert!(!result.success);
    assert!(result.error.is_some());
    assert_eq!(result.error.unwrap(), "没有要更新的行");
}

#[tokio::test]
async fn test_empty_inserts() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("无法连接到测试数据库: {}. 跳过测试", e);
            return;
        }
    };

    // 测试空的插入列表
    let result = transaction_manager::batch_insert_rows(&client, "public", "test_table", vec![]).await;
    assert!(!result.success);
    assert!(result.error.is_some());
    assert_eq!(result.error.unwrap(), "没有要插入的行");
}

#[tokio::test]
async fn test_empty_deletes() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("无法连接到测试数据库: {}. 跳过测试", e);
            return;
        }
    };

    // 测试空的删除列表
    let result = transaction_manager::batch_delete_rows(&client, "public", "test_table", vec![]).await;
    assert!(!result.success);
    assert!(result.error.is_some());
    assert_eq!(result.error.unwrap(), "没有要删除的行");
}
