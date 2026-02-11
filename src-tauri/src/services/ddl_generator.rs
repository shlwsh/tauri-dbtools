/**
 * DDL Generator Service
 * 
 * This module provides DDL (Data Definition Language) generation functionality including:
 * - Generating CREATE TABLE statements from table designs
 * - Generating ALTER TABLE statements for table modifications
 * - Generating CREATE INDEX statements
 * - Generating constraint definitions
 * 
 * Validates: Requirements 7.1, 7.2, 7.3, 7.4, 7.5
 */

use crate::models::schema::{
    TableDesign, TableChanges, ColumnDefinition, ConstraintDefinition, 
    IndexDefinition, ColumnModification,
};

/// Generate CREATE TABLE DDL statement from table design
/// 
/// # Arguments
/// * `design` - Table design specification
/// 
/// # Returns
/// * `String` - Complete CREATE TABLE statement with constraints and indexes
pub fn generate_create_table(design: &TableDesign) -> String {
    let mut ddl = Vec::new();
    
    // CREATE TABLE header
    ddl.push(format!(
        "CREATE TABLE {}.{} (",
        escape_identifier(&design.schema),
        escape_identifier(&design.table_name)
    ));
    
    // Column definitions
    let column_defs: Vec<String> = design
        .columns
        .iter()
        .map(|col| format!("  {}", generate_column_definition(col)))
        .collect();
    
    ddl.push(column_defs.join(",\n"));
    
    // Table-level constraints
    let table_constraints: Vec<String> = design
        .constraints
        .iter()
        .filter(|c| should_include_in_create_table(c))
        .map(|c| format!("  {}", generate_constraint_definition(c)))
        .collect();
    
    if !table_constraints.is_empty() {
        ddl.push(",\n".to_string());
        ddl.push(table_constraints.join(",\n"));
    }
    
    ddl.push("\n);".to_string());
    
    // Index definitions (separate statements)
    let index_statements: Vec<String> = design
        .indexes
        .iter()
        .map(|idx| generate_create_index(&design.schema, &design.table_name, idx))
        .collect();
    
    if !index_statements.is_empty() {
        ddl.push("\n\n".to_string());
        ddl.push(index_statements.join("\n\n"));
    }
    
    ddl.concat()
}

/// Generate ALTER TABLE DDL statements from table changes
/// 
/// # Arguments
/// * `schema` - Schema name
/// * `table` - Table name
/// * `changes` - Table modification specification
/// 
/// # Returns
/// * `Vec<String>` - List of ALTER TABLE statements
pub fn generate_alter_table(
    schema: &str,
    table: &str,
    changes: &TableChanges,
) -> Vec<String> {
    let mut statements = Vec::new();
    let table_name = format!("{}.{}", escape_identifier(schema), escape_identifier(table));
    
    // Drop constraints first (they may depend on columns)
    for constraint_name in &changes.dropped_constraints {
        statements.push(format!(
            "ALTER TABLE {} DROP CONSTRAINT {};",
            table_name,
            escape_identifier(constraint_name)
        ));
    }
    
    // Drop indexes
    for index_name in &changes.dropped_indexes {
        statements.push(format!(
            "DROP INDEX {}.{};",
            escape_identifier(schema),
            escape_identifier(index_name)
        ));
    }
    
    // Drop columns
    for column_name in &changes.dropped_columns {
        statements.push(format!(
            "ALTER TABLE {} DROP COLUMN {};",
            table_name,
            escape_identifier(column_name)
        ));
    }
    
    // Add columns
    for column in &changes.added_columns {
        statements.push(format!(
            "ALTER TABLE {} ADD COLUMN {};",
            table_name,
            generate_column_definition(column)
        ));
    }
    
    // Modify columns
    for modification in &changes.modified_columns {
        statements.extend(generate_column_modifications(
            &table_name,
            modification,
        ));
    }
    
    // Add constraints
    for constraint in &changes.added_constraints {
        statements.push(format!(
            "ALTER TABLE {} ADD {};",
            table_name,
            generate_constraint_definition(constraint)
        ));
    }
    
    // Add indexes
    for index in &changes.added_indexes {
        statements.push(generate_create_index(schema, table, index));
    }
    
    statements
}

