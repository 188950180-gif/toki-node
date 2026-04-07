//! 日志配置和初始化模块

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::metadata::LevelFilter;
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

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

    let fmt_layer = if config.json {
        fmt::layer()
            .json()
            .with_timer(ChronoLocal::new("%Y-%m-%d %H:%M:%S%.3f".to_string()))
            .with_target(true)
            .with_level(true)
            .with_file(true)
            .with_line_number(true)
            .with_env_filter(env_filter)
    } else {
        fmt::layer()
            .with_timer(ChronoLocal::new("%Y-%m-%d %H:%M:%S%.3f".to_string()))
            .with_target(true)
            .with_level(true)
            .with_file(true)
            .with_line_number(true)
            .with_env_filter(env_filter)
    };

    let subscriber = tracing_subscriber::registry();

    if config.console {
        if let Some(file_path) = config.file {
            // 同时输出到控制台和文件
            let file_appender = make_file_appender(&file_path)?;
            let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
            let file_layer = fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_timer(ChronoLocal::new("%Y-%m-%d %H:%M:%S%.3f".to_string()))
                .with_target(true)
                .with_level(true)
                .with_file(true)
                .with_line_number(true);
            subscriber.with(fmt_layer).with(file_layer).init();
        } else {
            subscriber.with(fmt_layer).init();
        }
    } else if let Some(file_path) = config.file {
        // 仅输出到文件
        let file_appender = make_file_appender(&file_path)?;
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        let file_layer = fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .with_timer(ChronoLocal::new("%Y-%m-%d %H:%M:%S%.3f".to_string()))
            .with_target(true)
            .with_level(true)
            .with_file(true)
            .with_line_number(true);
        subscriber.with(file_layer).init();
    } else {
        // 无输出（不应该发生）
        subscriber.with(fmt_layer).init();
    }

    Ok(())
}

fn make_file_appender(file_path: &str) -> Result<std::fs::File> {
    let path = Path::new(file_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    Ok(file)
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
        // 等待异步写入完成
        std::thread::sleep(std::time::Duration::from_millis(100));
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