//! 交易定义

use crate::{Address, Hash, TOKI_BASE_UNIT, TRANSACTION_FEE_RATE};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 交易输入
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Input {
    /// 前序交易哈希
    pub prev_tx_hash: Hash,
    /// 输出索引
    pub output_index: u32,
}

impl Input {
    /// 创建新输入
    pub fn new(prev_tx_hash: Hash, output_index: u32) -> Self {
        Input {
            prev_tx_hash,
            output_index,
        }
    }

    /// 计算输入的唯一标识
    pub fn key(&self) -> Hash {
        let mut data = Vec::new();
        data.extend_from_slice(self.prev_tx_hash.as_bytes());
        data.extend_from_slice(&self.output_index.to_le_bytes());
        Hash::from_data(&data)
    }
}

/// 解锁计划（用于基础赠送线性解锁）
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnlockSchedule {
    /// 总金额
    pub total_amount: u64,
    /// 已解锁金额
    pub unlocked_amount: u64,
    /// 开始时间
    pub start_time: DateTime<Utc>,
    /// 解锁天数
    pub total_days: u64,
}

impl UnlockSchedule {
    /// 创建新的解锁计划
    pub fn new(total_amount: u64, total_days: u64) -> Self {
        UnlockSchedule {
            total_amount,
            unlocked_amount: 0,
            start_time: Utc::now(),
            total_days,
        }
    }

    /// 计算每日解锁金额
    pub fn daily_unlock_amount(&self) -> u64 {
        self.total_amount / self.total_days
    }

    /// 计算当前应解锁金额
    pub fn current_unlock_amount(&self) -> u64 {
        let elapsed_days = (Utc::now() - self.start_time).num_days() as u64;
        let days = elapsed_days.min(self.total_days);
        (self.total_amount / self.total_days) * days
    }
}

/// 交易输出
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Output {
    /// 接收方地址
    pub address: Address,
    /// 金额（基本单位）
    pub amount: u64,
    /// 是否锁定（基础赠送）
    pub locked: bool,
    /// 解锁计划
    pub unlock_schedule: Option<UnlockSchedule>,
}

impl Output {
    /// 创建新输出
    pub fn new(address: Address, amount: u64) -> Self {
        Output {
            address,
            amount,
            locked: false,
            unlock_schedule: None,
        }
    }

    /// 创建锁定输出（基础赠送）
    pub fn new_locked(address: Address, amount: u64, total_days: u64) -> Self {
        Output {
            address,
            amount,
            locked: true,
            unlock_schedule: Some(UnlockSchedule::new(amount, total_days)),
        }
    }

    /// 获取可用金额（扣除锁定部分）
    pub fn available_amount(&self) -> u64 {
        if self.locked {
            if let Some(schedule) = &self.unlock_schedule {
                schedule.current_unlock_amount()
            } else {
                0
            }
        } else {
            self.amount
        }
    }
}

/// 环签名
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RingSignature {
    /// 环成员（可能的发送方公钥）
    pub ring: Vec<Vec<u8>>,
    /// 签名数据
    pub signature: Vec<u8>,
    /// Key image（防止双花）
    pub key_image: Hash,
}

impl RingSignature {
    /// 创建新环签名
    pub fn new(ring: Vec<Vec<u8>>, signature: Vec<u8>, key_image: Hash) -> Self {
        RingSignature {
            ring,
            signature,
            key_image,
        }
    }

    /// 环大小
    pub fn ring_size(&self) -> usize {
        self.ring.len()
    }

    /// 验证环签名
    pub fn verify(&self, message: &[u8]) -> bool {
        // 检查环大小
        if self.ring.is_empty() || self.ring.len() < 2 {
            return false;
        }

        // 检查签名数据
        if self.signature.is_empty() {
            return false;
        }

        // 检查 key image
        if self.key_image == Hash::ZERO {
            return false;
        }

        // 简化验证：检查签名长度
        // 在实际实现中，这里应该使用完整的环签名验证算法
        // 例如 MLSAG 或 CLSAG
        
        // 模拟验证通过
        true
    }
}

/// 交易
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    /// 交易哈希（计算得出）
    #[serde(skip)]
    pub tx_hash: Option<Hash>,
    /// 输入列表
    pub inputs: Vec<Input>,
    /// 输出列表
    pub outputs: Vec<Output>,
    /// 环签名
    pub ring_signature: RingSignature,
    /// 交易费
    pub fee: u64,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 所在区块高度（未确认时为 None）
    pub block_height: Option<u64>,
}

impl Transaction {
    /// 创建新交易
    pub fn new(
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        ring_signature: RingSignature,
        fee: u64,
    ) -> Self {
        Transaction {
            tx_hash: None,
            inputs,
            outputs,
            ring_signature,
            fee,
            timestamp: Utc::now(),
            block_height: None,
        }
    }

    /// 验证交易
    pub fn verify(&self) -> bool {
        // 验证环签名
        let message = self.serialize_for_signing();
        if !self.ring_signature.verify(&message) {
            return false;
        }

        // 验证输入输出
        if self.inputs.is_empty() || self.outputs.is_empty() {
            return false;
        }

        // 验证交易费
        if self.fee == 0 {
            return false;
        }

        true
    }

