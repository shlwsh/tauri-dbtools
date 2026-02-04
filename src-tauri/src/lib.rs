// 使用 PostgreSQL 官方工具 (pg_dump/pg_restore) 的实现
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    database: DatabaseConfig,
}

#[derive(Serialize, Deserialize, Clone)]
struct DatabaseConfig {
    host: String,
    port: String,
    user: String,
    password: String,
    #[serde(default)]
    default_database: String,
}

#[derive(Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    message: String,
    data: Option<T>,
}

// New types for database explorer
#[derive(Serialize, Deserialize, Clone)]
struct TableInfo {
    name: String,
    schema: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    row_count: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ColumnInfo {
    name: String,
    #[serde(rename = "type")]
    data_type: String,
    nullable: bool,
    #[serde(rename = "isPrimaryKey")]
    is_primary_key: bool,
}

#[derive(Serialize, Deserialize)]
struct TableData {
    columns: Vec<ColumnInfo>,
    rows: Vec<serde_json::Value>,
    #[serde(rename = "totalRows")]
    total_rows: i64,
    page: u32,
    #[serde(rename = "pageSize")]
    page_size: u32,
}

fn get_config_path() -> PathBuf {
    let project_config = PathBuf::from("config.json");
    if project_config.exists() {
        return project_config;
    }
    
    if let Some(mut home) = dirs::home_dir() {
        home.push(".pg-db-tool");
        std::fs::create_dir_all(&home).ok();
        home.push("config.json");
        return home;
    }
    
    PathBuf::from("config.json")
}

fn load_config() -> Config {
    let config_path = get_config_path();
    
    if config_path.exists() {
        if let Ok(mut file) = File::open(&config_path) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                if let Ok(config) = serde_json::from_str::<Config>(&contents) {
                    log::info!("已加载配置文件: {}", config_path.display());
                    return config;
                }
            }
        }
    }
    
    log::warn!("使用默认配置");
    Config {
        database: DatabaseConfig {
            host: "localhost".to_string(),
            port: "5432".to_string(),
            user: "postgres".to_string(),
            password: "postgres".to_string(),
            default_database: "personnel_db".to_string(),
        }
    }
}

fn get_db_config() -> DatabaseConfig {
    let config = load_config();
    
    DatabaseConfig {
        host: env::var("PG_HOST").unwrap_or(config.database.host),
        port: env::var("PG_PORT").unwrap_or(config.database.port),
        user: env::var("PG_USER").unwrap_or(config.database.user),
        password: env::var("PG_PASSWORD").unwrap_or(config.database.password),
        default_database: config.database.default_database,
    }
}

fn get_export_dir() -> Result<PathBuf, String> {
    let mut export_dir = dirs::home_dir().ok_or("无法获取用户目录")?;
    export_dir.push("pg-db-tool-exports");
    std::fs::create_dir_all(&export_dir)
        .map_err(|e| format!("无法创建导出目录: {}", e))?;
    Ok(export_dir)
}

fn get_log_dir() -> Result<PathBuf, String> {
    let mut log_dir = dirs::home_dir().ok_or("无法获取用户目录")?;
    log_dir.push("pg-db-tool-logs");
    std::fs::create_dir_all(&log_dir)
        .map_err(|e| format!("无法创建日志目录: {}", e))?;
    Ok(log_dir)
}

// 使用 pg_dump 导出数据库
#[tauri::command]
async fn export_database(database: String) -> Result<ApiResponse<String>, String> {
    log::info!("========== 开始导出数据库 (pg_dump) ==========");
    log::info!("数据库: {}", database);
    
    let config = get_db_config();
    let export_dir = get_export_dir()?;
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}_{}.backup", database, timestamp);
    let file_path = export_dir.join(&filename);
    
    log::info!("导出文件: {}", file_path.display());
    
    // 使用 pg_dump 导出（自定义格式，压缩）
    let output = std::process::Command::new("pg_dump")
        .arg("-h").arg(&config.host)
        .arg("-p").arg(&config.port)
        .arg("-U").arg(&config.user)
        .arg("-F").arg("c")  // 自定义格式（压缩）
        .arg("-b")  // 包含大对象
        .arg("-v")  // 详细模式
        .arg("-f").arg(&file_path)
        .arg(&database)
        .env("PGPASSWORD", &config.password)
        .output()
        .map_err(|e| format!("无法执行 pg_dump: {}. 请确保 PostgreSQL 已安装并且 pg_dump 在 PATH 中", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::error!("pg_dump 失败: {}", stderr);
        return Err(format!("导出失败: {}", stderr));
    }
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    log::info!("pg_dump 输出: {}", stderr);
    
    // 获取文件大小
    if let Ok(metadata) = std::fs::metadata(&file_path) {
        let size_kb = metadata.len() / 1024;
        log::info!("导出文件大小: {} KB", size_kb);
    }
    
    log::info!("========== 导出完成 ==========");

    Ok(ApiResponse {
        success: true,
        message: format!("数据库已导出到 {}", file_path.display()),
        data: Some(file_path.to_string_lossy().to_string()),
    })
}

