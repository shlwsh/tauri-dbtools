/**
 * Property-Based Tests for Transaction Manager
 * 
 * 这些测试使用proptest库验证事务管理器的通用属性，特别是事务原子性。
 * 
 * Feature: database-advanced-features, Property 10: 事务原子性
 * 
 * Property 10: 事务原子性
 * For any 批量数据修改操作（多行UPDATE/INSERT/DELETE），事务管理器应该在单个事务中执行所有操作，
 * 并且如果任何操作失败，应该回滚所有更改，使数据库保持在操作前的状态
 * 
 * Validates: Requirements 10.2, 10.3, 16.1, 16.2
 */

use pg_db_tool::services::transaction_manager;
use pg_db_tool::models::data::RowUpdate;
use proptest::prelude::*;
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

/// 生成一组不重复ID的行更新
fn arbitrary_row_updates() -> impl Strategy<Value = Vec<RowUpdate>> {
    // 生成2-5个不重复的ID
    prop::collection::hash_set(1..=100i32, 2..=5)
        .prop_flat_map(|id_set| {
            let ids: Vec<i32> = id_set.into_iter().collect();
            let num_ids = ids.len();
            
            // 为每个ID生成更新数据
            prop::collection::vec(
                (1..=5usize).prop_flat_map(|num_changes| {
                    prop::collection::hash_map(
                        prop::sample::select(vec!["col1", "col2", "col3", "col4", "col5"]),
                        any::<i32>(),
                        1..=num_changes,
                    )
                }),
                num_ids..=num_ids
            ).prop_map(move |changes_list| {
                ids.iter().zip(changes_list.iter()).map(|(id, changes)| {
                    let changes_json: HashMap<String, serde_json::Value> = changes
                        .iter()
                        .map(|(k, v)| (k.to_string(), json!(v)))
                        .collect();
                    
                    RowUpdate {
                        primary_key: HashMap::from([("id".to_string(), json!(id))]),
                        changes: changes_json,
                    }
                }).collect()
            })
        })
}

/// 生成任意的行数据（用于插入）
/// 
/// 生成包含id和1-5个字段的行数据
fn arbitrary_row_data() -> impl Strategy<Value = HashMap<String, serde_json::Value>> {
    (1..=100i32, 1..=5usize).prop_flat_map(|(id, num_cols)| {
        prop::collection::hash_map(
            prop::sample::select(vec!["col1", "col2", "col3", "col4", "col5"]),
            any::<i32>(),
            1..=num_cols,
        ).prop_map(move |cols| {
            let mut row = HashMap::new();
            row.insert("id".to_string(), json!(id));
            for (k, v) in cols {
                row.insert(k.to_string(), json!(v));
            }
            row
        })
    })
}

/// 生成任意的主键（用于删除）
fn arbitrary_primary_key() -> impl Strategy<Value = HashMap<String, serde_json::Value>> {
    (1..=100i32).prop_map(|id| {
        HashMap::from([("id".to_string(), json!(id))])
    })
}

// Feature: database-advanced-features, Property 10: 事务原子性
// 测试批量更新的事务原子性：如果任何更新失败，所有更改应该被回滚
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn property_batch_update_atomicity(
        updates in arbitrary_row_updates()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let client = match get_test_client().await {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("无法连接到测试数据库: {}. 跳过测试", e);
                    return Ok(());
                }
            };

            // 创建测试表
            let table_name = format!("prop_test_update_{}", uuid::Uuid::new_v4().to_string().replace("-", "_"));
            let _ = client.execute(&format!("DROP TABLE IF EXISTS {}", table_name), &[]).await;
            let create_sql = format!(
                "CREATE TABLE {} (id INTEGER PRIMARY KEY, col1 INTEGER, col2 INTEGER, col3 INTEGER, col4 INTEGER, col5 INTEGER)",
                table_name
            );
            client.execute(&create_sql, &[]).await.unwrap();

            // 插入初始数据
            for update in &updates {
                let id: i32 = update.primary_key.get("id").unwrap().as_i64().unwrap() as i32;
                let insert_sql = format!(
                    "INSERT INTO {} (id, col1, col2, col3, col4, col5) VALUES ({}, 0, 0, 0, 0, 0)",
                    table_name, id
                );
                let _ = client.execute(&insert_sql, &[]).await;
            }

            // 获取原始数据
            let original_data = client.query(
                &format!("SELECT id, col1, col2, col3, col4, col5 FROM {} ORDER BY id", table_name),
                &[]
            ).await.unwrap();

            // 创建一个会失败的更新列表（在最后添加一个无效的更新）
            let mut failing_updates = updates.clone();
            let mut invalid_update = failing_updates.last().unwrap().clone();
            invalid_update.changes.insert("nonexistent_column".to_string(), json!(999));
            failing_updates.push(invalid_update);

            // 执行批量更新（应该失败）
            let result = transaction_manager::batch_update_rows(
                &client,
                "public",
                &table_name,
                failing_updates
            ).await;

            // 验证操作失败
            prop_assert!(!result.success, "批量更新应该失败");
            prop_assert!(result.error.is_some(), "应该有错误信息");

            // 验证所有数据都未被修改（事务已回滚）
            let current_data = client.query(
                &format!("SELECT id, col1, col2, col3, col4, col5 FROM {} ORDER BY id", table_name),
                &[]
            ).await.unwrap();

            prop_assert_eq!(
                current_data.len(),
                original_data.len(),
                "行数应该保持不变"
            );

            // 验证每一行的数据都未改变
            for (original_row, current_row) in original_data.iter().zip(current_data.iter()) {
                let original_id: i32 = original_row.get(0);
                let current_id: i32 = current_row.get(0);
                prop_assert_eq!(original_id, current_id, "ID应该保持不变");

                for col_idx in 1..6 {
                    let original_val: i32 = original_row.get(col_idx);
                    let current_val: i32 = current_row.get(col_idx);
                    prop_assert_eq!(
                        original_val,
                        current_val,
                        "列{}的值应该保持不变（事务已回滚）",
                        col_idx
                    );
                }
            }

            // 清理
            let _ = client.execute(&format!("DROP TABLE {}", table_name), &[]).await;

            Ok(())
        })?;
    }
}

