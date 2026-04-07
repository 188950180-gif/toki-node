//! 错误定义

use thiserror::Error;

/// 核心错误类型
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Account error: {0}")]
    Account(#[from] AccountError),

    #[error("Transaction error: {0}")]
    Transaction(#[from] TransactionError),

    #[error("Block error: {0}")]
    Block(#[from] BlockError),

    #[error("Exchange error: {0}")]
    Exchange(#[from] ExchangeError),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),
}

/// 账户错误
#[derive(Debug, Error)]
pub enum AccountError {
    #[error("Account already exists")]
    AlreadyExists,

    #[error("Account not found")]
    NotFound,

    #[error("Invalid account type")]
    InvalidType,

    #[error("Device fingerprint already registered")]
    DeviceFingerprintExists,

    #[error("Bio hash already registered")]
    BioHashExists,

    #[error("Account limit exceeded")]
    LimitExceeded,

    #[error("Balance limit exceeded")]
    BalanceLimitExceeded,

    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Account is locked")]
    AccountLocked,

    #[error("Invalid auth data")]
    InvalidAuthData,

    #[error("Collective account quota exceeded")]
    CollectiveQuotaExceeded,

    #[error("Nation account limit exceeded")]
    NationLimitExceeded,
}

/// 交易错误
#[derive(Debug, Error)]
pub enum TransactionError {
    #[error("Transaction not found")]
    NotFound,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Invalid ring signature")]
    InvalidRingSignature,

    #[error("Key image already spent")]
    KeyImageSpent,

    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Invalid input")]
    InvalidInput,

    #[error("Invalid output")]
    InvalidOutput,

    #[error("Fee too low")]
    FeeTooLow,

    #[error("Transaction already in pool")]
    AlreadyInPool,

    #[error("Transaction expired")]
    Expired,

    #[error("Double spend detected")]
    DoubleSpend,
}

/// 区块错误
#[derive(Debug, Error)]
pub enum BlockError {
    #[error("Block not found")]
    NotFound,

    #[error("Invalid proof of work")]
    InvalidPoW,

    #[error("Invalid difficulty")]
    InvalidDifficulty,

    #[error("Invalid merkle root")]
    InvalidMerkleRoot,

    #[error("Invalid previous hash")]
    InvalidPrevHash,

    #[error("Invalid timestamp")]
    InvalidTimestamp,

    #[error("Block already exists")]
    AlreadyExists,

    #[error("Orphan block")]
    Orphan,

    #[error("Invalid transaction in block")]
    InvalidTransaction,
}

/// 兑换错误
#[derive(Debug, Error)]
pub enum ExchangeError {
    #[error("Exchange not found")]
    NotFound,

    #[error("Exchange channel closed")]
    ChannelClosed,

    #[error("Exchange limit exceeded")]
    LimitExceeded,

    #[error("Invalid fiat amount")]
    InvalidFiatAmount,

    #[error("Invalid fiat type")]
    InvalidFiatType,

    #[error("Exchange rate unavailable")]
    RateUnavailable,

    #[error("Branch not found")]
    BranchNotFound,

    #[error("Destroy proof invalid")]
    DestroyProofInvalid,
}
