//! 平权削减模块
//! 
//! 检测并执行平权削减，防止财富过度集中

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::info;

use toki_core::TOKI_BASE_UNIT;

/// 平权阈值配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EqualizationThreshold {
    /// 余额阈值（toki）
    pub balance_threshold: u64,
    /// 账户数量阈值
    pub account_count_threshold: u64,
    /// 削减比例
    pub reduction_rate: f64,
}

/// 平权配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EqualizationConfig {
    /// 阈值列表（按余额升序）
    pub thresholds: Vec<EqualizationThreshold>,
    /// 检查周期（区块数）
    pub check_interval: u64,
    /// 是否启用
    pub enabled: bool,
}

impl Default for EqualizationConfig {
    fn default() -> Self {
        EqualizationConfig {
            thresholds: vec![
                EqualizationThreshold {
                    balance_threshold: 10_000_000,
                    account_count_threshold: 2_000_000,
                    reduction_rate: 0.2,
                },
                EqualizationThreshold {
                    balance_threshold: 100_000_000,
                    account_count_threshold: 200_000,
                    reduction_rate: 0.2,
                },
                EqualizationThreshold {
                    balance_threshold: 200_000_000,
                    account_count_threshold: 100_000,
                    reduction_rate: 0.2,
                },
                EqualizationThreshold {
                    balance_threshold: 500_000_000,
                    account_count_threshold: 20_000,
                    reduction_rate: 0.2,
                },
                EqualizationThreshold {
                    balance_threshold: 1_000_000_000,
                    account_count_threshold: 10_000,
                    reduction_rate: 0.2,
                },
                EqualizationThreshold {
                    balance_threshold: 1_500_000_000,
                    account_count_threshold: 10_000,
                    reduction_rate: 0.2,
                },
            ],
            check_interval: 1000,
            enabled: true,
        }
    }
}

/// 平权检测器
pub struct EqualizationDetector {
    config: EqualizationConfig,
}

impl EqualizationDetector {
    pub fn new(config: EqualizationConfig) -> Self {
        EqualizationDetector { config }
    }

    /// 检查是否需要执行平权
    pub fn check_equalization(&self, balance_stats: &BalanceStats) -> Option<EqualizationAction> {
        if !self.config.enabled {
            return None;
        }

        for threshold in &self.config.thresholds {
            let threshold_amount = threshold.balance_threshold * TOKI_BASE_UNIT;
            
            // 统计超过阈值的账户数量
            let count = balance_stats.count_above_threshold(threshold_amount);
            
            if count >= threshold.account_count_threshold as usize {
                info!(
                    "触发平权: 余额>={} toki 的账户数 {} >= {}",
                    threshold.balance_threshold,
                    count,
                    threshold.account_count_threshold
                );
                
                return Some(EqualizationAction {
                    balance_threshold: threshold.balance_threshold,
                    affected_accounts: count,
                    reduction_rate: threshold.reduction_rate,
                });
            }
        }

        None
    }

    /// 计算削减金额
    pub fn calculate_reduction(&self, balance: u64, rate: f64) -> u64 {
        (balance as f64 * rate) as u64
    }
}

impl Default for EqualizationDetector {
    fn default() -> Self {
        Self::new(EqualizationConfig::default())
    }
}

/// 余额统计
#[derive(Debug, Clone)]
pub struct BalanceStats {
    /// 按余额区间统计
    buckets: HashMap<u64, usize>,
}

impl BalanceStats {
    pub fn new() -> Self {
        BalanceStats {
            buckets: HashMap::new(),
        }
    }

    /// 添加账户
    pub fn add_account(&mut self, balance: u64) {
        // 找到对应的桶
        let bucket = Self::get_bucket(balance);
        *self.buckets.entry(bucket).or_insert(0) += 1;
    }

    /// 统计超过阈值的账户数
    pub fn count_above_threshold(&self, threshold: u64) -> usize {
        self.buckets.iter()
            .filter(|(&bucket, _)| bucket >= threshold)
            .map(|(_, &count)| count)
            .sum()
    }

    /// 获取桶边界
    fn get_bucket(balance: u64) -> u64 {
        // 使用 1000 万 toki 为单位
        let unit = 10_000_000 * TOKI_BASE_UNIT;
        (balance / unit) * unit
    }
}

impl Default for BalanceStats {
    fn default() -> Self {
        Self::new()
    }
}

/// 平权动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqualizationAction {
    pub balance_threshold: u64,
    pub affected_accounts: usize,
    pub reduction_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equalization_config() {
        let config = EqualizationConfig::default();
        assert!(config.enabled);
        assert_eq!(config.thresholds.len(), 6);
    }

    #[test]
    fn test_balance_stats() {
        let mut stats = BalanceStats::new();
        
        stats.add_account(5_000_000 * TOKI_BASE_UNIT);
        stats.add_account(15_000_000 * TOKI_BASE_UNIT);
        stats.add_account(25_000_000 * TOKI_BASE_UNIT);
        
        let count = stats.count_above_threshold(10_000_000 * TOKI_BASE_UNIT);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_reduction_calculation() {
        let detector = EqualizationDetector::default();
        let balance = 100_000_000 * TOKI_BASE_UNIT;
        let reduction = detector.calculate_reduction(balance, 0.2);
        
        assert_eq!(reduction, 20_000_000 * TOKI_BASE_UNIT);
    }
}
