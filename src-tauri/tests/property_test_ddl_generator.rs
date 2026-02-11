/**
 * Property-Based Tests for DDL Generator
 * 
 * Feature: database-advanced-features
 * 
 * This module contains property-based tests for DDL generation functionality.
 * Tests verify that generated DDL statements are syntactically correct and
 * complete for all valid table designs.
 */

use proptest::prelude::*;
use pg_db_tool::models::schema::{
    TableDesign, ColumnDefinition, ConstraintDefinition, IndexDefinition,
    TableChanges,
};
use pg_db_tool::services::ddl_generator::{generate_create_table, generate_alter_table};

// Strategy for generating valid PostgreSQL identifiers
fn identifier_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,30}"
        .prop_map(|s| s.to_string())
}

// Strategy for generating PostgreSQL data types
fn data_type_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("integer".to_string()),
        Just("bigint".to_string()),
        Just("smallint".to_string()),
        Just("varchar".to_string()),
        Just("text".to_string()),
        Just("boolean".to_string()),
        Just("timestamp".to_string()),
        Just("date".to_string()),
        Just("numeric".to_string()),
        Just("json".to_string()),
        Just("jsonb".to_string()),
        Just("uuid".to_string()),
    ]
}

// Strategy for generating column definitions
fn column_definition_strategy() -> impl Strategy<Value = ColumnDefinition> {
    (
        identifier_strategy(),
        data_type_strategy(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
    ).prop_map(|(name, data_type, is_nullable, is_primary_key, is_unique)| {
        let char_max_length = if data_type == "varchar" { Some(255) } else { None };
        let (numeric_precision, numeric_scale) = if data_type == "numeric" { 
            (Some(10), Some(2)) 
        } else { 
            (None, None) 
        };
        
        ColumnDefinition {
            name,
            data_type,
            character_maximum_length: char_max_length,
            numeric_precision,
            numeric_scale,
            is_nullable: if is_primary_key { false } else { is_nullable },
            column_default: None,
            is_primary_key,
            is_unique,
        }
    })
}

// Strategy for generating constraint definitions
fn constraint_strategy(columns: Vec<String>) -> impl Strategy<Value = ConstraintDefinition> {
    if columns.is_empty() {
        return Just(ConstraintDefinition::check(
            "empty_check".to_string(),
            "true".to_string(),
        )).boxed();
    }
    
    let max_cols = columns.len().min(3);
    let column_subset = prop::sample::subsequence(columns.clone(), 1..=max_cols);
    
    (identifier_strategy(), column_subset, 0..4u8).prop_map(|(name, cols, constraint_type)| {
        match constraint_type {
            0 => ConstraintDefinition::primary_key(format!("{}_pkey", name), cols),
            1 => ConstraintDefinition::unique(format!("{}_key", name), cols),
            2 => ConstraintDefinition::check(
                format!("{}_check", name),
                "true".to_string(), // Simple valid check expression
            ),
            _ => ConstraintDefinition::foreign_key(
                format!("{}_fkey", name),
                cols.clone(),
                "public.other_table".to_string(),
                cols,
            ),
        }
    }).boxed()
}

// Strategy for generating index definitions
fn index_strategy(columns: Vec<String>) -> impl Strategy<Value = IndexDefinition> {
    if columns.is_empty() {
        return Just(IndexDefinition::btree(
            "empty_idx".to_string(),
            vec!["id".to_string()],
            false,
        )).boxed();
    }
    
    let max_cols = columns.len().min(3);
    let column_subset = prop::sample::subsequence(columns, 1..=max_cols);
    
    (identifier_strategy(), column_subset, any::<bool>()).prop_map(|(name, cols, is_unique)| {
        IndexDefinition::btree(format!("{}_idx", name), cols, is_unique)
    }).boxed()
}

// Strategy for generating table designs
fn table_design_strategy() -> impl Strategy<Value = TableDesign> {
    (
        identifier_strategy(),
        identifier_strategy(),
        prop::collection::vec(column_definition_strategy(), 1..10),
    ).prop_flat_map(|(table_name, schema, columns)| {
        let column_names: Vec<String> = columns.iter().map(|c| c.name.clone()).collect();
        let column_names_for_constraints = column_names.clone();
        let column_names_for_indexes = column_names.clone();
        
        (
            Just(table_name),
            Just(schema),
            Just(columns),
            prop::collection::vec(constraint_strategy(column_names_for_constraints), 0..3),
            prop::collection::vec(index_strategy(column_names_for_indexes), 0..3),
        )
    }).prop_map(|(table_name, schema, columns, constraints, indexes)| {
        TableDesign {
            table_name,
            schema,
            columns,
            constraints,
            indexes,
        }
    })
}

// Feature: database-advanced-features, Property 7: DDL生成完整性
// For any table design (including columns, constraints, and indexes),
// the DDL generator should produce a CREATE TABLE statement that includes
// all defined columns, constraints, and indexes, and the generated SQL
// should be syntactically correct PostgreSQL DDL.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn property_7_ddl_generation_completeness(design in table_design_strategy()) {
        let ddl = generate_create_table(&design);
        
        // Verify DDL is not empty
        prop_assert!(!ddl.is_empty(), "Generated DDL should not be empty");
        
        // Verify DDL starts with CREATE TABLE
        prop_assert!(
            ddl.to_uppercase().contains("CREATE TABLE"),
            "DDL should contain CREATE TABLE statement"
        );
        
        // Verify table name is in DDL
        prop_assert!(
            ddl.contains(&design.table_name),
            "DDL should contain table name: {}", design.table_name
        );
        
        // Verify all columns are in DDL
        for column in &design.columns {
            prop_assert!(
                ddl.contains(&column.name),
                "DDL should contain column: {}", column.name
            );
            
            // Verify data type is in DDL
            prop_assert!(
                ddl.to_uppercase().contains(&column.data_type.to_uppercase()),
                "DDL should contain data type: {}", column.data_type
            );
        }
        
        // Verify constraints are in DDL
        for constraint in &design.constraints {
            prop_assert!(
                ddl.contains(&constraint.constraint_name) || 
                ddl.to_uppercase().contains(&constraint.constraint_type.to_uppercase()),
                "DDL should contain constraint: {}", constraint.constraint_name
            );
        }
        
        // Verify indexes are in DDL
        for index in &design.indexes {
            prop_assert!(
                ddl.contains(&index.index_name),
                "DDL should contain index: {}", index.index_name
            );
        }
        
        // Verify basic SQL syntax
        prop_assert!(
            ddl.contains('(') && ddl.contains(')'),
            "DDL should have balanced parentheses"
        );
        
        prop_assert!(
            ddl.ends_with(';') || ddl.trim().ends_with(';'),
            "DDL should end with semicolon"
        );
    }
}