// 使用 pg_restore 导入数据库
#[tauri::command]
async fn import_database(file_path: String, database: String) -> Result<ApiResponse<()>, String> {
    log::info!("========== 开始导入数据库 (pg_restore) ==========");
    log::info!("文件: {}", file_path);
    log::info!("目标数据库: {}", database);
    
    let config = get_db_config();
    let path = PathBuf::from(&file_path);

    if !path.exists() {
        return Err(format!("文件不存在: {}", file_path));
    }

    // 连接到 postgres 数据库来创建目标数据库
    let psql_check = std::process::Command::new("psql")
        .arg("-h").arg(&config.host)
        .arg("-p").arg(&config.port)
        .arg("-U").arg(&config.user)
        .arg("-d").arg("postgres")
        .arg("-t")
        .arg("-c").arg(format!("SELECT 1 FROM pg_database WHERE datname='{}'", database))
        .env("PGPASSWORD", &config.password)
        .output()
        .map_err(|e| format!("无法执行 psql: {}. 请确保 PostgreSQL 已安装并且 psql 在 PATH 中", e))?;

    let db_exists = String::from_utf8_lossy(&psql_check.stdout).trim().contains("1");

    if db_exists {
        log::info!("数据库 {} 已存在，正在删除...", database);
        
        // 终止所有连接
        let _ = std::process::Command::new("psql")
            .arg("-h").arg(&config.host)
            .arg("-p").arg(&config.port)
            .arg("-U").arg(&config.user)
            .arg("-d").arg("postgres")
            .arg("-c").arg(format!(
                "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}' AND pid <> pg_backend_pid()",
                database
            ))
            .env("PGPASSWORD", &config.password)
            .output();

        // 删除数据库
        let drop_output = std::process::Command::new("psql")
            .arg("-h").arg(&config.host)
            .arg("-p").arg(&config.port)
            .arg("-U").arg(&config.user)
            .arg("-d").arg("postgres")
            .arg("-c").arg(format!("DROP DATABASE IF EXISTS \"{}\"", database))
            .env("PGPASSWORD", &config.password)
            .output()
            .map_err(|e| format!("无法删除数据库: {}", e))?;

        if !drop_output.status.success() {
            let stderr = String::from_utf8_lossy(&drop_output.stderr);
            log::warn!("删除数据库警告: {}", stderr);
        }
    }

    // 创建新数据库
    log::info!("创建数据库 {}...", database);
    let create_output = std::process::Command::new("psql")
        .arg("-h").arg(&config.host)
        .arg("-p").arg(&config.port)
        .arg("-U").arg(&config.user)
        .arg("-d").arg("postgres")
        .arg("-c").arg(format!("CREATE DATABASE \"{}\"", database))
        .env("PGPASSWORD", &config.password)
        .output()
        .map_err(|e| format!("无法创建数据库: {}", e))?;

    if !create_output.status.success() {
        let stderr = String::from_utf8_lossy(&create_output.stderr);
        log::error!("创建数据库失败: {}", stderr);
        return Err(format!("创建数据库失败: {}", stderr));
    }

    // 使用 pg_restore 导入
    log::info!("正在导入数据...");
    let restore_output = std::process::Command::new("pg_restore")
        .arg("-h").arg(&config.host)
        .arg("-p").arg(&config.port)
        .arg("-U").arg(&config.user)
        .arg("-d").arg(&database)
        .arg("-v")  // 详细模式
        .arg("--no-owner")  // 不恢复所有权
        .arg("--no-acl")  // 不恢复访问权限
        .arg(&file_path)
        .env("PGPASSWORD", &config.password)
        .output()
        .map_err(|e| format!("无法执行 pg_restore: {}", e))?;

    let stderr = String::from_utf8_lossy(&restore_output.stderr);
    log::info!("pg_restore 输出: {}", stderr);

    if !restore_output.status.success() {
        log::warn!("pg_restore 返回非零状态码，但这可能是正常的（某些警告）");
    }

    log::info!("========== 导入完成 ==========");

    Ok(ApiResponse {
        success: true,
        message: format!("数据库 {} 导入成功", database),
        data: None,
    })
}

