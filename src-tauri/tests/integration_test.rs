// 集成测试 - 测试导出导入功能
use tokio_postgres::NoTls;
use std::fs::File;
use std::io::{Write, BufWriter, BufReader, BufRead};
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("集成测试：数据库导出导入");
    println!("========================================\n");

    let host = "localhost";
    let port = "5432";
    let user = "postgres";
    let password = "postgres";
    let source_db = "personnel_db";
    let target_db = "p14_test";
    let export_file = "test_export.sql.gz";

    // 步骤1：导出数据库
    println!("步骤1：导出数据库 {}...", source_db);
    export_database(host, port, user, password, source_db, export_file).await?;
    println!("✓ 导出完成\n");

    // 步骤2：导入数据库
    println!("步骤2：导入到数据库 {}...", target_db);
    import_database(host, port, user, password, target_db, export_file).await?;
    println!("✓ 导入完成\n");

    // 步骤3：验证数据
    println!("步骤3：验证数据...");
    verify_data(host, port, user, password, source_db, target_db).await?;
    
    println!("\n========================================");
    println!("测试完成！");
    println!("========================================");

    Ok(())
}

async fn export_database(
    host: &str,
    port: &str,
    user: &str,
    password: &str,
    database: &str,
    output_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn_str = format!(
        "host={} port={} user={} password={} dbname={}",
        host, port, user, password, database
    );

    let (client, connection) = tokio_postgres::connect(&conn_str, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("连接错误: {}", e);
        }
    });

    let file = File::create(output_file)?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut writer = BufWriter::new(encoder);

    writeln!(writer, "-- PostgreSQL database dump")?;
    writeln!(writer, "SET client_encoding = 'UTF8';")?;
    writeln!(writer, "SET standard_conforming_strings = on;")?;
    writeln!(writer, "")?;

    // 导出序列
    println!("   导出序列...");
    let sequences = client.query(
        "SELECT sequence_name FROM information_schema.sequences WHERE sequence_schema = 'public' ORDER BY sequence_name",
        &[]
    ).await?;

    for seq_row in &sequences {
        let seq_name: String = seq_row.get(0);
        
        if let Ok(info) = client.query_one(
            &format!("SELECT last_value, increment_by, min_value, max_value FROM \"{}\"", seq_name),
            &[]
        ).await {
            let last_value: i64 = info.get(0);
            let increment: i64 = info.get(1);
            let min_value: i64 = info.get(2);
            let max_value: i64 = info.get(3);
            
            writeln!(writer, "\n-- Sequence: {}", seq_name)?;
            writeln!(writer, "DROP SEQUENCE IF EXISTS \"{}\" CASCADE;", seq_name)?;
            writeln!(writer, "CREATE SEQUENCE \"{}\" INCREMENT {} MINVALUE {} MAXVALUE {} START {};", 
                seq_name, increment, min_value, max_value, last_value)?;
            writeln!(writer, "SELECT setval('\"{}\", {}, true);", seq_name, last_value)?;
        }
    }

    // 获取所有表
    let tables = client.query(
        "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE' ORDER BY table_name",
        &[]
    ).await?;

    println!("   找到 {} 个表", tables.len());

    let mut total_rows = 0;

    for table_row in &tables {
        let table_name: String = table_row.get(0);
        println!("   导出表: {}", table_name);

        // 获取列信息和数据类型
        let columns_query = format!(
            "SELECT column_name, udt_name, character_maximum_length, is_nullable, column_default
             FROM information_schema.columns 
             WHERE table_schema = 'public' AND table_name = $1
             ORDER BY ordinal_position"
        );

        let columns = client.query(&columns_query, &[&table_name]).await?;

        if columns.is_empty() {
            println!("     警告: 表 {} 没有列，跳过", table_name);
            continue;
        }

        // CREATE TABLE
        writeln!(writer, "\n-- Table: {}", table_name)?;
        writeln!(writer, "DROP TABLE IF EXISTS \"{}\" CASCADE;", table_name)?;
        write!(writer, "CREATE TABLE \"{}\" (\n", table_name)?;

        for (i, col) in columns.iter().enumerate() {
            let col_name: String = col.get(0);
            let data_type: String = col.get(1);
            let max_length: Option<i32> = col.get(2);
            let is_nullable: String = col.get(3);
            let col_default: Option<String> = col.get(4);

            if i > 0 {
                write!(writer, ",\n")?;
            }

            write!(writer, "  \"{}\" ", col_name)?;
            
            // 处理数据类型
            match data_type.as_str() {
                "varchar" | "bpchar" => {
                    if let Some(len) = max_length {
                        write!(writer, "character varying({})", len)?;
                    } else {
                        write!(writer, "character varying")?;
                    }
                },
                "int4" => write!(writer, "integer")?,
                "int8" => write!(writer, "bigint")?,
                "int2" => write!(writer, "smallint")?,
                "float4" => write!(writer, "real")?,
                "float8" => write!(writer, "double precision")?,
                "bool" => write!(writer, "boolean")?,
                "timestamptz" => write!(writer, "timestamp with time zone")?,
                "timestamp" => write!(writer, "timestamp without time zone")?,
                _ => write!(writer, "{}", data_type)?,
            }
            
            if is_nullable == "NO" {
                write!(writer, " NOT NULL")?;
            }
            
            // Skip DEFAULT clauses that reference sequences
            if let Some(default) = col_default {
                if !default.contains("nextval") {
                    write!(writer, " DEFAULT {}", default)?;
                }
            }
        }

        writeln!(writer, "\n);")?;

        // 导出数据
        let data_query = format!("SELECT * FROM \"{}\"", table_name);
        let rows = match client.query(&data_query, &[]).await {
            Ok(r) => r,
            Err(e) => {
                println!("     错误: 无法查询表 {}: {}", table_name, e);
                continue;
            }
        };

        if !rows.is_empty() {
            println!("     {} 行数据", rows.len());
            total_rows += rows.len();
            writeln!(writer, "\n-- Data for table: {} ({} rows)", table_name, rows.len())?;

            let col_info: Vec<(String, String)> = columns.iter().map(|col| {
                let name: String = col.get(0);
                let dtype: String = col.get(1);
                (name, dtype)
            }).collect();

            for row in rows {
                let mut values = Vec::new();
                for (i, (_, col_type)) in col_info.iter().enumerate() {
                    let value = match col_type.as_str() {
                        "int2" | "int4" | "int8" => {
                            if let Ok(Some(v)) = row.try_get::<_, Option<i64>>(i) {
                                v.to_string()
                            } else if let Ok(Some(v)) = row.try_get::<_, Option<i32>>(i) {
                                v.to_string()
                            } else if let Ok(Some(v)) = row.try_get::<_, Option<i16>>(i) {
                                v.to_string()
                            } else {
                                "NULL".to_string()
                            }
                        },
                        "float4" | "float8" | "numeric" => {
                            if let Ok(Some(v)) = row.try_get::<_, Option<f64>>(i) {
                                v.to_string()
                            } else if let Ok(Some(v)) = row.try_get::<_, Option<f32>>(i) {
                                v.to_string()
                            } else {
                                "NULL".to_string()
                            }
                        },
                        "bool" => {
                            if let Ok(Some(v)) = row.try_get::<_, Option<bool>>(i) {
                                if v { "true" } else { "false" }.to_string()
                            } else {
                                "NULL".to_string()
                            }
                        },
                        _ => {
                            if let Ok(Some(s)) = row.try_get::<_, Option<String>>(i) {
                                let escaped = s.replace("\\", "\\\\").replace("'", "''");
                                format!("'{}'", escaped)
                            } else {
                                "NULL".to_string()
                            }
                        }
                    };
                    values.push(value);
                }

                writeln!(
                    writer,
                    "INSERT INTO \"{}\" ({}) VALUES ({});",
                    table_name,
                    col_info.iter().map(|(c, _)| format!("\"{}\"", c)).collect::<Vec<_>>().join(", "),
                    values.join(", ")
                )?;
            }
        }
    }

    println!("   总计导出 {} 行数据", total_rows);

    // 添加序列默认值
    println!("   添加序列默认值...");
    for table_row in &tables {
        let table_name: String = table_row.get(0);
        
        let seq_defaults_query = "SELECT column_name, column_default 
                                  FROM information_schema.columns 
                                  WHERE table_schema = 'public' AND table_name = $1 
                                  AND column_default LIKE '%nextval%'";
        
        if let Ok(seq_cols) = client.query(seq_defaults_query, &[&table_name]).await {
            for seq_col in seq_cols {
                let col_name: String = seq_col.get(0);
                let col_default: String = seq_col.get(1);
                writeln!(writer, "\nALTER TABLE \"{}\" ALTER COLUMN \"{}\" SET DEFAULT {};",
                    table_name, col_name, col_default)?;
            }
        }
    }

    // 导出主键和约束
    println!("   导出约束...");
    for table_row in &tables {
        let table_name: String = table_row.get(0);
        
        let pk_query = "SELECT constraint_name, column_name 
                        FROM information_schema.key_column_usage 
                        WHERE table_schema = 'public' AND table_name = $1 
                        AND constraint_name IN (
                            SELECT constraint_name FROM information_schema.table_constraints 
                            WHERE table_schema = 'public' AND table_name = $1 AND constraint_type = 'PRIMARY KEY'
                        )
                        ORDER BY ordinal_position";
        
        if let Ok(pk_rows) = client.query(pk_query, &[&table_name]).await {
            if !pk_rows.is_empty() {
                let pk_name: String = pk_rows[0].get(0);
                let pk_cols: Vec<String> = pk_rows.iter().map(|r| {
                    let col: String = r.get(1);
                    format!("\"{}\"", col)
                }).collect();
                
                writeln!(writer, "\nALTER TABLE \"{}\" ADD CONSTRAINT \"{}\" PRIMARY KEY ({});",
                    table_name, pk_name, pk_cols.join(", "))?;
            }
        }
    }

    // 导出索引
    println!("   导出索引...");
    let indexes_query = "SELECT indexname, indexdef FROM pg_indexes WHERE schemaname = 'public' AND indexname NOT LIKE '%_pkey' ORDER BY indexname";
    if let Ok(indexes) = client.query(indexes_query, &[]).await {
        for idx_row in indexes {
            let _idx_name: String = idx_row.get(0);
            let idx_def: String = idx_row.get(1);
            writeln!(writer, "\n{};", idx_def)?;
        }
    }

    writer.flush()?;
    Ok(())
}

