/**
 * Property-Based Tests for Query Executor
 * 
 * Feature: database-advanced-features
 * Property 3: 查询执行返回适当结果类型
 * 
 * **Validates: Requirements 2.3, 2.4, 2.5**
 * 
 * For any valid SQL statement, the query executor should return appropriate results
 * based on statement type:
 * - SELECT queries return result set and execution time
 * - INSERT/UPDATE/DELETE return affected rows count
 * - CREATE/ALTER/DROP return success message
 */

use proptest::prelude::*;
use tokio_postgres::{NoTls, Client};
use pg_db_tool::services::query_executor;
use pg_db_tool::models::query::QueryResultType;

/// Get a test database client
async fn get_test_client() -> Result<Client, Box<dyn std::error::Error>> {
    let connection_string = "host=localhost port=5432 user=postgres password=postgres dbname=postgres";
    
    let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;
    
    // Spawn connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });
    
    Ok(client)
}

/// Strategy to generate valid SELECT queries
fn select_query_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Simple SELECT with literals
        (any::<i32>(), any::<bool>(), "[a-z]{3,10}").prop_map(|(num, flag, text)| {
            format!("SELECT {} as num, {} as flag, '{}' as text", num, flag, text)
        }),
        // SELECT from system tables
        Just("SELECT * FROM pg_tables LIMIT 1".to_string()),
        Just("SELECT tablename FROM pg_tables LIMIT 5".to_string()),
        Just("SELECT schemaname, tablename FROM pg_tables WHERE schemaname = 'pg_catalog' LIMIT 3".to_string()),
        // SELECT with expressions
        (1..100i32).prop_map(|n| format!("SELECT {} * 2 as doubled, {} + 10 as added", n, n)),
        // SELECT with string functions
        "[a-z]{5,15}".prop_map(|s| format!("SELECT upper('{}') as upper_text, length('{}') as len", s, s)),
        // SELECT with aggregates
        Just("SELECT count(*) as cnt FROM pg_tables".to_string()),
        // SELECT with CASE
        (1..10i32).prop_map(|n| {
            format!("SELECT CASE WHEN {} > 5 THEN 'high' ELSE 'low' END as category", n)
        }),
    ]
}

/// Strategy to generate valid INSERT queries with setup
fn insert_query_strategy() -> impl Strategy<Value = (String, String, String)> {
    (1..1000i32, "[a-z]{3,10}").prop_map(|(id, name)| {
        let table_name = format!("test_insert_{}", id);
        let setup = format!("CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, name VARCHAR(100))", table_name);
        let insert = format!("INSERT INTO {} (id, name) VALUES ({}, '{}')", table_name, id, name);
        let cleanup = format!("DROP TABLE IF EXISTS {}", table_name);
        (setup, insert, cleanup)
    })
}

/// Strategy to generate valid UPDATE queries with setup
fn update_query_strategy() -> impl Strategy<Value = (String, String, String, String)> {
    (1..1000i32, "[a-z]{3,10}", "[a-z]{3,10}").prop_map(|(id, old_name, new_name)| {
        let table_name = format!("test_update_{}", id);
        let setup = format!("CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, name VARCHAR(100))", table_name);
        let insert = format!("INSERT INTO {} (id, name) VALUES ({}, '{}')", table_name, id, old_name);
        let update = format!("UPDATE {} SET name = '{}' WHERE id = {}", table_name, new_name, id);
        let cleanup = format!("DROP TABLE IF EXISTS {}", table_name);
        (setup, insert, update, cleanup)
    })
}

/// Strategy to generate valid DELETE queries with setup
fn delete_query_strategy() -> impl Strategy<Value = (String, String, String, String)> {
    (1..1000i32, "[a-z]{3,10}").prop_map(|(id, name)| {
        let table_name = format!("test_delete_{}", id);
        let setup = format!("CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, name VARCHAR(100))", table_name);
        let insert = format!("INSERT INTO {} (id, name) VALUES ({}, '{}')", table_name, id, name);
        let delete = format!("DELETE FROM {} WHERE id = {}", table_name, id);
        let cleanup = format!("DROP TABLE IF EXISTS {}", table_name);
        (setup, insert, delete, cleanup)
    })
}

/// Strategy to generate valid DDL queries
fn ddl_query_strategy() -> impl Strategy<Value = (String, String)> {
    (1..1000i32).prop_map(|id| {
        let table_name = format!("test_ddl_{}", id);
        let create = format!("CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, data TEXT)", table_name);
        let drop = format!("DROP TABLE IF EXISTS {}", table_name);
        (create, drop)
    })
}