/// Generate column definition for CREATE TABLE or ALTER TABLE ADD COLUMN
fn generate_column_definition(column: &ColumnDefinition) -> String {
    let mut parts = vec![escape_identifier(&column.name)];
    
    // Data type with length/precision
    let data_type = format_data_type(column);
    parts.push(data_type);
    
    // NULL constraint
    if !column.is_nullable {
        parts.push("NOT NULL".to_string());
    }
    
    // Default value
    if let Some(ref default) = column.column_default {
        parts.push(format!("DEFAULT {}", default));
    }
    
    // UNIQUE constraint (column-level)
    if column.is_unique {
        parts.push("UNIQUE".to_string());
    }
    
    parts.join(" ")
}

/// Format data type with length or precision
fn format_data_type(column: &ColumnDefinition) -> String {
    let base_type = column.data_type.to_uppercase();
    
    // Handle character types with length
    if let Some(length) = column.character_maximum_length {
        if base_type.contains("CHAR") || base_type.contains("VARCHAR") {
            return format!("{}({})", base_type, length);
        }
    }
    
    // Handle numeric types with precision and scale
    if let Some(precision) = column.numeric_precision {
        if base_type == "NUMERIC" || base_type == "DECIMAL" {
            if let Some(scale) = column.numeric_scale {
                return format!("{}({}, {})", base_type, precision, scale);
            } else {
                return format!("{}({})", base_type, precision);
            }
        }
    }
    
    base_type
}

/// Generate constraint definition
fn generate_constraint_definition(constraint: &ConstraintDefinition) -> String {
    let constraint_name = escape_identifier(&constraint.constraint_name);
    
    match constraint.constraint_type.as_str() {
        "PRIMARY KEY" => {
            let columns = constraint
                .columns
                .iter()
                .map(|c| escape_identifier(c))
                .collect::<Vec<_>>()
                .join(", ");
            format!("CONSTRAINT {} PRIMARY KEY ({})", constraint_name, columns)
        }
        "FOREIGN KEY" => {
            let columns = constraint
                .columns
                .iter()
                .map(|c| escape_identifier(c))
                .collect::<Vec<_>>()
                .join(", ");
            
            let mut fk_def = format!(
                "CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {}",
                constraint_name,
                columns,
                constraint.referenced_table.as_ref().unwrap_or(&"".to_string())
            );
            
            if let Some(ref ref_cols) = constraint.referenced_columns {
                let ref_columns = ref_cols
                    .iter()
                    .map(|c| escape_identifier(c))
                    .collect::<Vec<_>>()
                    .join(", ");
                fk_def.push_str(&format!(" ({})", ref_columns));
            }
            
            if let Some(ref on_delete) = constraint.on_delete {
                fk_def.push_str(&format!(" ON DELETE {}", on_delete));
            }
            
            if let Some(ref on_update) = constraint.on_update {
                fk_def.push_str(&format!(" ON UPDATE {}", on_update));
            }
            
            fk_def
        }
        "UNIQUE" => {
            let columns = constraint
                .columns
                .iter()
                .map(|c| escape_identifier(c))
                .collect::<Vec<_>>()
                .join(", ");
            format!("CONSTRAINT {} UNIQUE ({})", constraint_name, columns)
        }
        "CHECK" => {
            let default_expr = String::new();
            let check_expr = constraint.check_clause.as_ref().unwrap_or(&default_expr);
            format!("CONSTRAINT {} CHECK ({})", constraint_name, check_expr)
        }
        _ => format!("-- Unknown constraint type: {}", constraint.constraint_type),
    }
}

/// Generate CREATE INDEX statement
fn generate_create_index(schema: &str, table: &str, index: &IndexDefinition) -> String {
    let unique = if index.is_unique { "UNIQUE " } else { "" };
    
    let columns = index
        .columns
        .iter()
        .map(|c| escape_identifier(c))
        .collect::<Vec<_>>()
        .join(", ");
    
    let index_type = if index.index_type.to_uppercase() != "BTREE" {
        format!(" USING {}", index.index_type.to_uppercase())
    } else {
        "".to_string()
    };
    
    format!(
        "CREATE {}INDEX {}.{}{} ON {}.{} ({});",
        unique,
        escape_identifier(schema),
        escape_identifier(&index.index_name),
        index_type,
        escape_identifier(schema),
        escape_identifier(table),
        columns
    )
}

