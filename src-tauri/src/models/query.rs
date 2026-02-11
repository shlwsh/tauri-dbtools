/**
 * Query Result Type Definitions
 * 
 * This module defines types for SQL query execution results including:
 * - Query result structure with columns and rows
 * - Query result types (SELECT, INSERT, UPDATE, DELETE, DDL, Error)
 * - Error position information
 * - Column metadata
 * 
 * Validates: Requirements 2.1, 2.3, 2.4, 2.5, 2.7
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of a SQL query execution
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueryResult {
    /// Type of query result
    pub result_type: QueryResultType,
    /// Column information for SELECT queries
    pub columns: Option<Vec<ColumnInfo>>,
    /// Row data for SELECT queries (as JSON values)
    pub rows: Option<Vec<HashMap<String, serde_json::Value>>>,
    /// Number of rows affected by DML operations
    pub affected_rows: Option<u64>,
    /// Query execution duration in milliseconds
    pub duration_ms: u64,
    /// Error message if query failed
    pub error: Option<String>,
    /// Position of error in SQL (if available)
    pub error_position: Option<ErrorPosition>,
}

/// Type of query result
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum QueryResultType {
    /// SELECT query that returns data
    Select,
    /// INSERT operation
    Insert,
    /// UPDATE operation
    Update,
    /// DELETE operation
    Delete,
    /// DDL operation (CREATE, ALTER, DROP, etc.)
    Ddl,
    /// Query execution error
    Error,
}

/// Position of an error in SQL text
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorPosition {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
}

/// Information about a database column
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ColumnInfo {
    /// Column name
    pub name: String,
    /// PostgreSQL type name (e.g., "integer", "varchar", "timestamp")
    pub type_name: String,
    /// Whether the column allows NULL values
    pub nullable: bool,
    /// Whether this column is part of the primary key
    pub is_primary_key: bool,
}

impl QueryResult {
    /// Create a successful SELECT result
    pub fn select(
        columns: Vec<ColumnInfo>,
        rows: Vec<HashMap<String, serde_json::Value>>,
        duration_ms: u64,
    ) -> Self {
        Self {
            result_type: QueryResultType::Select,
            columns: Some(columns),
            rows: Some(rows),
            affected_rows: None,
            duration_ms,
            error: None,
            error_position: None,
        }
    }

    /// Create a successful DML result (INSERT, UPDATE, DELETE)
    pub fn dml(result_type: QueryResultType, affected_rows: u64, duration_ms: u64) -> Self {
        Self {
            result_type,
            columns: None,
            rows: None,
            affected_rows: Some(affected_rows),
            duration_ms,
            error: None,
            error_position: None,
        }
    }

    /// Create a successful DDL result
    pub fn ddl(duration_ms: u64) -> Self {
        Self {
            result_type: QueryResultType::Ddl,
            columns: None,
            rows: None,
            affected_rows: None,
            duration_ms,
            error: None,
            error_position: None,
        }
    }

    /// Create an error result
    pub fn error(error: String, error_position: Option<ErrorPosition>, duration_ms: u64) -> Self {
        Self {
            result_type: QueryResultType::Error,
            columns: None,
            rows: None,
            affected_rows: None,
            duration_ms,
            error: Some(error),
            error_position,
        }
    }
}

impl ColumnInfo {
    /// Create a new ColumnInfo
    pub fn new(name: String, type_name: String, nullable: bool, is_primary_key: bool) -> Self {
        Self {
            name,
            type_name,
            nullable,
            is_primary_key,
        }
    }
}

impl ErrorPosition {
    /// Create a new ErrorPosition
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_result_select() {
        let columns = vec![ColumnInfo::new(
            "id".to_string(),
            "integer".to_string(),
            false,
            true,
        )];
        let rows = vec![HashMap::from([(
            "id".to_string(),
            serde_json::Value::Number(1.into()),
        )])];
        let result = QueryResult::select(columns, rows, 100);

        assert_eq!(result.result_type, QueryResultType::Select);
        assert!(result.columns.is_some());
        assert!(result.rows.is_some());
        assert_eq!(result.duration_ms, 100);
    }

    #[test]
    fn test_query_result_dml() {
        let result = QueryResult::dml(QueryResultType::Update, 5, 50);

        assert_eq!(result.result_type, QueryResultType::Update);
        assert_eq!(result.affected_rows, Some(5));
        assert_eq!(result.duration_ms, 50);
    }

    #[test]
    fn test_query_result_error() {
        let error_pos = ErrorPosition::new(1, 10);
        let result = QueryResult::error("Syntax error".to_string(), Some(error_pos), 10);

        assert_eq!(result.result_type, QueryResultType::Error);
        assert!(result.error.is_some());
        assert!(result.error_position.is_some());
    }
}
