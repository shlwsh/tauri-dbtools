use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use tokio::process::Command;
use tokio_postgres::NoTls;
use tauri::Manager;

#[derive(Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    message: String,
    data: Option<T>,
}

#[derive(Serialize, Deserialize)]
struct DatabaseConfig {
    host: String,
    port: String,
    user: String,
    password: String,
}

fn get_export_dir() -> Result<PathBuf, String> {
    let mut export_dir = dirs::home_dir().ok_or("Failed to get home directory")?;
    export_dir.push("pg-db-tool-exports");
    std::fs::create_dir_all(&export_dir)
        .map_err(|e| format!("Failed to create export directory: {}", e))?;
    Ok(export_dir)
}

fn get_db_config() -> DatabaseConfig {
    DatabaseConfig {
        host: env::var("PG_HOST").unwrap_or_else(|_| "localhost".to_string()),
        port: env::var("PG_PORT").unwrap_or_else(|_| "5432".to_string()),
        user: env::var("PG_USER").unwrap_or_else(|_| "postgres".to_string()),
        password: env::var("PG_PASSWORD").unwrap_or_else(|_| "postgres".to_string()),
    }
}

#[tauri::command]
async fn export_database(database: String) -> Result<ApiResponse<String>, String> {
    let config = get_db_config();
    let export_dir = get_export_dir()?;
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}_{}.sql", database, timestamp);
    let file_path = export_dir.join(&filename);

    let output = Command::new("pg_dump")
        .arg("-h")
        .arg(&config.host)
        .arg("-p")
        .arg(&config.port)
        .arg("-U")
        .arg(&config.user)
        .arg("-d")
        .arg(&database)
        .arg("-f")
        .arg(&file_path)
        .env("PGPASSWORD", &config.password)
        .output()
        .await
        .map_err(|e| format!("Failed to execute pg_dump: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("pg_dump failed: {}", stderr));
    }

    Ok(ApiResponse {
        success: true,
        message: format!("Database exported successfully to {}", file_path.display()),
        data: Some(file_path.to_string_lossy().to_string()),
    })
}

#[tauri::command]
async fn import_database(file_path: String, database: String) -> Result<ApiResponse<()>, String> {
    let config = get_db_config();
    let path = PathBuf::from(&file_path);

    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    let conn_str = format!(
        "host={} port={} user={} password={}",
        config.host, config.port, config.user, config.password
    );

    let (client, connection) = tokio_postgres::connect(&conn_str, NoTls)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let check_db = client
        .query("SELECT 1 FROM pg_database WHERE datname = $1", &[&database])
        .await
        .map_err(|e| format!("Failed to check database: {}", e))?;

    if !check_db.is_empty() {
        client
            .execute(&format!("DROP DATABASE IF EXISTS \"{}\"", database), &[])
            .await
            .map_err(|e| format!("Failed to drop existing database: {}", e))?;
    }

    client
        .execute(&format!("CREATE DATABASE \"{}\"", database), &[])
        .await
        .map_err(|e| format!("Failed to create database: {}", e))?;

    let output = Command::new("psql")
        .arg("-h")
        .arg(&config.host)
        .arg("-p")
        .arg(&config.port)
        .arg("-U")
        .arg(&config.user)
        .arg("-d")
        .arg(&database)
        .arg("-f")
        .arg(&path)
        .env("PGPASSWORD", &config.password)
        .output()
        .await
        .map_err(|e| format!("Failed to execute psql: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("psql import failed: {}", stderr));
    }

    Ok(ApiResponse {
        success: true,
        message: format!("Database {} imported successfully", database),
        data: None,
    })
}

#[tauri::command]
async fn list_databases() -> Result<ApiResponse<Vec<String>>, String> {
    let config = get_db_config();

    let conn_str = format!(
        "host={} port={} user={} password={}",
        config.host, config.port, config.user, config.password
    );

    let (client, connection) = tokio_postgres::connect(&conn_str, NoTls)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let rows = client
        .query(
            "SELECT datname FROM pg_database WHERE datistemplate = false ORDER BY datname",
            &[],
        )
        .await
        .map_err(|e| format!("Failed to query databases: {}", e))?;

    let databases: Vec<String> = rows.iter().map(|row| row.get(0)).collect();

    Ok(ApiResponse {
        success: true,
        message: "Databases listed successfully".to_string(),
        data: Some(databases),
    })
}

#[tauri::command]
async fn check_health() -> Result<ApiResponse<()>, String> {
    Ok(ApiResponse {
        success: true,
        message: "Server is running".to_string(),
        data: None,
    })
}

#[tauri::command]
async fn get_export_dir_path() -> Result<String, String> {
    let export_dir = get_export_dir()?;
    Ok(export_dir.to_string_lossy().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            export_database,
            import_database,
            list_databases,
            check_health,
            get_export_dir_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