#[tauri::command]
async fn list_databases() -> Result<ApiResponse<Vec<String>>, String> {
    let config = get_db_config();
    
    let output = std::process::Command::new("psql")
        .arg("-h").arg(&config.host)
        .arg("-p").arg(&config.port)
        .arg("-U").arg(&config.user)
        .arg("-d").arg("postgres")
        .arg("-t")
        .arg("-c").arg("SELECT datname FROM pg_database WHERE datistemplate = false ORDER BY datname")
        .env("PGPASSWORD", &config.password)
        .output()
        .map_err(|e| format!("无法执行 psql: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("查询数据库列表失败: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let databases: Vec<String> = stdout
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    Ok(ApiResponse {
        success: true,
        message: "数据库列表获取成功".to_string(),
        data: Some(databases),
    })
}

#[tauri::command]
async fn check_health() -> Result<ApiResponse<()>, String> {
    Ok(ApiResponse {
        success: true,
        message: "服务运行正常".to_string(),
        data: None,
    })
}

#[tauri::command]
async fn get_export_dir_path() -> Result<String, String> {
    let export_dir = get_export_dir()?;
    Ok(export_dir.to_string_lossy().to_string())
}

#[tauri::command]
async fn get_log_dir_path() -> Result<String, String> {
    let log_dir = get_log_dir()?;
    Ok(log_dir.to_string_lossy().to_string())
}

// Database Explorer APIs
#[tauri::command]
async fn list_tables(database: String) -> Result<ApiResponse<Vec<TableInfo>>, String> {
    log::info!("========== 列出表 ==========");
    log::info!("数据库: {}", database);
    
    let config = get_db_config();
    
    let query = "SELECT 
        schemaname as schema, 
        tablename as name,
        n_live_tup as row_count
    FROM pg_stat_user_tables 
    ORDER BY schemaname, tablename";
    
    let output = std::process::Command::new("psql")
        .arg("-h").arg(&config.host)
        .arg("-p").arg(&config.port)
        .arg("-U").arg(&config.user)
        .arg("-d").arg(&database)
        .arg("-t")
        .arg("-A")
        .arg("-F").arg("|")
        .arg("-c").arg(query)
        .env("PGPASSWORD", &config.password)
        .output()
        .map_err(|e| format!("无法执行 psql: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("查询表列表失败: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let tables: Vec<TableInfo> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 2 {
                Some(TableInfo {
                    schema: parts[0].trim().to_string(),
                    name: parts[1].trim().to_string(),
                    row_count: parts.get(2).and_then(|s| s.trim().parse().ok()),
                })
            } else {
                None
            }
        })
        .collect();

    log::info!("找到 {} 个表", tables.len());

    Ok(ApiResponse {
        success: true,
        message: format!("找到 {} 个表", tables.len()),
        data: Some(tables),
    })
}

#[tauri::command]
async fn get_table_data(
    database: String,
    table: String,
    page: u32,
    page_size: u32,
) -> Result<ApiResponse<TableData>, String> {
    log::info!("========== 查询表数据 ==========");
    log::info!("数据库: {}, 表: {}, 页: {}, 每页: {}", database, table, page, page_size);
    
    let config = get_db_config();
    
    // Get column information
    let column_query = format!(
        "SELECT 
            a.attname as name,
            pg_catalog.format_type(a.atttypid, a.atttypmod) as type,
            NOT a.attnotnull as nullable,
            COALESCE((SELECT true FROM pg_index i WHERE i.indrelid = a.attrelid AND a.attnum = ANY(i.indkey) AND i.indisprimary), false) as is_primary_key
        FROM pg_catalog.pg_attribute a
        WHERE a.attrelid = '{}'::regclass
        AND a.attnum > 0
        AND NOT a.attisdropped
        ORDER BY a.attnum",
        table
    );
    
    let column_output = std::process::Command::new("psql")
        .arg("-h").arg(&config.host)
        .arg("-p").arg(&config.port)
        .arg("-U").arg(&config.user)
        .arg("-d").arg(&database)
        .arg("-t")
        .arg("-A")
        .arg("-F").arg("|")
        .arg("-c").arg(&column_query)
        .env("PGPASSWORD", &config.password)
        .output()
        .map_err(|e| format!("无法查询列信息: {}", e))?;

    if !column_output.status.success() {
        let stderr = String::from_utf8_lossy(&column_output.stderr);
        return Err(format!("查询列信息失败: {}", stderr));
    }

    let column_stdout = String::from_utf8_lossy(&column_output.stdout);
    let columns: Vec<ColumnInfo> = column_stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                Some(ColumnInfo {
                    name: parts[0].trim().to_string(),
                    data_type: parts[1].trim().to_string(),
                    nullable: parts[2].trim() == "t",
                    is_primary_key: parts[3].trim() == "t",
                })
            } else {
                None
            }
        })
        .collect();

    // Get total row count
    let count_query = format!("SELECT COUNT(*) FROM {}", table);
    let count_output = std::process::Command::new("psql")
        .arg("-h").arg(&config.host)
        .arg("-p").arg(&config.port)
        .arg("-U").arg(&config.user)
        .arg("-d").arg(&database)
        .arg("-t")
        .arg("-c").arg(&count_query)
        .env("PGPASSWORD", &config.password)
        .output()
        .map_err(|e| format!("无法查询行数: {}", e))?;

    let total_rows: i64 = String::from_utf8_lossy(&count_output.stdout)
        .trim()
        .parse()
        .unwrap_or(0);

    // Get paginated data
    let offset = (page - 1) * page_size;
    let data_query = format!(
        "SELECT * FROM {} LIMIT {} OFFSET {}",
        table, page_size, offset
    );
    
    let data_output = std::process::Command::new("psql")
        .arg("-h").arg(&config.host)
        .arg("-p").arg(&config.port)
        .arg("-U").arg(&config.user)
        .arg("-d").arg(&database)
        .arg("-t")
        .arg("-A")
        .arg("-F").arg("|")
        .arg("-c").arg(&data_query)
        .env("PGPASSWORD", &config.password)
        .output()
        .map_err(|e| format!("无法查询数据: {}", e))?;

    if !data_output.status.success() {
        let stderr = String::from_utf8_lossy(&data_output.stderr);
        return Err(format!("查询数据失败: {}", stderr));
    }

    let data_stdout = String::from_utf8_lossy(&data_output.stdout);
    let rows: Vec<serde_json::Value> = data_stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let values: Vec<&str> = line.split('|').collect();
            let mut row = serde_json::Map::new();
            for (i, col) in columns.iter().enumerate() {
                if let Some(value) = values.get(i) {
                    row.insert(col.name.clone(), serde_json::Value::String(value.to_string()));
                }
            }
            serde_json::Value::Object(row)
        })
        .collect();

    log::info!("返回 {} 行数据，总共 {} 行", rows.len(), total_rows);

    Ok(ApiResponse {
        success: true,
        message: format!("查询成功，返回 {} 行", rows.len()),
        data: Some(TableData {
            columns,
            rows,
            total_rows,
            page,
            page_size,
        }),
    })
}

