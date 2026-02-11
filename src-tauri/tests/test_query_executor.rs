/**
 * Integration tests for Query Executor
 * 
 * These tests verify that the query executor correctly handles different SQL statement types
 * and returns appropriate results.
 */

use tokio_postgres::{NoTls, Client};
use pg_db_tool::services::query_executor;
use pg_db_tool::models::query::QueryResultType;

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

#[tokio::test]
async fn test_execute_select_query() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    let sql = "SELECT 1 as id, 'test' as name";
    let result = query_executor::execute_sql(&client, sql).await;
    
    assert_eq!(result.result_type, QueryResultType::Select);
    assert!(result.columns.is_some());
    assert!(result.rows.is_some());
    assert!(result.error.is_none());
    
    let columns = result.columns.unwrap();
    assert_eq!(columns.len(), 2);
    assert_eq!(columns[0].name, "id");
    assert_eq!(columns[1].name, "name");
    
    let rows = result.rows.unwrap();
    assert_eq!(rows.len(), 1);
}

#[tokio::test]
async fn test_execute_empty_select() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    let sql = "SELECT * FROM pg_tables WHERE tablename = 'nonexistent_table_xyz'";
    let result = query_executor::execute_sql(&client, sql).await;
    
    assert_eq!(result.result_type, QueryResultType::Select);
    assert!(result.columns.is_some());
    assert!(result.rows.is_some());
    assert!(result.error.is_none());
    
    let rows = result.rows.unwrap();
    assert_eq!(rows.len(), 0);
}

#[tokio::test]
async fn test_execute_ddl_create_table() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    // Drop table if exists
    let _ = query_executor::execute_sql(&client, "DROP TABLE IF EXISTS test_query_executor").await;
    
    // Create table
    let sql = "CREATE TABLE test_query_executor (id INTEGER PRIMARY KEY, name VARCHAR(100))";
    let result = query_executor::execute_sql(&client, sql).await;
    
    assert_eq!(result.result_type, QueryResultType::Ddl);
    assert!(result.error.is_none());
    
    // Clean up
    let _ = query_executor::execute_sql(&client, "DROP TABLE test_query_executor").await;
}

#[tokio::test]
async fn test_execute_insert() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    // Setup
    let _ = query_executor::execute_sql(&client, "DROP TABLE IF EXISTS test_insert").await;
    let _ = query_executor::execute_sql(&client, "CREATE TABLE test_insert (id INTEGER PRIMARY KEY, name VARCHAR(100))").await;
    
    // Insert
    let sql = "INSERT INTO test_insert (id, name) VALUES (1, 'Alice'), (2, 'Bob')";
    let result = query_executor::execute_sql(&client, sql).await;
    
    assert_eq!(result.result_type, QueryResultType::Insert);
    assert_eq!(result.affected_rows, Some(2));
    assert!(result.error.is_none());
    
    // Clean up
    let _ = query_executor::execute_sql(&client, "DROP TABLE test_insert").await;
}

#[tokio::test]
async fn test_execute_update() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    // Setup
    let _ = query_executor::execute_sql(&client, "DROP TABLE IF EXISTS test_update").await;
    let _ = query_executor::execute_sql(&client, "CREATE TABLE test_update (id INTEGER PRIMARY KEY, name VARCHAR(100))").await;
    let _ = query_executor::execute_sql(&client, "INSERT INTO test_update (id, name) VALUES (1, 'Alice'), (2, 'Bob')").await;
    
    // Update
    let sql = "UPDATE test_update SET name = 'Charlie' WHERE id = 1";
    let result = query_executor::execute_sql(&client, sql).await;
    
    assert_eq!(result.result_type, QueryResultType::Update);
    assert_eq!(result.affected_rows, Some(1));
    assert!(result.error.is_none());
    
    // Clean up
    let _ = query_executor::execute_sql(&client, "DROP TABLE test_update").await;
}