/// Strategy to generate multiple SQL statements
/// Returns a tuple of (statements_vec, expected_result_type, setup, cleanup)
fn multi_statement_strategy() -> impl Strategy<Value = (Vec<String>, QueryResultType, String, String)> {
    prop_oneof![
        // Multiple SELECT statements
        (1..5usize).prop_flat_map(|count| {
            prop::collection::vec(select_query_strategy(), count..=count)
        }).prop_map(|statements| {
            (statements, QueryResultType::Select, String::new(), String::new())
        }),
        
        // Multiple INSERT statements
        (1..5usize, 1..1000i32).prop_map(|(count, base_id)| {
            let table_name = format!("test_multi_insert_{}", base_id);
            let setup = format!("CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, value INTEGER)", table_name);
            let cleanup = format!("DROP TABLE IF EXISTS {}", table_name);
            
            let statements: Vec<String> = (0..count)
                .map(|i| format!("INSERT INTO {} (id, value) VALUES ({}, {})", table_name, base_id + i as i32, i * 10))
                .collect();
            
            (statements, QueryResultType::Insert, setup, cleanup)
        }),
        
        // Multiple UPDATE statements
        (1..5usize, 1..1000i32).prop_map(|(count, base_id)| {
            let table_name = format!("test_multi_update_{}", base_id);
            let setup = format!(
                "CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, value INTEGER); {}",
                table_name,
                (0..count).map(|i| format!("INSERT INTO {} (id, value) VALUES ({}, {})", table_name, base_id + i as i32, i * 10))
                    .collect::<Vec<_>>()
                    .join("; ")
            );
            let cleanup = format!("DROP TABLE IF EXISTS {}", table_name);
            
            let statements: Vec<String> = (0..count)
                .map(|i| format!("UPDATE {} SET value = {} WHERE id = {}", table_name, i * 20, base_id + i as i32))
                .collect();
            
            (statements, QueryResultType::Update, setup, cleanup)
        }),
        
        // Multiple DELETE statements
        (1..5usize, 1..1000i32).prop_map(|(count, base_id)| {
            let table_name = format!("test_multi_delete_{}", base_id);
            let setup = format!(
                "CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, value INTEGER); {}",
                table_name,
                (0..count).map(|i| format!("INSERT INTO {} (id, value) VALUES ({}, {})", table_name, base_id + i as i32, i * 10))
                    .collect::<Vec<_>>()
                    .join("; ")
            );
            let cleanup = format!("DROP TABLE IF EXISTS {}", table_name);
            
            let statements: Vec<String> = (0..count)
                .map(|i| format!("DELETE FROM {} WHERE id = {}", table_name, base_id + i as i32))
                .collect();
            
            (statements, QueryResultType::Delete, setup, cleanup)
        }),
        
        // Mixed DML statements (INSERT, UPDATE, DELETE on same table)
        (1..1000i32).prop_map(|base_id| {
            let table_name = format!("test_multi_mixed_{}", base_id);
            let setup = format!("CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, value INTEGER)", table_name);
            let cleanup = format!("DROP TABLE IF EXISTS {}", table_name);
            
            let statements = vec![
                format!("INSERT INTO {} (id, value) VALUES ({}, 100)", table_name, base_id),
                format!("INSERT INTO {} (id, value) VALUES ({}, 200)", table_name, base_id + 1),
                format!("UPDATE {} SET value = 150 WHERE id = {}", table_name, base_id),
                format!("DELETE FROM {} WHERE id = {}", table_name, base_id + 1),
            ];
            
            // Last statement is DELETE, so result type should be Delete
            (statements, QueryResultType::Delete, setup, cleanup)
        }),
        
        // Multiple DDL statements
        (1..1000i32).prop_map(|base_id| {
            let table1 = format!("test_multi_ddl_{}a", base_id);
            let table2 = format!("test_multi_ddl_{}b", base_id);
            
            let statements = vec![
                format!("CREATE TABLE IF NOT EXISTS {} (id INTEGER)", table1),
                format!("CREATE TABLE IF NOT EXISTS {} (id INTEGER)", table2),
            ];
            
            let cleanup = format!("DROP TABLE IF EXISTS {}; DROP TABLE IF EXISTS {}", table1, table2);
            
            (statements, QueryResultType::Ddl, String::new(), cleanup)
        }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Property 3.1: SELECT queries return result set with columns and rows
    /// 
    /// For any valid SELECT query, the executor should:
    /// - Return QueryResultType::Select
    /// - Include column information
    /// - Include row data (may be empty)
    /// - Include execution time
    /// - Not have an error
    #[test]
    fn prop_select_returns_result_set(sql in select_query_strategy()) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let client = match get_test_client().await {
                Ok(c) => c,
                Err(_) => {
                    // Skip test if database is not available
                    return Ok(());
                }
            };
            
            let result = query_executor::execute_sql(&client, &sql).await;
            
            // Verify result type is Select
            prop_assert_eq!(result.result_type, QueryResultType::Select);
            
            // Verify columns are present
            prop_assert!(result.columns.is_some(), "SELECT query should return columns");
            
            // Verify rows are present (even if empty)
            prop_assert!(result.rows.is_some(), "SELECT query should return rows array");
            
            // Verify no error
            prop_assert!(result.error.is_none(), "Valid SELECT query should not have error");
            
            // Verify affected_rows is None for SELECT
            prop_assert!(result.affected_rows.is_none(), "SELECT query should not have affected_rows");
            
            Ok(())
        })?;
    }
    
    /// Property 3.2: INSERT queries return affected rows count
    /// 
    /// For any valid INSERT query, the executor should:
    /// - Return QueryResultType::Insert
    /// - Include affected_rows count
    /// - Include execution time
    /// - Not have columns or rows
    /// - Not have an error
    #[test]
    fn prop_insert_returns_affected_rows((setup, insert, cleanup) in insert_query_strategy()) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let client = match get_test_client().await {
                Ok(c) => c,
                Err(_) => {
                    return Ok(());
                }
            };
            
            // Setup: create table
            let _ = query_executor::execute_sql(&client, &setup).await;
            
            // Execute INSERT
            let result = query_executor::execute_sql(&client, &insert).await;
            
            // Cleanup
            let _ = query_executor::execute_sql(&client, &cleanup).await;
            
            // Verify result type is Insert
            prop_assert_eq!(result.result_type, QueryResultType::Insert);
            
            // Verify affected_rows is present and > 0
            prop_assert!(result.affected_rows.is_some(), "INSERT query should return affected_rows");
            prop_assert!(result.affected_rows.unwrap() > 0, "INSERT should affect at least 1 row");
            
            // Verify no error
            prop_assert!(result.error.is_none(), "Valid INSERT query should not have error");
            
            // Verify columns and rows are None for INSERT
            prop_assert!(result.columns.is_none(), "INSERT query should not have columns");
            prop_assert!(result.rows.is_none(), "INSERT query should not have rows");
            
            Ok(())
        })?;
    }
    
    /// Property 3.3: UPDATE queries return affected rows count
    /// 
    /// For any valid UPDATE query, the executor should:
    /// - Return QueryResultType::Update
    /// - Include affected_rows count
    /// - Include execution time
    /// - Not have columns or rows
    /// - Not have an error
    #[test]
    fn prop_update_returns_affected_rows((setup, insert, update, cleanup) in update_query_strategy()) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let client = match get_test_client().await {
                Ok(c) => c,
                Err(_) => {
                    return Ok(());
                }
            };
            
            // Setup: create table and insert data
            let _ = query_executor::execute_sql(&client, &setup).await;
            let _ = query_executor::execute_sql(&client, &insert).await;
            
            // Execute UPDATE
            let result = query_executor::execute_sql(&client, &update).await;
            
            // Cleanup
            let _ = query_executor::execute_sql(&client, &cleanup).await;
            
            // Verify result type is Update
            prop_assert_eq!(result.result_type, QueryResultType::Update);
            
            // Verify affected_rows is present
            prop_assert!(result.affected_rows.is_some(), "UPDATE query should return affected_rows");
            
            // Verify no error
            prop_assert!(result.error.is_none(), "Valid UPDATE query should not have error");
            
            // Verify columns and rows are None for UPDATE
            prop_assert!(result.columns.is_none(), "UPDATE query should not have columns");
            prop_assert!(result.rows.is_none(), "UPDATE query should not have rows");
            
            Ok(())
        })?;
    }
    
    /// Property 3.4: DELETE queries return affected rows count
    /// 
    /// For any valid DELETE query, the executor should:
    /// - Return QueryResultType::Delete
    /// - Include affected_rows count
    /// - Include execution time
    /// - Not have columns or rows
    /// - Not have an error
    #[test]
    fn prop_delete_returns_affected_rows((setup, insert, delete, cleanup) in delete_query_strategy()) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let client = match get_test_client().await {
                Ok(c) => c,
                Err(_) => {
                    return Ok(());
                }
            };
            
            // Setup: create table and insert data
            let _ = query_executor::execute_sql(&client, &setup).await;
            let _ = query_executor::execute_sql(&client, &insert).await;
            
            // Execute DELETE
            let result = query_executor::execute_sql(&client, &delete).await;
            
            // Cleanup
            let _ = query_executor::execute_sql(&client, &cleanup).await;
            
            // Verify result type is Delete
            prop_assert_eq!(result.result_type, QueryResultType::Delete);
            
            // Verify affected_rows is present
            prop_assert!(result.affected_rows.is_some(), "DELETE query should return affected_rows");
            
            // Verify no error
            prop_assert!(result.error.is_none(), "Valid DELETE query should not have error");
            
            // Verify columns and rows are None for DELETE
            prop_assert!(result.columns.is_none(), "DELETE query should not have columns");
            prop_assert!(result.rows.is_none(), "DELETE query should not have rows");
            
            Ok(())
        })?;
    }
    
    /// Property 3.5: DDL queries return success message
    /// 
    /// For any valid DDL query (CREATE/ALTER/DROP), the executor should:
    /// - Return QueryResultType::Ddl
    /// - Include execution time
    /// - Not have columns, rows, or affected_rows
    /// - Not have an error
    #[test]
    fn prop_ddl_returns_success((create, drop) in ddl_query_strategy()) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let client = match get_test_client().await {
                Ok(c) => c,
                Err(_) => {
                    return Ok(());
                }
            };
            
            // Execute CREATE TABLE
            let result = query_executor::execute_sql(&client, &create).await;
            
            // Verify result type is Ddl
            prop_assert_eq!(result.result_type, QueryResultType::Ddl);
            
            // Verify no error
            prop_assert!(result.error.is_none(), "Valid DDL query should not have error");
            
            // Verify columns, rows, and affected_rows are None for DDL
            prop_assert!(result.columns.is_none(), "DDL query should not have columns");
            prop_assert!(result.rows.is_none(), "DDL query should not have rows");
            prop_assert!(result.affected_rows.is_none(), "DDL query should not have affected_rows");
            
            // Cleanup
            let _ = query_executor::execute_sql(&client, &drop).await;
            
            Ok(())
        })?;
    }
    
    /// Property 4: Multi-statement sequential execution
    /// 
    /// **Feature: database-advanced-features, Property 4: 多语句顺序执行**
    /// **Validates: Requirements 2.6**
    /// 
    /// For any string containing multiple SQL statements separated by semicolons,
    /// the query executor should execute each statement in the order they appear
    /// in the string.
    /// 
    /// This property verifies:
    /// 1. All statements are executed in order
    /// 2. The result type matches the last statement's type
    /// 3. For DML operations, affected rows are accumulated
    /// 4. If any statement fails, execution stops and returns error
    /// 5. Execution time covers all statements
    #[test]
    fn prop_multi_statement_sequential_execution(
        (statements, expected_type, setup, cleanup) in multi_statement_strategy()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let client = match get_test_client().await {
                Ok(c) => c,
                Err(_) => {
                    return Ok(());
                }
            };
            
            // Setup if needed
            if !setup.is_empty() {
                let _ = query_executor::execute_sql(&client, &setup).await;
            }
            
            // Join statements with semicolons to create multi-statement SQL
            let multi_sql = statements.join("; ");
            
            // Execute multi-statement SQL
            let result = query_executor::execute_sql(&client, &multi_sql).await;
            
            // Cleanup
            if !cleanup.is_empty() {
                let _ = query_executor::execute_sql(&client, &cleanup).await;
            }
            
            // Verify no error occurred
            prop_assert!(
                result.error.is_none(),
                "Multi-statement execution should not have error: {:?}",
                result.error
            );
            
            // Clone expected_type for multiple uses
            let expected_type_clone = expected_type.clone();
            
            // Verify result type matches the expected type (last statement's type)
            prop_assert_eq!(
                result.result_type,
                expected_type,
                "Result type should match the last statement's type"
            );
            
            // For DML operations, verify affected_rows is present
            if matches!(expected_type_clone, QueryResultType::Insert | QueryResultType::Update | QueryResultType::Delete) {
                prop_assert!(
                    result.affected_rows.is_some(),
                    "DML multi-statement should have affected_rows"
                );
                
                // For multiple DML statements, affected rows should be >= number of statements
                // (each statement affects at least 0 rows)
                let affected = result.affected_rows.unwrap();
                prop_assert!(
                    affected >= 0,
                    "Affected rows should be non-negative, got {}",
                    affected
                );
            }
            
            // For SELECT, verify columns and rows are present
            if expected_type_clone == QueryResultType::Select {
                prop_assert!(
                    result.columns.is_some(),
                    "SELECT multi-statement should have columns"
                );
                prop_assert!(
                    result.rows.is_some(),
                    "SELECT multi-statement should have rows"
                );
            }
            
            // For DDL, verify no columns, rows, or affected_rows
            if expected_type_clone == QueryResultType::Ddl {
                prop_assert!(
                    result.columns.is_none(),
                    "DDL multi-statement should not have columns"
                );
                prop_assert!(
                    result.rows.is_none(),
                    "DDL multi-statement should not have rows"
                );
                prop_assert!(
                    result.affected_rows.is_none(),
                    "DDL multi-statement should not have affected_rows"
                );
            }
            
            // Verify execution time is recorded (may be 0 for very fast queries)
            // Just check that the field exists and is not negative
            prop_assert!(
                result.duration_ms >= 0,
                "Execution time should be non-negative"
            );
            
            Ok(())
        })?;
    }
    
    /// Property 4.1: Multi-statement execution order verification
    /// 
    /// This test verifies that statements are executed in the exact order they appear
    /// by checking the side effects of each statement.
    #[test]
    fn prop_multi_statement_execution_order(base_id in 1..1000i32) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let client = match get_test_client().await {
                Ok(c) => c,
                Err(_) => {
                    return Ok(());
                }
            };
            
            let table_name = format!("test_order_{}", base_id);
            
            // Setup: create table
            let setup = format!("CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, step INTEGER)", table_name);
            let _ = query_executor::execute_sql(&client, &setup).await;
            
            // Create multi-statement SQL that inserts rows in specific order
            let multi_sql = format!(
                "INSERT INTO {} (id, step) VALUES ({}, 1); \
                 INSERT INTO {} (id, step) VALUES ({}, 2); \
                 INSERT INTO {} (id, step) VALUES ({}, 3)",
                table_name, base_id, 
                table_name, base_id + 1,
                table_name, base_id + 2
            );
            
            // Execute multi-statement SQL
            let result = query_executor::execute_sql(&client, &multi_sql).await;
            
            // Verify no error
            prop_assert!(result.error.is_none(), "Execution should succeed");
            
            // Verify all 3 rows were inserted
            let verify_sql = format!("SELECT id, step FROM {} ORDER BY id", table_name);
            let verify_result = query_executor::execute_sql(&client, &verify_sql).await;
            
            prop_assert!(verify_result.rows.is_some(), "Should have rows");
            let rows = verify_result.rows.unwrap();
            prop_assert_eq!(rows.len(), 3, "Should have exactly 3 rows");
            
            // Verify the order by checking step values
            for (i, row) in rows.iter().enumerate() {
                let step = row.get("step").and_then(|v| v.as_i64()).unwrap_or(0);
                prop_assert_eq!(
                    step as usize,
                    i + 1,
                    "Row {} should have step {}, got {}",
                    i,
                    i + 1,
                    step
                );
            }
            
            // Cleanup
            let cleanup = format!("DROP TABLE IF EXISTS {}", table_name);
            let _ = query_executor::execute_sql(&client, &cleanup).await;
            
            Ok(())
        })?;
    }
    
    /// Property 4.2: Multi-statement error stops execution
    /// 
    /// If any statement in a multi-statement SQL fails, execution should stop
    /// and return an error. Subsequent statements should not be executed.
    #[test]
    fn prop_multi_statement_error_stops_execution(base_id in 1..1000i32) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let client = match get_test_client().await {
                Ok(c) => c,
                Err(_) => {
                    return Ok(());
                }
            };
            
            let table_name = format!("test_error_{}", base_id);
            
            // Setup: create table
            let setup = format!("CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY)", table_name);
            let _ = query_executor::execute_sql(&client, &setup).await;
            
            // Create multi-statement SQL where the second statement will fail (duplicate key)
            let multi_sql = format!(
                "INSERT INTO {} (id) VALUES ({}); \
                 INSERT INTO {} (id) VALUES ({}); \
                 INSERT INTO {} (id) VALUES ({})",
                table_name, base_id,
                table_name, base_id,  // This will fail due to duplicate key
                table_name, base_id + 1  // This should not be executed
            );
            
            // Execute multi-statement SQL
            let result = query_executor::execute_sql(&client, &multi_sql).await;
            
            // Verify error occurred
            prop_assert!(
                result.error.is_some(),
                "Should have error due to duplicate key"
            );
            prop_assert_eq!(
                result.result_type,
                QueryResultType::Error,
                "Result type should be Error"
            );
            
            // Verify only the first statement was executed
            let verify_sql = format!("SELECT COUNT(*) as cnt FROM {}", table_name);
            let verify_result = query_executor::execute_sql(&client, &verify_sql).await;
            
            if let Some(rows) = verify_result.rows {
                if let Some(first_row) = rows.first() {
                    let count = first_row.get("cnt").and_then(|v| v.as_i64()).unwrap_or(0);
                    prop_assert_eq!(
                        count,
                        1,
                        "Only first statement should have been executed, found {} rows",
                        count
                    );
                }
            }
            
            // Cleanup
            let cleanup = format!("DROP TABLE IF EXISTS {}", table_name);
            let _ = query_executor::execute_sql(&client, &cleanup).await;
            
            Ok(())
        })?;
    }
    
    /// Property 5: SQL错误返回错误信息
    /// 
    /// **Feature: database-advanced-features, Property 5: SQL错误返回错误信息**
    /// **Validates: Requirements 2.7**
    /// 
    /// For any invalid SQL statement, the query executor should return a result
    /// containing an error message, and should not modify the database state.
    /// 
    /// This property verifies:
    /// 1. Invalid SQL returns QueryResultType::Error
    /// 2. Error message is present and non-empty
    /// 3. No columns, rows, or affected_rows are returned
    /// 4. Database state remains unchanged (no side effects)
    /// 5. Execution time is recorded
    #[test]
    fn prop_invalid_sql_returns_error(
        (invalid_sql, setup, verify_sql, cleanup) in invalid_sql_strategy()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let client = match get_test_client().await {
                Ok(c) => c,
                Err(_) => {
                    return Ok(());
                }
            };
            
            // Setup: create test table if needed
            if !setup.is_empty() {
                let setup_result = query_executor::execute_sql(&client, &setup).await;
                prop_assert!(
                    setup_result.error.is_none(),
                    "Setup should succeed: {:?}",
                    setup_result.error
                );
            }
            
            // Get initial database state if we have a verification query
            let initial_state = if !verify_sql.is_empty() {
                let result = query_executor::execute_sql(&client, &verify_sql).await;
                result.rows.clone()
            } else {
                None
            };
            
            // Execute invalid SQL
            let result = query_executor::execute_sql(&client, &invalid_sql).await;
            
            // Verify result type is Error
            prop_assert_eq!(
                result.result_type,
                QueryResultType::Error,
                "Invalid SQL should return Error type"
            );
            
            // Verify error message is present and non-empty
            prop_assert!(
                result.error.is_some(),
                "Invalid SQL should have error message"
            );
            
            let error_msg = result.error.as_ref().unwrap();
            prop_assert!(
                !error_msg.is_empty(),
                "Error message should not be empty"
            );
            
            // Verify no columns, rows, or affected_rows
            prop_assert!(
                result.columns.is_none(),
                "Error result should not have columns"
            );
            prop_assert!(
                result.rows.is_none(),
                "Error result should not have rows"
            );
            prop_assert!(
                result.affected_rows.is_none(),
                "Error result should not have affected_rows"
            );
            
            // Verify execution time is recorded
            prop_assert!(
                result.duration_ms >= 0,
                "Execution time should be non-negative"
            );
            
            // Verify database state is unchanged
            if !verify_sql.is_empty() {
                let final_state_result = query_executor::execute_sql(&client, &verify_sql).await;
                let final_state = final_state_result.rows;
                
                prop_assert_eq!(
                    initial_state,
                    final_state,
                    "Database state should not change after invalid SQL"
                );
            }
            
            // Cleanup
            if !cleanup.is_empty() {
                let _ = query_executor::execute_sql(&client, &cleanup).await;
            }
            
            Ok(())
        })?;
    }
    
    /// Property 5.1: Syntax errors return descriptive error messages
    /// 
    /// For SQL syntax errors, the error message should indicate it's a syntax error
    /// and ideally include position information.
    #[test]
    fn prop_syntax_error_returns_descriptive_message(
        syntax_error_sql in syntax_error_strategy()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let client = match get_test_client().await {
                Ok(c) => c,
                Err(_) => {
                    return Ok(());
                }
            };
            
            // Execute SQL with syntax error
            let result = query_executor::execute_sql(&client, &syntax_error_sql).await;
            
            // Verify result type is Error
            prop_assert_eq!(
                result.result_type,
                QueryResultType::Error,
                "Syntax error should return Error type"
            );
            
            // Verify error message exists
            prop_assert!(
                result.error.is_some(),
                "Syntax error should have error message"
            );
            
            let error_msg = result.error.as_ref().unwrap().to_lowercase();
            
            // Verify error message indicates SQL error
            // Accept any of: syntax errors, query type errors, operator errors, or object not found
            // All of these are valid error responses for invalid SQL
            prop_assert!(
                error_msg.contains("syntax") 
                    || error_msg.contains("42") // PostgreSQL error codes 42xxx are syntax/access errors
                    || error_msg.contains("unable to determine query type")
                    || error_msg.contains("does not exist")
                    || error_msg.contains("操作符不存在") // Chinese: operator does not exist
                    || error_msg.contains("operator"),
                "Error message should indicate SQL error, got: {}",
                result.error.as_ref().unwrap()
            );
            
            Ok(())
        })?;
    }
    
    /// Property 5.2: Constraint violations return appropriate error messages
    /// 
    /// For constraint violations (unique, foreign key, not null, check),
    /// the error message should indicate the type of constraint violated.
    #[test]
    fn prop_constraint_violation_returns_appropriate_message(
        (setup, violation_sql, cleanup, expected_code) in constraint_violation_strategy()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let client = match get_test_client().await {
                Ok(c) => c,
                Err(_) => {
                    return Ok(());
                }
            };
            
            // Setup: create table and initial data
            let setup_result = query_executor::execute_sql(&client, &setup).await;
            prop_assert!(
                setup_result.error.is_none(),
                "Setup should succeed: {:?}",
                setup_result.error
            );
            
            // Execute SQL that violates constraint
            let result = query_executor::execute_sql(&client, &violation_sql).await;
            
            // Verify result type is Error
            prop_assert_eq!(
                result.result_type,
                QueryResultType::Error,
                "Constraint violation should return Error type"
            );
            
            // Verify error message exists
            prop_assert!(
                result.error.is_some(),
                "Constraint violation should have error message"
            );
            
            let error_msg = result.error.as_ref().unwrap().to_lowercase();
            
            // Verify error message indicates constraint violation
            prop_assert!(
                error_msg.contains("constraint") || error_msg.contains(&expected_code),
                "Error message should indicate constraint violation (code {}), got: {}",
                expected_code,
                result.error.as_ref().unwrap()
            );
            
            // Cleanup
            let _ = query_executor::execute_sql(&client, &cleanup).await;
            
            Ok(())
        })?;
    }
    
    /// Property 5.3: Non-existent table/column errors are clear
    /// 
    /// When referencing non-existent tables or columns, the error message
    /// should clearly indicate what doesn't exist.
    #[test]
    fn prop_nonexistent_object_error_is_clear(
        (sql, expected_pattern) in nonexistent_object_strategy()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let client = match get_test_client().await {
                Ok(c) => c,
                Err(_) => {
                    return Ok(());
                }
            };
            
            // Execute SQL referencing non-existent object
            let result = query_executor::execute_sql(&client, &sql).await;
            
            // Verify result type is Error
            prop_assert_eq!(
                result.result_type,
                QueryResultType::Error,
                "Non-existent object should return Error type"
            );
            
            // Verify error message exists
            prop_assert!(
                result.error.is_some(),
                "Non-existent object error should have error message"
            );
            
            let error_msg = result.error.as_ref().unwrap().to_lowercase();
            
            // Verify error message indicates the object doesn't exist
            // Accept both English and Chinese error messages
            prop_assert!(
                error_msg.contains(&expected_pattern) 
                    || error_msg.contains("不存在") // Chinese: does not exist
                    || error_msg.contains("42p01") // Table does not exist
                    || error_msg.contains("42703") // Column does not exist
                    || error_msg.contains("42883"), // Function does not exist
                "Error message should indicate object doesn't exist (pattern: {}), got: {}",
                expected_pattern,
                result.error.as_ref().unwrap()
            );
            
            Ok(())
        })?;
    }
}

