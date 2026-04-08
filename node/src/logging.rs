//! 日志配置和初始化模块

use std::fs::OpenOptions;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::metadata::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

/// 日志配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogConfig {
    /// 日志级别 (info, debug, warn, error)
    pub level: String,
    /// 日志文件路径
    pub file: Option<String>,
    /// 是否输出到控制台
    pub console: bool,
    /// 是否输出 JSON 格式
    pub json: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file: Some("logs/toki.log".to_string()),
            console: true,
            json: false,
        }
    }
}

/// 初始化日志系统
pub fn init_logging(config: LogConfig) -> Result<()> {
    let level = match config.level.as_str() {
        "debug" => LevelFilter::DEBUG,
        "warn" => LevelFilter::WARN,
        "error" => LevelFilter::ERROR,
        _ => LevelFilter::INFO,
    };

    let env_filter = EnvFilter::from_default_env().add_directive(level.into());

    let subscriber = tracing_subscriber::registry();

    if config.console {
        let fmt_layer = fmt::layer()
            .with_target(true)
            .with_level(true)
            .with_file(true)
            .with_line_number(true)
            .with_env_filter(env_filter.clone());
        subscriber.with(fmt_layer).init();
    }

    if let Some(file_path) = config.file {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;
        let file_layer = fmt::layer()
            .with_writer(std::sync::Arc::new(file))
            .with_ansi(false)
            .with_target(true)
            .with_level(true)
            .with_file(true)
            .with_line_number(true)
            .with_env_filter(env_filter);
        subscriber.with(file_layer).init();
    } else if !config.console {
        // 如果既没有控制台也没有文件，至少设置一个默认的
        subscriber.with(fmt::layer()).init();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_initialization() {
        let config = LogConfig::default();
        let result = init_logging(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_logging_with_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let config = LogConfig {
            file: Some(log_path.to_str().unwrap().to_string()),
            console: false,
            ..Default::default()
        };
        let result = init_logging(config);
        assert!(result.is_ok());
        tracing::info!("Test log message");
        std::thread::sleep(std::time::Duration::from_millis(50));
        let metadata = std::fs::metadata(&log_path).unwrap();
        assert!(metadata.len() > 0);
    }

    #[test]
    fn test_logging_invalid_level() {
        let config = LogConfig {
            level: "invalid".to_string(),
            ..Default::default()
        };
        let result = init_logging(config);
        assert!(result.is_ok());
    }
}