#[tokio::test]
async fn test_execute_delete() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    // Setup
    let _ = query_executor::execute_sql(&client, "DROP TABLE IF EXISTS test_delete").await;
    let _ = query_executor::execute_sql(&client, "CREATE TABLE test_delete (id INTEGER PRIMARY KEY, name VARCHAR(100))").await;
    let _ = query_executor::execute_sql(&client, "INSERT INTO test_delete (id, name) VALUES (1, 'Alice'), (2, 'Bob')").await;
    
    // Delete
    let sql = "DELETE FROM test_delete WHERE id = 1";
    let result = query_executor::execute_sql(&client, sql).await;
    
    assert_eq!(result.result_type, QueryResultType::Delete);
    assert_eq!(result.affected_rows, Some(1));
    assert!(result.error.is_none());
    
    // Clean up
    let _ = query_executor::execute_sql(&client, "DROP TABLE test_delete").await;
}

#[tokio::test]
async fn test_execute_invalid_sql() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    let sql = "SELECT * FROM nonexistent_table_xyz";
    let result = query_executor::execute_sql(&client, sql).await;
    
    assert_eq!(result.result_type, QueryResultType::Error);
    assert!(result.error.is_some());
    
    let error = result.error.unwrap();
    // PostgreSQL error messages may vary, just check that we got an error
    assert!(!error.is_empty(), "Error message should not be empty");
}

#[tokio::test]
async fn test_execute_empty_sql() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    let sql = "   ";
    let result = query_executor::execute_sql(&client, sql).await;
    
    assert_eq!(result.result_type, QueryResultType::Error);
    assert!(result.error.is_some());
    assert_eq!(result.error.unwrap(), "SQL statement is empty");
}

#[tokio::test]
async fn test_execute_with_comments() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    let sql = "-- This is a comment\nSELECT 1 as value";
    let result = query_executor::execute_sql(&client, sql).await;
    
    assert_eq!(result.result_type, QueryResultType::Select);
    assert!(result.error.is_none());
}

#[tokio::test]
async fn test_execute_with_cte() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    let sql = "WITH cte AS (SELECT 1 as id) SELECT * FROM cte";
    let result = query_executor::execute_sql(&client, sql).await;
    
    assert_eq!(result.result_type, QueryResultType::Select);
    assert!(result.error.is_none());
    
    let rows = result.rows.unwrap();
    assert_eq!(rows.len(), 1);
}

#[tokio::test]
async fn test_execute_multiple_statements() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    // Setup
    let _ = query_executor::execute_sql(&client, "DROP TABLE IF EXISTS test_multi").await;
    
    // Execute multiple statements
    let sql = "CREATE TABLE test_multi (id INTEGER PRIMARY KEY, name VARCHAR(100)); INSERT INTO test_multi (id, name) VALUES (1, 'Alice'); INSERT INTO test_multi (id, name) VALUES (2, 'Bob')";
    let result = query_executor::execute_sql(&client, sql).await;
    
    // Should return the result of the last statement (INSERT)
    assert_eq!(result.result_type, QueryResultType::Insert);
    // Should accumulate affected rows from both INSERTs
    assert_eq!(result.affected_rows, Some(2));
    assert!(result.error.is_none());
    
    // Verify data was inserted
    let verify_result = query_executor::execute_sql(&client, "SELECT * FROM test_multi ORDER BY id").await;
    assert_eq!(verify_result.result_type, QueryResultType::Select);
    let rows = verify_result.rows.unwrap();
    assert_eq!(rows.len(), 2);
    
    // Clean up
    let _ = query_executor::execute_sql(&client, "DROP TABLE test_multi").await;
}

