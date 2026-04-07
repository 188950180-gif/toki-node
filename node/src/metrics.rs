//! Prometheus 监控模块
//!
//! 实现运维监控指标导出

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

/// 指标类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetricType {
    /// 计数器
    Counter,
    /// 仪表盘
    Gauge,
    /// 直方图
    Histogram,
    /// 摘要
    Summary,
}

/// 指标值
#[derive(Debug, Clone)]
pub enum MetricValue {
    /// 单值
    Single(f64),
    /// 直方图
    Histogram {
        buckets: Vec<(f64, u64)>,
        sum: f64,
        count: u64,
    },
}

/// 指标
#[derive(Debug, Clone)]
pub struct Metric {
    /// 名称
    pub name: String,
    /// 帮助文本
    pub help: String,
    /// 类型
    pub metric_type: MetricType,
    /// 标签
    pub labels: HashMap<String, String>,
    /// 值
    pub value: MetricValue,
    /// 更新时间
    pub updated_at: Instant,
}

/// Prometheus 监控器
pub struct PrometheusMetrics {
    /// 指标集合
    metrics: HashMap<String, Metric>,
    /// 命名空间
    namespace: String,
    /// 子系统
    subsystem: String,
}

impl PrometheusMetrics {
    /// 创建新的监控器
    pub fn new(namespace: &str, subsystem: &str) -> Self {
        PrometheusMetrics {
            metrics: HashMap::new(),
            namespace: namespace.to_string(),
            subsystem: subsystem.to_string(),
        }
    }

    /// 注册计数器
    pub fn register_counter(&mut self, name: &str, help: &str) {
        let full_name = self.full_name(name);
        self.metrics.insert(
            full_name.clone(),
            Metric {
                name: full_name,
                help: help.to_string(),
                metric_type: MetricType::Counter,
                labels: HashMap::new(),
                value: MetricValue::Single(0.0),
                updated_at: Instant::now(),
            },
        );
    }

    /// 注册仪表盘
    pub fn register_gauge(&mut self, name: &str, help: &str) {
        let full_name = self.full_name(name);
        self.metrics.insert(
            full_name.clone(),
            Metric {
                name: full_name,
                help: help.to_string(),
                metric_type: MetricType::Gauge,
                labels: HashMap::new(),
                value: MetricValue::Single(0.0),
                updated_at: Instant::now(),
            },
        );
    }

    /// 注册直方图
    pub fn register_histogram(&mut self, name: &str, help: &str, buckets: Vec<f64>) {
        let full_name = self.full_name(name);
        let bucket_values: Vec<(f64, u64)> = buckets.into_iter().map(|b| (b, 0)).collect();

        self.metrics.insert(
            full_name.clone(),
            Metric {
                name: full_name,
                help: help.to_string(),
                metric_type: MetricType::Histogram,
                labels: HashMap::new(),
                value: MetricValue::Histogram {
                    buckets: bucket_values,
                    sum: 0.0,
                    count: 0,
                },
                updated_at: Instant::now(),
            },
        );
    }

    /// 递增计数器
    pub fn inc_counter(&mut self, name: &str) {
        let full_name = self.full_name(name);
        if let Some(metric) = self.metrics.get_mut(&full_name) {
            if let MetricValue::Single(ref mut value) = metric.value {
                *value += 1.0;
                metric.updated_at = Instant::now();
            }
        }
    }

    /// 增加计数器
    pub fn add_counter(&mut self, name: &str, delta: f64) {
        let full_name = self.full_name(name);
        if let Some(metric) = self.metrics.get_mut(&full_name) {
            if let MetricValue::Single(ref mut value) = metric.value {
                *value += delta;
                metric.updated_at = Instant::now();
            }
        }
    }

    /// 设置仪表盘
    pub fn set_gauge(&mut self, name: &str, value: f64) {
        let full_name = self.full_name(name);
        if let Some(metric) = self.metrics.get_mut(&full_name) {
            if let MetricValue::Single(ref mut v) = metric.value {
                *v = value;
                metric.updated_at = Instant::now();
            }
        }
    }

    /// 观察直方图
    pub fn observe_histogram(&mut self, name: &str, value: f64) {
        let full_name = self.full_name(name);
        if let Some(metric) = self.metrics.get_mut(&full_name) {
            if let MetricValue::Histogram {
                ref mut buckets,
                ref mut sum,
                ref mut count,
            } = metric.value
            {
                *sum += value;
                *count += 1;

                for (bucket, bucket_count) in buckets.iter_mut() {
                    if value <= *bucket {
                        *bucket_count += 1;
                    }
                }

                metric.updated_at = Instant::now();
            }
        }
    }

