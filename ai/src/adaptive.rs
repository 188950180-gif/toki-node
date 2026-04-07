//! AI 自适应控制器
//!
//! 根据网络状态自动调整系统参数

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tracing::info;

/// 网络指标
#[derive(Clone, Debug, Default)]
pub struct NetworkMetrics {
    /// 交易数量
    pub tx_count: u64,
    /// 容量
    pub capacity: u64,
    /// 平均出块时间
    pub avg_block_time: u64,
    /// 连接节点数
    pub peer_count: usize,
    /// 内存使用率
    pub memory_usage: f64,
    /// CPU 使用率
    pub cpu_usage: f64,
}

/// 自适应参数
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdaptiveParams {
    /// 目标出块时间（秒）
    pub target_block_time: u64,
    /// 基础交易费
    pub base_tx_fee: u64,
    /// 平权触发阈值
    pub equalization_threshold: u64,
    /// 派发速率
    pub distribution_rate: f64,
    /// 最大交易池大小
    pub max_tx_pool_size: u64,
    /// 最大连接数
    pub max_connections: usize,
}

impl Default for AdaptiveParams {
    fn default() -> Self {
        AdaptiveParams {
            target_block_time: 10,
            base_tx_fee: 1000,
            equalization_threshold: 10_000_000,
            distribution_rate: 1.0,
            max_tx_pool_size: 100_000,
            max_connections: 100,
        }
    }
}

/// AI 自适应控制器
pub struct AdaptiveController {
    /// 历史数据窗口
    history_window: VecDeque<NetworkMetrics>,
    /// 窗口大小
    window_size: usize,
    /// 学习率
    learning_rate: f64,
    /// 当前参数
    current_params: AdaptiveParams,
    /// 调整计数
    adjustment_count: u64,
}

impl AdaptiveController {
    /// 创建新的自适应控制器
    pub fn new() -> Self {
        AdaptiveController {
            history_window: VecDeque::with_capacity(100),
            window_size: 100,
            learning_rate: 0.1,
            current_params: AdaptiveParams::default(),
            adjustment_count: 0,
        }
    }

    /// 使用指定参数创建
    pub fn with_params(params: AdaptiveParams) -> Self {
        AdaptiveController {
            history_window: VecDeque::with_capacity(100),
            window_size: 100,
            learning_rate: 0.1,
            current_params: params,
            adjustment_count: 0,
        }
    }

    /// 获取当前参数
    pub fn current_params(&self) -> &AdaptiveParams {
        &self.current_params
    }

    /// 记录指标
    pub fn record_metrics(&mut self, metrics: NetworkMetrics) {
        if self.history_window.len() >= self.window_size {
            self.history_window.pop_front();
        }
        self.history_window.push_back(metrics);
    }

    /// 根据网络状态自动调整参数
    pub fn adjust(&mut self, metrics: &NetworkMetrics) -> AdaptiveParams {
        self.record_metrics(metrics.clone());

        // 分析网络负载
        let load_factor = if metrics.capacity > 0 {
            metrics.tx_count as f64 / metrics.capacity as f64
        } else {
            0.0
        };

        // 动态调整交易费
        if load_factor > 0.8 {
            // 高负载，提高交易费
            self.current_params.base_tx_fee = (self.current_params.base_tx_fee as f64 * 1.1) as u64;
            info!("高负载，提高交易费至 {}", self.current_params.base_tx_fee);
        } else if load_factor < 0.3 {
            // 低负载，降低交易费
            self.current_params.base_tx_fee =
                (self.current_params.base_tx_fee as f64 * 0.95).max(100.0) as u64;
            info!("低负载，降低交易费至 {}", self.current_params.base_tx_fee);
        }

        // 动态调整出块时间
        if metrics.avg_block_time > self.current_params.target_block_time + 3 {
            // 出块太慢，降低难度
            if self.current_params.target_block_time > 5 {
                self.current_params.target_block_time -= 1;
                info!(
                    "出块慢，调整目标时间为 {} 秒",
                    self.current_params.target_block_time
                );
            }
        } else if metrics.avg_block_time < self.current_params.target_block_time - 3 {
            // 出块太快，提高难度
            if self.current_params.target_block_time < 20 {
                self.current_params.target_block_time += 1;
                info!(
                    "出块快，调整目标时间为 {} 秒",
                    self.current_params.target_block_time
                );
            }
        }

        // 动态调整连接数
        if metrics.peer_count < 5 {
            // 连接太少，增加最大连接数
            self.current_params.max_connections += 10;
            info!(
                "连接少，增加最大连接数至 {}",
                self.current_params.max_connections
            );
        }

        // 动态调整交易池大小
        if metrics.memory_usage > 0.8 {
            // 内存紧张，减小交易池
            self.current_params.max_tx_pool_size =
                (self.current_params.max_tx_pool_size as f64 * 0.9) as u64;
            info!(
                "内存紧张，减小交易池至 {}",
                self.current_params.max_tx_pool_size
            );
        }

        self.adjustment_count += 1;
        self.current_params.clone()
    }

