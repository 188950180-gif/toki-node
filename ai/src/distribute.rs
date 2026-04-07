//! 自动分配模块
//!
//! 实现基础赠送、集体账户分配、国家账户分配

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;

use toki_core::{AccountType, Region, TOKI_BASE_UNIT};

/// 分配配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DistributionConfig {
    /// 人均基准额度（toki）
    pub per_capita_amount: u64,
    /// 基础赠送解锁天数
    pub unlock_days: u64,
    /// 每日解锁比例
    pub daily_unlock_rate: f64,
}

impl Default for DistributionConfig {
    fn default() -> Self {
        DistributionConfig {
            per_capita_amount: 100_000,
            unlock_days: 365,
            daily_unlock_rate: 1.0 / 365.0,
        }
    }
}

/// 分配器
pub struct Distributor {
    config: DistributionConfig,
}

impl Distributor {
    pub fn new(config: DistributionConfig) -> Self {
        Distributor { config }
    }

    /// 计算基础赠送金额
    pub fn calculate_basic_grant(&self, account_type: AccountType) -> u64 {
        match account_type {
            AccountType::Personal => self.config.per_capita_amount * TOKI_BASE_UNIT,
            AccountType::Collective => self.config.per_capita_amount * TOKI_BASE_UNIT * 10,
            AccountType::Nation => self.config.per_capita_amount * TOKI_BASE_UNIT * 100,
            _ => 0,
        }
    }

    /// 计算解锁金额
    pub fn calculate_unlock_amount(&self, total: u64, elapsed_days: u64) -> u64 {
        if elapsed_days >= self.config.unlock_days {
            return total;
        }

        let unlock_ratio = elapsed_days as f64 * self.config.daily_unlock_rate;
        (total as f64 * unlock_ratio) as u64
    }

    /// 执行区域分配
    pub fn distribute_by_region(&self, region: Region, population: u64) -> DistributionResult {
        let amount_per_person = self.config.per_capita_amount * TOKI_BASE_UNIT;
        let total = population.saturating_mul(amount_per_person);

        info!(
            "区域分配: {:?} 人口={} 总额={} toki",
            region,
            population,
            total / TOKI_BASE_UNIT
        );

        DistributionResult {
            region,
            population,
            total_amount: total,
            amount_per_person,
        }
    }
}

impl Default for Distributor {
    fn default() -> Self {
        Self::new(DistributionConfig::default())
    }
}

/// 分配结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionResult {
    pub region: Region,
    pub population: u64,
    pub total_amount: u64,
    pub amount_per_person: u64,
}

/// 解锁计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockPlan {
    pub total_amount: u64,
    pub unlocked_amount: u64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub daily_amount: u64,
}

impl UnlockPlan {
    /// 创建新的解锁计划
    pub fn new(total: u64, days: u64) -> Self {
        let now = Utc::now();
        let end = now + chrono::Duration::days(days as i64);
        let daily = total / days;

        UnlockPlan {
            total_amount: total,
            unlocked_amount: 0,
            start_time: now,
            end_time: end,
            daily_amount: daily,
        }
    }

    /// 计算当前应解锁金额
    pub fn current_unlock(&self) -> u64 {
        let now = Utc::now();
        if now >= self.end_time {
            return self.total_amount;
        }

        let elapsed = (now - self.start_time).num_days();
        if elapsed <= 0 {
            return 0;
        }

        self.daily_amount.saturating_mul(elapsed as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_grant() {
        let distributor = Distributor::default();

        let personal = distributor.calculate_basic_grant(AccountType::Personal);
        assert_eq!(personal, 100_000 * TOKI_BASE_UNIT);

        let collective = distributor.calculate_basic_grant(AccountType::Collective);
        assert_eq!(collective, 1_000_000 * TOKI_BASE_UNIT);
    }

    #[test]
    fn test_unlock_calculation() {
        let distributor = Distributor::default();
        let total = 100_000 * TOKI_BASE_UNIT;

        // 0 天解锁
        let unlock_0 = distributor.calculate_unlock_amount(total, 0);
        assert_eq!(unlock_0, 0);

        // 365 天完全解锁
        let unlock_full = distributor.calculate_unlock_amount(total, 365);
        assert_eq!(unlock_full, total);

        // 182 天解锁一半
        let unlock_half = distributor.calculate_unlock_amount(total, 182);
        assert!(unlock_half > 0 && unlock_half < total);
    }

    #[test]
    fn test_unlock_plan() {
        let total = 365_000 * TOKI_BASE_UNIT;
        let plan = UnlockPlan::new(total, 365);

        assert_eq!(plan.daily_amount, 1_000 * TOKI_BASE_UNIT);
    }
}