/// Generate ALTER TABLE statements for column modifications
fn generate_column_modifications(
    table_name: &str,
    modification: &ColumnModification,
) -> Vec<String> {
    let mut statements = Vec::new();
    let old_name = escape_identifier(&modification.old_name);
    let new_col = &modification.new_definition;
    let new_name = escape_identifier(&new_col.name);
    
    // Rename column if name changed
    if modification.old_name != new_col.name {
        statements.push(format!(
            "ALTER TABLE {} RENAME COLUMN {} TO {};",
            table_name, old_name, new_name
        ));
    }
    
    // Change data type
    let data_type = format_data_type(new_col);
    statements.push(format!(
        "ALTER TABLE {} ALTER COLUMN {} TYPE {};",
        table_name, new_name, data_type
    ));
    
    // Change nullable
    if new_col.is_nullable {
        statements.push(format!(
            "ALTER TABLE {} ALTER COLUMN {} DROP NOT NULL;",
            table_name, new_name
        ));
    } else {
        statements.push(format!(
            "ALTER TABLE {} ALTER COLUMN {} SET NOT NULL;",
            table_name, new_name
        ));
    }
    
    // Change default value
    if let Some(ref default) = new_col.column_default {
        statements.push(format!(
            "ALTER TABLE {} ALTER COLUMN {} SET DEFAULT {};",
            table_name, new_name, default
        ));
    } else {
        statements.push(format!(
            "ALTER TABLE {} ALTER COLUMN {} DROP DEFAULT;",
            table_name, new_name
        ));
    }
    
    statements
}

/// Check if constraint should be included in CREATE TABLE statement
/// (vs. added separately with ALTER TABLE)
fn should_include_in_create_table(_constraint: &ConstraintDefinition) -> bool {
    // All constraints can be included in CREATE TABLE
    true
}

/// Escape SQL identifier (table name, column name, etc.)
/// 
/// Wraps identifier in double quotes if it contains special characters
/// or is a reserved keyword.
fn escape_identifier(identifier: &str) -> String {
    // Check if identifier needs quoting
    let needs_quoting = identifier.chars().any(|c| !c.is_alphanumeric() && c != '_')
        || identifier.chars().next().map_or(false, |c| c.is_numeric())
        || is_reserved_keyword(identifier);
    
    if needs_quoting {
        format!("\"{}\"", identifier.replace('"', "\"\""))
    } else {
        identifier.to_string()
    }
}