#[tokio::test]
async fn test_execute_multiple_statements_with_select() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    // Setup
    let _ = query_executor::execute_sql(&client, "DROP TABLE IF EXISTS test_multi_select").await;
    let _ = query_executor::execute_sql(&client, "CREATE TABLE test_multi_select (id INTEGER PRIMARY KEY, name VARCHAR(100))").await;
    let _ = query_executor::execute_sql(&client, "INSERT INTO test_multi_select (id, name) VALUES (1, 'Alice'), (2, 'Bob')").await;
    
    // Execute multiple statements ending with SELECT
    let sql = "UPDATE test_multi_select SET name = 'Charlie' WHERE id = 1; SELECT * FROM test_multi_select ORDER BY id";
    let result = query_executor::execute_sql(&client, sql).await;
    
    // Should return the result of the last statement (SELECT)
    assert_eq!(result.result_type, QueryResultType::Select);
    assert!(result.error.is_none());
    
    let rows = result.rows.unwrap();
    assert_eq!(rows.len(), 2);
    
    // Clean up
    let _ = query_executor::execute_sql(&client, "DROP TABLE test_multi_select").await;
}

#[tokio::test]
async fn test_execute_multiple_statements_with_error() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    // Setup
    let _ = query_executor::execute_sql(&client, "DROP TABLE IF EXISTS test_multi_error").await;
    let _ = query_executor::execute_sql(&client, "CREATE TABLE test_multi_error (id INTEGER PRIMARY KEY, name VARCHAR(100))").await;
    
    // Execute multiple statements where the second one fails
    let sql = "INSERT INTO test_multi_error (id, name) VALUES (1, 'Alice'); SELECT * FROM nonexistent_table; INSERT INTO test_multi_error (id, name) VALUES (2, 'Bob')";
    let result = query_executor::execute_sql(&client, sql).await;
    
    // Should return error from the second statement
    assert_eq!(result.result_type, QueryResultType::Error);
    assert!(result.error.is_some());
    
    let error = result.error.unwrap();
    assert!(error.contains("statement 2"), "Error should indicate which statement failed");
    
    // Verify that only the first INSERT was executed (execution stopped at error)
    let verify_result = query_executor::execute_sql(&client, "SELECT * FROM test_multi_error").await;
    let rows = verify_result.rows.unwrap();
    assert_eq!(rows.len(), 1, "Only the first INSERT should have been executed");
    
    // Clean up
    let _ = query_executor::execute_sql(&client, "DROP TABLE test_multi_error").await;
}

#[tokio::test]
async fn test_execute_multiple_statements_with_semicolon_in_string() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    // Setup
    let _ = query_executor::execute_sql(&client, "DROP TABLE IF EXISTS test_multi_string").await;
    let _ = query_executor::execute_sql(&client, "CREATE TABLE test_multi_string (id INTEGER PRIMARY KEY, name VARCHAR(100))").await;
    
    // Execute statements with semicolon inside string literal
    let sql = "INSERT INTO test_multi_string (id, name) VALUES (1, 'John; Doe'); SELECT * FROM test_multi_string";
    let result = query_executor::execute_sql(&client, sql).await;
    
    // Should return SELECT result
    assert_eq!(result.result_type, QueryResultType::Select);
    assert!(result.error.is_none());
    
    let rows = result.rows.unwrap();
    assert_eq!(rows.len(), 1);
    
    // Verify the semicolon was preserved in the string
    let name_value = rows[0].get("name").unwrap();
    assert_eq!(name_value.as_str().unwrap(), "John; Doe");
    
    // Clean up
    let _ = query_executor::execute_sql(&client, "DROP TABLE test_multi_string").await;
}

#[tokio::test]
async fn test_execute_multiple_statements_with_comments() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    // Setup
    let _ = query_executor::execute_sql(&client, "DROP TABLE IF EXISTS test_multi_comments").await;
    
    // Execute statements with comments containing semicolons
    let sql = "-- First statement; this semicolon is in a comment\nCREATE TABLE test_multi_comments (id INTEGER); /* Another comment; with semicolon */ INSERT INTO test_multi_comments VALUES (1)";
    let result = query_executor::execute_sql(&client, sql).await;
    
    // Should succeed
    assert_eq!(result.result_type, QueryResultType::Insert);
    assert!(result.error.is_none());
    
    // Clean up
    let _ = query_executor::execute_sql(&client, "DROP TABLE test_multi_comments").await;
}