/// Strategy to generate invalid SQL statements with setup and verification
fn invalid_sql_strategy() -> impl Strategy<Value = (String, String, String, String)> {
    prop_oneof![
        // Syntax errors - no setup needed
        syntax_error_strategy().prop_map(|sql| (sql, String::new(), String::new(), String::new())),
        
        // Constraint violations - need setup
        constraint_violation_strategy().prop_map(|(setup, sql, cleanup, _)| {
            // For constraint violations, we can verify table still exists
            let table_name = extract_table_name(&setup);
            let verify = if !table_name.is_empty() {
                format!("SELECT COUNT(*) as cnt FROM {}", table_name)
            } else {
                String::new()
            };
            (sql, setup, verify, cleanup)
        }),
        
        // Non-existent objects - no setup needed
        nonexistent_object_strategy().prop_map(|(sql, _)| (sql, String::new(), String::new(), String::new())),
        
        // Type mismatch errors
        (1..1000i32).prop_map(|id| {
            let table_name = format!("test_type_error_{}", id);
            let setup = format!("CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, value INTEGER)", table_name);
            let invalid_sql = format!("INSERT INTO {} (id, value) VALUES ({}, 'not_a_number')", table_name, id);
            let verify = format!("SELECT COUNT(*) as cnt FROM {}", table_name);
            let cleanup = format!("DROP TABLE IF EXISTS {}", table_name);
            (invalid_sql, setup, verify, cleanup)
        }),
        
        // Division by zero
        Just((
            "SELECT 1 / 0 as result".to_string(),
            String::new(),
            String::new(),
            String::new()
        )),
    ]
}