// Feature: database-advanced-features, Property 10: 事务原子性
// 测试批量插入的事务原子性：如果任何插入失败，所有插入应该被回滚
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn property_batch_insert_atomicity(
        rows in prop::collection::vec(arbitrary_row_data(), 2..5)
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let client = match get_test_client().await {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("无法连接到测试数据库: {}. 跳过测试", e);
                    return Ok(());
                }
            };

            // 创建测试表
            let table_name = format!("prop_test_insert_{}", uuid::Uuid::new_v4().to_string().replace("-", "_"));
            let _ = client.execute(&format!("DROP TABLE IF EXISTS {}", table_name), &[]).await;
            let create_sql = format!(
                "CREATE TABLE {} (id INTEGER PRIMARY KEY, col1 INTEGER, col2 INTEGER, col3 INTEGER, col4 INTEGER, col5 INTEGER)",
                table_name
            );
            client.execute(&create_sql, &[]).await.unwrap();

            // 获取原始行数（应该为0）
            let original_count_rows = client.query(
                &format!("SELECT COUNT(*) FROM {}", table_name),
                &[]
            ).await.unwrap();
            let original_count: i64 = original_count_rows[0].get(0);

            // 创建一个会失败的插入列表（添加一个重复的ID）
            let mut failing_rows = rows.clone();
            if let Some(first_row) = failing_rows.first() {
                let mut duplicate_row = first_row.clone();
                // 确保ID相同但其他字段不同
                if let Some(col1) = duplicate_row.get_mut("col1") {
                    *col1 = json!(999);
                }
                failing_rows.push(duplicate_row);
            }

            // 执行批量插入（应该失败，因为有重复的主键）
            let result = transaction_manager::batch_insert_rows(
                &client,
                "public",
                &table_name,
                failing_rows
            ).await;

            // 验证操作失败
            prop_assert!(!result.success, "批量插入应该失败（重复主键）");
            prop_assert!(result.error.is_some(), "应该有错误信息");

            // 验证没有数据被插入（事务已回滚）
            let current_count_rows = client.query(
                &format!("SELECT COUNT(*) FROM {}", table_name),
                &[]
            ).await.unwrap();
            let current_count: i64 = current_count_rows[0].get(0);

            prop_assert_eq!(
                current_count,
                original_count,
                "表中不应该有任何数据（事务已回滚）"
            );

            // 清理
            let _ = client.execute(&format!("DROP TABLE {}", table_name), &[]).await;

            Ok(())
        })?;
    }
}

