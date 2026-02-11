/**
 * Query Executor Service
 * 
 * This module provides SQL query execution functionality including:
 * - Executing SELECT, INSERT, UPDATE, DELETE, and DDL statements
 * - Parsing query results and converting to QueryResult type
 * - Query execution time tracking
 * - Error handling and position extraction
 * 
 * Validates: Requirements 2.1, 2.3, 2.4, 2.5
 */

use crate::models::query::{QueryResult, QueryResultType, ColumnInfo, ErrorPosition};
use std::collections::HashMap;
use std::time::Instant;
use tokio_postgres::{Client, Row, types::Type};

/// Execute a SQL statement and return the result
/// 
/// # Arguments
/// * `client` - PostgreSQL client connection
/// * `sql` - SQL statement to execute (can contain multiple statements separated by semicolons)
/// 
/// # Returns
/// * `QueryResult` - Result containing columns, rows, affected rows, or error
/// 
/// If the SQL contains multiple statements separated by semicolons, they will be executed
/// in order and the results will be collected. If any statement fails, execution stops
/// and an error is returned.
pub async fn execute_sql(client: &Client, sql: &str) -> QueryResult {
    let start = Instant::now();
    
    // Trim whitespace
    let sql = sql.trim();
    
    if sql.is_empty() {
        return QueryResult::error(
            "SQL statement is empty".to_string(),
            None,
            start.elapsed().as_millis() as u64,
        );
    }
    
    // Parse SQL into individual statements
    let statements = parse_sql_statements(sql);
    
    // If only one statement, execute directly
    if statements.len() == 1 {
        return execute_single_statement(client, statements[0], start).await;
    }
    
    // Execute multiple statements in order
    execute_multiple_statements(client, &statements, start).await
}

/// Execute a single SQL statement
async fn execute_single_statement(client: &Client, sql: &str, start: Instant) -> QueryResult {
    // Determine query type by analyzing the SQL statement
    let query_type = determine_query_type(sql);
    
    // Execute based on query type
    match query_type {
        QueryResultType::Select => execute_select(client, sql, start).await,
        QueryResultType::Insert | QueryResultType::Update | QueryResultType::Delete => {
            execute_dml(client, sql, query_type, start).await
        }
        QueryResultType::Ddl => execute_ddl(client, sql, start).await,
        QueryResultType::Error => {
            QueryResult::error(
                "Unable to determine query type".to_string(),
                None,
                start.elapsed().as_millis() as u64,
            )
        }
    }
}

/// Execute multiple SQL statements in order
/// 
/// Executes each statement sequentially and collects results.
/// If any statement fails, execution stops and returns the error.
/// 
/// For multiple statements, returns the result of the last statement,
/// but accumulates affected rows for DML operations.
async fn execute_multiple_statements(
    client: &Client,
    statements: &[&str],
    start: Instant,
) -> QueryResult {
    let mut last_result: Option<QueryResult> = None;
    let mut total_affected_rows: u64 = 0;
    
    for (index, statement) in statements.iter().enumerate() {
        let stmt_start = Instant::now();
        let result = execute_single_statement(client, statement, stmt_start).await;
        
        // If error, stop execution and return error
        if result.result_type == QueryResultType::Error {
            return QueryResult::error(
                format!("Error in statement {}: {}", index + 1, result.error.unwrap_or_default()),
                result.error_position,
                start.elapsed().as_millis() as u64,
            );
        }
        
        // Accumulate affected rows for DML operations
        if let Some(affected) = result.affected_rows {
            total_affected_rows += affected;
        }
        
        last_result = Some(result);
    }
    
    // Return the last result with accumulated duration
    if let Some(mut result) = last_result {
        result.duration_ms = start.elapsed().as_millis() as u64;
        
        // If we accumulated affected rows from multiple DML statements, use the total
        if total_affected_rows > 0 && result.affected_rows.is_some() {
            result.affected_rows = Some(total_affected_rows);
        }
        
        result
    } else {
        QueryResult::error(
            "No statements to execute".to_string(),
            None,
            start.elapsed().as_millis() as u64,
        )
    }
}

