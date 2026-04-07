//! 交易池（内存池）
//! 
//! 管理待确认的交易

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use parking_lot::RwLock;
use tracing::{debug, info};

use toki_core::{Hash, Transaction};

/// 交易池配置
#[derive(Clone, Debug)]
pub struct TxPoolConfig {
    /// 最大交易数量
    pub max_tx_count: usize,
    /// 最大交易大小（字节）
    pub max_tx_size: usize,
    /// 最小交易费
    pub min_fee: u64,
    /// 交易过期时间（秒）
    pub tx_timeout_secs: u64,
}

impl Default for TxPoolConfig {
    fn default() -> Self {
        TxPoolConfig {
            max_tx_count: 10000,
            max_tx_size: 100 * 1024, // 100 KB
            min_fee: 1000,
            tx_timeout_secs: 3600, // 1 小时
        }
    }
}

/// 交易池条目
#[derive(Clone, Debug)]
pub struct PoolEntry {
    /// 交易
    pub transaction: Transaction,
    /// 加入时间
    pub added_at: i64,
    /// 交易大小
    pub size: usize,
}

/// 交易池
pub struct TransactionPool {
    /// 配置
    config: TxPoolConfig,
    /// 待处理交易（按费率排序）
    pending: Arc<RwLock<VecDeque<Hash>>>,
    /// 所有交易
    transactions: Arc<RwLock<HashMap<Hash, PoolEntry>>>,
    /// 已使用的 key image（防止双花）
    key_images: Arc<RwLock<HashSet<Hash>>>,
}

