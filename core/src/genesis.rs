//! 创世区块配置
//!
//! 定义 Toki 区块链的创世区块参数

use crate::{constants::*, Address, Hash};
use serde::{Deserialize, Serialize};

/// 创世区块配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenesisConfig {
    /// 链 ID
    pub chain_id: String,
    /// 创世时间
    pub genesis_time: i64,
    /// 初始难度
    pub initial_difficulty: u64,
    /// 分配池初始余额
    pub distribution_pool_balance: u64,
    /// 储备池初始余额
    pub reserve_pool_balance: u64,
    /// AI 聚合账户地址
    pub ai_aggregate_address: Address,
    /// 开发者账户地址
    pub developer_address: Address,
    /// 初始账户分配
    pub initial_allocations: Vec<InitialAllocation>,
}

/// 初始账户分配
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitialAllocation {
    /// 账户地址
    pub address: Address,
    /// 余额
    pub balance: u64,
    /// 是否锁定
    pub locked: bool,
    /// 锁定到期时间（Unix 时间戳）
    pub lock_until: Option<i64>,
}

impl GenesisConfig {
    /// 创建主网创世配置
    pub fn mainnet() -> Self {
        GenesisConfig {
            chain_id: "toki-mainnet-1".to_string(),
            genesis_time: 1704067200, // 2024-01-01 00:00:00 UTC
            initial_difficulty: 1_000_000,
            distribution_pool_balance: (TOTAL_SUPPLY as f64 * DISTRIBUTION_POOL_RATIO) as u64,
            reserve_pool_balance: (TOTAL_SUPPLY as f64 * RESERVE_POOL_RATIO) as u64,
            ai_aggregate_address: Address::from_base58(
                "toki1ai0000000000000000000000000000000000000000000000000000000",
            )
            .unwrap_or(Address::ZERO),
            developer_address: Address::from_base58(
                "toki1dev00000000000000000000000000000000000000000000000000000",
            )
            .unwrap_or(Address::ZERO),
            initial_allocations: vec![],
        }
    }

    /// 创建测试网创世配置
    pub fn testnet() -> Self {
        GenesisConfig {
            chain_id: "toki-testnet-1".to_string(),
            genesis_time: chrono::Utc::now().timestamp(),
            initial_difficulty: 1000, // 测试网难度较低
            distribution_pool_balance: (TOTAL_SUPPLY as f64 * DISTRIBUTION_POOL_RATIO) as u64,
            reserve_pool_balance: (TOTAL_SUPPLY as f64 * RESERVE_POOL_RATIO) as u64,
            ai_aggregate_address: Address::ZERO,
            developer_address: Address::ZERO,
            initial_allocations: vec![],
        }
    }

    /// 验证创世配置
    pub fn validate(&self) -> Result<(), String> {
        // 验证链 ID 不为空
        if self.chain_id.is_empty() {
            return Err("Chain ID cannot be empty".to_string());
        }

        // 验证初始难度
        if self.initial_difficulty == 0 {
            return Err("Initial difficulty cannot be zero".to_string());
        }

        // 验证总分配不超过总量
        let allocated: u64 = self.initial_allocations.iter().map(|a| a.balance).sum();

        let total_pools = self
            .distribution_pool_balance
            .saturating_add(self.reserve_pool_balance);

        if allocated.saturating_add(total_pools) > TOTAL_SUPPLY {
            return Err("Total allocation exceeds total supply".to_string());
        }

        Ok(())
    }

    /// 从 JSON 文件加载
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// 导出为 JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl Default for GenesisConfig {
    fn default() -> Self {
        Self::testnet()
    }
}

/// 创世区块信息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenesisInfo {
    /// 配置
    pub config: GenesisConfig,
    /// 创世区块哈希
    pub genesis_hash: Hash,
    /// 状态根
    pub state_root: Hash,
}

impl GenesisInfo {
    /// 创建创世信息
    pub fn new(config: GenesisConfig, genesis_hash: Hash, state_root: Hash) -> Self {
        GenesisInfo {
            config,
            genesis_hash,
            state_root,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mainnet_genesis() {
        let config = GenesisConfig::mainnet();
        assert!(config.validate().is_ok());
        assert_eq!(config.chain_id, "toki-mainnet-1");
    }

    #[test]
    fn test_testnet_genesis() {
        let config = GenesisConfig::testnet();
        assert!(config.validate().is_ok());
        assert_eq!(config.chain_id, "toki-testnet-1");
    }

    #[test]
    fn test_genesis_json() {
        let config = GenesisConfig::testnet();
        let json = config.to_json().unwrap();
        let parsed = GenesisConfig::from_json(&json).unwrap();
        assert_eq!(config.chain_id, parsed.chain_id);
    }
}