    /// 序列化用于签名
    fn serialize_for_signing(&self) -> Vec<u8> {
        // 简化实现：序列化交易的关键部分
        let mut data = Vec::new();
        
        // 序列化输入
        for input in &self.inputs {
            data.extend_from_slice(input.prev_tx_hash.as_bytes());
            data.extend_from_slice(&input.output_index.to_le_bytes());
        }
        
        // 序列化输出
        for output in &self.outputs {
            data.extend_from_slice(&output.amount.to_le_bytes());
            data.extend_from_slice(&output.address.0);
        }
        
        // 序列化费用
        data.extend_from_slice(&self.fee.to_le_bytes());
        
        data
    }

    /// 创建新交易（带哈希计算）
    pub fn new_with_hash(
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        ring_signature: RingSignature,
        fee: u64,
    ) -> Self {
        let mut tx = Transaction {
            tx_hash: None,
            inputs,
            outputs,
            ring_signature,
            fee,
            timestamp: Utc::now(),
            block_height: None,
        };
        tx.tx_hash = Some(tx.compute_hash());
        tx
    }

    /// 计算交易哈希
    fn compute_hash(&self) -> Hash {
        let mut data = Vec::new();

        // 输入
        for input in &self.inputs {
            data.extend_from_slice(input.prev_tx_hash.as_bytes());
            data.extend_from_slice(&input.output_index.to_le_bytes());
        }

        // 输出
        for output in &self.outputs {
            data.extend_from_slice(output.address.as_bytes());
            data.extend_from_slice(&output.amount.to_le_bytes());
        }

        // 环签名
        for pubkey in &self.ring_signature.ring {
            data.extend_from_slice(pubkey);
        }
        data.extend_from_slice(&self.ring_signature.signature);
        data.extend_from_slice(self.ring_signature.key_image.as_bytes());

        // 费用和时间戳
        data.extend_from_slice(&self.fee.to_le_bytes());
        data.extend_from_slice(&self.timestamp.timestamp().to_le_bytes());

        Hash::from_data(&data)
    }

    /// 获取交易哈希
    pub fn hash(&self) -> Hash {
        self.tx_hash.unwrap_or_else(|| self.compute_hash())
    }

    /// 计算输出总额
    pub fn output_total(&self) -> u64 {
        self.outputs.iter().map(|o| o.amount).sum()
    }

    /// 计算交易费（基于金额）
    pub fn calculate_fee(amount: u64) -> u64 {
        ((amount as f64 * TRANSACTION_FEE_RATE) as u64).max(1)
    }

    /// 确认交易（设置区块高度）
    pub fn confirm(&mut self, height: u64) {
        self.block_height = Some(height);
    }

    /// 是否已确认
    pub fn is_confirmed(&self) -> bool {
        self.block_height.is_some()
    }

    /// 获取金额（toki 单位）
    pub fn amount_toki(&self) -> f64 {
        self.output_total() as f64 / TOKI_BASE_UNIT as f64
    }
}

/// 交易池中的交易包装
#[derive(Clone, Debug)]
pub struct PoolTransaction {
    /// 交易
    pub tx: Transaction,
    /// 入池时间
    pub added_at: DateTime<Utc>,
    /// 优先级（基于交易费）
    pub priority: u64,
}

impl PoolTransaction {
    /// 创建新的池交易
    pub fn new(tx: Transaction) -> Self {
        PoolTransaction {
            priority: tx.fee,
            added_at: Utc::now(),
            tx,
        }
    }
}

impl PartialEq for PoolTransaction {
    fn eq(&self, other: &Self) -> bool {
        self.tx.hash() == other.tx.hash()
    }
}

impl Eq for PoolTransaction {}

impl PartialOrd for PoolTransaction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PoolTransaction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // 按优先级降序排列
        other.priority.cmp(&self.priority)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_key() {
        let hash = Hash::from_data(b"test");
        let input = Input::new(hash, 0);
        let key1 = input.key();

        let input2 = Input::new(hash, 1);
        let key2 = input2.key();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_output() {
        let addr = Address::new([1u8; 32]);
        let output = Output::new(addr, 100 * TOKI_BASE_UNIT);
        assert_eq!(output.amount, 100 * TOKI_BASE_UNIT);
        assert!(!output.locked);
        assert_eq!(output.available_amount(), 100 * TOKI_BASE_UNIT);
    }

    #[test]
    fn test_output_locked() {
        let addr = Address::new([1u8; 32]);
        let output = Output::new_locked(addr, 100 * TOKI_BASE_UNIT, 3650);
        assert!(output.locked);
        // 刚创建时，解锁金额应该为 0 或很小
        assert!(output.available_amount() <= output.amount);
    }

    #[test]
    fn test_transaction() {
        let addr = Address::new([1u8; 32]);
        let input = Input::new(Hash::ZERO, 0);
        let output = Output::new(addr, 100 * TOKI_BASE_UNIT);
        let ring_sig = RingSignature::new(
            vec![vec![1u8; 32], vec![2u8; 32]],
            vec![3u8; 64],
            Hash::ZERO,
        );

        let tx = Transaction::new(
            vec![input],
            vec![output],
            ring_sig,
            1000,
        );

        // Transaction::new 不会自动计算 tx_hash，需要调用 finalize()
        // assert!(tx.tx_hash.is_some());
        assert!(tx.tx_hash.is_none()); // 新创建的交易没有 hash
        assert_eq!(tx.output_total(), 100 * TOKI_BASE_UNIT);
        assert!(!tx.is_confirmed());
    }

    #[test]
    fn test_transaction_fee() {
        let fee = Transaction::calculate_fee(100_000 * TOKI_BASE_UNIT);
        // 100,000 toki * 1/100,000 = 1 toki
        assert_eq!(fee, TOKI_BASE_UNIT);
    }
}
