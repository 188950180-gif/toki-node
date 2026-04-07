//! AI 经济聚合器
//!
//! 实现区块链自主运行的核心功能
//! - 账户聚合
//! - 收益分配
//! - 平权削减
//! - 公益执行

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use anyhow::Result;
use tracing::{info, warn, debug};

use toki_core::{Address, Hash, Block, TOKI_BASE_UNIT};

/// AI 经济配置
#[derive(Clone, Debug)]
pub struct AIConfig {
    /// 聚合比例（0-1）
    pub aggregate_ratio: f64,
    /// 平权阈值
    pub equalize_threshold: u64,
    /// 公益比例（0-1）
    pub charity_ratio: f64,
    /// 公益地址
    pub charity_address: Address,
}

impl Default for AIConfig {
    fn default() -> Self {
        AIConfig {
            aggregate_ratio: 0.8,
            equalize_threshold: 1_000_000 * TOKI_BASE_UNIT,
            charity_ratio: 0.05,
            charity_address: Address::new([1u8; 32]),
        }
    }
}

/// AI 经济聚合器
pub struct AIAggregator {
    config: AIConfig,
    /// 账户余额
    balances: Arc<RwLock<HashMap<Address, u64>>>,
    /// 聚合账户
    aggregate_account: Address,
}

impl AIAggregator {
    /// 创建新的 AI 聚合器
    pub fn new(config: AIConfig) -> Self {
        let aggregate_account = Address::new([2u8; 32]); // AI 聚合账户
        
        info!("创建 AI 聚合器");
        info!("聚合比例: {}", config.aggregate_ratio);
        info!("平权阈值: {}", config.equalize_threshold);
        info!("公益比例: {}", config.charity_ratio);
        
        AIAggregator {
            config,
            balances: Arc::new(RwLock::new(HashMap::new())),
            aggregate_account,
        }
    }

    /// 处理区块
    pub fn process_block(&self, block: &Block) -> Result<()> {
        info!("处理区块: {}", block.height());
        
        // 1. 收集区块奖励
        let block_reward = self.calculate_block_reward(block);
        
        // 2. 聚合收益
        self.aggregate_rewards(block_reward)?;
        
        // 3. 平权削减
        self.equalize()?;
        
        // 4. 公益执行
        self.donate_to_charity()?;
        
        Ok(())
    }

    /// 计算区块奖励
    fn calculate_block_reward(&self, block: &Block) -> u64 {
        // 简化实现：固定奖励
        100 * TOKI_BASE_UNIT
    }

    /// 聚合收益
    fn aggregate_rewards(&self, total_reward: u64) -> Result<()> {
        let aggregate_amount = (total_reward as f64 * self.config.aggregate_ratio) as u64;
        
        let mut balances = self.balances.write();
        let current = balances.entry(self.aggregate_account.clone()).or_insert(0);
        *current += aggregate_amount;
        
        info!("聚合收益: {} -> 聚合账户", aggregate_amount);
        Ok(())
    }

    /// 平权削减
    fn equalize(&self) -> Result<()> {
        let mut balances = self.balances.write();
        let mut total_excess = 0u64;
        
        // 计算超额部分
        for (_, balance) in balances.iter_mut() {
            if *balance > self.config.equalize_threshold {
                let excess = *balance - self.config.equalize_threshold;
                *balance = self.config.equalize_threshold;
                total_excess += excess;
            }
        }
        
        // 重新分配
        if total_excess > 0 {
            self.redistribute(&mut balances, total_excess)?;
        }
        
        Ok(())
    }

    /// 重新分配
    fn redistribute(&self, balances: &mut HashMap<Address, u64>, total: u64) -> Result<()> {
        let count = balances.len();
        if count == 0 {
            return Ok(());
        }
        
        let per_account = total / count as u64;
        for (_, balance) in balances.iter_mut() {
            *balance += per_account;
        }
        
        info!("重新分配: {} -> {} 个账户", total, count);
        Ok(())
    }

    /// 公益执行
    fn donate_to_charity(&self) -> Result<()> {
        let mut balances = self.balances.write();
        
        // 从所有账户中提取公益金
        let total_donation = balances.values().sum::<u64>() as f64 * self.config.charity_ratio;
        let donation = total_donation as u64;
        
        if donation > 0 {
            let charity_balance = balances.entry(self.config.charity_address.clone()).or_insert(0);
            *charity_balance += donation;
            
            info!("公益捐赠: {} -> 公益账户", donation);
        }
        
        Ok(())
    }

    /// 获取账户余额
    pub fn get_balance(&self, address: &Address) -> u64 {
        let balances = self.balances.read();
        *balances.get(address).unwrap_or(&0)
    }

    /// 获取聚合账户余额
    pub fn get_aggregate_balance(&self) -> u64 {
        self.get_balance(&self.aggregate_account)
    }

    /// 获取公益账户余额
    pub fn get_charity_balance(&self) -> u64 {
        self.get_balance(&self.config.charity_address)
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> AggregatorStats {
        let balances = self.balances.read();
        
        AggregatorStats {
            total_accounts: balances.len(),
            aggregate_balance: self.get_aggregate_balance(),
            charity_balance: self.get_charity_balance(),
            total_balance: balances.values().sum(),
        }
    }
}

/// 聚合器统计信息
#[derive(Debug, Clone)]
pub struct AggregatorStats {
    /// 总账户数
    pub total_accounts: usize,
    /// 聚合账户余额
    pub aggregate_balance: u64,
    /// 公益账户余额
    pub charity_balance: u64,
    /// 总余额
    pub total_balance: u64,
}

impl Default for AIAggregator {
    fn default() -> Self {
        Self::new(AIConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregator_creation() {
        let aggregator = AIAggregator::default();
        assert_eq!(aggregator.get_aggregate_balance(), 0);
    }

    #[test]
    fn test_process_block() {
        let aggregator = AIAggregator::default();
        let block = Block::genesis();
        
        let result = aggregator.process_block(&block);
        assert!(result.is_ok());
        assert!(aggregator.get_aggregate_balance() > 0);
    }

    #[test]
    fn test_get_balance() {
        let aggregator = AIAggregator::default();
        let address = Address::new([1u8; 32]);
        
        let balance = aggregator.get_balance(&address);
        assert_eq!(balance, 0);
    }

    #[test]
    fn test_get_stats() {
        let aggregator = AIAggregator::default();
        let stats = aggregator.get_stats();
        
        assert_eq!(stats.total_accounts, 0);
        assert_eq!(stats.total_balance, 0);
    }
}
