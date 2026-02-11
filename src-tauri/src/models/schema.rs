/**
 * Schema Type Definitions
 * 
 * This module defines types for database schema management including:
 * - Table schema with columns, constraints, and indexes
 * - Column definitions with data types and properties
 * - Constraint definitions (primary key, foreign key, unique, check)
 * - Index definitions
 * - Table design and modification structures
 * 
 * Validates: Requirements 5.1, 6.1, 7.1, 8.1
 */

use serde::{Deserialize, Serialize};

/// Complete schema information for a table
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TableSchema {
    /// Table name
    pub table_name: String,
    /// Schema name
    pub schema: String,
    /// List of column definitions
    pub columns: Vec<ColumnDefinition>,
    /// List of constraints
    pub constraints: Vec<ConstraintDefinition>,
    /// List of indexes
    pub indexes: Vec<IndexDefinition>,
}

/// Definition of a table column
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ColumnDefinition {
    /// Column name
    pub name: String,
    /// PostgreSQL data type (e.g., "integer", "character varying")
    pub data_type: String,
    /// Maximum length for character types
    pub character_maximum_length: Option<i32>,
    /// Numeric precision
    pub numeric_precision: Option<i32>,
    /// Numeric scale
    pub numeric_scale: Option<i32>,
    /// Whether the column allows NULL values
    pub is_nullable: bool,
    /// Default value expression
    pub column_default: Option<String>,
    /// Whether this column is part of the primary key
    pub is_primary_key: bool,
    /// Whether this column has a unique constraint
    pub is_unique: bool,
}

/// Definition of a table constraint
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConstraintDefinition {
    /// Type of constraint (PRIMARY KEY, FOREIGN KEY, UNIQUE, CHECK)
    pub constraint_type: String,
    /// Constraint name
    pub constraint_name: String,
    /// Columns involved in the constraint
    pub columns: Vec<String>,
    /// Referenced table (for foreign key)
    pub referenced_table: Option<String>,
    /// Referenced columns (for foreign key)
    pub referenced_columns: Option<Vec<String>>,
    /// ON DELETE action (for foreign key)
    pub on_delete: Option<String>,
    /// ON UPDATE action (for foreign key)
    pub on_update: Option<String>,
    /// Check expression (for check constraint)
    pub check_clause: Option<String>,
}

/// Definition of a table index
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexDefinition {
    /// Index name
    pub index_name: String,
    /// Columns included in the index
    pub columns: Vec<String>,
    /// Index type (btree, hash, gist, gin)
    pub index_type: String,
    /// Whether this is a unique index
    pub is_unique: bool,
}

/// Design for creating or modifying a table
#[derive(Debug, Deserialize, Clone)]
pub struct TableDesign {
    /// Table name
    pub table_name: String,
    /// Schema name
    pub schema: String,
    /// List of column definitions
    pub columns: Vec<ColumnDefinition>,
    /// List of constraints
    pub constraints: Vec<ConstraintDefinition>,
    /// List of indexes
    pub indexes: Vec<IndexDefinition>,
}

/// Changes to be applied to an existing table
#[derive(Debug, Deserialize, Clone)]
pub struct TableChanges {
    /// Columns to be added
    pub added_columns: Vec<ColumnDefinition>,
    /// Columns to be modified
    pub modified_columns: Vec<ColumnModification>,
    /// Column names to be dropped
    pub dropped_columns: Vec<String>,
    /// Constraints to be added
    pub added_constraints: Vec<ConstraintDefinition>,
    /// Constraint names to be dropped
    pub dropped_constraints: Vec<String>,
    /// Indexes to be added
    pub added_indexes: Vec<IndexDefinition>,
    /// Index names to be dropped
    pub dropped_indexes: Vec<String>,
}

/// Modification to an existing column
#[derive(Debug, Deserialize, Clone)]
pub struct ColumnModification {
    /// Original column name
    pub old_name: String,
    /// New column definition
    pub new_definition: ColumnDefinition,
}

impl TableSchema {
    /// Create a new TableSchema
    pub fn new(table_name: String, schema: String) -> Self {
        Self {
            table_name,
            schema,
            columns: Vec::new(),
            constraints: Vec::new(),
            indexes: Vec::new(),
        }
    }

    /// Add a column to the schema
    pub fn add_column(&mut self, column: ColumnDefinition) {
        self.columns.push(column);
    }

    /// Add a constraint to the schema
    pub fn add_constraint(&mut self, constraint: ConstraintDefinition) {
        self.constraints.push(constraint);
    }

    /// Add an index to the schema
    pub fn add_index(&mut self, index: IndexDefinition) {
        self.indexes.push(index);
    }
}

impl ColumnDefinition {
    /// Create a new ColumnDefinition with basic properties
    pub fn new(name: String, data_type: String, is_nullable: bool) -> Self {
        Self {
            name,
            data_type,
            character_maximum_length: None,
            numeric_precision: None,
            numeric_scale: None,
            is_nullable,
            column_default: None,
            is_primary_key: false,
            is_unique: false,
        }
    }