    /// 分析趋势
    pub fn analyze_trend(&self) -> SystemTrend {
        if self.history_window.len() < 10 {
            return SystemTrend::Stable;
        }

        let recent: Vec<_> = self.history_window.iter().rev().take(10).collect();

        let older: Vec<_> = self.history_window.iter().rev().skip(10).take(10).collect();

        if older.is_empty() {
            return SystemTrend::Stable;
        }

        let recent_load: f64 = recent
            .iter()
            .map(|m| {
                if m.capacity > 0 {
                    m.tx_count as f64 / m.capacity as f64
                } else {
                    0.0
                }
            })
            .sum::<f64>()
            / recent.len() as f64;

        let older_load: f64 = older
            .iter()
            .map(|m| {
                if m.capacity > 0 {
                    m.tx_count as f64 / m.capacity as f64
                } else {
                    0.0
                }
            })
            .sum::<f64>()
            / older.len() as f64;

        if recent_load > older_load * 1.2 {
            SystemTrend::Increasing
        } else if recent_load < older_load * 0.8 {
            SystemTrend::Decreasing
        } else {
            SystemTrend::Stable
        }
    }

    /// 获取调整次数
    pub fn adjustment_count(&self) -> u64 {
        self.adjustment_count
    }
}

impl Default for AdaptiveController {
    fn default() -> Self {
        Self::new()
    }
}

/// 系统趋势
#[derive(Debug, Clone, PartialEq)]
pub enum SystemTrend {
    /// 增长
    Increasing,
    /// 稳定
    Stable,
    /// 下降
    Decreasing,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_controller_creation() {
        let controller = AdaptiveController::new();
        assert_eq!(controller.current_params().target_block_time, 10);
    }

    #[test]
    fn test_adjust_high_load() {
        let mut controller = AdaptiveController::new();
        let metrics = NetworkMetrics {
            tx_count: 900,
            capacity: 1000,
            avg_block_time: 10,
            peer_count: 10,
            memory_usage: 0.5,
            cpu_usage: 0.5,
        };

        let params = controller.adjust(&metrics);
        // 高负载应该提高交易费
        assert!(params.base_tx_fee >= 1000);
    }

    #[test]
    fn test_adjust_slow_blocks() {
        let mut controller = AdaptiveController::new();
        let metrics = NetworkMetrics {
            tx_count: 100,
            capacity: 1000,
            avg_block_time: 15, // 比目标慢
            peer_count: 10,
            memory_usage: 0.5,
            cpu_usage: 0.5,
        };

        let params = controller.adjust(&metrics);
        // 出块慢应该降低目标时间
        assert!(params.target_block_time <= 10);
    }

    #[test]
    fn test_trend_analysis() {
        let mut controller = AdaptiveController::new();

        // 添加历史数据
        for i in 0..20 {
            let metrics = NetworkMetrics {
                tx_count: 100 + i as u64 * 10,
                capacity: 1000,
                avg_block_time: 10,
                peer_count: 10,
                memory_usage: 0.5,
                cpu_usage: 0.5,
            };
            controller.record_metrics(metrics);
        }

        let trend = controller.analyze_trend();
        assert_eq!(trend, SystemTrend::Increasing);
    }
}
