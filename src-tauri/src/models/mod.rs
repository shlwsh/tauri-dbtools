/**
 * Models Module
 * 
 * This module exports all data model types used throughout the application.
 * It includes types for:
 * - Query execution results (query.rs)
 * - Database schema definitions (schema.rs)
 * - Data manipulation operations (data.rs)
 */

pub mod query;
pub mod schema;
pub mod data;

// Re-export commonly used types for convenience
pub use query::{QueryResult, QueryResultType, ColumnInfo, ErrorPosition};
pub use schema::{
    TableSchema, ColumnDefinition, ConstraintDefinition, IndexDefinition,
    TableDesign, TableChanges, ColumnModification,
};
pub use data::{
    RowUpdate, BatchUpdateRequest, BatchInsertRequest, BatchDeleteRequest,
    BatchOperationResponse,
};