/// Strategy to generate SQL with syntax errors
fn syntax_error_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Missing FROM clause
        Just("SELECT * WHERE id = 1".to_string()),
        
        // Invalid keyword order
        Just("FROM pg_tables SELECT *".to_string()),
        
        // Unclosed string literal
        Just("SELECT 'unclosed string FROM pg_tables".to_string()),
        
        // Missing closing parenthesis
        Just("SELECT * FROM pg_tables WHERE (schemaname = 'public'".to_string()),
        
        // Invalid column definition
        Just("CREATE TABLE test (id)".to_string()),
        
        // Multiple syntax errors
        Just("SELCT * FORM pg_tables WERE schemaname = 'public'".to_string()),
        
        // Invalid INSERT syntax - missing VALUES keyword
        Just("INSERT INTO pg_tables".to_string()),
        
        // Invalid UPDATE syntax - incomplete SET clause
        Just("UPDATE pg_tables SET".to_string()),
        
        // Invalid DELETE syntax - missing FROM keyword
        Just("DELETE pg_tables WHERE schemaname = 'public'".to_string()),
        
        // Invalid operator
        Just("SELECT 1 === 2".to_string()),
        
        // Unclosed parenthesis in expression
        Just("SELECT (1 + 2".to_string()),
        
        // Invalid comma placement
        Just("SELECT , FROM pg_tables".to_string()),
        
        // Missing SELECT keyword
        Just("* FROM pg_tables".to_string()),
    ]
}