#[tokio::test]
async fn test_execute_multiple_dml_accumulates_affected_rows() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    // Setup
    let _ = query_executor::execute_sql(&client, "DROP TABLE IF EXISTS test_multi_dml").await;
    let _ = query_executor::execute_sql(&client, "CREATE TABLE test_multi_dml (id INTEGER PRIMARY KEY, name VARCHAR(100))").await;
    let _ = query_executor::execute_sql(&client, "INSERT INTO test_multi_dml (id, name) VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie')").await;
    
    // Execute multiple UPDATE statements
    let sql = "UPDATE test_multi_dml SET name = 'Updated1' WHERE id = 1; UPDATE test_multi_dml SET name = 'Updated2' WHERE id = 2";
    let result = query_executor::execute_sql(&client, sql).await;
    
    // Should accumulate affected rows from both UPDATEs
    assert_eq!(result.result_type, QueryResultType::Update);
    assert_eq!(result.affected_rows, Some(2));
    assert!(result.error.is_none());
    
    // Clean up
    let _ = query_executor::execute_sql(&client, "DROP TABLE test_multi_dml").await;
}

// ============================================================================
// Error Handling Tests
// These tests verify that SQL errors are properly caught, parsed, and 
// converted to user-friendly messages
// ============================================================================

#[tokio::test]
async fn test_error_handling_syntax_error() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    // Invalid SQL syntax - missing FROM keyword
    let sql = "SELECT * pg_tables";  // Missing FROM
    let result = query_executor::execute_sql(&client, sql).await;
    
    assert_eq!(result.result_type, QueryResultType::Error);
    assert!(result.error.is_some());
    
    let error = result.error.unwrap();
    assert!(error.contains("Syntax error") || error.contains("syntax"), 
        "Error should mention syntax error: {}", error);
}

#[tokio::test]
async fn test_error_handling_table_not_exists() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    // Reference non-existent table
    let sql = "SELECT * FROM nonexistent_table_xyz_12345";
    let result = query_executor::execute_sql(&client, sql).await;
    
    assert_eq!(result.result_type, QueryResultType::Error);
    assert!(result.error.is_some());
    
    let error = result.error.unwrap();
    assert!(error.contains("Table does not exist") || error.contains("does not exist"), 
        "Error should mention table not found: {}", error);
}

#[tokio::test]
async fn test_error_handling_column_not_exists() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    // Reference non-existent column
    let sql = "SELECT nonexistent_column_xyz FROM pg_tables LIMIT 1";
    let result = query_executor::execute_sql(&client, sql).await;
    
    assert_eq!(result.result_type, QueryResultType::Error);
    assert!(result.error.is_some());
    
    let error = result.error.unwrap();
    assert!(error.contains("Column does not exist") || error.contains("column"), 
        "Error should mention column not found: {}", error);
}

#[tokio::test]
async fn test_error_handling_unique_constraint_violation() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    // Setup
    let _ = query_executor::execute_sql(&client, "DROP TABLE IF EXISTS test_unique_error").await;
    let _ = query_executor::execute_sql(&client, "CREATE TABLE test_unique_error (id INTEGER PRIMARY KEY, email VARCHAR(100) UNIQUE)").await;
    let _ = query_executor::execute_sql(&client, "INSERT INTO test_unique_error (id, email) VALUES (1, 'test@example.com')").await;
    
    // Try to insert duplicate email
    let sql = "INSERT INTO test_unique_error (id, email) VALUES (2, 'test@example.com')";
    let result = query_executor::execute_sql(&client, sql).await;
    
    assert_eq!(result.result_type, QueryResultType::Error);
    assert!(result.error.is_some());
    
    let error = result.error.unwrap();
    assert!(error.contains("Unique constraint violation") || error.contains("unique"), 
        "Error should mention unique constraint: {}", error);
    
    // Clean up
    let _ = query_executor::execute_sql(&client, "DROP TABLE test_unique_error").await;
}

