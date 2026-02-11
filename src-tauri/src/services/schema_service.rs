/**
 * Schema Service
 * 
 * This module provides database schema management functionality including:
 * - Retrieving table schema (columns, constraints, indexes)
 * - Querying information_schema for column definitions
 * - Querying pg_constraint for constraint information
 * - Querying pg_indexes for index information
 * 
 * Validates: Requirements 8.1, 8.2, 8.3, 8.4
 */

use crate::models::schema::{TableSchema, ColumnDefinition, ConstraintDefinition, IndexDefinition};
use tokio_postgres::Client;

/// Get complete schema information for a table
/// 
/// # Arguments
/// * `client` - PostgreSQL client connection
/// * `schema` - Schema name (e.g., "public")
/// * `table` - Table name
/// 
/// # Returns
/// * `Result<TableSchema, String>` - Complete table schema or error message
pub async fn get_table_schema(
    client: &Client,
    schema: &str,
    table: &str,
) -> Result<TableSchema, String> {
    // Get column definitions
    let columns = get_columns(client, schema, table).await?;
    
    // Get constraints
    let constraints = get_constraints(client, schema, table).await?;
    
    // Get indexes
    let indexes = get_indexes(client, schema, table).await?;
    
    // Mark primary key columns
    let mut columns_with_pk = mark_primary_key_columns(columns, &constraints);
    
    // Mark unique columns
    columns_with_pk = mark_unique_columns(columns_with_pk, &constraints);
    
    Ok(TableSchema {
        table_name: table.to_string(),
        schema: schema.to_string(),
        columns: columns_with_pk,
        constraints,
        indexes,
    })
}

/// Get column definitions from information_schema
async fn get_columns(
    client: &Client,
    schema: &str,
    table: &str,
) -> Result<Vec<ColumnDefinition>, String> {
    let query = r#"
        SELECT 
            column_name,
            data_type,
            character_maximum_length,
            numeric_precision,
            numeric_scale,
            is_nullable,
            column_default
        FROM information_schema.columns
        WHERE table_schema = $1 AND table_name = $2
        ORDER BY ordinal_position
    "#;
    
    let rows = client
        .query(query, &[&schema, &table])
        .await
        .map_err(|e| format!("Failed to query columns: {}", e))?;
    
    let columns = rows
        .iter()
        .map(|row| {
            let column_name: String = row.get(0);
            let data_type: String = row.get(1);
            let char_max_length: Option<i32> = row.get(2);
            let numeric_precision: Option<i32> = row.get(3);
            let numeric_scale: Option<i32> = row.get(4);
            let is_nullable: String = row.get(5);
            let column_default: Option<String> = row.get(6);
            
            ColumnDefinition {
                name: column_name,
                data_type,
                character_maximum_length: char_max_length,
                numeric_precision,
                numeric_scale,
                is_nullable: is_nullable == "YES",
                column_default,
                is_primary_key: false, // Will be set later
                is_unique: false, // Will be set later
            }
        })
        .collect();
    
    Ok(columns)
}

