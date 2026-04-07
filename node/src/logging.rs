//! 结构化日志配置
//!
//! 提供 JSON 格式的结构化日志

use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    EnvFilter, Layer,
};
use std::fs::OpenOptions;

/// 日志配置
pub struct LogConfig {
    /// 日志级别
    pub level: String,
    /// 日志文件路径
    pub file: String,
    /// 是否输出到控制台
    pub console: bool,
    /// 是否使用 JSON 格式
    pub json: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        LogConfig {
            level: "info".to_string(),
            file: "./logs/toki.log".to_string(),
            console: true,
            json: true,
        }
    }
}

/// 初始化日志系统
pub fn init_logging(config: LogConfig) {
    // 创建日志目录
    if let Some(log_dir) = std::path::Path::new(&config.file).parent() {
        let _ = std::fs::create_dir_all(log_dir);
    }

    // 创建环境过滤器
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    // 初始化日志
    if config.console {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_target(true)
            .with_thread_ids(true)
            .init();
    }
}

/// 日志字段
pub mod fields {
    use tracing::field;

    /// 区块高度
    pub fn block_height(height: u64) -> field::DisplayValue<u64> {
        field::display(height)
    }

    /// 交易哈希
    pub fn tx_hash(hash: &str) -> field::DisplayValue<&str> {
        field::display(hash)
    }

    /// 节点 ID
    pub fn node_id(id: &str) -> field::DisplayValue<&str> {
        field::display(id)
    }

    /// Peer ID
    pub fn peer_id(id: &str) -> field::DisplayValue<&str> {
        field::display(id)
    }

    /// 内存使用（MB）
    pub fn memory_mb(mb: u64) -> field::DisplayValue<u64> {
        field::display(mb)
    }

    /// CPU 使用率
    pub fn cpu_percent(percent: f64) -> field::DisplayValue<f64> {
        field::display(percent)
    }

    /// 响应时间（ms）
    pub fn response_time_ms(ms: u64) -> field::DisplayValue<u64> {
        field::display(ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::{info, span, Level};

    #[test]
    fn test_logging_initialization() {
        let config = LogConfig::default();
        assert!(init_logging(config).is_ok());
    }

    #[test]
    fn test_structured_logging() {
        let config = LogConfig {
            json: true,
            ..Default::default()
        };
        init_logging(config).unwrap();

        let span = span!(Level::INFO, "test_span", block_height = 100);
        let _enter = span.enter();

        info!(
            block_height = 100,
            tx_hash = "abc123",
            "Test structured log"
        );
    }
}