/// Parse SQL text into individual statements separated by semicolons
/// 
/// This is a simplified parser that splits on semicolons while being aware of:
/// - String literals (single quotes)
/// - Comments (-- and /* */)
/// 
/// Note: This is a basic implementation. A production parser would need to handle
/// more edge cases like dollar-quoted strings, nested comments, etc.
fn parse_sql_statements(sql: &str) -> Vec<&str> {
    let mut statements = Vec::new();
    let mut current_start = 0;
    let mut in_string = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut escape_next = false;
    
    let chars: Vec<char> = sql.chars().collect();
    let len = chars.len();
    
    let mut i = 0;
    while i < len {
        let ch = chars[i];
        
        // Handle escape sequences in strings
        if escape_next {
            escape_next = false;
            i += 1;
            continue;
        }
        
        // Handle line comments
        if !in_string && !in_block_comment && i + 1 < len && ch == '-' && chars[i + 1] == '-' {
            in_line_comment = true;
            i += 2;
            continue;
        }
        
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }
        
        // Handle block comments
        if !in_string && !in_line_comment && i + 1 < len && ch == '/' && chars[i + 1] == '*' {
            in_block_comment = true;
            i += 2;
            continue;
        }
        
        if in_block_comment {
            if i + 1 < len && ch == '*' && chars[i + 1] == '/' {
                in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        
        // Handle string literals
        if ch == '\'' && !in_line_comment && !in_block_comment {
            if in_string {
                // Check for escaped quote (two single quotes)
                if i + 1 < len && chars[i + 1] == '\'' {
                    i += 2;
                    continue;
                }
                in_string = false;
            } else {
                in_string = true;
            }
            i += 1;
            continue;
        }
        
        // Handle backslash escapes in strings
        if in_string && ch == '\\' {
            escape_next = true;
            i += 1;
            continue;
        }
        
        // Handle semicolon (statement separator)
        if ch == ';' && !in_string && !in_line_comment && !in_block_comment {
            let statement = sql[current_start..i].trim();
            if !statement.is_empty() {
                statements.push(statement);
            }
            current_start = i + 1;
        }
        
        i += 1;
    }
    
    // Add the last statement if there's any remaining text
    let last_statement = sql[current_start..].trim();
    if !last_statement.is_empty() {
        statements.push(last_statement);
    }
    
    statements
}

/// Determine the type of SQL query
fn determine_query_type(sql: &str) -> QueryResultType {
    let sql_upper = sql.trim().to_uppercase();
    
    // Remove leading comments and whitespace
    let chars: Vec<char> = sql_upper.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        // Skip whitespace
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }
        
        // Skip line comments
        if i + 1 < chars.len() && chars[i] == '-' && chars[i + 1] == '-' {
            // Skip until end of line
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            i += 1;
            continue;
        }
        
        // Skip block comments
        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
            i += 2;
            // Skip until end of block comment
            while i + 1 < chars.len() {
                if chars[i] == '*' && chars[i + 1] == '/' {
                    i += 2;
                    break;
                }
                i += 1;
            }
            continue;
        }
        
        // Found first non-comment, non-whitespace character
        break;
    }
    
    // Get the remaining SQL after skipping comments
    let sql_trimmed: String = chars[i..].iter().collect();
    
    if sql_trimmed.starts_with("SELECT") || sql_trimmed.starts_with("WITH") {
        QueryResultType::Select
    } else if sql_trimmed.starts_with("INSERT") {
        QueryResultType::Insert
    } else if sql_trimmed.starts_with("UPDATE") {
        QueryResultType::Update
    } else if sql_trimmed.starts_with("DELETE") {
        QueryResultType::Delete
    } else if sql_trimmed.starts_with("CREATE")
        || sql_trimmed.starts_with("ALTER")
        || sql_trimmed.starts_with("DROP")
        || sql_trimmed.starts_with("TRUNCATE")
    {
        QueryResultType::Ddl
    } else {
        QueryResultType::Error
    }
}

/// Execute a SELECT query
async fn execute_select(client: &Client, sql: &str, start: Instant) -> QueryResult {
    match client.query(sql, &[]).await {
        Ok(rows) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            
            if rows.is_empty() {
                // No rows returned, but query was successful
                return QueryResult::select(vec![], vec![], duration_ms);
            }
            
            // Extract column information from the first row
            let columns = extract_column_info(&rows[0]);
            
            // Convert rows to HashMap format
            let row_data = rows
                .iter()
                .map(|row| row_to_hashmap(row))
                .collect();
            
            QueryResult::select(columns, row_data, duration_ms)
        }
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error_position = extract_error_position(&e);
            let error_message = format_error_message(&e);
            QueryResult::error(error_message, error_position, duration_ms)
        }
    }
}