    /// Set the column as primary key
    pub fn with_primary_key(mut self) -> Self {
        self.is_primary_key = true;
        self.is_nullable = false;
        self
    }

    /// Set the column as unique
    pub fn with_unique(mut self) -> Self {
        self.is_unique = true;
        self
    }

    /// Set the default value
    pub fn with_default(mut self, default: String) -> Self {
        self.column_default = Some(default);
        self
    }

    /// Set character maximum length
    pub fn with_length(mut self, length: i32) -> Self {
        self.character_maximum_length = Some(length);
        self
    }

    /// Set numeric precision and scale
    pub fn with_precision(mut self, precision: i32, scale: Option<i32>) -> Self {
        self.numeric_precision = Some(precision);
        self.numeric_scale = scale;
        self
    }
}

impl ConstraintDefinition {
    /// Create a primary key constraint
    pub fn primary_key(name: String, columns: Vec<String>) -> Self {
        Self {
            constraint_type: "PRIMARY KEY".to_string(),
            constraint_name: name,
            columns,
            referenced_table: None,
            referenced_columns: None,
            on_delete: None,
            on_update: None,
            check_clause: None,
        }
    }

    /// Create a foreign key constraint
    pub fn foreign_key(
        name: String,
        columns: Vec<String>,
        referenced_table: String,
        referenced_columns: Vec<String>,
    ) -> Self {
        Self {
            constraint_type: "FOREIGN KEY".to_string(),
            constraint_name: name,
            columns,
            referenced_table: Some(referenced_table),
            referenced_columns: Some(referenced_columns),
            on_delete: None,
            on_update: None,
            check_clause: None,
        }
    }

    /// Create a unique constraint
    pub fn unique(name: String, columns: Vec<String>) -> Self {
        Self {
            constraint_type: "UNIQUE".to_string(),
            constraint_name: name,
            columns,
            referenced_table: None,
            referenced_columns: None,
            on_delete: None,
            on_update: None,
            check_clause: None,
        }
    }

    /// Create a check constraint
    pub fn check(name: String, check_clause: String) -> Self {
        Self {
            constraint_type: "CHECK".to_string(),
            constraint_name: name,
            columns: Vec::new(),
            referenced_table: None,
            referenced_columns: None,
            on_delete: None,
            on_update: None,
            check_clause: Some(check_clause),
        }
    }

    /// Set ON DELETE action for foreign key
    pub fn with_on_delete(mut self, action: String) -> Self {
        self.on_delete = Some(action);
        self
    }

    /// Set ON UPDATE action for foreign key
    pub fn with_on_update(mut self, action: String) -> Self {
        self.on_update = Some(action);
        self
    }
}

impl IndexDefinition {
    /// Create a new index definition
    pub fn new(name: String, columns: Vec<String>, index_type: String, is_unique: bool) -> Self {
        Self {
            index_name: name,
            columns,
            index_type,
            is_unique,
        }
    }

    /// Create a B-tree index
    pub fn btree(name: String, columns: Vec<String>, is_unique: bool) -> Self {
        Self::new(name, columns, "btree".to_string(), is_unique)
    }

    /// Create a hash index
    pub fn hash(name: String, columns: Vec<String>) -> Self {
        Self::new(name, columns, "hash".to_string(), false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_schema_creation() {
        let mut schema = TableSchema::new("users".to_string(), "public".to_string());
        
        let id_column = ColumnDefinition::new("id".to_string(), "integer".to_string(), false)
            .with_primary_key();
        schema.add_column(id_column);

        assert_eq!(schema.table_name, "users");
        assert_eq!(schema.columns.len(), 1);
        assert!(schema.columns[0].is_primary_key);
    }

    #[test]
    fn test_column_definition_builder() {
        let column = ColumnDefinition::new("email".to_string(), "varchar".to_string(), false)
            .with_length(255)
            .with_unique();

        assert_eq!(column.name, "email");
        assert_eq!(column.character_maximum_length, Some(255));
        assert!(column.is_unique);
    }

    #[test]
    fn test_constraint_definitions() {
        let pk = ConstraintDefinition::primary_key(
            "users_pkey".to_string(),
            vec!["id".to_string()],
        );
        assert_eq!(pk.constraint_type, "PRIMARY KEY");

        let fk = ConstraintDefinition::foreign_key(
            "orders_user_id_fkey".to_string(),
            vec!["user_id".to_string()],
            "users".to_string(),
            vec!["id".to_string()],
        )
        .with_on_delete("CASCADE".to_string());
        
        assert_eq!(fk.constraint_type, "FOREIGN KEY");
        assert_eq!(fk.on_delete, Some("CASCADE".to_string()));
    }

    #[test]
    fn test_index_definition() {
        let index = IndexDefinition::btree(
            "users_email_idx".to_string(),
            vec!["email".to_string()],
            true,
        );

        assert_eq!(index.index_type, "btree");
        assert!(index.is_unique);
    }
}