#[tokio::test]
async fn test_error_handling_not_null_constraint_violation() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    // Setup
    let _ = query_executor::execute_sql(&client, "DROP TABLE IF EXISTS test_not_null_error").await;
    let _ = query_executor::execute_sql(&client, "CREATE TABLE test_not_null_error (id INTEGER PRIMARY KEY, name VARCHAR(100) NOT NULL)").await;
    
    // Try to insert NULL into NOT NULL column
    let sql = "INSERT INTO test_not_null_error (id, name) VALUES (1, NULL)";
    let result = query_executor::execute_sql(&client, sql).await;
    
    assert_eq!(result.result_type, QueryResultType::Error);
    assert!(result.error.is_some());
    
    let error = result.error.unwrap();
    assert!(error.contains("Not null constraint violation") || error.contains("null"), 
        "Error should mention not null constraint: {}", error);
    
    // Clean up
    let _ = query_executor::execute_sql(&client, "DROP TABLE test_not_null_error").await;
}

#[tokio::test]
async fn test_error_handling_foreign_key_constraint_violation() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    // Setup
    let _ = query_executor::execute_sql(&client, "DROP TABLE IF EXISTS test_fk_child").await;
    let _ = query_executor::execute_sql(&client, "DROP TABLE IF EXISTS test_fk_parent").await;
    let _ = query_executor::execute_sql(&client, "CREATE TABLE test_fk_parent (id INTEGER PRIMARY KEY)").await;
    let _ = query_executor::execute_sql(&client, "CREATE TABLE test_fk_child (id INTEGER PRIMARY KEY, parent_id INTEGER REFERENCES test_fk_parent(id))").await;
    
    // Try to insert with non-existent parent
    let sql = "INSERT INTO test_fk_child (id, parent_id) VALUES (1, 999)";
    let result = query_executor::execute_sql(&client, sql).await;
    
    assert_eq!(result.result_type, QueryResultType::Error);
    assert!(result.error.is_some());
    
    let error = result.error.unwrap();
    assert!(error.contains("Foreign key constraint violation") || error.contains("foreign key"), 
        "Error should mention foreign key constraint: {}", error);
    
    // Clean up
    let _ = query_executor::execute_sql(&client, "DROP TABLE test_fk_child").await;
    let _ = query_executor::execute_sql(&client, "DROP TABLE test_fk_parent").await;
}

#[tokio::test]
async fn test_error_handling_check_constraint_violation() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    // Setup
    let _ = query_executor::execute_sql(&client, "DROP TABLE IF EXISTS test_check_error").await;
    let _ = query_executor::execute_sql(&client, "CREATE TABLE test_check_error (id INTEGER PRIMARY KEY, age INTEGER CHECK (age >= 0))").await;
    
    // Try to insert negative age
    let sql = "INSERT INTO test_check_error (id, age) VALUES (1, -5)";
    let result = query_executor::execute_sql(&client, sql).await;
    
    assert_eq!(result.result_type, QueryResultType::Error);
    assert!(result.error.is_some());
    
    let error = result.error.unwrap();
    assert!(error.contains("Check constraint violation") || error.contains("check"), 
        "Error should mention check constraint: {}", error);
    
    // Clean up
    let _ = query_executor::execute_sql(&client, "DROP TABLE test_check_error").await;
}