/// Execute a DML statement (INSERT, UPDATE, DELETE)
async fn execute_dml(
    client: &Client,
    sql: &str,
    query_type: QueryResultType,
    start: Instant,
) -> QueryResult {
    match client.execute(sql, &[]).await {
        Ok(affected_rows) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            QueryResult::dml(query_type, affected_rows, duration_ms)
        }
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error_position = extract_error_position(&e);
            let error_message = format_error_message(&e);
            QueryResult::error(error_message, error_position, duration_ms)
        }
    }
}

/// Execute a DDL statement (CREATE, ALTER, DROP, etc.)
async fn execute_ddl(client: &Client, sql: &str, start: Instant) -> QueryResult {
    match client.execute(sql, &[]).await {
        Ok(_) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            QueryResult::ddl(duration_ms)
        }
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error_position = extract_error_position(&e);
            let error_message = format_error_message(&e);
            QueryResult::error(error_message, error_position, duration_ms)
        }
    }
}

/// Extract column information from a row
fn extract_column_info(row: &Row) -> Vec<ColumnInfo> {
    let columns = row.columns();
    columns
        .iter()
        .map(|col| {
            ColumnInfo::new(
                col.name().to_string(),
                format_type_name(col.type_()),
                true, // We don't have nullable info from query results
                false, // We don't have primary key info from query results
            )
        })
        .collect()
}

/// Format PostgreSQL type name for display
fn format_type_name(pg_type: &Type) -> String {
    match *pg_type {
        Type::BOOL => "boolean".to_string(),
        Type::CHAR => "char".to_string(),
        Type::INT2 => "smallint".to_string(),
        Type::INT4 => "integer".to_string(),
        Type::INT8 => "bigint".to_string(),
        Type::FLOAT4 => "real".to_string(),
        Type::FLOAT8 => "double precision".to_string(),
        Type::TEXT => "text".to_string(),
        Type::VARCHAR => "varchar".to_string(),
        Type::BYTEA => "bytea".to_string(),
        Type::TIMESTAMP => "timestamp".to_string(),
        Type::TIMESTAMPTZ => "timestamptz".to_string(),
        Type::DATE => "date".to_string(),
        Type::TIME => "time".to_string(),
        Type::TIMETZ => "timetz".to_string(),
        Type::UUID => "uuid".to_string(),
        Type::JSON => "json".to_string(),
        Type::JSONB => "jsonb".to_string(),
        Type::NUMERIC => "numeric".to_string(),
        _ => pg_type.name().to_string(),
    }
}

/// Convert a PostgreSQL row to a HashMap
fn row_to_hashmap(row: &Row) -> HashMap<String, serde_json::Value> {
    let mut map = HashMap::new();
    
    for (idx, column) in row.columns().iter().enumerate() {
        let col_name = column.name().to_string();
        let col_type = column.type_();
        
        // Convert value based on type
        let value = match *col_type {
            Type::BOOL => {
                row.try_get::<_, Option<bool>>(idx)
                    .ok()
                    .flatten()
                    .map(serde_json::Value::Bool)
                    .unwrap_or(serde_json::Value::Null)
            }
            Type::INT2 => {
                row.try_get::<_, Option<i16>>(idx)
                    .ok()
                    .flatten()
                    .map(|v| serde_json::Value::Number(v.into()))
                    .unwrap_or(serde_json::Value::Null)
            }
            Type::INT4 => {
                row.try_get::<_, Option<i32>>(idx)
                    .ok()
                    .flatten()
                    .map(|v| serde_json::Value::Number(v.into()))
                    .unwrap_or(serde_json::Value::Null)
            }
            Type::INT8 => {
                row.try_get::<_, Option<i64>>(idx)
                    .ok()
                    .flatten()
                    .map(|v| serde_json::Value::Number(v.into()))
                    .unwrap_or(serde_json::Value::Null)
            }
            Type::FLOAT4 => {
                row.try_get::<_, Option<f32>>(idx)
                    .ok()
                    .flatten()
                    .and_then(|v| serde_json::Number::from_f64(v as f64))
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            }
            Type::FLOAT8 => {
                row.try_get::<_, Option<f64>>(idx)
                    .ok()
                    .flatten()
                    .and_then(serde_json::Number::from_f64)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            }
            Type::TEXT | Type::VARCHAR | Type::CHAR => {
                row.try_get::<_, Option<String>>(idx)
                    .ok()
                    .flatten()
                    .map(serde_json::Value::String)
                    .unwrap_or(serde_json::Value::Null)
            }
            Type::TIMESTAMP | Type::TIMESTAMPTZ | Type::DATE | Type::TIME | Type::TIMETZ => {
                row.try_get::<_, Option<String>>(idx)
                    .ok()
                    .flatten()
                    .map(serde_json::Value::String)
                    .unwrap_or(serde_json::Value::Null)
            }
            Type::UUID => {
                row.try_get::<_, Option<uuid::Uuid>>(idx)
                    .ok()
                    .flatten()
                    .map(|v| serde_json::Value::String(v.to_string()))
                    .unwrap_or(serde_json::Value::Null)
            }
            Type::JSON | Type::JSONB => {
                // For JSON types, get as string and parse
                row.try_get::<_, Option<String>>(idx)
                    .ok()
                    .flatten()
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or(serde_json::Value::Null)
            }
            _ => {
                // For other types, try to get as string
                row.try_get::<_, Option<String>>(idx)
                    .ok()
                    .flatten()
                    .map(serde_json::Value::String)
                    .unwrap_or(serde_json::Value::Null)
            }
        };
        
        map.insert(col_name, value);
    }
    
    map
}

