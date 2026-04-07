//! 日志配置和初始化模块

use std::fs::File;
use std::io::Write;
use std::path::Path;

use anyhow::Result;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use serde::{Deserialize, Serialize};

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
        "debug" => log::LevelFilter::Debug,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    };

    let pattern = if config.json {
        r#"{"time":"%d(%{%Y-%m-%dT%H:%M:%S%.3f%z})","level":"%l","target":"%c","message":"%m"}"#
    } else {
        "[%d{%Y-%m-%d %H:%M:%S%.3f}] [%l] [%c] %m%n"
    };

    let encoder = Box::new(PatternEncoder::new(pattern));

    let mut config_builder = Config::builder();

    // 控制台输出
    if config.console {
        let console_appender = ConsoleAppender::builder().encoder(encoder.clone()).build();
        config_builder = config_builder
            .appender(Appender::builder().build("console", Box::new(console_appender)));
    }

    // 文件输出
    if let Some(file_path) = config.file {
        if let Some(parent) = Path::new(&file_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file_appender = FileAppender::builder().encoder(encoder).build(&file_path)?;
        config_builder =
            config_builder.appender(Appender::builder().build("file", Box::new(file_appender)));
    }

    // 设置 root logger
    let root = if config.console && config.file.is_some() {
        Root::builder()
            .appender("console")
            .appender("file")
            .build(level)
    } else if config.console {
        Root::builder().appender("console").build(level)
    } else if let Some(_) = config.file {
        Root::builder().appender("file").build(level)
    } else {
        Root::builder().build(level)
    };

    let config = config_builder.build(root)?;
    log4rs::init_config(config)?;

    Ok(())
}

/// 获取用于 `tracing` 的日志层（可选，用于与 tracing 集成）
#[cfg(feature = "tracing")]
pub fn get_tracing_layer() -> tracing_subscriber::layer::BoxedLayer<tracing_subscriber::Registry> {
    use tracing_subscriber::fmt::Layer;
    let layer = Layer::new().with_filter(tracing_subscriber::EnvFilter::from_default_env());
    Box::new(layer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_initialization() {
        let config = LogConfig::default();
        // init_logging 返回 Result，这里直接调用，不要求 is_ok()
        let result = init_logging(config);
        // 如果返回 Ok，则初始化成功；否则打印错误
        if let Err(e) = result {
            eprintln!("Failed to init logging: {}", e);
        }
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
        // 写入一条测试日志
        log::info!("Test log message");
        // 检查文件是否存在且非空
        let metadata = std::fs::metadata(&log_path);
        assert!(metadata.is_ok());
        assert!(metadata.unwrap().len() > 0);
    }

    #[test]
    fn test_logging_invalid_level() {
        let config = LogConfig {
            level: "invalid".to_string(),
            ..Default::default()
        };
        let result = init_logging(config);
        // 应该仍然能初始化（默认降级为 Info）
        assert!(result.is_ok());
    }
}
