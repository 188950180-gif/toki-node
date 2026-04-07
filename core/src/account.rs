//! 账户定义

use crate::{Address, Hash, Region, TOKI_BASE_UNIT};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 账户类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccountType {
    /// 个人账户
    Personal,
    /// 集体账户（企业/组织）
    Collective,
    /// 国家账户
    Nation,
    /// 开发者账户
    Developer {
        /// 是否为总收款账户
        is_main: bool,
    },
    /// AI 归集账户
    AIAggregate,
}

impl Default for AccountType {
    fn default() -> Self {
        AccountType::Personal
    }
}

impl std::fmt::Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountType::Personal => write!(f, "Personal"),
            AccountType::Collective => write!(f, "Collective"),
            AccountType::Nation => write!(f, "Nation"),
            AccountType::Developer { is_main } => {
                if *is_main {
                    write!(f, "Developer(Main)")
                } else {
                    write!(f, "Developer")
                }
            }
            AccountType::AIAggregate => write!(f, "AIAggregate"),
        }
    }
}

/// 账户
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    /// 账户地址
    pub address: Address,
    /// 账户类型
    pub account_type: AccountType,
    /// 余额（基本单位）
    pub balance: u64,
    /// 锁定余额（用于基础赠送线性解锁）
    pub locked_balance: u64,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后活跃时间
    pub last_active: DateTime<Utc>,
    /// 认证信息哈希（实名认证、营业执照等）
    pub auth_hash: Option<Hash>,
    /// 区域标识（仅开发者超级账号需要）
    pub region: Option<Region>,
    /// 设备指纹
    pub device_fingerprint: Option<Hash>,
    /// 生物特征哈希
    pub bio_hash: Option<Hash>,
    /// 国家代码（用于集体/国家账户）
    pub country_code: Option<String>,
    /// 累计兑换金额（用于限额检查）
    pub total_exchanged: u64,
    /// 基础赠送解锁进度（已解锁天数）
    pub gift_unlock_days: u64,
}

impl Account {
    /// 创建新账户
    pub fn new(address: Address, account_type: AccountType) -> Self {
        let now = Utc::now();
        Account {
            address,
            account_type,
            balance: 0,
            locked_balance: 0,
            created_at: now,
            last_active: now,
            auth_hash: None,
            region: None,
            device_fingerprint: None,
            bio_hash: None,
            country_code: None,
            total_exchanged: 0,
            gift_unlock_days: 0,
        }
    }

    /// 创建个人账户
    pub fn new_personal(
        address: Address,
        device_fingerprint: Hash,
        bio_hash: Hash,
    ) -> Self {
        let mut account = Self::new(address, AccountType::Personal);
        account.device_fingerprint = Some(device_fingerprint);
        account.bio_hash = Some(bio_hash);
        account
    }

    /// 创建开发者账户
    pub fn new_developer(address: Address, is_main: bool, region: Option<Region>) -> Self {
        let mut account = Self::new(address, AccountType::Developer { is_main });
        account.region = region;
        account
    }

    /// 创建 AI 归集账户
    pub fn new_ai_aggregate(address: Address) -> Self {
        Self::new(address, AccountType::AIAggregate)
    }

    /// 获取余额（toki 单位）
    pub fn balance_toki(&self) -> f64 {
        self.balance as f64 / TOKI_BASE_UNIT as f64
    }

    /// 增加余额
    pub fn add_balance(&mut self, amount: u64) {
        self.balance = self.balance.saturating_add(amount);
        self.last_active = Utc::now();
    }

    /// 减少余额
    pub fn sub_balance(&mut self, amount: u64) -> bool {
        if self.balance >= amount {
            self.balance -= amount;
            self.last_active = Utc::now();
            true
        } else {
            false
        }
    }

    /// 增加锁定余额
    pub fn add_locked_balance(&mut self, amount: u64) {
        self.locked_balance = self.locked_balance.saturating_add(amount);
    }

    /// 解锁部分余额
    pub fn unlock_balance(&mut self, amount: u64) -> bool {
        if self.locked_balance >= amount {
            self.locked_balance -= amount;
            self.balance = self.balance.saturating_add(amount);
            self.gift_unlock_days += 1;
            true
        } else {
            false
        }
    }

    /// 是否为开发者超级账号
    pub fn is_super_account(&self) -> bool {
        matches!(self.account_type, AccountType::Developer { is_main: false })
    }

    /// 是否为开发者总收款账户
    pub fn is_main_developer_account(&self) -> bool {
        matches!(self.account_type, AccountType::Developer { is_main: true })
    }

    /// 更新最后活跃时间
    pub fn touch(&mut self) {
        self.last_active = Utc::now();
    }

    /// 计算不活跃天数
    pub fn inactive_days(&self) -> i64 {
        let now = Utc::now();
        (now - self.last_active).num_days()
    }
}

impl Default for Account {
    fn default() -> Self {
        Self::new(Address::ZERO, AccountType::Personal)
    }
}

/// 账户创建请求
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateAccountRequest {
    /// 账户类型
    pub account_type: AccountType,
    /// 设备指纹
    pub device_fingerprint: Option<Hash>,
    /// 生物特征哈希
    pub bio_hash: Option<Hash>,
    /// 认证数据哈希
    pub auth_hash: Option<Hash>,
    /// 国家代码
    pub country_code: Option<String>,
    /// 区域（开发者超级账号）
    pub region: Option<Region>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_new() {
        let addr = Address::new([1u8; 32]);
        let account = Account::new(addr, AccountType::Personal);
        assert_eq!(account.address, addr);
        assert_eq!(account.balance, 0);
        assert_eq!(account.locked_balance, 0);
    }

    #[test]
    fn test_account_balance() {
        let addr = Address::new([1u8; 32]);
        let mut account = Account::new(addr, AccountType::Personal);

        account.add_balance(100 * TOKI_BASE_UNIT);
        assert_eq!(account.balance, 100 * TOKI_BASE_UNIT);
        assert!((account.balance_toki() - 100.0).abs() < f64::EPSILON);

        assert!(account.sub_balance(50 * TOKI_BASE_UNIT));
        assert_eq!(account.balance, 50 * TOKI_BASE_UNIT);

        assert!(!account.sub_balance(100 * TOKI_BASE_UNIT));
        assert_eq!(account.balance, 50 * TOKI_BASE_UNIT);
    }

    #[test]
    fn test_account_locked_balance() {
        let addr = Address::new([1u8; 32]);
        let mut account = Account::new(addr, AccountType::Personal);

        account.add_locked_balance(100 * TOKI_BASE_UNIT);
        assert_eq!(account.locked_balance, 100 * TOKI_BASE_UNIT);

        assert!(account.unlock_balance(10 * TOKI_BASE_UNIT));
        assert_eq!(account.locked_balance, 90 * TOKI_BASE_UNIT);
        assert_eq!(account.balance, 10 * TOKI_BASE_UNIT);
        assert_eq!(account.gift_unlock_days, 1);
    }

    #[test]
    fn test_developer_account() {
        let addr = Address::new([1u8; 32]);
        let main = Account::new_developer(addr, true, None);
        assert!(main.is_main_developer_account());
        assert!(!main.is_super_account());

        let super_acc = Account::new_developer(addr, false, Some(Region::US));
        assert!(!super_acc.is_main_developer_account());
        assert!(super_acc.is_super_account());
        assert_eq!(super_acc.region, Some(Region::US));
    }
}