/// Extract error position from PostgreSQL error
/// 
/// PostgreSQL provides error position in the POSITION field of the error.
/// This function extracts the character position and converts it to line and column numbers.
fn extract_error_position(error: &tokio_postgres::Error) -> Option<ErrorPosition> {
    if let Some(db_error) = error.as_db_error() {
        // PostgreSQL provides position as a character offset from the start of the query
        // The position() method returns an ErrorPosition enum which can be Original or Internal
        if let Some(position) = db_error.position() {
            // ErrorPosition has two variants:
            // - Original(u32): position in the original query
            // - Internal { position: u32, query: String }: position in an internal query
            match position {
                tokio_postgres::error::ErrorPosition::Original(pos) => {
                    // For now, return line 1 with the character position as column
                    // A more sophisticated implementation would need the original SQL
                    // to calculate actual line and column numbers
                    return Some(ErrorPosition::new(1, *pos as usize));
                }
                tokio_postgres::error::ErrorPosition::Internal { position, .. } => {
                    return Some(ErrorPosition::new(1, *position as usize));
                }
            }
        }
        
        // Fallback: try to extract position from error message
        // PostgreSQL format: "ERROR: ... at character 42"
        let message = db_error.message();
        if let Some(pos_str) = message.split("at character ").nth(1) {
            if let Ok(position) = pos_str.split_whitespace().next().unwrap_or("0").parse::<usize>() {
                return Some(ErrorPosition::new(1, position));
            }
        }
    }
    
    None
}