#[tauri::command]
async fn create_record(
    database: String,
    table: String,
    data: serde_json::Value,
) -> Result<ApiResponse<()>, String> {
    log::info!("========== 创建记录 ==========");
    log::info!("数据库: {}, 表: {}", database, table);
    
    let config = get_db_config();
    
    let obj = data.as_object().ok_or("数据必须是对象")?;
    
    let columns: Vec<String> = obj.keys().cloned().collect();
    let values: Vec<String> = obj.values()
        .map(|v| match v {
            serde_json::Value::String(s) => format!("'{}'", s.replace("'", "''")),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => "NULL".to_string(),
            _ => format!("'{}'", v.to_string().replace("'", "''")),
        })
        .collect();
    
    let insert_query = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table,
        columns.join(", "),
        values.join(", ")
    );
    
    log::info!("执行 SQL: {}", insert_query);
    
    let output = std::process::Command::new("psql")
        .arg("-h").arg(&config.host)
        .arg("-p").arg(&config.port)
        .arg("-U").arg(&config.user)
        .arg("-d").arg(&database)
        .arg("-c").arg(&insert_query)
        .env("PGPASSWORD", &config.password)
        .output()
        .map_err(|e| format!("无法执行插入: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("插入失败: {}", stderr));
    }

    log::info!("记录创建成功");

    Ok(ApiResponse {
        success: true,
        message: "记录创建成功".to_string(),
        data: None,
    })
}