    /// 导出 Prometheus 格式
    pub fn export(&self) -> String {
        let mut output = String::new();

        for metric in self.metrics.values() {
            // 输出帮助文本
            output.push_str(&format!("# HELP {} {}\n", metric.name, metric.help));

            // 输出类型
            let type_str = match metric.metric_type {
                MetricType::Counter => "counter",
                MetricType::Gauge => "gauge",
                MetricType::Histogram => "histogram",
                MetricType::Summary => "summary",
            };
            output.push_str(&format!("# TYPE {} {}\n", metric.name, type_str));

            // 输出值
            match &metric.value {
                MetricValue::Single(value) => {
                    output.push_str(&format!("{} {}\n", metric.name, value));
                }
                MetricValue::Histogram {
                    buckets,
                    sum,
                    count,
                } => {
                    // 输出桶
                    for (bucket, bucket_count) in buckets {
                        output.push_str(&format!(
                            "{}_bucket{{le=\"{}\"}} {}\n",
                            metric.name, bucket, bucket_count
                        ));
                    }
                    output.push_str(&format!(
                        "{}_bucket{{le=\"+Inf\"}} {}\n",
                        metric.name, count
                    ));
                    output.push_str(&format!("{}_sum {}\n", metric.name, sum));
                    output.push_str(&format!("{}_count {}\n", metric.name, count));
                }
            }

            output.push('\n');
        }

        output
    }

    /// 获取完整名称
    fn full_name(&self, name: &str) -> String {
        format!("{}_{}_{}", self.namespace, self.subsystem, name)
    }

    /// 获取指标
    pub fn get(&self, name: &str) -> Option<&Metric> {
        let full_name = self.full_name(name);
        self.metrics.get(&full_name)
    }
}

/// Toki 节点监控指标
pub struct NodeMetrics {
    /// 内部监控器
    metrics: PrometheusMetrics,
}

impl NodeMetrics {
    /// 创建节点监控
    pub fn new() -> Self {
        let mut metrics = PrometheusMetrics::new("toki", "node");

        // 注册基础指标
        metrics.register_counter("blocks_total", "Total number of blocks processed");
        metrics.register_counter(
            "transactions_total",
            "Total number of transactions processed",
        );
        metrics.register_counter("blockchain_height", "Current blockchain height");
        metrics.register_gauge("peers_connected", "Number of connected peers");
        metrics.register_gauge("mining_threads", "Number of mining threads");
        metrics.register_gauge("memory_usage_bytes", "Memory usage in bytes");
        metrics.register_gauge("cpu_usage_percent", "CPU usage percentage");
        metrics.register_gauge("disk_usage_bytes", "Disk usage in bytes");
        metrics.register_histogram(
            "block_time_seconds",
            "Block time in seconds",
            vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0],
        );
        metrics.register_histogram(
            "transaction_size_bytes",
            "Transaction size in bytes",
            vec![100.0, 500.0, 1000.0, 5000.0, 10000.0, 50000.0],
        );

        NodeMetrics { metrics }
    }

    /// 记录新区块
    pub fn record_block(&mut self, block_time: f64) {
        self.metrics.inc_counter("blocks_total");
        self.metrics
            .observe_histogram("block_time_seconds", block_time);
    }

    /// 记录新交易
    pub fn record_transaction(&mut self, tx_size: f64) {
        self.metrics.inc_counter("transactions_total");
        self.metrics
            .observe_histogram("transaction_size_bytes", tx_size);
    }

    /// 更新区块高度
    pub fn update_height(&mut self, height: u64) {
        self.metrics.set_gauge("blockchain_height", height as f64);
    }

    /// 更新连接数
    pub fn update_peers(&mut self, count: usize) {
        self.metrics.set_gauge("peers_connected", count as f64);
    }

    /// 更新挖矿线程数
    pub fn update_mining_threads(&mut self, count: usize) {
        self.metrics.set_gauge("mining_threads", count as f64);
    }

    /// 更新资源使用
    pub fn update_resources(&mut self, memory: u64, cpu: f64, disk: u64) {
        self.metrics.set_gauge("memory_usage_bytes", memory as f64);
        self.metrics.set_gauge("cpu_usage_percent", cpu);
        self.metrics.set_gauge("disk_usage_bytes", disk as f64);
    }

    /// 导出指标
    pub fn export(&self) -> String {
        self.metrics.export()
    }
}

impl Default for NodeMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = PrometheusMetrics::new("toki", "node");
        assert!(metrics.metrics.is_empty());
    }

    #[test]
    fn test_counter() {
        let mut metrics = PrometheusMetrics::new("toki", "node");
        metrics.register_counter("test_counter", "Test counter");
        metrics.inc_counter("test_counter");
        metrics.inc_counter("test_counter");

        let metric = metrics.get("test_counter").unwrap();
        if let MetricValue::Single(value) = metric.value {
            assert_eq!(value, 2.0);
        }
    }

    #[test]
    fn test_gauge() {
        let mut metrics = PrometheusMetrics::new("toki", "node");
        metrics.register_gauge("test_gauge", "Test gauge");
        metrics.set_gauge("test_gauge", 42.0);

        let metric = metrics.get("test_gauge").unwrap();
        if let MetricValue::Single(value) = metric.value {
            assert_eq!(value, 42.0);
        }
    }

    #[test]
    fn test_node_metrics() {
        let mut metrics = NodeMetrics::new();
        metrics.record_block(10.5);
        metrics.record_transaction(500.0);
        metrics.update_height(100);
        metrics.update_peers(10);

        let export = metrics.export();
        assert!(export.contains("toki_node_blocks_total"));
        assert!(export.contains("toki_node_transactions_total"));
    }
}
