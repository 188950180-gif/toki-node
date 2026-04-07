//! 法币兑换记录定义

use crate::{Address, Hash, FiatType, TOKI_BASE_UNIT};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 法币销毁凭证
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DestroyProof {
    /// 凭证哈希
    pub proof_hash: Hash,
    /// 销毁时间
    pub destroyed_at: DateTime<Utc>,
    /// 见证数据（销毁凭证）
    pub witness: Vec<u8>,
}

impl DestroyProof {
    /// 创建新销毁凭证
    pub fn new(witness: Vec<u8>) -> Self {
        let proof_hash = Hash::from_data(&witness);
        DestroyProof {
            proof_hash,
            destroyed_at: Utc::now(),
            witness,
        }
    }
}

/// 兑换记录
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExchangeTransaction {
    /// 兑换唯一标识
    pub exchange_id: Hash,
    /// 法币类型
    pub fiat_type: FiatType,
    /// 法币金额（最小单位，如分）
    pub fiat_amount: u64,
    /// 兑换时的汇率（相对于 EUR）
    pub exchange_rate: f64,
    /// 发放的 toki 数量（基本单位）
    pub toki_amount: u64,
    /// 奖励 toki 数量（基本单位）
    pub bonus_amount: u64,
    /// 用户账户地址
    pub user_address: Address,
    /// 物理网点标识
    pub branch_id: String,
    /// 兑换时间戳
    pub timestamp: DateTime<Utc>,
    /// 法币销毁凭证
    pub destroy_proof: Option<DestroyProof>,
    /// 兑换状态
    pub status: ExchangeStatus,
}

impl ExchangeTransaction {
    /// 创建新兑换记录
    pub fn new(
        fiat_type: FiatType,
        fiat_amount: u64,
        exchange_rate: f64,
        user_address: Address,
        branch_id: String,
    ) -> Self {
        // 计算基础 toki 数量
        let toki_amount = if fiat_type == FiatType::EUR {
            // 1 EUR = 1 toki
            fiat_amount
        } else {
            // 其他法币按汇率折算
            (fiat_amount as f64 / exchange_rate) as u64
        };

        // 生成唯一 ID
        let mut data = Vec::new();
        data.extend_from_slice(&fiat_amount.to_le_bytes());
        data.extend_from_slice(user_address.as_bytes());
        data.extend_from_slice(branch_id.as_bytes());
        data.extend_from_slice(&Utc::now().timestamp().to_le_bytes());
        let exchange_id = Hash::from_data(&data);

        ExchangeTransaction {
            exchange_id,
            fiat_type,
            fiat_amount,
            exchange_rate,
            toki_amount,
            bonus_amount: 0,
            user_address,
            branch_id,
            timestamp: Utc::now(),
            destroy_proof: None,
            status: ExchangeStatus::Pending,
        }
    }

    /// 设置奖励金额
    pub fn set_bonus(&mut self, bonus: u64) {
        self.bonus_amount = bonus;
    }

    /// 设置销毁凭证
    pub fn set_destroy_proof(&mut self, proof: DestroyProof) {
        self.destroy_proof = Some(proof);
        self.status = ExchangeStatus::Completed;
    }

    /// 获取总发放金额
    pub fn total_toki(&self) -> u64 {
        self.toki_amount.saturating_add(self.bonus_amount)
    }

    /// 获取金额（toki 单位）
    pub fn toki_amount_display(&self) -> f64 {
        self.toki_amount as f64 / TOKI_BASE_UNIT as f64
    }

    /// 确认兑换
    pub fn confirm(&mut self) {
        if self.status == ExchangeStatus::Pending {
            self.status = ExchangeStatus::Confirmed;
        }
    }
}

/// 兑换状态
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExchangeStatus {
    /// 待处理
    Pending,
    /// 已确认（法币已到账）
    Confirmed,
    /// 已完成（toki 已发放）
    Completed,
    /// 已取消
    Cancelled,
}

impl Default for ExchangeStatus {
    fn default() -> Self {
        ExchangeStatus::Pending
    }
}