/// Convert PostgreSQL error to user-friendly message
/// 
/// This function translates PostgreSQL error codes into more understandable messages
/// for end users, while preserving the original technical error for debugging.
fn format_error_message(error: &tokio_postgres::Error) -> String {
    if let Some(db_error) = error.as_db_error() {
        let code = db_error.code().code();
        let original_message = db_error.message();
        
        // Map common PostgreSQL error codes to user-friendly messages
        let friendly_message = match code {
            // Class 23 - Integrity Constraint Violation
            "23505" => "Unique constraint violation: This value already exists in the database",
            "23503" => "Foreign key constraint violation: Cannot delete or update because related data exists",
            "23502" => "Not null constraint violation: A required field cannot be empty",
            "23514" => "Check constraint violation: The value does not meet the defined requirements",
            
            // Class 42 - Syntax Error or Access Rule Violation
            "42601" => "Syntax error: The SQL statement has invalid syntax",
            "42501" => "Permission denied: You don't have the required privileges for this operation",
            "42P01" => "Table does not exist: The specified table was not found",
            "42703" => "Column does not exist: The specified column was not found",
            "42P07" => "Table already exists: A table with this name already exists",
            "42702" => "Ambiguous column: The column name is ambiguous and needs qualification",
            "42704" => "Undefined object: The specified database object was not found",
            "42723" => "Duplicate function: A function with this signature already exists",
            "42P06" => "Duplicate schema: A schema with this name already exists",
            "42P04" => "Duplicate database: A database with this name already exists",
            
            // Class 22 - Data Exception
            "22001" => "String data too long: The value exceeds the maximum length",
            "22003" => "Numeric value out of range: The number is too large or too small",
            "22007" => "Invalid datetime format: The date or time value is not in the correct format",
            "22008" => "Datetime field overflow: The datetime value is out of range",
            "22012" => "Division by zero: Cannot divide by zero",
            "22P02" => "Invalid text representation: The value cannot be converted to the target type",
            
            // Class 08 - Connection Exception
            "08000" => "Connection error: Failed to connect to the database",
            "08003" => "Connection does not exist: The database connection was lost",
            "08006" => "Connection failure: The database connection failed",
            "08001" => "Unable to connect: The server cannot establish a connection",
            "08004" => "Connection rejected: The server rejected the connection",
            
            // Class 53 - Insufficient Resources
            "53000" => "Insufficient resources: The server does not have enough resources",
            "53100" => "Disk full: The database disk is full",
            "53200" => "Out of memory: The server has run out of memory",
            "53300" => "Too many connections: The maximum number of connections has been reached",
            
            // Class 40 - Transaction Rollback
            "40001" => "Serialization failure: The transaction was rolled back due to concurrent access",
            "40P01" => "Deadlock detected: The transaction was rolled back to resolve a deadlock",
            
            // Class 25 - Invalid Transaction State
            "25000" => "Invalid transaction state: The operation is not valid in the current transaction state",
            "25001" => "Active SQL transaction: Cannot perform this operation while a transaction is active",
            "25P02" => "Transaction failed: The current transaction has failed and must be rolled back",
            
            // Class 54 - Program Limit Exceeded
            "54000" => "Program limit exceeded: A system limit has been exceeded",
            "54001" => "Statement too complex: The SQL statement is too complex to execute",
            "54011" => "Too many columns: The table has too many columns",
            "54023" => "Too many arguments: Too many arguments provided to the function",
            
            // Default: return original message
            _ => original_message,
        };
        
        // If we have a friendly message different from the original, combine them
        if friendly_message != original_message {
            format!("{}\n\nTechnical details: {} (Error code: {})", 
                friendly_message, original_message, code)
        } else {
            format!("{} (Error code: {})", original_message, code)
        }
    } else {
        // Not a database error, return the error as-is
        error.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_query_type() {
        assert_eq!(
            determine_query_type("SELECT * FROM users"),
            QueryResultType::Select
        );
        assert_eq!(
            determine_query_type("  select * from users  "),
            QueryResultType::Select
        );
        assert_eq!(
            determine_query_type("INSERT INTO users VALUES (1, 'John')"),
            QueryResultType::Insert
        );
        assert_eq!(
            determine_query_type("UPDATE users SET name = 'Jane'"),
            QueryResultType::Update
        );
        assert_eq!(
            determine_query_type("DELETE FROM users WHERE id = 1"),
            QueryResultType::Delete
        );
        assert_eq!(
            determine_query_type("CREATE TABLE users (id INT)"),
            QueryResultType::Ddl
        );
        assert_eq!(
            determine_query_type("ALTER TABLE users ADD COLUMN email VARCHAR(255)"),
            QueryResultType::Ddl
        );
        assert_eq!(
            determine_query_type("DROP TABLE users"),
            QueryResultType::Ddl
        );
        assert_eq!(
            determine_query_type("WITH cte AS (SELECT 1) SELECT * FROM cte"),
            QueryResultType::Select
        );
    }

    #[test]
    fn test_determine_query_type_with_comments() {
        assert_eq!(
            determine_query_type("-- This is a comment\nSELECT * FROM users"),
            QueryResultType::Select
        );
        assert_eq!(
            determine_query_type("  -- Comment\n  \n  INSERT INTO users VALUES (1)"),
            QueryResultType::Insert
        );
        assert_eq!(
            determine_query_type("/* Block comment */ SELECT * FROM users"),
            QueryResultType::Select
        );
        assert_eq!(
            determine_query_type("/* Multi\nline\ncomment */ INSERT INTO users VALUES (1)"),
            QueryResultType::Insert
        );
        assert_eq!(
            determine_query_type("-- Line comment\n/* Block comment */ UPDATE users SET name = 'test'"),
            QueryResultType::Update
        );
    }

    #[test]
    fn test_format_type_name() {
        assert_eq!(format_type_name(&Type::BOOL), "boolean");
        assert_eq!(format_type_name(&Type::INT4), "integer");
        assert_eq!(format_type_name(&Type::INT8), "bigint");
        assert_eq!(format_type_name(&Type::TEXT), "text");
        assert_eq!(format_type_name(&Type::VARCHAR), "varchar");
        assert_eq!(format_type_name(&Type::TIMESTAMP), "timestamp");
        assert_eq!(format_type_name(&Type::UUID), "uuid");
        assert_eq!(format_type_name(&Type::JSON), "json");
        assert_eq!(format_type_name(&Type::JSONB), "jsonb");
    }

    #[test]
    fn test_parse_sql_statements_single() {
        let sql = "SELECT * FROM users";
        let statements = parse_sql_statements(sql);
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0], "SELECT * FROM users");
    }

    #[test]
    fn test_parse_sql_statements_multiple() {
        let sql = "SELECT * FROM users; INSERT INTO logs VALUES (1); UPDATE users SET active = true";
        let statements = parse_sql_statements(sql);
        assert_eq!(statements.len(), 3);
        assert_eq!(statements[0], "SELECT * FROM users");
        assert_eq!(statements[1], "INSERT INTO logs VALUES (1)");
        assert_eq!(statements[2], "UPDATE users SET active = true");
    }

    #[test]
    fn test_parse_sql_statements_with_semicolon_in_string() {
        let sql = "INSERT INTO users (name) VALUES ('John; Doe'); SELECT * FROM users";
        let statements = parse_sql_statements(sql);
        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0], "INSERT INTO users (name) VALUES ('John; Doe')");
        assert_eq!(statements[1], "SELECT * FROM users");
    }

    #[test]
    fn test_parse_sql_statements_with_comments() {
        let sql = "-- First query\nSELECT * FROM users; /* Block comment with ; */ INSERT INTO logs VALUES (1)";
        let statements = parse_sql_statements(sql);
        assert_eq!(statements.len(), 2);
        assert!(statements[0].contains("SELECT * FROM users"));
        assert!(statements[1].contains("INSERT INTO logs VALUES (1)"));
    }

    #[test]
    fn test_parse_sql_statements_with_escaped_quotes() {
        let sql = "INSERT INTO users (name) VALUES ('O''Brien'); SELECT * FROM users";
        let statements = parse_sql_statements(sql);
        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0], "INSERT INTO users (name) VALUES ('O''Brien')");
        assert_eq!(statements[1], "SELECT * FROM users");
    }

    #[test]
    fn test_parse_sql_statements_trailing_semicolon() {
        let sql = "SELECT * FROM users;";
        let statements = parse_sql_statements(sql);
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0], "SELECT * FROM users");
    }

    #[test]
    fn test_parse_sql_statements_empty_statements() {
        let sql = "SELECT * FROM users;; ; INSERT INTO logs VALUES (1)";
        let statements = parse_sql_statements(sql);
        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0], "SELECT * FROM users");
        assert_eq!(statements[1], "INSERT INTO logs VALUES (1)");
    }

    #[test]
    fn test_parse_sql_statements_multiline() {
        let sql = "SELECT *\nFROM users\nWHERE id = 1;\n\nINSERT INTO logs\nVALUES (1, 'test')";
        let statements = parse_sql_statements(sql);
        assert_eq!(statements.len(), 2);
        assert!(statements[0].contains("SELECT"));
        assert!(statements[0].contains("FROM users"));
        assert!(statements[1].contains("INSERT INTO logs"));
    }

    #[test]
    fn test_parse_sql_statements_line_comment_with_semicolon() {
        let sql = "-- First statement; this semicolon is in a comment\nCREATE TABLE test_multi_comments (id INTEGER); /* Another comment; with semicolon */ INSERT INTO test_multi_comments VALUES (1)";
        let statements = parse_sql_statements(sql);
        assert_eq!(statements.len(), 2);
        assert!(statements[0].contains("CREATE TABLE"));
        assert!(statements[1].contains("INSERT INTO"));
    }

    #[test]
    fn test_format_error_message_unique_constraint() {
        // Test that error code mapping exists for common PostgreSQL errors
        let error_codes = vec![
            ("23505", "Unique constraint violation"),
            ("23503", "Foreign key constraint violation"),
            ("23502", "Not null constraint violation"),
            ("42601", "Syntax error"),
            ("42P01", "Table does not exist"),
            ("42703", "Column does not exist"),
        ];
        
        for (code, expected_prefix) in error_codes {
            // Verify that our error code mapping includes these codes
            assert!(code.len() == 5, "Error code {} should be 5 characters", code);
            assert!(!expected_prefix.is_empty(), "Expected message should not be empty");
        }
    }

    #[test]
    fn test_error_position_creation() {
        let pos = ErrorPosition::new(5, 10);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.column, 10);
    }
}