#[tokio::test]
async fn test_error_handling_data_type_mismatch() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    // Setup
    let _ = query_executor::execute_sql(&client, "DROP TABLE IF EXISTS test_type_error").await;
    let _ = query_executor::execute_sql(&client, "CREATE TABLE test_type_error (id INTEGER PRIMARY KEY, value INTEGER)").await;
    
    // Try to insert string into integer column
    let sql = "INSERT INTO test_type_error (id, value) VALUES (1, 'not a number')";
    let result = query_executor::execute_sql(&client, sql).await;
    
    assert_eq!(result.result_type, QueryResultType::Error);
    assert!(result.error.is_some());
    
    let error = result.error.unwrap();
    assert!(error.contains("Invalid text representation") || error.contains("invalid input"), 
        "Error should mention type conversion error: {}", error);
    
    // Clean up
    let _ = query_executor::execute_sql(&client, "DROP TABLE test_type_error").await;
}

#[tokio::test]
async fn test_error_handling_division_by_zero() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    // Try to divide by zero
    let sql = "SELECT 1 / 0";
    let result = query_executor::execute_sql(&client, sql).await;
    
    assert_eq!(result.result_type, QueryResultType::Error);
    assert!(result.error.is_some());
    
    let error = result.error.unwrap();
    assert!(error.contains("Division by zero") || error.contains("division by zero"), 
        "Error should mention division by zero: {}", error);
}

#[tokio::test]
async fn test_error_handling_table_already_exists() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    // Setup - create table
    let _ = query_executor::execute_sql(&client, "DROP TABLE IF EXISTS test_duplicate_table").await;
    let _ = query_executor::execute_sql(&client, "CREATE TABLE test_duplicate_table (id INTEGER)").await;
    
    // Try to create the same table again
    let sql = "CREATE TABLE test_duplicate_table (id INTEGER)";
    let result = query_executor::execute_sql(&client, sql).await;
    
    assert_eq!(result.result_type, QueryResultType::Error);
    assert!(result.error.is_some());
    
    let error = result.error.unwrap();
    assert!(error.contains("Table already exists") || error.contains("already exists"), 
        "Error should mention table already exists: {}", error);
    
    // Clean up
    let _ = query_executor::execute_sql(&client, "DROP TABLE test_duplicate_table").await;
}

#[tokio::test]
async fn test_error_position_extraction() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    // SQL with syntax error at a specific position
    let sql = "SELECT * FORM pg_tables";  // Typo: FORM instead of FROM
    let result = query_executor::execute_sql(&client, sql).await;
    
    assert_eq!(result.result_type, QueryResultType::Error);
    assert!(result.error.is_some());
    
    // Check if error position is extracted
    // Note: PostgreSQL may or may not provide position for all errors
    if let Some(pos) = result.error_position {
        assert!(pos.line > 0, "Line number should be positive");
        assert!(pos.column > 0, "Column number should be positive");
    }
}

#[tokio::test]
async fn test_error_message_includes_technical_details() {
    let client = match get_test_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test - cannot connect to database: {}", e);
            return;
        }
    };
    
    // Setup
    let _ = query_executor::execute_sql(&client, "DROP TABLE IF EXISTS test_error_details").await;
    let _ = query_executor::execute_sql(&client, "CREATE TABLE test_error_details (id INTEGER PRIMARY KEY)").await;
    let _ = query_executor::execute_sql(&client, "INSERT INTO test_error_details (id) VALUES (1)").await;
    
    // Try to insert duplicate primary key
    let sql = "INSERT INTO test_error_details (id) VALUES (1)";
    let result = query_executor::execute_sql(&client, sql).await;
    
    assert_eq!(result.result_type, QueryResultType::Error);
    assert!(result.error.is_some());
    
    let error = result.error.unwrap();
    // Should contain both user-friendly message and technical details
    assert!(error.contains("Technical details") || error.contains("Error code"), 
        "Error should include technical details: {}", error);
    
    // Clean up
    let _ = query_executor::execute_sql(&client, "DROP TABLE test_error_details").await;
}
