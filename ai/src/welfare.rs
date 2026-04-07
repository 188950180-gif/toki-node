//! 低保系统
//!
//! 区块链自主运行核心功能
//! - 低保资格管理
//! - 低保发放
//! - 资格审核

use anyhow::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

use toki_core::{Address, TOKI_BASE_UNIT};

/// 低保配置
#[derive(Clone, Debug)]
pub struct WelfareConfig {
    /// 低保金额
    pub welfare_amount: u64,
    /// 发放周期（区块数）
    pub distribution_interval: u64,
    /// 资格审核周期（区块数）
    pub review_interval: u64,
    /// 最低余额要求
    pub min_balance_threshold: u64,
}

impl Default for WelfareConfig {
    fn default() -> Self {
        WelfareConfig {
            welfare_amount: 10 * TOKI_BASE_UNIT,
            distribution_interval: 100,
            review_interval: 1000,
            min_balance_threshold: 100 * TOKI_BASE_UNIT,
        }
    }
}

/// 低保申请
#[derive(Debug, Clone)]
pub struct WelfareApplication {
    /// 申请地址
    pub address: Address,
    /// 申请时间
    pub applied_at: i64,
    /// 申请理由
    pub reason: String,
    /// 审核状态
    pub status: ApplicationStatus,
}

/// 申请状态
#[derive(Debug, Clone, PartialEq)]
pub enum ApplicationStatus {
    /// 待审核
    Pending,
    /// 已批准
    Approved,
    /// 已拒绝
    Rejected,
}

/// 低保系统
pub struct WelfareSystem {
    config: WelfareConfig,
    /// 低保资金池
    welfare_pool: Arc<RwLock<u64>>,
    /// 低保申请
    applications: Arc<RwLock<HashMap<Address, WelfareApplication>>>,
    /// 低保资格
    eligible_addresses: Arc<RwLock<HashMap<Address, bool>>>,
    /// 发放记录
    distribution_records: Arc<RwLock<Vec<DistributionRecord>>>,
}

/// 发放记录
#[derive(Debug, Clone)]
pub struct DistributionRecord {
    /// 接收地址
    pub address: Address,
    /// 发放金额
    pub amount: u64,
    /// 发放区块高度
    pub block_height: u64,
    /// 发放时间
    pub timestamp: i64,
}

