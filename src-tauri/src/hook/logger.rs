use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use chrono::Local;

use super::types::*;

pub trait Logger: Send + Sync {
    fn log_hook_start(&self, hook_type: &HookType, handler_name: &str);
    fn log_hook_end(&self, hook_type: &HookType, handler_name: &str, duration: Duration);
    fn log_hook_error(&self, hook_type: &HookType, handler_name: &str, error: &HookError);
    fn log_hook_cancel(&self, hook_type: &HookType, handler_name: &str, reason: &str);
}

pub struct NoopLogger;

impl Logger for NoopLogger {
    fn log_hook_start(&self, _hook_type: &HookType, _handler_name: &str) {}
    fn log_hook_end(&self, _hook_type: &HookType, _handler_name: &str, _duration: Duration) {}
    fn log_hook_error(&self, _hook_type: &HookType, _handler_name: &str, _error: &HookError) {}
    fn log_hook_cancel(&self, _hook_type: &HookType, _handler_name: &str, _reason: &str) {}
}

pub struct FileLogger {
    log_path: PathBuf,
}

impl FileLogger {
    pub fn new(log_path: PathBuf) -> Self {
        Self { log_path }
    }

    pub fn default_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".sourcesvn").join("logs").join("hooks.log")
    }

    fn write_log(&self, message: &str) {
        if let Some(parent) = self.log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let file_result = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path);
        match file_result {
            Ok(mut file) => {
                if writeln!(file, "{}", message).is_err() {
                    eprintln!("[log write error] {}", message);
                }
            }
            Err(_) => {
                eprintln!("[log file error] {}", message);
            }
        }
    }
}

impl Logger for FileLogger {
    fn log_hook_start(&self, hook_type: &HookType, handler_name: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        self.write_log(&format!(
            "[{}] Hook开始执行: {} - {}",
            timestamp,
            hook_type.as_str(),
            handler_name
        ));
    }

    fn log_hook_end(&self, hook_type: &HookType, handler_name: &str, duration: Duration) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        self.write_log(&format!(
            "[{}] Hook执行完成: {} - {} (耗时: {:?})",
            timestamp,
            hook_type.as_str(),
            handler_name,
            duration
        ));
    }

    fn log_hook_error(&self, hook_type: &HookType, handler_name: &str, error: &HookError) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        self.write_log(&format!(
            "[{}] Hook执行失败: {} - {} (错误: {})",
            timestamp,
            hook_type.as_str(),
            handler_name,
            error
        ));
    }

    fn log_hook_cancel(&self, hook_type: &HookType, handler_name: &str, reason: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        self.write_log(&format!(
            "[{}] Hook执行取消: {} - {} (原因: {})",
            timestamp,
            hook_type.as_str(),
            handler_name,
            reason
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_noop_logger_implements_all_methods() {
        let logger = NoopLogger;
        logger.log_hook_start(&HookType::PreCommit, "test");
        logger.log_hook_end(&HookType::PreCommit, "test", Duration::from_millis(100));
        logger.log_hook_error(
            &HookType::PreCommit,
            "test",
            &HookError::ExecutionFailed("err".into()),
        );
        logger.log_hook_cancel(&HookType::PreCommit, "test", "cancelled");
    }

    #[test]
    fn test_file_logger_default_path() {
        let path = FileLogger::default_path();
        assert!(path.ends_with(".sourcesvn/logs/hooks.log"));
    }

    #[test]
    fn test_file_logger_writes_start_log() {
        let dir = std::env::temp_dir().join("sourcesvn_test_logs");
        let log_path = dir.join("test_start.log");
        let _ = fs::remove_file(&log_path);

        let logger = FileLogger::new(log_path.clone());
        logger.log_hook_start(&HookType::PreCommit, "test_handler");

        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("Hook开始执行"));
        assert!(content.contains("PreCommit"));
        assert!(content.contains("test_handler"));

        let _ = fs::remove_file(&log_path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn test_file_logger_writes_end_log() {
        let dir = std::env::temp_dir().join("sourcesvn_test_logs");
        let log_path = dir.join("test_end.log");
        let _ = fs::remove_file(&log_path);

        let logger = FileLogger::new(log_path.clone());
        logger.log_hook_end(&HookType::PostCommit, "handler", Duration::from_millis(250));

        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("Hook执行完成"));
        assert!(content.contains("PostCommit"));
        assert!(content.contains("250"));

        let _ = fs::remove_file(&log_path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn test_file_logger_writes_error_log() {
        let dir = std::env::temp_dir().join("sourcesvn_test_logs");
        let log_path = dir.join("test_error.log");
        let _ = fs::remove_file(&log_path);

        let logger = FileLogger::new(log_path.clone());
        logger.log_hook_error(
            &HookType::PreUpdate,
            "handler",
            &HookError::TimeoutExpired,
        );

        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("Hook执行失败"));
        assert!(content.contains("PreUpdate"));
        assert!(content.contains("Hook执行超时"));

        let _ = fs::remove_file(&log_path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn test_file_logger_writes_cancel_log() {
        let dir = std::env::temp_dir().join("sourcesvn_test_logs");
        let log_path = dir.join("test_cancel.log");
        let _ = fs::remove_file(&log_path);

        let logger = FileLogger::new(log_path.clone());
        logger.log_hook_cancel(&HookType::PreCommit, "handler", "user abort");

        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("Hook执行取消"));
        assert!(content.contains("PreCommit"));
        assert!(content.contains("user abort"));

        let _ = fs::remove_file(&log_path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn test_file_logger_appends_to_existing_file() {
        let dir = std::env::temp_dir().join("sourcesvn_test_logs");
        let log_path = dir.join("test_append.log");
        let _ = fs::remove_file(&log_path);

        let logger = FileLogger::new(log_path.clone());
        logger.log_hook_start(&HookType::PreCommit, "h1");
        logger.log_hook_end(&HookType::PreCommit, "h1", Duration::from_millis(50));

        let content = fs::read_to_string(&log_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);

        let _ = fs::remove_file(&log_path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn test_file_logger_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<FileLogger>();
    }
}
