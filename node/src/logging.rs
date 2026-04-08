//! 极简日志配置（仅控制台输出，确保编译通过）

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::metadata::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogConfig {
    pub level: String,
    pub file: Option<String>,
    pub console: bool,
    pub json: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file: None,
            console: true,
            json: false,
        }
    }
}

pub fn init_logging(config: LogConfig) -> Result<()> {
    let level = match config.level.as_str() {
        "debug" => LevelFilter::DEBUG,
        "warn" => LevelFilter::WARN,
        "error" => LevelFilter::ERROR,
        _ => LevelFilter::INFO,
    };
    let env_filter = EnvFilter::from_default_env().add_directive(level.into());

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_file(true)
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let config = LogConfig::default();
        assert!(init_logging(config).is_ok());
    }
}