impl std::fmt::Display for ExchangeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExchangeStatus::Pending => write!(f, "Pending"),
            ExchangeStatus::Confirmed => write!(f, "Confirmed"),
            ExchangeStatus::Completed => write!(f, "Completed"),
            ExchangeStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// 汇率信息
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExchangeRate {
    /// 法币类型
    pub fiat_type: FiatType,
    /// 相对于 EUR 的汇率
    pub rate: f64,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl ExchangeRate {
    /// 创建新汇率
    pub fn new(fiat_type: FiatType, rate: f64) -> Self {
        ExchangeRate {
            fiat_type,
            rate,
            updated_at: Utc::now(),
        }
    }

    /// EUR 汇率（恒为 1.0）
    pub fn eur() -> Self {
        Self::new(FiatType::EUR, 1.0)
    }
}

/// 兑换限额
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExchangeLimit {
    /// 单日限额
    pub daily_limit: u64,
    /// 累计限额
    pub total_limit: u64,
}

impl ExchangeLimit {
    /// 个人兑换限额
    pub fn personal() -> Self {
        ExchangeLimit {
            daily_limit: crate::PERSONAL_EXCHANGE_LIMIT * TOKI_BASE_UNIT,
            total_limit: crate::PERSONAL_EXCHANGE_LIMIT * TOKI_BASE_UNIT,
        }
    }

    /// 集体兑换限额
    pub fn collective() -> Self {
        ExchangeLimit {
            daily_limit: crate::COLLECTIVE_EXCHANGE_LIMIT * TOKI_BASE_UNIT,
            total_limit: crate::COLLECTIVE_EXCHANGE_LIMIT * TOKI_BASE_UNIT,
        }
    }

    /// 国家兑换限额
    pub fn nation() -> Self {
        ExchangeLimit {
            daily_limit: crate::NATION_EXCHANGE_LIMIT * TOKI_BASE_UNIT,
            total_limit: crate::NATION_EXCHANGE_LIMIT * TOKI_BASE_UNIT,
        }
    }
}

/// 法币通道状态
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FiatChannelStatus {
    /// 系统启动时间
    pub launch_time: DateTime<Utc>,
    /// 倒计时开始时间（启动后 365 天）
    pub countdown_start: Option<DateTime<Utc>>,
    /// 是否已关闭
    pub is_closed: bool,
}

impl FiatChannelStatus {
    /// 创建新状态
    pub fn new(launch_time: DateTime<Utc>) -> Self {
        FiatChannelStatus {
            launch_time,
            countdown_start: None,
            is_closed: false,
        }
    }

    /// 检查并更新状态
    pub fn update(&mut self) {
        if self.is_closed {
            return;
        }

        let now = Utc::now();
        let elapsed_days = (now - self.launch_time).num_days();

        // 启动后 365 天开始倒计时
        if elapsed_days >= crate::FIAT_CHANNEL_START_DELAY_DAYS as i64 {
            if self.countdown_start.is_none() {
                self.countdown_start = Some(now);
            }

            // 倒计时 365 天后关闭
            if let Some(start) = self.countdown_start {
                let countdown_days = (now - start).num_days();
                if countdown_days >= crate::FIAT_CHANNEL_COUNTDOWN_DAYS as i64 {
                    self.is_closed = true;
                }
            }
        }
    }

    /// 获取剩余天数
    pub fn remaining_days(&self) -> Option<i64> {
        if self.is_closed {
            return Some(0);
        }

        if let Some(start) = self.countdown_start {
            let elapsed = (Utc::now() - start).num_days();
            let remaining = crate::FIAT_CHANNEL_COUNTDOWN_DAYS as i64 - elapsed;
            Some(remaining.max(0))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_transaction() {
        let addr = Address::new([1u8; 32]);
        let exchange = ExchangeTransaction::new(
            FiatType::EUR,
            100 * TOKI_BASE_UNIT,
            1.0,
            addr,
            "branch-001".to_string(),
        );

        assert_eq!(exchange.fiat_type, FiatType::EUR);
        assert_eq!(exchange.toki_amount, 100 * TOKI_BASE_UNIT);
        assert_eq!(exchange.status, ExchangeStatus::Pending);
    }

    #[test]
    fn test_exchange_with_bonus() {
        let addr = Address::new([1u8; 32]);
        let mut exchange = ExchangeTransaction::new(
            FiatType::EUR,
            100 * TOKI_BASE_UNIT,
            1.0,
            addr,
            "branch-001".to_string(),
        );

        exchange.set_bonus(5 * TOKI_BASE_UNIT);
        assert_eq!(exchange.total_toki(), 105 * TOKI_BASE_UNIT);
    }

    #[test]
    fn test_exchange_rate() {
        let eur_rate = ExchangeRate::eur();
        assert_eq!(eur_rate.rate, 1.0);

        let usd_rate = ExchangeRate::new(FiatType::USD, 1.1);
        assert_eq!(usd_rate.rate, 1.1);
    }

    #[test]
    fn test_fiat_channel_status() {
        let launch_time = Utc::now() - chrono::Duration::days(400);
        let mut status = FiatChannelStatus::new(launch_time);

        status.update();
        assert!(status.countdown_start.is_some());
    }
}