// Feature: database-advanced-features, Property 10: 事务原子性
// 测试批量删除的事务原子性：如果任何删除失败，所有删除应该被回滚
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn property_batch_delete_atomicity(
        primary_keys in prop::collection::vec(arbitrary_primary_key(), 2..5)
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let client = match get_test_client().await {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("无法连接到测试数据库: {}. 跳过测试", e);
                    return Ok(());
                }
            };

            // 创建测试表
            let table_name = format!("prop_test_delete_{}", uuid::Uuid::new_v4().to_string().replace("-", "_"));
            let _ = client.execute(&format!("DROP TABLE IF EXISTS {}", table_name), &[]).await;
            let create_sql = format!(
                "CREATE TABLE {} (id INTEGER PRIMARY KEY, data VARCHAR(100))",
                table_name
            );
            client.execute(&create_sql, &[]).await.unwrap();

            // 插入测试数据
            for pk in &primary_keys {
                let id: i32 = pk.get("id").unwrap().as_i64().unwrap() as i32;
                let insert_sql = format!(
                    "INSERT INTO {} (id, data) VALUES ({}, 'test_data_{}')",
                    table_name, id, id
                );
                let _ = client.execute(&insert_sql, &[]).await;
            }

            // 创建一个引用表来建立外键约束
            let ref_table_name = format!("{}_ref", table_name);
            let create_ref_sql = format!(
                "CREATE TABLE {} (ref_id INTEGER PRIMARY KEY, parent_id INTEGER REFERENCES {}(id))",
                ref_table_name, table_name
            );
            client.execute(&create_ref_sql, &[]).await.unwrap();

            // 为第一个ID创建外键引用，使其无法删除
            if let Some(first_pk) = primary_keys.first() {
                let id: i32 = first_pk.get("id").unwrap().as_i64().unwrap() as i32;
                let insert_ref_sql = format!(
                    "INSERT INTO {} (ref_id, parent_id) VALUES (1, {})",
                    ref_table_name, id
                );
                let _ = client.execute(&insert_ref_sql, &[]).await;
            }

            // 获取原始数据
            let original_data = client.query(
                &format!("SELECT id, data FROM {} ORDER BY id", table_name),
                &[]
            ).await.unwrap();

            // 执行批量删除（应该失败，因为第一个ID有外键引用）
            let result = transaction_manager::batch_delete_rows(
                &client,
                "public",
                &table_name,
                primary_keys.clone()
            ).await;

            // 验证操作失败
            prop_assert!(!result.success, "批量删除应该失败（外键约束）");
            prop_assert!(result.error.is_some(), "应该有错误信息");

            // 验证所有数据都未被删除（事务已回滚）
            let current_data = client.query(
                &format!("SELECT id, data FROM {} ORDER BY id", table_name),
                &[]
            ).await.unwrap();

            prop_assert_eq!(
                current_data.len(),
                original_data.len(),
                "所有行应该仍然存在（事务已回滚）"
            );

            // 验证每一行的数据都未改变
            for (original_row, current_row) in original_data.iter().zip(current_data.iter()) {
                let original_id: i32 = original_row.get(0);
                let current_id: i32 = current_row.get(0);
                prop_assert_eq!(original_id, current_id, "ID应该保持不变");

                let original_data: String = original_row.get(1);
                let current_data: String = current_row.get(1);
                prop_assert_eq!(original_data, current_data, "数据应该保持不变");
            }

            // 清理
            let _ = client.execute(&format!("DROP TABLE {} CASCADE", table_name), &[]).await;

            Ok(())
        })?;
    }
}

// Feature: database-advanced-features, Property 10: 事务原子性
// 测试成功的批量操作确实会提交所有更改
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn property_batch_update_success_commits_all(
        updates in arbitrary_row_updates()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let client = match get_test_client().await {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("无法连接到测试数据库: {}. 跳过测试", e);
                    return Ok(());
                }
            };

            // 创建测试表
            let table_name = format!("prop_test_success_{}", uuid::Uuid::new_v4().to_string().replace("-", "_"));
            let _ = client.execute(&format!("DROP TABLE IF EXISTS {}", table_name), &[]).await;
            let create_sql = format!(
                "CREATE TABLE {} (id INTEGER PRIMARY KEY, col1 INTEGER, col2 INTEGER, col3 INTEGER, col4 INTEGER, col5 INTEGER)",
                table_name
            );
            client.execute(&create_sql, &[]).await.unwrap();

            // 插入初始数据
            for update in &updates {
                let id: i32 = update.primary_key.get("id").unwrap().as_i64().unwrap() as i32;
                let insert_sql = format!(
                    "INSERT INTO {} (id, col1, col2, col3, col4, col5) VALUES ({}, 0, 0, 0, 0, 0)",
                    table_name, id
                );
                let _ = client.execute(&insert_sql, &[]).await;
            }

            // 执行批量更新（应该成功）
            let result = transaction_manager::batch_update_rows(
                &client,
                "public",
                &table_name,
                updates.clone()
            ).await;

            // 验证操作成功
            prop_assert!(result.success, "批量更新应该成功");
            prop_assert!(result.error.is_none(), "不应该有错误");
            prop_assert_eq!(
                result.rows_affected as usize,
                updates.len(),
                "应该影响正确数量的行"
            );

            // 验证所有更改都已提交
            for update in &updates {
                let id: i32 = update.primary_key.get("id").unwrap().as_i64().unwrap() as i32;
                let rows = client.query(
                    &format!("SELECT col1, col2, col3, col4, col5 FROM {} WHERE id = {}", table_name, id),
                    &[]
                ).await.unwrap();

                prop_assert_eq!(rows.len(), 1, "应该找到对应的行");

                // 验证每个更改的字段都已更新
                for (col_name, expected_value) in &update.changes {
                    let col_idx = match col_name.as_str() {
                        "col1" => 0,
                        "col2" => 1,
                        "col3" => 2,
                        "col4" => 3,
                        "col5" => 4,
                        _ => continue,
                    };

                    let actual_value: i32 = rows[0].get(col_idx);
                    let expected: i32 = expected_value.as_i64().unwrap() as i32;
                    prop_assert_eq!(
                        actual_value,
                        expected,
                        "字段{}应该被更新为正确的值",
                        col_name
                    );
                }
            }

            // 清理
            let _ = client.execute(&format!("DROP TABLE {}", table_name), &[]).await;

            Ok(())
        })?;
    }
}