#[tauri::command]
async fn update_record(
    database: String,
    table: String,
    primary_key: serde_json::Value,
    data: serde_json::Value,
) -> Result<ApiResponse<()>, String> {
    log::info!("========== 更新记录 ==========");
    log::info!("数据库: {}, 表: {}", database, table);
    
    let config = get_db_config();
    
    let pk_obj = primary_key.as_object().ok_or("主键必须是对象")?;
    let data_obj = data.as_object().ok_or("数据必须是对象")?;
    
    let set_clauses: Vec<String> = data_obj.iter()
        .map(|(k, v)| {
            let value_str = match v {
                serde_json::Value::String(s) => format!("'{}'", s.replace("'", "''")),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Null => "NULL".to_string(),
                _ => format!("'{}'", v.to_string().replace("'", "''")),
            };
            format!("{} = {}", k, value_str)
        })
        .collect();
    
    let where_clauses: Vec<String> = pk_obj.iter()
        .map(|(k, v)| {
            let value_str = match v {
                serde_json::Value::String(s) => format!("'{}'", s.replace("'", "''")),
                serde_json::Value::Number(n) => n.to_string(),
                _ => format!("'{}'", v.to_string().replace("'", "''")),
            };
            format!("{} = {}", k, value_str)
        })
        .collect();
    
    let update_query = format!(
        "UPDATE {} SET {} WHERE {}",
        table,
        set_clauses.join(", "),
        where_clauses.join(" AND ")
    );
    
    log::info!("执行 SQL: {}", update_query);
    
    let output = std::process::Command::new("psql")
        .arg("-h").arg(&config.host)
        .arg("-p").arg(&config.port)
        .arg("-U").arg(&config.user)
        .arg("-d").arg(&database)
        .arg("-c").arg(&update_query)
        .env("PGPASSWORD", &config.password)
        .output()
        .map_err(|e| format!("无法执行更新: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("更新失败: {}", stderr));
    }

    log::info!("记录更新成功");

    Ok(ApiResponse {
        success: true,
        message: "记录更新成功".to_string(),
        data: None,
    })
}

#[tauri::command]
async fn delete_record(
    database: String,
    table: String,
    primary_key: serde_json::Value,
) -> Result<ApiResponse<()>, String> {
    log::info!("========== 删除记录 ==========");
    log::info!("数据库: {}, 表: {}", database, table);
    
    let config = get_db_config();
    
    let pk_obj = primary_key.as_object().ok_or("主键必须是对象")?;
    
    let where_clauses: Vec<String> = pk_obj.iter()
        .map(|(k, v)| {
            let value_str = match v {
                serde_json::Value::String(s) => format!("'{}'", s.replace("'", "''")),
                serde_json::Value::Number(n) => n.to_string(),
                _ => format!("'{}'", v.to_string().replace("'", "''")),
            };
            format!("{} = {}", k, value_str)
        })
        .collect();
    
    let delete_query = format!(
        "DELETE FROM {} WHERE {}",
        table,
        where_clauses.join(" AND ")
    );
    
    log::info!("执行 SQL: {}", delete_query);
    
    let output = std::process::Command::new("psql")
        .arg("-h").arg(&config.host)
        .arg("-p").arg(&config.port)
        .arg("-U").arg(&config.user)
        .arg("-d").arg(&database)
        .arg("-c").arg(&delete_query)
        .env("PGPASSWORD", &config.password)
        .output()
        .map_err(|e| format!("无法执行删除: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("删除失败: {}", stderr));
    }

    log::info!("记录删除成功");

    Ok(ApiResponse {
        success: true,
        message: "记录删除成功".to_string(),
        data: None,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    setup_logger().expect("无法设置日志");
    
    log::info!("========================================");
    log::info!("PostgreSQL 数据库工具启动中 (pg_dump/pg_restore)...");
    log::info!("========================================");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            export_database,
            import_database,
            list_databases,
            check_health,
            get_export_dir_path,
            get_log_dir_path,
            list_tables,
            get_table_data,
            create_record,
            update_record,
            delete_record
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}

fn setup_logger() -> Result<(), fern::InitError> {
    let log_dir = get_log_dir().expect("无法获取日志目录");
    let log_file = log_dir.join(format!(
        "pg-db-tool_{}.log",
        chrono::Local::now().format("%Y%m%d")
    ));

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file(log_file)?)
        .apply()?;

    Ok(())
}
