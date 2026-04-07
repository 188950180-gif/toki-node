//! 交易存储

use crate::{Database, StorageError, CF_KEY_IMAGES, CF_TRANSACTIONS};
use bincode;
use std::sync::Arc;
use toki_core::{Hash, Transaction};

/// 交易存储
pub struct TransactionStore {
    db: Arc<Database>,
}

impl TransactionStore {
    /// 创建新交易存储
    pub fn new(db: Arc<Database>) -> Self {
        TransactionStore { db }
    }

    /// 保存交易
    pub fn save_transaction(&self, tx: &Transaction) -> Result<(), StorageError> {
        let key = tx.hash().as_bytes().to_vec();
        let data = bincode::serialize(tx)?;
        self.db.put(CF_TRANSACTIONS, &key, &data)?;
        Ok(())
    }

    /// 获取交易
    pub fn get_transaction(&self, tx_hash: &Hash) -> Result<Option<Transaction>, StorageError> {
        let key = tx_hash.as_bytes().to_vec();
        let data = self.db.get(CF_TRANSACTIONS, &key)?;

        match data {
            Some(bytes) => {
                let tx: Transaction = bincode::deserialize(&bytes)?;
                Ok(Some(tx))
            }
            None => Ok(None),
        }
    }

    /// 检查交易是否存在
    pub fn transaction_exists(&self, tx_hash: &Hash) -> Result<bool, StorageError> {
        let key = tx_hash.as_bytes().to_vec();
        self.db.exists(CF_TRANSACTIONS, &key)
    }

    /// 检查 key_image 是否已使用（双花检查）
    pub fn key_image_exists(&self, key_image: &Hash) -> Result<bool, StorageError> {
        let key = key_image.as_bytes().to_vec();
        self.db.exists(CF_KEY_IMAGES, &key)
    }

    /// 标记 key_image 为已使用
    pub fn mark_key_image_used(
        &self,
        key_image: &Hash,
        tx_hash: &Hash,
    ) -> Result<(), StorageError> {
        let key = key_image.as_bytes().to_vec();
        let value = tx_hash.as_bytes().to_vec();
        self.db.put(CF_KEY_IMAGES, &key, &value)?;
        Ok(())
    }

    /// 获取 key_image 对应的交易哈希
    pub fn get_key_image_tx(&self, key_image: &Hash) -> Result<Option<Hash>, StorageError> {
        let key = key_image.as_bytes().to_vec();
        let data = self.db.get(CF_KEY_IMAGES, &key)?;

        match data {
            Some(bytes) => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Ok(Some(Hash::new(arr)))
            }
            None => Ok(None),
        }
    }

    /// 批量保存交易
    pub fn save_transactions_batch(&self, txs: &[Transaction]) -> Result<(), StorageError> {
        let mut writes = Vec::new();

        for tx in txs {
            let key = tx.hash().as_bytes().to_vec();
            let data = bincode::serialize(tx)?;
            writes.push(crate::WriteOp::put(CF_TRANSACTIONS, key, data));

            // 同时标记 key_image
            let ki_key = tx.ring_signature.key_image.as_bytes().to_vec();
            let ki_value = tx.hash().as_bytes().to_vec();
            writes.push(crate::WriteOp::put(CF_KEY_IMAGES, ki_key, ki_value));
        }

        self.db.write_batch(writes)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use toki_core::{Address, Output, RingSignature, TOKI_BASE_UNIT};

    #[test]
    fn test_transaction_store() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();
        let store = TransactionStore::new(Arc::new(db));

        // 创建测试交易
        let addr = Address::new([1u8; 32]);
        let output = Output::new(addr, 100 * TOKI_BASE_UNIT);
        let ring_sig = RingSignature::new(vec![vec![1u8; 32]], vec![2u8; 64], Hash::ZERO);
        let tx = Transaction::new(vec![], vec![output], ring_sig, 1000);

        // 保存
        store.save_transaction(&tx).unwrap();

        // 读取
        let loaded = store.get_transaction(&tx.hash()).unwrap();
        assert!(loaded.is_some());

        // 检查 key_image
        assert!(!store.key_image_exists(&Hash::ZERO).unwrap());
        store.mark_key_image_used(&Hash::ZERO, &tx.hash()).unwrap();
        assert!(store.key_image_exists(&Hash::ZERO).unwrap());
    }
}