impl WelfareSystem {
    /// 创建新的低保系统
    pub fn new(config: WelfareConfig) -> Self {
        info!("创建低保系统");
        info!("低保金额: {}", config.welfare_amount);
        info!("发放周期: {} 区块", config.distribution_interval);

        WelfareSystem {
            config,
            welfare_pool: Arc::new(RwLock::new(0)),
            applications: Arc::new(RwLock::new(HashMap::new())),
            eligible_addresses: Arc::new(RwLock::new(HashMap::new())),
            distribution_records: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 添加低保资金
    pub fn add_funds(&self, amount: u64) -> Result<()> {
        let mut pool = self.welfare_pool.write();
        *pool += amount;
        info!("低保资金增加: {}", amount);
        Ok(())
    }

    /// 申请低保
    pub fn apply_for_welfare(&self, address: Address, reason: String) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        let application = WelfareApplication {
            address: address.clone(),
            applied_at: now,
            reason,
            status: ApplicationStatus::Pending,
        };

        let mut applications = self.applications.write();
        applications.insert(address.clone(), application);

        info!("收到低保申请: {}", address);
        Ok(())
    }

    /// 审核申请
    pub fn review_application(
        &self,
        address: &Address,
        approved: bool,
        reason: String,
    ) -> Result<()> {
        let mut applications = self.applications.write();

        if let Some(app) = applications.get_mut(address) {
            if approved {
                app.status = ApplicationStatus::Approved;

                // 添加到资格列表
                let mut eligible = self.eligible_addresses.write();
                eligible.insert(address.clone(), true);

                info!("低保申请已批准: {} (理由: {})", address, reason);
            } else {
                app.status = ApplicationStatus::Rejected;

                // 从资格列表移除
                let mut eligible = self.eligible_addresses.write();
                eligible.remove(address);

                info!("低保申请已拒绝: {} (理由: {})", address, reason);
            }
        }

        Ok(())
    }

    /// 发放低保
    pub fn distribute_welfare(&self, block_height: u64) -> Result<u64> {
        let eligible = self.eligible_addresses.read();
        let mut pool = self.welfare_pool.write();

        let eligible_count = eligible.len();
        if eligible_count == 0 {
            return Ok(0);
        }

        let amount_per_address = self.config.welfare_amount;
        let total_required = amount_per_address * eligible_count as u64;

        if *pool < total_required {
            warn!("低保资金不足: 需要 {}, 可用 {}", total_required, *pool);
            return Ok(0);
        }

        let now = chrono::Utc::now().timestamp();
        let mut distributed = 0u64;

        for (address, _) in eligible.iter() {
            if *pool >= amount_per_address {
                *pool -= amount_per_address;
                distributed += amount_per_address;

                let record = DistributionRecord {
                    address: address.clone(),
                    amount: amount_per_address,
                    block_height,
                    timestamp: now,
                };

                self.distribution_records.write().push(record);

                debug!("发放低保: {} -> {}", amount_per_address, address);
            }
        }

        info!(
            "低保发放完成: {} 地址, 总计 {}",
            eligible_count, distributed
        );
        Ok(distributed)
    }

    /// 获取低保资金池余额
    pub fn get_pool_balance(&self) -> u64 {
        *self.welfare_pool.read()
    }

    /// 获取申请信息
    pub fn get_application(&self, address: &Address) -> Option<WelfareApplication> {
        self.applications.read().get(address).cloned()
    }

    /// 获取所有申请
    pub fn get_all_applications(&self) -> Vec<WelfareApplication> {
        self.applications.read().values().cloned().collect()
    }

    /// 检查资格
    pub fn is_eligible(&self, address: &Address) -> bool {
        *self
            .eligible_addresses
            .read()
            .get(address)
            .unwrap_or(&false)
    }

    /// 获取发放记录
    pub fn get_distribution_records(&self, limit: usize) -> Vec<DistributionRecord> {
        let records = self.distribution_records.read();
        records.iter().rev().take(limit).cloned().collect()
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> WelfareStats {
        let pool_balance = self.get_pool_balance();
        let applications = self.get_all_applications();
        let eligible_count = self.eligible_addresses.read().len();
        let records = self.get_distribution_records(1000);

        let pending_count = applications
            .iter()
            .filter(|a| a.status == ApplicationStatus::Pending)
            .count();

        let approved_count = applications
            .iter()
            .filter(|a| a.status == ApplicationStatus::Approved)
            .count();

        let total_distributed = records.iter().map(|r| r.amount).sum();

        WelfareStats {
            pool_balance,
            application_count: applications.len(),
            pending_count,
            approved_count,
            eligible_count,
            total_distributed,
        }
    }
}

/// 低保统计信息
#[derive(Debug, Clone)]
pub struct WelfareStats {
    /// 低保资金池余额
    pub pool_balance: u64,
    /// 申请总数
    pub application_count: usize,
    /// 待审核申请数
    pub pending_count: usize,
    /// 已批准申请数
    pub approved_count: usize,
    /// 符合资格地址数
    pub eligible_count: usize,
    /// 总发放金额
    pub total_distributed: u64,
}

impl Default for WelfareSystem {
    fn default() -> Self {
        Self::new(WelfareConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_welfare_system_creation() {
        let welfare = WelfareSystem::default();
        assert_eq!(welfare.get_pool_balance(), 0);
    }

    #[test]
    fn test_add_funds() {
        let welfare = WelfareSystem::default();
        welfare.add_funds(1000 * TOKI_BASE_UNIT).unwrap();

        assert_eq!(welfare.get_pool_balance(), 1000 * TOKI_BASE_UNIT);
    }

    #[test]
    fn test_apply_for_welfare() {
        let welfare = WelfareSystem::default();
        let address = Address::new([1u8; 32]);

        welfare
            .apply_for_welfare(address.clone(), "Need help".to_string())
            .unwrap();
        assert_eq!(welfare.get_all_applications().len(), 1);
    }

    #[test]
    fn test_review_application() {
        let welfare = WelfareSystem::default();
        let address = Address::new([1u8; 32]);

        welfare
            .apply_for_welfare(address.clone(), "Need help".to_string())
            .unwrap();
        welfare
            .review_application(&address, true, "Valid".to_string())
            .unwrap();

        assert!(welfare.is_eligible(&address));
    }

    #[test]
    fn test_distribute_welfare() {
        let welfare = WelfareSystem::default();
        let address = Address::new([1u8; 32]);

        welfare.add_funds(1000 * TOKI_BASE_UNIT).unwrap();
        welfare
            .apply_for_welfare(address.clone(), "Need help".to_string())
            .unwrap();
        welfare
            .review_application(&address, true, "Valid".to_string())
            .unwrap();

        let distributed = welfare.distribute_welfare(100).unwrap();
        assert_eq!(distributed, 10 * TOKI_BASE_UNIT);
    }

    #[test]
    fn test_get_stats() {
        let welfare = WelfareSystem::default();
        let stats = welfare.get_stats();

        assert_eq!(stats.pool_balance, 0);
        assert_eq!(stats.application_count, 0);
        assert_eq!(stats.eligible_count, 0);
    }
}
