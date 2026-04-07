//! 区块验证器

use toki_core::{Block, Hash, Transaction};

/// 验证结果
#[derive(Debug)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }
}

/// 区块验证器
#[allow(dead_code)]
pub struct BlockValidator {
    /// 当前难度
    current_difficulty: u64,
}

impl BlockValidator {
    pub fn new(current_difficulty: u64) -> Self {
        BlockValidator { current_difficulty }
    }

    /// 验证区块
    pub fn validate_block(&self, block: &Block) -> ValidationResult {
        // 验证区块头
        if let Err(e) = self.validate_header(block) {
            return ValidationResult::Invalid(e);
        }

        // 验证交易
        for tx in &block.transactions {
            if let Err(e) = self.validate_transaction(tx) {
                return ValidationResult::Invalid(e);
            }
        }

        // 验证 Merkle 根
        if !block.verify_merkle_root() {
            return ValidationResult::Invalid("Invalid merkle root".to_string());
        }

        ValidationResult::Valid
    }

    /// 验证区块头
    fn validate_header(&self, block: &Block) -> Result<(), String> {
        // 验证难度
        if !block.meets_difficulty() {
            return Err("Block does not meet difficulty requirement".to_string());
        }

        // 验证时间戳
        let now = chrono::Utc::now().timestamp() as u64;
        if block.header.timestamp.timestamp() as u64 > now + 7200 {
            return Err("Block timestamp too far in future".to_string());
        }

        Ok(())
    }

    /// 验证交易
    fn validate_transaction(&self, tx: &Transaction) -> Result<(), String> {
        // 验证交易费
        // 注意：180天内允许0费用（延迟收费机制）
        // 这里简化处理，实际应该在验证时传入创世时间和当前时间
        // 完整实现见 validate_transaction_with_delay 方法

        // 暂时允许0费用，由交易池和API层控制
        // if tx.fee == 0 {
        //     return Err("Transaction fee cannot be zero".to_string());
        // }

        // 验证输入输出平衡
        // TODO: 实现完整的 UTXO 验证

        Ok(())
    }

    /// 验证交易（带延迟收费）
    ///
    /// # 参数
    /// - `tx`: 交易
    /// - `genesis_time`: 创世区块时间戳（秒）
    /// - `current_time`: 当前时间戳（秒）
    pub fn validate_transaction_with_delay(
        &self,
        tx: &Transaction,
        genesis_time: u64,
        current_time: u64,
    ) -> Result<(), String> {
        use toki_core::constants::FEE_DELAY_DAYS;

        // 计算运行天数
        let running_days = (current_time.saturating_sub(genesis_time)) / 86400;

        // 180天内允许0费用
        if running_days < FEE_DELAY_DAYS && tx.fee == 0 {
            return Ok(());
        }

        // 180天后必须收费
        if running_days >= FEE_DELAY_DAYS && tx.fee == 0 {
            return Err("Transaction fee cannot be zero after 180 days".to_string());
        }

        // 验证输入输出平衡
        // TODO: 实现完整的 UTXO 验证

        Ok(())
    }
}

/// 交易验证器
pub struct TransactionValidator;

impl TransactionValidator {
    /// 验证交易签名
    pub fn verify_signature(_tx: &Transaction) -> bool {
        // TODO: 实现环签名验证
        true
    }

    /// 验证双花
    pub fn check_double_spend(_tx: &Transaction, _spent_keys: &[Hash]) -> bool {
        // TODO: 检查 key_image 是否已使用
        true
    }
}
