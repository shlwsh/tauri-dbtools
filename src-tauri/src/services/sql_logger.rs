/**
 * SQL Logger Service
 * 
 * 详细记录所有 SQL 命令执行的过程，包括：
 * - SQL 语句内容
 * - 执行时间
 * - 执行结果（成功/失败）
 * - 影响的行数
 * - 错误信息
 * - 用户和数据库信息
 */

use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

/// SQL 执行日志条目
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SqlLogEntry {
    /// 时间戳
    pub timestamp: String,
    /// 数据库名称
    pub database: String,
    /// SQL 语句
    pub sql: String,
    /// 执行状态（success/error）
    pub status: String,
    /// 执行耗时（毫秒）
    pub duration_ms: u64,
    /// 查询类型（SELECT/INSERT/UPDATE/DELETE/DDL）
    pub query_type: String,
    /// 影响的行数（DML 操作）
    pub affected_rows: Option<u64>,
    /// 返回的行数（SELECT 操作）
    pub returned_rows: Option<usize>,
    /// 错误信息（如果失败）
    pub error: Option<String>,
    /// 错误位置（如果有）
    pub error_position: Option<String>,
}

impl SqlLogEntry {
    /// 创建成功的日志条目
    pub fn success(
        database: String,
        sql: String,
        duration_ms: u64,
        query_type: String,
        affected_rows: Option<u64>,
        returned_rows: Option<usize>,
    ) -> Self {
        Self {
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            database,
            sql,
            status: "success".to_string(),
            duration_ms,
            query_type,
            affected_rows,
            returned_rows,
            error: None,
            error_position: None,
        }
    }

    /// 创建失败的日志条目
    pub fn error(
        database: String,
        sql: String,
        duration_ms: u64,
        error: String,
        error_position: Option<String>,
    ) -> Self {
        Self {
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            database,
            sql,
            status: "error".to_string(),
            duration_ms,
            query_type: "ERROR".to_string(),
            affected_rows: None,
            returned_rows: None,
            error: Some(error),
            error_position,
        }
    }

    /// 格式化为可读的日志字符串
    pub fn format_log(&self) -> String {
        let mut log = format!(
            "[{}] [{}] [{}] Database: {}\n",
            self.timestamp, self.status.to_uppercase(), self.query_type, self.database
        );

        // SQL 语句（截断过长的语句）
        let sql_preview = if self.sql.len() > 200 {
            format!("{}...", &self.sql[..200])
        } else {
            self.sql.clone()
        };
        log.push_str(&format!("SQL: {}\n", sql_preview));

        // 执行结果
        if self.status == "success" {
            log.push_str(&format!("Duration: {}ms\n", self.duration_ms));
            
            if let Some(rows) = self.affected_rows {
                log.push_str(&format!("Affected Rows: {}\n", rows));
            }
            
            if let Some(rows) = self.returned_rows {
                log.push_str(&format!("Returned Rows: {}\n", rows));
            }
        } else {
            log.push_str(&format!("Duration: {}ms\n", self.duration_ms));
            
            if let Some(error) = &self.error {
                log.push_str(&format!("Error: {}\n", error));
            }
            
            if let Some(pos) = &self.error_position {
                log.push_str(&format!("Error Position: {}\n", pos));
            }
        }

        log.push_str("----------------------------------------\n");
        log
    }

    /// 格式化为 JSON 字符串
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// SQL 日志记录器
pub struct SqlLogger {
    log_file_path: PathBuf,
    json_log_path: PathBuf,
}

impl SqlLogger {
    /// 创建新的日志记录器
    pub fn new(log_dir: PathBuf) -> Result<Self, String> {
        // 确保日志目录存在
        std::fs::create_dir_all(&log_dir)
            .map_err(|e| format!("无法创建日志目录: {}", e))?;

        // 生成日志文件名（按日期）
        let date = Local::now().format("%Y-%m-%d").to_string();
        let log_file_path = log_dir.join(format!("sql_execution_{}.log", date));
        let json_log_path = log_dir.join(format!("sql_execution_{}.jsonl", date));

        Ok(Self {
            log_file_path,
            json_log_path,
        })
    }

    /// 记录 SQL 执行日志
    pub fn log(&self, entry: &SqlLogEntry) -> Result<(), String> {
        // 写入文本日志
        self.write_text_log(entry)?;
        
        // 写入 JSON 日志（用于程序化分析）
        self.write_json_log(entry)?;

        Ok(())
    }

    /// 写入文本格式日志
    fn write_text_log(&self, entry: &SqlLogEntry) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)
            .map_err(|e| format!("无法打开日志文件: {}", e))?;

        file.write_all(entry.format_log().as_bytes())
            .map_err(|e| format!("无法写入日志: {}", e))?;

        Ok(())
    }

    /// 写入 JSON 格式日志（每行一个 JSON 对象）
    fn write_json_log(&self, entry: &SqlLogEntry) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.json_log_path)
            .map_err(|e| format!("无法打开 JSON 日志文件: {}", e))?;

        let json = entry.to_json()
            .map_err(|e| format!("无法序列化日志条目: {}", e))?;

        writeln!(file, "{}", json)
            .map_err(|e| format!("无法写入 JSON 日志: {}", e))?;

        Ok(())
    }

    /// 获取日志文件路径
    pub fn get_log_file_path(&self) -> &PathBuf {
        &self.log_file_path
    }

    /// 获取 JSON 日志文件路径
    pub fn get_json_log_path(&self) -> &PathBuf {
        &self.json_log_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_sql_log_entry_success() {
        let entry = SqlLogEntry::success(
            "test_db".to_string(),
            "SELECT * FROM users".to_string(),
            150,
            "SELECT".to_string(),
            None,
            Some(10),
        );

        assert_eq!(entry.status, "success");
        assert_eq!(entry.database, "test_db");
        assert_eq!(entry.duration_ms, 150);
        assert_eq!(entry.returned_rows, Some(10));
        assert!(entry.error.is_none());
    }

    #[test]
    fn test_sql_log_entry_error() {
        let entry = SqlLogEntry::error(
            "test_db".to_string(),
            "SELECT * FROM invalid_table".to_string(),
            50,
            "Table does not exist".to_string(),
            Some("Line 1, Column 15".to_string()),
        );

        assert_eq!(entry.status, "error");
        assert_eq!(entry.database, "test_db");
        assert!(entry.error.is_some());
        assert_eq!(entry.error.as_ref().unwrap(), "Table does not exist");
    }

    #[test]
    fn test_format_log() {
        let entry = SqlLogEntry::success(
            "test_db".to_string(),
            "INSERT INTO users VALUES (1, 'John')".to_string(),
            75,
            "INSERT".to_string(),
            Some(1),
            None,
        );

        let log = entry.format_log();
        assert!(log.contains("test_db"));
        assert!(log.contains("INSERT"));
        assert!(log.contains("75ms"));
        assert!(log.contains("Affected Rows: 1"));
    }

    #[test]
    fn test_to_json() {
        let entry = SqlLogEntry::success(
            "test_db".to_string(),
            "SELECT 1".to_string(),
            10,
            "SELECT".to_string(),
            None,
            Some(1),
        );

        let json = entry.to_json();
        assert!(json.is_ok());
        assert!(json.unwrap().contains("test_db"));
    }

    #[test]
    fn test_sql_logger_creation() {
        let temp_dir = env::temp_dir().join("sql_logger_test");
        let logger = SqlLogger::new(temp_dir.clone());
        
        assert!(logger.is_ok());
        assert!(temp_dir.exists());

        // Cleanup
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
