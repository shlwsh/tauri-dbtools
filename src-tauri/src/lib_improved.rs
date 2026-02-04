// 改进的导出函数 - 使用 COPY 命令
async fn export_database_improved(database: String) -> Result<ApiResponse<String>, String> {
    log::info!("========== Starting database export (Improved) ==========");
    
    let config = get_db_config();
    let export_dir = get_export_dir()?;
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}_{}.sql", database, timestamp);
    let file_path = export_dir.join(&filename);
    
    // 连接到数据库
    let conn_str = format!(
        "host={} port={} user={} password={} dbname={}",
        config.host, config.port, config.user, config.password, database
    );
    
    let (mut client, connection) = tokio_postgres::connect(&conn_str, NoTls).await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            log::error!("Connection error: {}", e);
        }
    });

    let file = File::create(&file_path).map_err(|e| format!("Failed to create file: {}", e))?;
    let mut writer = BufWriter::new(file);

    // 写入头部
    writeln!(writer, "-- PostgreSQL database dump").unwrap();
    writeln!(writer, "SET client_encoding = 'UTF8';").unwrap();
    writeln!(writer, "SET standard_conforming_strings = on;").unwrap();
    writeln!(writer, "").unwrap();

    // 获取所有表
    let tables = client.query(
        "SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename",
        &[]
    ).await.map_err(|e| format!("Failed to fetch tables: {}", e))?;

    for table_row in &tables {
        let table_name: String = table_row.get(0);
        log::info!("Exporting table: {}", table_name);
        
        // 使用 pg_dump 风格的 DDL
        writeln!(writer, "\n--").unwrap();
        writeln!(writer, "-- Table structure for table {}", table_name).unwrap();
        writeln!(writer, "--").unwrap();
        writeln!(writer, "DROP TABLE IF EXISTS {} CASCADE;", table_name).unwrap();
        
        // 获取完整的 CREATE TABLE 语句
        let create_stmt_query = format!(
            "SELECT 
                'CREATE TABLE ' || quote_ident('{}') || ' (' || 
                string_agg(
                    quote_ident(column_name) || ' ' || 
                    CASE 
                        WHEN data_type = 'character varying' THEN 'varchar(' || character_maximum_length || ')'
                        WHEN data_type = 'character' THEN 'char(' || character_maximum_length || ')'
                        WHEN data_type = 'numeric' THEN 'numeric(' || numeric_precision || ',' || numeric_scale || ')'
                        ELSE data_type
                    END ||
                    CASE WHEN is_nullable = 'NO' THEN ' NOT NULL' ELSE '' END,
                    ', '
                ) || ');'
            FROM information_schema.columns
            WHERE table_name = '{}'
            GROUP BY table_name",
            table_name, table_name
        );
        
        match client.query_one(&create_stmt_query, &[]).await {
            Ok(row) => {
                let create_stmt: String = row.get(0);
                writeln!(writer, "{}", create_stmt).unwrap();
            }
            Err(e) => {
                log::warn!("Failed to get CREATE TABLE: {}", e);
                continue;
            }
        }

        // 导出数据 - 使用简单的 INSERT 语句
        let count_query = format!("SELECT COUNT(*) FROM {}", table_name);
        let count: i64 = client.query_one(&count_query, &[]).await
            .map_err(|e| format!("Failed to count rows: {}", e))?
            .get(0);
        
        if count > 0 {
            writeln!(writer, "\n--").unwrap();
            writeln!(writer, "-- Data for table {} ({} rows)", table_name, count).unwrap();
            writeln!(writer, "--").unwrap();
            
            // 获取列名
            let cols_query = format!(
                "SELECT column_name FROM information_schema.columns 
                 WHERE table_name = '{}' ORDER BY ordinal_position",
                table_name
            );
            let col_rows = client.query(&cols_query, &[]).await
                .map_err(|e| format!("Failed to get columns: {}", e))?;
            let col_names: Vec<String> = col_rows.iter().map(|r| r.get(0)).collect();
            
            // 分批导出数据
            let batch_size = 1000;
            let mut offset = 0;
            
            loop {
                let data_query = format!(
                    "SELECT * FROM {} LIMIT {} OFFSET {}",
                    table_name, batch_size, offset
                );
                
                let rows = client.query(&data_query, &[]).await
                    .map_err(|e| format!("Failed to export data: {}", e))?;
                
                if rows.is_empty() {
                    break;
                }
                
                for row in &rows {
                    let mut values = Vec::new();
                    for i in 0..row.len() {
                        let value = format_sql_value(&row, i);
                        values.push(value);
                    }
                    
                    writeln!(
                        writer,
                        "INSERT INTO {} ({}) VALUES ({});",
                        table_name,
                        col_names.join(", "),
                        values.join(", ")
                    ).unwrap();
                }
                
                offset += batch_size;
                if rows.len() < batch_size {
                    break;
                }
            }
        }
    }

    writer.flush().unwrap();
    log::info!("========== Export completed ==========");

    Ok(ApiResponse {
        success: true,
        message: format!("Database exported to {}", file_path.display()),
        data: Some(file_path.to_string_lossy().to_string()),
    })
}

// 辅助函数：格式化 SQL 值
fn format_sql_value(row: &tokio_postgres::Row, index: usize) -> String {
    use tokio_postgres::types::Type;
    
    let column_type = row.columns()[index].type_();
    
    match column_type {
        &Type::BOOL => {
            match row.try_get::<_, Option<bool>>(index) {
                Ok(Some(v)) => v.to_string(),
                _ => "NULL".to_string(),
            }
        }
        &Type::INT2 | &Type::INT4 => {
            match row.try_get::<_, Option<i32>>(index) {
                Ok(Some(v)) => v.to_string(),
                _ => "NULL".to_string(),
            }
        }
        &Type::INT8 => {
            match row.try_get::<_, Option<i64>>(index) {
                Ok(Some(v)) => v.to_string(),
                _ => "NULL".to_string(),
            }
        }
        &Type::FLOAT4 | &Type::FLOAT8 => {
            match row.try_get::<_, Option<f64>>(index) {
                Ok(Some(v)) => v.to_string(),
                _ => "NULL".to_string(),
            }
        }
        _ => {
            // 默认作为字符串处理
            match row.try_get::<_, Option<String>>(index) {
                Ok(Some(v)) => format!("'{}'", v.replace("'", "''")),
                _ => "NULL".to_string(),
            }
        }
    }
}