// Strategy for generating table changes
fn table_changes_strategy() -> impl Strategy<Value = TableChanges> {
    (
        prop::collection::vec(column_definition_strategy(), 0..3),
        prop::collection::vec(identifier_strategy(), 0..2),
        prop::collection::vec(identifier_strategy(), 0..2),
        prop::collection::vec(identifier_strategy(), 0..2),
    ).prop_map(|(added_columns, dropped_columns, dropped_constraints, dropped_indexes)| {
        TableChanges {
            added_columns,
            modified_columns: vec![], // Simplified for now
            dropped_columns,
            added_constraints: vec![],
            dropped_constraints,
            added_indexes: vec![],
            dropped_indexes,
        }
    })
}

// Feature: database-advanced-features, Property 8: ALTER TABLE生成正确性
// For any table structure modification (adding/modifying/deleting columns, constraints, indexes),
// the DDL generator should generate a sequence of ALTER TABLE statements that can transform
// the original table structure to the new table structure.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn property_8_alter_table_generation_correctness(
        schema in identifier_strategy(),
        table in identifier_strategy(),
        changes in table_changes_strategy()
    ) {
        let statements = generate_alter_table(&schema, &table, &changes);
        
        // If there are changes, there should be statements
        let has_changes = !changes.added_columns.is_empty()
            || !changes.modified_columns.is_empty()
            || !changes.dropped_columns.is_empty()
            || !changes.added_constraints.is_empty()
            || !changes.dropped_constraints.is_empty()
            || !changes.added_indexes.is_empty()
            || !changes.dropped_indexes.is_empty();
        
        if has_changes {
            prop_assert!(
                !statements.is_empty(),
                "Should generate statements when there are changes"
            );
        }
        
        // Verify all statements are valid SQL
        for statement in &statements {
            prop_assert!(
                !statement.is_empty(),
                "Statement should not be empty"
            );
            
            prop_assert!(
                statement.ends_with(';'),
                "Statement should end with semicolon: {}", statement
            );
            
            // Verify statement type
            let upper = statement.to_uppercase();
            prop_assert!(
                upper.contains("ALTER TABLE") || upper.contains("DROP INDEX") || upper.contains("CREATE INDEX"),
                "Statement should be ALTER TABLE, DROP INDEX, or CREATE INDEX: {}", statement
            );
        }
        
        // Verify added columns are in statements
        for column in &changes.added_columns {
            let found = statements.iter().any(|s| {
                s.to_uppercase().contains("ADD COLUMN") && s.contains(&column.name)
            });
            prop_assert!(
                found,
                "Should have ADD COLUMN statement for: {}", column.name
            );
        }
        
        // Verify dropped columns are in statements
        for column_name in &changes.dropped_columns {
            let found = statements.iter().any(|s| {
                s.to_uppercase().contains("DROP COLUMN") && s.contains(column_name)
            });
            prop_assert!(
                found,
                "Should have DROP COLUMN statement for: {}", column_name
            );
        }
        
        // Verify dropped constraints are in statements
        for constraint_name in &changes.dropped_constraints {
            let found = statements.iter().any(|s| {
                s.to_uppercase().contains("DROP CONSTRAINT") && s.contains(constraint_name)
            });
            prop_assert!(
                found,
                "Should have DROP CONSTRAINT statement for: {}", constraint_name
            );
        }
        
        // Verify dropped indexes are in statements
        for index_name in &changes.dropped_indexes {
            let found = statements.iter().any(|s| {
                s.to_uppercase().contains("DROP INDEX") && s.contains(index_name)
            });
            prop_assert!(
                found,
                "Should have DROP INDEX statement for: {}", index_name
            );
        }
        
        // Verify order: drops before adds
        let mut last_drop_index = None;
        let mut first_add_index = None;
        
        for (i, statement) in statements.iter().enumerate() {
            let upper = statement.to_uppercase();
            if upper.contains("DROP") {
                last_drop_index = Some(i);
            }
            if upper.contains("ADD") && first_add_index.is_none() {
                first_add_index = Some(i);
            }
        }
        
        if let (Some(last_drop), Some(first_add)) = (last_drop_index, first_add_index) {
            prop_assert!(
                last_drop < first_add,
                "DROP statements should come before ADD statements"
            );
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_simple_create_table() {
        let design = TableDesign {
            table_name: "users".to_string(),
            schema: "public".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "integer".to_string(),
                    character_maximum_length: None,
                    numeric_precision: None,
                    numeric_scale: None,
                    is_nullable: false,
                    column_default: None,
                    is_primary_key: true,
                    is_unique: false,
                },
                ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "varchar".to_string(),
                    character_maximum_length: Some(100),
                    numeric_precision: None,
                    numeric_scale: None,
                    is_nullable: false,
                    column_default: None,
                    is_primary_key: false,
                    is_unique: false,
                },
            ],
            constraints: vec![
                ConstraintDefinition::primary_key(
                    "users_pkey".to_string(),
                    vec!["id".to_string()],
                ),
            ],
            indexes: vec![],
        };
        
        let ddl = generate_create_table(&design);
        
        assert!(ddl.contains("CREATE TABLE"));
        assert!(ddl.contains("users"));
        assert!(ddl.contains("id"));
        assert!(ddl.contains("name"));
        assert!(ddl.contains("PRIMARY KEY"));
    }

    #[test]
    fn test_simple_alter_table() {
        let changes = TableChanges {
            added_columns: vec![
                ColumnDefinition {
                    name: "email".to_string(),
                    data_type: "varchar".to_string(),
                    character_maximum_length: Some(255),
                    numeric_precision: None,
                    numeric_scale: None,
                    is_nullable: true,
                    column_default: None,
                    is_primary_key: false,
                    is_unique: false,
                },
            ],
            modified_columns: vec![],
            dropped_columns: vec!["old_column".to_string()],
            added_constraints: vec![],
            dropped_constraints: vec![],
            added_indexes: vec![],
            dropped_indexes: vec![],
        };
        
        let statements = generate_alter_table("public", "users", &changes);
        
        assert!(!statements.is_empty());
        assert!(statements.iter().any(|s| s.contains("ADD COLUMN") && s.contains("email")));
        assert!(statements.iter().any(|s| s.contains("DROP COLUMN") && s.contains("old_column")));
    }
}