async fn import_database(
    host: &str,
    port: &str,
    user: &str,
    password: &str,
    database: &str,
    input_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn_str = format!("host={} port={} user={} password={}", host, port, user, password);

    let (client, connection) = tokio_postgres::connect(&conn_str, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("连接错误: {}", e);
        }
    });

    // 删除旧数据库
    let check_db = client.query("SELECT 1 FROM pg_database WHERE datname = $1", &[&database]).await?;
    if !check_db.is_empty() {
        println!("   删除旧数据库...");
        client.execute(
            &format!("SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}' AND pid <> pg_backend_pid()", database),
            &[],
        ).await.ok();
        client.execute(&format!("DROP DATABASE IF EXISTS \"{}\"", database), &[]).await?;
    }

    // 创建新数据库
    println!("   创建数据库...");
    client.execute(&format!("CREATE DATABASE \"{}\"", database), &[]).await?;

    drop(client);

    // 连接到新数据库
    let target_conn_str = format!(
        "host={} port={} user={} password={} dbname={}",
        host, port, user, password, database
    );

    let (target_client, target_connection) = tokio_postgres::connect(&target_conn_str, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = target_connection.await {
            eprintln!("连接错误: {}", e);
        }
    });

    // 读取并执行 SQL
    println!("   执行 SQL 语句...");
    let file = File::open(input_file)?;
    let decoder = GzDecoder::new(file);
    let reader = BufReader::new(decoder);

    let mut sql_buffer = String::new();
    let mut statement_count = 0;
    let mut error_count = 0;
    let mut create_table_count = 0;
    let mut insert_count = 0;

    for (_line_num, line) in reader.lines().enumerate() {
        let line = line?;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with("--") {
            continue;
        }

        sql_buffer.push_str(&line);
        sql_buffer.push(' ');

        if trimmed.ends_with(';') {
            let sql = sql_buffer.trim();
            
            match target_client.execute(sql, &[]).await {
                Ok(_) => {
                    statement_count += 1;
                    if sql.to_uppercase().starts_with("CREATE TABLE") {
                        create_table_count += 1;
                    } else if sql.to_uppercase().starts_with("INSERT INTO") {
                        insert_count += 1;
                    }
                },
                Err(e) => {
                    error_count += 1;
                    if error_count <= 5 {
                        println!("   警告: {} - SQL: {}", e, sql.chars().take(100).collect::<String>());
                    }
                }
            }
            sql_buffer.clear();
        }
    }

    println!("   成功执行 {} 条语句 ({} CREATE TABLE, {} INSERT)", statement_count, create_table_count, insert_count);
    println!("   {} 条失败", error_count);
    Ok(())
}