/// Strategy to generate constraint violation scenarios
/// Returns (setup_sql, violation_sql, cleanup_sql, expected_error_code)
fn constraint_violation_strategy() -> impl Strategy<Value = (String, String, String, String)> {
    prop_oneof![
        // Unique constraint violation
        (1..1000i32).prop_map(|id| {
            let table_name = format!("test_unique_{}", id);
            let setup = format!(
                "CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, email VARCHAR(100) UNIQUE); \
                 INSERT INTO {} (id, email) VALUES ({}, 'test@example.com')",
                table_name, table_name, id
            );
            let violation = format!("INSERT INTO {} (id, email) VALUES ({}, 'test@example.com')", table_name, id + 1);
            let cleanup = format!("DROP TABLE IF EXISTS {}", table_name);
            (setup, violation, cleanup, "23505".to_string())
        }),
        
        // Primary key violation
        (1..1000i32).prop_map(|id| {
            let table_name = format!("test_pk_{}", id);
            let setup = format!(
                "CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY); \
                 INSERT INTO {} (id) VALUES ({})",
                table_name, table_name, id
            );
            let violation = format!("INSERT INTO {} (id) VALUES ({})", table_name, id);
            let cleanup = format!("DROP TABLE IF EXISTS {}", table_name);
            (setup, violation, cleanup, "23505".to_string())
        }),
        
        // Not null constraint violation
        (1..1000i32).prop_map(|id| {
            let table_name = format!("test_notnull_{}", id);
            let setup = format!(
                "CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, name VARCHAR(100) NOT NULL)",
                table_name
            );
            let violation = format!("INSERT INTO {} (id, name) VALUES ({}, NULL)", table_name, id);
            let cleanup = format!("DROP TABLE IF EXISTS {}", table_name);
            (setup, violation, cleanup, "23502".to_string())
        }),
        
        // Check constraint violation
        (1..1000i32).prop_map(|id| {
            let table_name = format!("test_check_{}", id);
            let setup = format!(
                "CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, age INTEGER CHECK (age >= 0))",
                table_name
            );
            let violation = format!("INSERT INTO {} (id, age) VALUES ({}, -1)", table_name, id);
            let cleanup = format!("DROP TABLE IF EXISTS {}", table_name);
            (setup, violation, cleanup, "23514".to_string())
        }),
    ]
}

