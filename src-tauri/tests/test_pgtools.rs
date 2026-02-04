// 集成测试 - 使用 pg_dump/pg_restore 测试导出导入功能
use std::process::Command;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("集成测试：数据库导出导入 (pg_dump/pg_restore)");
    println!("========================================\n");

    let host = "localhost";
    let port = "5432";
    let user = "postgres";
    let password = "postgres";
    let source_db = "personnel_db";
    let target_db = "p14_pgtools";
    
    // 获取导出目录
    let mut export_dir = dirs::home_dir().expect("无法获取用户目录");
    export_dir.push("pg-db-tool-exports");
    std::fs::create_dir_all(&export_dir)?;
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let export_file = export_dir.join(format!("{}_{}.backup", source_db, timestamp));

    // 步骤1：使用 pg_dump 导出数据库
    println!("步骤1：导出数据库 {}...", source_db);
    let export_output = Command::new("pg_dump")
        .arg("-h").arg(host)
        .arg("-p").arg(port)
        .arg("-U").arg(user)
        .arg("-F").arg("c")  // 自定义格式（压缩）
        .arg("-b")  // 包含大对象
        .arg("-v")  // 详细模式
        .arg("-f").arg(&export_file)
        .arg(source_db)
        .env("PGPASSWORD", password)
        .output()?;

    if !export_output.status.success() {
        let stderr = String::from_utf8_lossy(&export_output.stderr);
        eprintln!("✗ 导出失败: {}", stderr);
        return Err(format!("导出失败: {}", stderr).into());
    }

    let stderr = String::from_utf8_lossy(&export_output.stderr);
    println!("   pg_dump 输出:");
    for line in stderr.lines().take(10) {
        println!("   {}", line);
    }

    // 检查文件大小
    if let Ok(metadata) = std::fs::metadata(&export_file) {
        let size_kb = metadata.len() / 1024;
        println!("   ✓ 导出完成，文件大小: {} KB", size_kb);
    }
    println!();

    // 步骤2：检查并删除旧的目标数据库
    println!("步骤2：准备目标数据库 {}...", target_db);
    
    let check_output = Command::new("psql")
        .arg("-h").arg(host)
        .arg("-p").arg(port)
        .arg("-U").arg(user)
        .arg("-d").arg("postgres")
        .arg("-t")
        .arg("-c").arg(format!("SELECT 1 FROM pg_database WHERE datname='{}'", target_db))
        .env("PGPASSWORD", password)
        .output()?;

    let db_exists = String::from_utf8_lossy(&check_output.stdout).trim().contains("1");

    if db_exists {
        println!("   数据库已存在，正在删除...");
        
        // 终止所有连接
        let _ = Command::new("psql")
            .arg("-h").arg(host)
            .arg("-p").arg(port)
            .arg("-U").arg(user)
            .arg("-d").arg("postgres")
            .arg("-c").arg(format!(
                "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}' AND pid <> pg_backend_pid()",
                target_db
            ))
            .env("PGPASSWORD", password)
            .output();

        // 删除数据库
        let drop_output = Command::new("psql")
            .arg("-h").arg(host)
            .arg("-p").arg(port)
            .arg("-U").arg(user)
            .arg("-d").arg("postgres")
            .arg("-c").arg(format!("DROP DATABASE IF EXISTS \"{}\"", target_db))
            .env("PGPASSWORD", password)
            .output()?;

        if !drop_output.status.success() {
            let stderr = String::from_utf8_lossy(&drop_output.stderr);
            println!("   警告: {}", stderr);
        }
    }

    // 创建新数据库
    println!("   创建新数据库...");
    let create_output = Command::new("psql")
        .arg("-h").arg(host)
        .arg("-p").arg(port)
        .arg("-U").arg(user)
        .arg("-d").arg("postgres")
        .arg("-c").arg(format!("CREATE DATABASE \"{}\"", target_db))
        .env("PGPASSWORD", password)
        .output()?;

    if !create_output.status.success() {
        let stderr = String::from_utf8_lossy(&create_output.stderr);
        eprintln!("   ✗ 创建数据库失败: {}", stderr);
        return Err(format!("创建数据库失败: {}", stderr).into());
    }
    println!("   ✓ 数据库创建成功");
    println!();

    // 步骤3：使用 pg_restore 导入数据库
    println!("步骤3：导入到数据库 {}...", target_db);
    let restore_output = Command::new("pg_restore")
        .arg("-h").arg(host)
        .arg("-p").arg(port)
        .arg("-U").arg(user)
        .arg("-d").arg(target_db)
        .arg("-v")  // 详细模式
        .arg("--no-owner")  // 不恢复所有权
        .arg("--no-acl")  // 不恢复访问权限
        .arg(&export_file)
        .env("PGPASSWORD", password)
        .output()?;

    let stderr = String::from_utf8_lossy(&restore_output.stderr);
    println!("   pg_restore 输出:");
    for line in stderr.lines().take(10) {
        println!("   {}", line);
    }

    if !restore_output.status.success() {
        println!("   ⚠ pg_restore 返回非零状态码（这可能是正常的）");
    } else {
        println!("   ✓ 导入完成");
    }
    println!();

    // 步骤4：验证数据
    println!("步骤4：验证数据...");
    
    // 获取源数据库表数量
    let source_tables_output = Command::new("psql")
        .arg("-h").arg(host)
        .arg("-p").arg(port)
        .arg("-U").arg(user)
        .arg("-d").arg(source_db)
        .arg("-t")
        .arg("-c").arg("SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public' AND table_type='BASE TABLE'")
        .env("PGPASSWORD", password)
        .output()?;

    let source_table_count = String::from_utf8_lossy(&source_tables_output.stdout)
        .trim()
        .parse::<i32>()
        .unwrap_or(0);

    // 获取目标数据库表数量
    let target_tables_output = Command::new("psql")
        .arg("-h").arg(host)
        .arg("-p").arg(port)
        .arg("-U").arg(user)
        .arg("-d").arg(target_db)
        .arg("-t")
        .arg("-c").arg("SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public' AND table_type='BASE TABLE'")
        .env("PGPASSWORD", password)
        .output()?;

    let target_table_count = String::from_utf8_lossy(&target_tables_output.stdout)
        .trim()
        .parse::<i32>()
        .unwrap_or(0);

    println!("   源数据库表数: {}", source_table_count);
    println!("   目标数据库表数: {}", target_table_count);

    if source_table_count == target_table_count {
        println!("   ✓ 表数量匹配");
    } else {
        println!("   ⚠ 表数量不匹配！");
    }

    // 检查几个示例表的数据
    let sample_tables = vec!["departments", "employees", "product"];
    
    for table in sample_tables {
        // 源数据库行数
        let source_count_output = Command::new("psql")
            .arg("-h").arg(host)
            .arg("-p").arg(port)
            .arg("-U").arg(user)
            .arg("-d").arg(source_db)
            .arg("-t")
            .arg("-c").arg(format!("SELECT COUNT(*) FROM \"{}\"", table))
            .env("PGPASSWORD", password)
            .output();

        // 目标数据库行数
        let target_count_output = Command::new("psql")
            .arg("-h").arg(host)
            .arg("-p").arg(port)
            .arg("-U").arg(user)
            .arg("-d").arg(target_db)
            .arg("-t")
            .arg("-c").arg(format!("SELECT COUNT(*) FROM \"{}\"", table))
            .env("PGPASSWORD", password)
            .output();

        if let (Ok(source), Ok(target)) = (source_count_output, target_count_output) {
            let source_count = String::from_utf8_lossy(&source.stdout).trim().parse::<i32>().unwrap_or(-1);
            let target_count = String::from_utf8_lossy(&target.stdout).trim().parse::<i32>().unwrap_or(-1);

            if source_count == target_count && source_count >= 0 {
                println!("   ✓ {}: {} 行", table, source_count);
            } else if source_count >= 0 && target_count >= 0 {
                println!("   ⚠ {}: 源 {} 行, 目标 {} 行", table, source_count, target_count);
            } else {
                println!("   ⚠ {}: 表可能不存在", table);
            }
        }
    }

    println!("\n========================================");
    println!("测试完成！");
    println!("导出文件: {}", export_file.display());
    println!("========================================");

    Ok(())
}