/// Check if identifier is a PostgreSQL reserved keyword
fn is_reserved_keyword(identifier: &str) -> bool {
    let keywords = [
        "SELECT", "INSERT", "UPDATE", "DELETE", "FROM", "WHERE", "JOIN",
        "TABLE", "CREATE", "ALTER", "DROP", "INDEX", "PRIMARY", "FOREIGN",
        "KEY", "CONSTRAINT", "UNIQUE", "CHECK", "DEFAULT", "NULL", "NOT",
        "AND", "OR", "IN", "EXISTS", "BETWEEN", "LIKE", "IS", "AS",
        "ORDER", "BY", "GROUP", "HAVING", "LIMIT", "OFFSET", "UNION",
        "INTERSECT", "EXCEPT", "CASE", "WHEN", "THEN", "ELSE", "END",
        "USER", "CURRENT", "SESSION", "SYSTEM", "CATALOG", "SCHEMA",
    ];
    
    keywords.contains(&identifier.to_uppercase().as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_column_definition() {
        let col = ColumnDefinition {
            name: "id".to_string(),
            data_type: "integer".to_string(),
            character_maximum_length: None,
            numeric_precision: None,
            numeric_scale: None,
            is_nullable: false,
            column_default: None,
            is_primary_key: false,
            is_unique: false,
        };
        
        let def = generate_column_definition(&col);
        assert_eq!(def, "id INTEGER NOT NULL");
    }

    #[test]
    fn test_generate_column_definition_with_length() {
        let col = ColumnDefinition {
            name: "email".to_string(),
            data_type: "varchar".to_string(),
            character_maximum_length: Some(255),
            numeric_precision: None,
            numeric_scale: None,
            is_nullable: true,
            column_default: None,
            is_primary_key: false,
            is_unique: true,
        };
        
        let def = generate_column_definition(&col);
        assert_eq!(def, "email VARCHAR(255) UNIQUE");
    }

    #[test]
    fn test_generate_column_definition_with_default() {
        let col = ColumnDefinition {
            name: "created_at".to_string(),
            data_type: "timestamp".to_string(),
            character_maximum_length: None,
            numeric_precision: None,
            numeric_scale: None,
            is_nullable: false,
            column_default: Some("CURRENT_TIMESTAMP".to_string()),
            is_primary_key: false,
            is_unique: false,
        };
        
        let def = generate_column_definition(&col);
        assert_eq!(def, "created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP");
    }

    #[test]
    fn test_generate_primary_key_constraint() {
        let constraint = ConstraintDefinition::primary_key(
            "users_pkey".to_string(),
            vec!["id".to_string()],
        );
        
        let def = generate_constraint_definition(&constraint);
        assert_eq!(def, "CONSTRAINT users_pkey PRIMARY KEY (id)");
    }

    #[test]
    fn test_generate_foreign_key_constraint() {
        let constraint = ConstraintDefinition::foreign_key(
            "orders_user_id_fkey".to_string(),
            vec!["user_id".to_string()],
            "public.users".to_string(),
            vec!["id".to_string()],
        )
        .with_on_delete("CASCADE".to_string())
        .with_on_update("NO ACTION".to_string());
        
        let def = generate_constraint_definition(&constraint);
        assert!(def.contains("FOREIGN KEY (user_id)"));
        assert!(def.contains("REFERENCES public.users (id)"));
        assert!(def.contains("ON DELETE CASCADE"));
        assert!(def.contains("ON UPDATE NO ACTION"));
    }

    #[test]
    fn test_generate_unique_constraint() {
        let constraint = ConstraintDefinition::unique(
            "users_email_key".to_string(),
            vec!["email".to_string()],
        );
        
        let def = generate_constraint_definition(&constraint);
        assert_eq!(def, "CONSTRAINT users_email_key UNIQUE (email)");
    }

    #[test]
    fn test_generate_check_constraint() {
        let constraint = ConstraintDefinition::check(
            "users_age_check".to_string(),
            "age >= 18".to_string(),
        );
        
        let def = generate_constraint_definition(&constraint);
        assert_eq!(def, "CONSTRAINT users_age_check CHECK (age >= 18)");
    }

    #[test]
    fn test_generate_create_index() {
        let index = IndexDefinition::btree(
            "users_email_idx".to_string(),
            vec!["email".to_string()],
            true,
        );
        
        let stmt = generate_create_index("public", "users", &index);
        assert!(stmt.contains("CREATE UNIQUE INDEX"));
        assert!(stmt.contains("public.users_email_idx"));
        assert!(stmt.contains("ON public.users (email)"));
    }

    #[test]
    fn test_escape_identifier() {
        assert_eq!(escape_identifier("simple"), "simple");
        assert_eq!(escape_identifier("with-dash"), "\"with-dash\"");
        assert_eq!(escape_identifier("with space"), "\"with space\"");
        assert_eq!(escape_identifier("123numeric"), "\"123numeric\"");
        assert_eq!(escape_identifier("SELECT"), "\"SELECT\"");
        assert_eq!(escape_identifier("user"), "\"user\"");
    }

    #[test]
    fn test_format_data_type() {
        let col1 = ColumnDefinition {
            name: "test".to_string(),
            data_type: "varchar".to_string(),
            character_maximum_length: Some(100),
            numeric_precision: None,
            numeric_scale: None,
            is_nullable: true,
            column_default: None,
            is_primary_key: false,
            is_unique: false,
        };
        assert_eq!(format_data_type(&col1), "VARCHAR(100)");
        
        let col2 = ColumnDefinition {
            name: "test".to_string(),
            data_type: "numeric".to_string(),
            character_maximum_length: None,
            numeric_precision: Some(10),
            numeric_scale: Some(2),
            is_nullable: true,
            column_default: None,
            is_primary_key: false,
            is_unique: false,
        };
        assert_eq!(format_data_type(&col2), "NUMERIC(10, 2)");
    }
}