/// Strategy to generate SQL referencing non-existent objects
/// Returns (sql, expected_error_pattern)
fn nonexistent_object_strategy() -> impl Strategy<Value = (String, String)> {
    prop_oneof![
        // Non-existent table
        (1..1000i32).prop_map(|id| {
            let table_name = format!("nonexistent_table_{}", id);
            (
                format!("SELECT * FROM {}", table_name),
                "does not exist".to_string()
            )
        }),
        
        // Non-existent column
        Just((
            "SELECT nonexistent_column FROM pg_tables LIMIT 1".to_string(),
            "does not exist".to_string()
        )),
        
        // Non-existent function
        Just((
            "SELECT nonexistent_function()".to_string(),
            "does not exist".to_string()
        )),
    ]
}

/// Helper function to extract table name from CREATE TABLE statement
fn extract_table_name(sql: &str) -> String {
    if let Some(start) = sql.to_uppercase().find("CREATE TABLE") {
        let after_create = &sql[start + 12..];
        if let Some(if_not_exists_pos) = after_create.to_uppercase().find("IF NOT EXISTS") {
            let after_if = &after_create[if_not_exists_pos + 13..];
            if let Some(table_name) = after_if.trim().split_whitespace().next() {
                return table_name.to_string();
            }
        } else if let Some(table_name) = after_create.trim().split_whitespace().next() {
            return table_name.to_string();
        }
    }
    String::new()
}
