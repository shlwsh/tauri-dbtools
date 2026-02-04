// 独立测试脚本 - 测试数据库导出导入功能
use tokio_postgres::NoTls;
use std::fs::File;
use std::io::{Write, BufWriter, BufReader, BufRead};
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;

#[tokio::main]
async fn main() {
    println!("========================================");
    println!("PostgreSQL 导出导入功能测试");
    println!("========================================\n");

    // 数据库配置
    let host = "localhost";
    let port = "5432";
    let user = "postgres";
    let password = "postgres";
    let source_db = "personnel_db";
    let target_db = "p14_test";

    // 测试1: 连接数据库
    println!("1. 测试数据库连接...");
    let conn_str = format!("host={} port={} user={} password={}", host, port, user, password);
    
    match tokio_postgres::connect(&conn_str, NoTls).await {
        Ok((client, connection)) => {
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("连接错误: {}", e);
                }
            });
            println!("   ✓ 数据库连接成功\n");
            
            // 测试2: 检查源数据库
            println!("2. 检查源数据库 {}...", source_db);
            match client.query("SELECT 1 FROM pg_database WHERE datname = $1", &[&source_db]).await {
                Ok(rows) if !rows.is_empty() => {
                    println!("   ✓ 源数据库存在");
                    
                    // 获取表信息
                    let conn_str_db = format!("host={} port={} user={} password={} dbname={}", 
                        host, port, user, password, source_db);
                    
                    if let Ok((db_client, db_connection)) = tokio_postgres::connect(&conn_str_db, NoTls).await {
                        tokio::spawn(async move {
                            if let Err(e) = db_connection.await {
                                eprintln!("连接错误: {}", e);
                            }
                        });
                        
                        match db_client.query(
                            "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public' AND table_type='BASE TABLE'",
                            &[]
                        ).await {
                            Ok(rows) => {
                                let count: i64 = rows[0].get(0);
                                println!("   数据库包含 {} 个表\n", count);
                            }
                            Err(e) => println!("   警告: 无法获取表数量: {}\n", e),
                        }
                    }
                }
                Ok(_) => {
                    println!("   ✗ 源数据库不存在");
                    println!("   请确保数据库 {} 存在", source_db);
                    return;
                }
                Err(e) => {
                    println!("   ✗ 查询失败: {}", e);
                    return;
                }
            }
            
            println!("========================================");
            println!("测试完成 - 数据库连接正常");
            println!("========================================");
        }
        Err(e) => {
            println!("   ✗ 连接失败: {}", e);
            println!("\n请检查:");
            println!("  - PostgreSQL 是否运行");
            println!("  - 用户名密码是否正确");
            println!("  - 端口是否正确");
        }
    }
}