/// Get constraint definitions from pg_constraint
async fn get_constraints(
    client: &Client,
    schema: &str,
    table: &str,
) -> Result<Vec<ConstraintDefinition>, String> {
    let query = r#"
        SELECT 
            con.conname AS constraint_name,
            con.contype AS constraint_type,
            ARRAY(
                SELECT att.attname
                FROM unnest(con.conkey) AS u(attnum)
                JOIN pg_attribute att ON att.attnum = u.attnum AND att.attrelid = con.conrelid
                ORDER BY u.attnum
            ) AS columns,
            ref_ns.nspname AS referenced_schema,
            ref_cl.relname AS referenced_table,
            ARRAY(
                SELECT att.attname
                FROM unnest(con.confkey) AS u(attnum)
                JOIN pg_attribute att ON att.attnum = u.attnum AND att.attrelid = con.confrelid
                ORDER BY u.attnum
            ) AS referenced_columns,
            con.confdeltype AS on_delete_code,
            con.confupdtype AS on_update_code,
            pg_get_constraintdef(con.oid) AS constraint_def
        FROM pg_constraint con
        JOIN pg_class cl ON cl.oid = con.conrelid
        JOIN pg_namespace ns ON ns.oid = cl.relnamespace
        LEFT JOIN pg_class ref_cl ON ref_cl.oid = con.confrelid
        LEFT JOIN pg_namespace ref_ns ON ref_ns.oid = ref_cl.relnamespace
        WHERE ns.nspname = $1 AND cl.relname = $2
        ORDER BY con.conname
    "#;
    
    let rows = client
        .query(query, &[&schema, &table])
        .await
        .map_err(|e| format!("Failed to query constraints: {}", e))?;
    
    let constraints = rows
        .iter()
        .map(|row| {
            let constraint_name: String = row.get(0);
            let constraint_type_code: String = row.get(1);
            let columns: Vec<String> = row.get(2);
            let referenced_schema: Option<String> = row.get(3);
            let referenced_table: Option<String> = row.get(4);
            let referenced_columns: Option<Vec<String>> = row.get(5);
            let on_delete_code: Option<String> = row.get(6);
            let on_update_code: Option<String> = row.get(7);
            let constraint_def: String = row.get(8);
            
            // Convert constraint type code to readable name
            let constraint_type = match constraint_type_code.as_str() {
                "p" => "PRIMARY KEY",
                "f" => "FOREIGN KEY",
                "u" => "UNIQUE",
                "c" => "CHECK",
                _ => "UNKNOWN",
            }.to_string();
            
            // Build full referenced table name with schema
            let full_referenced_table = if let (Some(ref_schema), Some(ref_table)) = 
                (referenced_schema, referenced_table) {
                Some(format!("{}.{}", ref_schema, ref_table))
            } else {
                None
            };
            
            // Convert referential action codes to readable names
            let on_delete = on_delete_code.map(|code| match code.as_str() {
                "a" => "NO ACTION",
                "r" => "RESTRICT",
                "c" => "CASCADE",
                "n" => "SET NULL",
                "d" => "SET DEFAULT",
                _ => "NO ACTION",
            }.to_string());
            
            let on_update = on_update_code.map(|code| match code.as_str() {
                "a" => "NO ACTION",
                "r" => "RESTRICT",
                "c" => "CASCADE",
                "n" => "SET NULL",
                "d" => "SET DEFAULT",
                _ => "NO ACTION",
            }.to_string());
            
            // Extract check clause from constraint definition
            let check_clause = if constraint_type == "CHECK" {
                // Parse "CHECK (expression)" from constraint_def
                extract_check_clause(&constraint_def)
            } else {
                None
            };
            
            ConstraintDefinition {
                constraint_type,
                constraint_name,
                columns,
                referenced_table: full_referenced_table,
                referenced_columns,
                on_delete,
                on_update,
                check_clause,
            }
        })
        .collect();
    
    Ok(constraints)
}

/// Get index definitions from pg_indexes
async fn get_indexes(
    client: &Client,
    schema: &str,
    table: &str,
) -> Result<Vec<IndexDefinition>, String> {
    let query = r#"
        SELECT 
            i.indexname AS index_name,
            ARRAY(
                SELECT att.attname
                FROM unnest(ix.indkey) AS u(attnum)
                JOIN pg_attribute att ON att.attnum = u.attnum AND att.attrelid = ix.indrelid
                WHERE att.attnum > 0
                ORDER BY u.attnum
            ) AS columns,
            am.amname AS index_type,
            ix.indisunique AS is_unique,
            ix.indisprimary AS is_primary
        FROM pg_indexes i
        JOIN pg_class c ON c.relname = i.indexname
        JOIN pg_index ix ON ix.indexrelid = c.oid
        JOIN pg_class t ON t.oid = ix.indrelid
        JOIN pg_am am ON am.oid = c.relam
        WHERE i.schemaname = $1 AND i.tablename = $2
        ORDER BY i.indexname
    "#;
    
    let rows = client
        .query(query, &[&schema, &table])
        .await
        .map_err(|e| format!("Failed to query indexes: {}", e))?;
    
    let indexes = rows
        .iter()
        .filter_map(|row| {
            let index_name: String = row.get(0);
            let columns: Vec<String> = row.get(1);
            let index_type: String = row.get(2);
            let is_unique: bool = row.get(3);
            let is_primary: bool = row.get(4);
            
            // Skip primary key indexes (they're represented as constraints)
            if is_primary {
                return None;
            }
            
            Some(IndexDefinition {
                index_name,
                columns,
                index_type,
                is_unique,
            })
        })
        .collect();
    
    Ok(indexes)
}

/// Mark columns that are part of the primary key
fn mark_primary_key_columns(
    mut columns: Vec<ColumnDefinition>,
    constraints: &[ConstraintDefinition],
) -> Vec<ColumnDefinition> {
    // Find primary key constraint
    let pk_columns: Vec<String> = constraints
        .iter()
        .filter(|c| c.constraint_type == "PRIMARY KEY")
        .flat_map(|c| c.columns.clone())
        .collect();
    
    // Mark columns
    for column in &mut columns {
        if pk_columns.contains(&column.name) {
            column.is_primary_key = true;
        }
    }
    
    columns
}

/// Mark columns that have unique constraints
fn mark_unique_columns(
    mut columns: Vec<ColumnDefinition>,
    constraints: &[ConstraintDefinition],
) -> Vec<ColumnDefinition> {
    // Find unique constraints with single column
    let unique_columns: Vec<String> = constraints
        .iter()
        .filter(|c| c.constraint_type == "UNIQUE" && c.columns.len() == 1)
        .flat_map(|c| c.columns.clone())
        .collect();
    
    // Mark columns
    for column in &mut columns {
        if unique_columns.contains(&column.name) {
            column.is_unique = true;
        }
    }
    
    columns
}