impl TransactionPool {
    /// 创建新交易池
    pub fn new(config: TxPoolConfig) -> Self {
        TransactionPool {
            config,
            pending: Arc::new(RwLock::new(VecDeque::new())),
            transactions: Arc::new(RwLock::new(HashMap::new())),
            key_images: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// 添加交易
    pub fn add_transaction(&self, tx: Transaction) -> Result<Hash, TxPoolError> {
        let tx_hash = tx.hash();
        let now = chrono::Utc::now().timestamp();

        // 检查是否已存在
        {
            let txs = self.transactions.read();
            if txs.contains_key(&tx_hash) {
                return Err(TxPoolError::AlreadyExists);
            }
        }

        // 检查交易数量限制
        {
            let txs = self.transactions.read();
            if txs.len() >= self.config.max_tx_count {
                return Err(TxPoolError::PoolFull);
            }
        }

        // 检查交易费
        if tx.fee < self.config.min_fee {
            return Err(TxPoolError::FeeTooLow);
        }

        // 检查交易大小
        let size = estimate_tx_size(&tx);
        if size > self.config.max_tx_size {
            return Err(TxPoolError::TxTooLarge);
        }

        // 检查双花（使用输入的 key）
        {
            let key_images = self.key_images.read();
            for input in &tx.inputs {
                let input_key = input.key();
                if key_images.contains(&input_key) {
                    return Err(TxPoolError::DoubleSpend);
                }
            }
        }

        // 添加交易
        {
            let mut txs = self.transactions.write();
            txs.insert(tx_hash, PoolEntry {
                transaction: tx.clone(),
                added_at: now,
                size,
            });
        }

        // 添加到待处理队列
        {
            let mut pending = self.pending.write();
            pending.push_back(tx_hash);
        }

        // 记录输入 key（用于双花检测）
        {
            let mut key_images = self.key_images.write();
            for input in &tx.inputs {
                key_images.insert(input.key());
            }
        }

        debug!("交易加入交易池: {} (size: {} bytes)", tx_hash, size);
        Ok(tx_hash)
    }

    /// 获取交易
    pub fn get_transaction(&self, hash: &Hash) -> Option<Transaction> {
        let txs = self.transactions.read();
        txs.get(hash).map(|e| e.transaction.clone())
    }

    /// 移除交易
    pub fn remove_transaction(&self, hash: &Hash) {
        // 从交易列表移除
        let tx = {
            let mut txs = self.transactions.write();
            txs.remove(hash)
        };

        // 从待处理队列移除
        {
            let mut pending = self.pending.write();
            pending.retain(|h| h != hash);
        }

        // 移除 key image
        if let Some(entry) = tx {
            let mut key_images = self.key_images.write();
            for input in &entry.transaction.inputs {
                key_images.remove(&input.key());
            }
        }

        debug!("交易从交易池移除: {}", hash);
    }

    /// 获取待打包交易
    pub fn get_pending_transactions(&self, max_count: usize) -> Vec<Transaction> {
        let pending = self.pending.read();
        let txs = self.transactions.read();

        pending.iter()
            .take(max_count)
            .filter_map(|h| txs.get(h).map(|e| e.transaction.clone()))
            .collect()
    }

    /// 获取交易数量
    pub fn tx_count(&self) -> usize {
        self.transactions.read().len()
    }

    /// 清理过期交易
    pub fn cleanup_expired(&self) {
        let now = chrono::Utc::now().timestamp();
        let timeout = self.config.tx_timeout_secs as i64;

        let expired: Vec<Hash> = {
            let txs = self.transactions.read();
            txs.iter()
                .filter(|(_, e)| now - e.added_at > timeout)
                .map(|(h, _)| *h)
                .collect()
        };

        for hash in expired {
            self.remove_transaction(&hash);
            info!("清理过期交易: {}", hash);
        }
    }

    /// 获取交易池状态
    pub fn status(&self) -> TxPoolStatus {
        TxPoolStatus {
            tx_count: self.tx_count(),
            pending_count: self.pending.read().len(),
            key_image_count: self.key_images.read().len(),
        }
    }
}

/// 交易池错误
#[derive(Debug, Clone)]
pub enum TxPoolError {
    /// 交易已存在
    AlreadyExists,
    /// 交易池已满
    PoolFull,
    /// 交易费太低
    FeeTooLow,
    /// 交易太大
    TxTooLarge,
    /// 双花
    DoubleSpend,
}

/// 交易池状态
#[derive(Debug, Clone)]
pub struct TxPoolStatus {
    pub tx_count: usize,
    pub pending_count: usize,
    pub key_image_count: usize,
}

/// 估算交易大小
fn estimate_tx_size(tx: &Transaction) -> usize {
    // 简化估算
    let base_size = 100; // 基础开销
    let input_size = tx.inputs.len() * 40;
    let output_size = tx.outputs.len() * 72;
    let ring_size = tx.ring_signature.ring.len() * 32;
    let sig_size = tx.ring_signature.signature.len();
    
    base_size + input_size + output_size + ring_size + sig_size
}

impl Default for TransactionPool {
    fn default() -> Self {
        Self::new(TxPoolConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toki_core::{Address, Input, Output, RingSignature, TOKI_BASE_UNIT};

    fn create_test_tx() -> Transaction {
        let addr = Address::new([1u8; 32]);
        let output = Output::new(addr, 100 * TOKI_BASE_UNIT);
        let ring_sig = RingSignature::new(vec![], vec![1u8; 64], Hash::ZERO);
        Transaction::new(vec![], vec![output], ring_sig, 10000)
    }

    #[test]
    fn test_add_transaction() {
        let pool = TransactionPool::default();
        let tx = create_test_tx();
        
        let result = pool.add_transaction(tx);
        assert!(result.is_ok());
        assert_eq!(pool.tx_count(), 1);
    }

    #[test]
    fn test_duplicate_transaction() {
        let pool = TransactionPool::default();
        let tx = create_test_tx();
        
        pool.add_transaction(tx.clone()).unwrap();
        let result = pool.add_transaction(tx);
        assert!(matches!(result, Err(TxPoolError::AlreadyExists)));
    }

    #[test]
    fn test_get_pending() {
        let pool = TransactionPool::default();
        let tx = create_test_tx();
        
        pool.add_transaction(tx).unwrap();
        
        let pending = pool.get_pending_transactions(10);
        assert_eq!(pending.len(), 1);
    }
}
