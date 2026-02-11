/**
 * Data Operation Type Definitions
 * 
 * This module defines types for data manipulation operations including:
 * - Row update operations
 * - Batch update, insert, and delete requests
 * - Data modification tracking
 * 
 * Validates: Requirements 9.1, 10.1, 12.1
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Update operation for a single row
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RowUpdate {
    /// Primary key values identifying the row to update
    pub primary_key: HashMap<String, serde_json::Value>,
    /// Column values to update
    pub changes: HashMap<String, serde_json::Value>,
}

/// Request to update multiple rows in a batch
#[derive(Debug, Deserialize, Clone)]
pub struct BatchUpdateRequest {
    /// Database name
    pub database: String,
    /// Schema name
    pub schema: String,
    /// Table name
    pub table: String,
    /// Array of row updates
    pub updates: Vec<RowUpdate>,
}

/// Request to insert multiple rows in a batch
#[derive(Debug, Deserialize, Clone)]
pub struct BatchInsertRequest {
    /// Database name
    pub database: String,
    /// Schema name
    pub schema: String,
    /// Table name
    pub table: String,
    /// Array of rows to insert (each row is a map of column name to value)
    pub rows: Vec<HashMap<String, serde_json::Value>>,
}

/// Request to delete multiple rows in a batch
#[derive(Debug, Deserialize, Clone)]
pub struct BatchDeleteRequest {
    /// Database name
    pub database: String,
    /// Schema name
    pub schema: String,
    /// Table name
    pub table: String,
    /// Array of primary key values identifying rows to delete
    pub primary_keys: Vec<HashMap<String, serde_json::Value>>,
}

/// Response from a batch operation
#[derive(Debug, Serialize, Clone)]
pub struct BatchOperationResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Number of rows affected
    pub rows_affected: u64,
    /// Error message if operation failed
    pub error: Option<String>,
}

impl RowUpdate {
    /// Create a new RowUpdate
    pub fn new(
        primary_key: HashMap<String, serde_json::Value>,
        changes: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            primary_key,
            changes,
        }
    }
}

impl BatchUpdateRequest {
    /// Create a new BatchUpdateRequest
    pub fn new(database: String, schema: String, table: String, updates: Vec<RowUpdate>) -> Self {
        Self {
            database,
            schema,
            table,
            updates,
        }
    }
}

impl BatchInsertRequest {
    /// Create a new BatchInsertRequest
    pub fn new(
        database: String,
        schema: String,
        table: String,
        rows: Vec<HashMap<String, serde_json::Value>>,
    ) -> Self {
        Self {
            database,
            schema,
            table,
            rows,
        }
    }
}

impl BatchDeleteRequest {
    /// Create a new BatchDeleteRequest
    pub fn new(
        database: String,
        schema: String,
        table: String,
        primary_keys: Vec<HashMap<String, serde_json::Value>>,
    ) -> Self {
        Self {
            database,
            schema,
            table,
            primary_keys,
        }
    }
}

impl BatchOperationResponse {
    /// Create a successful response
    pub fn success(rows_affected: u64) -> Self {
        Self {
            success: true,
            rows_affected,
            error: None,
        }
    }

    /// Create an error response
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            rows_affected: 0,
            error: Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_update_creation() {
        let mut primary_key = HashMap::new();
        primary_key.insert("id".to_string(), serde_json::Value::Number(1.into()));

        let mut changes = HashMap::new();
        changes.insert(
            "name".to_string(),
            serde_json::Value::String("John".to_string()),
        );

        let update = RowUpdate::new(primary_key, changes);

        assert_eq!(update.primary_key.len(), 1);
        assert_eq!(update.changes.len(), 1);
    }

    #[test]
    fn test_batch_update_request() {
        let updates = vec![RowUpdate::new(
            HashMap::from([("id".to_string(), serde_json::Value::Number(1.into()))]),
            HashMap::from([(
                "name".to_string(),
                serde_json::Value::String("John".to_string()),
            )]),
        )];

        let request = BatchUpdateRequest::new(
            "mydb".to_string(),
            "public".to_string(),
            "users".to_string(),
            updates,
        );

        assert_eq!(request.database, "mydb");
        assert_eq!(request.table, "users");
        assert_eq!(request.updates.len(), 1);
    }

    #[test]
    fn test_batch_insert_request() {
        let rows = vec![HashMap::from([
            ("id".to_string(), serde_json::Value::Number(1.into())),
            (
                "name".to_string(),
                serde_json::Value::String("John".to_string()),
            ),
        ])];

        let request = BatchInsertRequest::new(
            "mydb".to_string(),
            "public".to_string(),
            "users".to_string(),
            rows,
        );

        assert_eq!(request.rows.len(), 1);
    }

    #[test]
    fn test_batch_delete_request() {
        let primary_keys = vec![HashMap::from([(
            "id".to_string(),
            serde_json::Value::Number(1.into()),
        )])];

        let request = BatchDeleteRequest::new(
            "mydb".to_string(),
            "public".to_string(),
            "users".to_string(),
            primary_keys,
        );

        assert_eq!(request.primary_keys.len(), 1);
    }

    #[test]
    fn test_batch_operation_response() {
        let success = BatchOperationResponse::success(5);
        assert!(success.success);
        assert_eq!(success.rows_affected, 5);
        assert!(success.error.is_none());

        let error = BatchOperationResponse::error("Database error".to_string());
        assert!(!error.success);
        assert_eq!(error.rows_affected, 0);
        assert!(error.error.is_some());
    }
}