/// Extract check clause expression from constraint definition
fn extract_check_clause(constraint_def: &str) -> Option<String> {
    // PostgreSQL constraint definition format: "CHECK (expression)"
    if let Some(start) = constraint_def.find("CHECK (") {
        let expr_start = start + 7; // Length of "CHECK ("
        if let Some(end) = constraint_def[expr_start..].rfind(')') {
            let expression = &constraint_def[expr_start..expr_start + end];
            return Some(expression.to_string());
        }
    }
    None
}

/// Get list of database objects for auto-completion
/// 
/// # Arguments
/// * `client` - PostgreSQL client connection
/// * `database` - Database name
/// * `object_type` - Type of objects to retrieve ("tables", "columns", "functions")
/// 
/// # Returns
/// * `Result<Vec<String>, String>` - List of object names or error message
pub async fn get_database_objects(
    client: &Client,
    object_type: &str,
) -> Result<Vec<String>, String> {
    match object_type {
        "tables" => get_table_names(client).await,
        "columns" => get_all_column_names(client).await,
        "functions" => get_function_names(client).await,
        _ => Err(format!("Unknown object type: {}", object_type)),
    }
}

/// Get all table names in the database
async fn get_table_names(client: &Client) -> Result<Vec<String>, String> {
    let query = r#"
        SELECT table_schema || '.' || table_name AS full_name
        FROM information_schema.tables
        WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
        ORDER BY table_schema, table_name
    "#;
    
    let rows = client
        .query(query, &[])
        .await
        .map_err(|e| format!("Failed to query tables: {}", e))?;
    
    let tables = rows
        .iter()
        .map(|row| row.get::<_, String>(0))
        .collect();
    
    Ok(tables)
}

/// Get all column names across all tables
async fn get_all_column_names(client: &Client) -> Result<Vec<String>, String> {
    let query = r#"
        SELECT DISTINCT column_name
        FROM information_schema.columns
        WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
        ORDER BY column_name
    "#;
    
    let rows = client
        .query(query, &[])
        .await
        .map_err(|e| format!("Failed to query columns: {}", e))?;
    
    let columns = rows
        .iter()
        .map(|row| row.get::<_, String>(0))
        .collect();
    
    Ok(columns)
}

/// Get all function names in the database
async fn get_function_names(client: &Client) -> Result<Vec<String>, String> {
    let query = r#"
        SELECT routine_schema || '.' || routine_name AS full_name
        FROM information_schema.routines
        WHERE routine_schema NOT IN ('pg_catalog', 'information_schema')
        ORDER BY routine_schema, routine_name
    "#;
    
    let rows = client
        .query(query, &[])
        .await
        .map_err(|e| format!("Failed to query functions: {}", e))?;
    
    let functions = rows
        .iter()
        .map(|row| row.get::<_, String>(0))
        .collect();
    
    Ok(functions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_check_clause() {
        let def1 = "CHECK (age >= 18)";
        assert_eq!(extract_check_clause(def1), Some("age >= 18".to_string()));
        
        let def2 = "CHECK ((status)::text = ANY (ARRAY['active'::text, 'inactive'::text]))";
        assert_eq!(
            extract_check_clause(def2),
            Some("(status)::text = ANY (ARRAY['active'::text, 'inactive'::text])".to_string())
        );
        
        let def3 = "FOREIGN KEY (user_id) REFERENCES users(id)";
        assert_eq!(extract_check_clause(def3), None);
    }

    #[test]
    fn test_mark_primary_key_columns() {
        let columns = vec![
            ColumnDefinition::new("id".to_string(), "integer".to_string(), false),
            ColumnDefinition::new("name".to_string(), "varchar".to_string(), true),
        ];
        
        let constraints = vec![
            ConstraintDefinition::primary_key(
                "users_pkey".to_string(),
                vec!["id".to_string()],
            ),
        ];
        
        let marked = mark_primary_key_columns(columns, &constraints);
        assert!(marked[0].is_primary_key);
        assert!(!marked[1].is_primary_key);
    }

    #[test]
    fn test_mark_unique_columns() {
        let columns = vec![
            ColumnDefinition::new("id".to_string(), "integer".to_string(), false),
            ColumnDefinition::new("email".to_string(), "varchar".to_string(), true),
        ];
        
        let constraints = vec![
            ConstraintDefinition::unique(
                "users_email_key".to_string(),
                vec!["email".to_string()],
            ),
        ];
        
        let marked = mark_unique_columns(columns, &constraints);
        assert!(!marked[0].is_unique);
        assert!(marked[1].is_unique);
    }
}