async fn verify_data(
    host: &str,
    port: &str,
    user: &str,
    password: &str,
    source_db: &str,
    target_db: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 连接到源数据库
    let source_conn_str = format!(
        "host={} port={} user={} password={} dbname={}",
        host, port, user, password, source_db
    );
    let (source_client, source_connection) = tokio_postgres::connect(&source_conn_str, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = source_connection.await {
            eprintln!("连接错误: {}", e);
        }
    });

    // 连接到目标数据库
    let target_conn_str = format!(
        "host={} port={} user={} password={} dbname={}",
        host, port, user, password, target_db
    );
    let (target_client, target_connection) = tokio_postgres::connect(&target_conn_str, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = target_connection.await {
            eprintln!("连接错误: {}", e);
        }
    });

    // 获取表列表
    let source_tables = source_client.query(
        "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE' ORDER BY table_name",
        &[]
    ).await?;

    let target_tables = target_client.query(
        "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE' ORDER BY table_name",
        &[]
    ).await?;

    println!("   源数据库表数: {}", source_tables.len());
    println!("   目标数据库表数: {}", target_tables.len());

    if source_tables.len() != target_tables.len() {
        println!("   ⚠ 表数量不匹配！");
    } else {
        println!("   ✓ 表数量匹配");
    }

    // 检查几个示例表的数据
    for table_row in source_tables.iter().take(5) {
        let table_name: String = table_row.get(0);
        
        let source_count: i64 = source_client.query_one(
            &format!("SELECT COUNT(*) FROM \"{}\"", table_name),
            &[]
        ).await?.get(0);

        match target_client.query_one(
            &format!("SELECT COUNT(*) FROM \"{}\"", table_name),
            &[]
        ).await {
            Ok(row) => {
                let target_count: i64 = row.get(0);
                if source_count == target_count {
                    println!("   ✓ {}: {} 行", table_name, source_count);
                } else {
                    println!("   ⚠ {}: 源 {} 行, 目标 {} 行", table_name, source_count, target_count);
                }
            }
            Err(e) => {
                println!("   ✗ {}: 查询失败 - {}", table_name, e);
            }
        }
    }

    Ok(())
}
