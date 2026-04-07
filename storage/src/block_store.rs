//! 区块存储

use crate::{Database, StorageError, CF_BLOCKS, CF_METADATA};
use bincode;
use std::sync::Arc;
use toki_core::Block;

/// 区块存储
pub struct BlockStore {
    db: Arc<Database>,
}

impl BlockStore {
    /// 创建新区块存储
    pub fn new(db: Arc<Database>) -> Self {
        BlockStore { db }
    }

    /// 保存区块
    pub fn save_block(&self, block: &Block) -> Result<(), StorageError> {
        let height = block.height();
        let _hash = block.hash();

        // 序列化区块
        let data = bincode::serialize(block)?;

        // 按高度存储
        let height_key = height.to_be_bytes().to_vec();
        self.db.put(CF_BLOCKS, &height_key, &data)?;

        // 更新最新高度
        self.update_latest_height(height)?;

        Ok(())
    }

    /// 获取区块（按高度）
    pub fn get_block_by_height(&self, height: u64) -> Result<Option<Block>, StorageError> {
        let key = height.to_be_bytes().to_vec();
        let data = self.db.get(CF_BLOCKS, &key)?;

        match data {
            Some(bytes) => {
                let block: Block = bincode::deserialize(&bytes)?;
                Ok(Some(block))
            }
            None => Ok(None),
        }
    }

    /// 获取最新区块高度
    pub fn get_latest_height(&self) -> Result<Option<u64>, StorageError> {
        let data = self.db.get(CF_METADATA, b"latest_height")?;

        match data {
            Some(bytes) => {
                let height: u64 = bincode::deserialize(&bytes)?;
                Ok(Some(height))
            }
            None => Ok(None),
        }
    }

    /// 更新最新高度
    fn update_latest_height(&self, height: u64) -> Result<(), StorageError> {
        let data = bincode::serialize(&height)?;
        self.db.put(CF_METADATA, b"latest_height", &data)?;
        Ok(())
    }

    /// 获取最新区块
    pub fn get_latest_block(&self) -> Result<Option<Block>, StorageError> {
        let height = self.get_latest_height()?;
        match height {
            Some(h) => self.get_block_by_height(h),
            None => Ok(None),
        }
    }

    /// 检查区块是否存在
    pub fn block_exists(&self, height: u64) -> Result<bool, StorageError> {
        let key = height.to_be_bytes().to_vec();
        self.db.exists(CF_BLOCKS, &key)
    }

    /// 获取区块数量
    pub fn block_count(&self) -> Result<u64, StorageError> {
        let height = self.get_latest_height()?;
        Ok(height.unwrap_or(0) + 1)
    }

    /// 获取区块范围
    pub fn get_blocks_range(
        &self,
        start: u64,
        end: u64,
    ) -> Result<Vec<Block>, StorageError> {
        let mut blocks = Vec::new();
        for height in start..=end {
            if let Some(block) = self.get_block_by_height(height)? {
                blocks.push(block);
            }
        }
        Ok(blocks)
    }

    /// 获取创世区块时间戳
    /// 
    /// # 返回
    /// - Some(timestamp): 创世区块的时间戳（秒）
    /// - None: 创世区块不存在
    pub fn get_genesis_timestamp(&self) -> Result<Option<u64>, StorageError> {
        match self.get_block_by_height(0)? {
            Some(genesis) => Ok(Some(genesis.header.timestamp.timestamp() as u64)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use toki_core::{Address, Hash};

    #[test]
    fn test_block_store() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();
        let store = BlockStore::new(Arc::new(db));

        // 创建测试区块
        let block = Block::new(0, Hash::ZERO, vec![], 1000, Address::ZERO);

        // 保存
        store.save_block(&block).unwrap();

        // 读取
        let loaded = store.get_block_by_height(0).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().height(), 0);

        // 检查最新高度
        let height = store.get_latest_height().unwrap();
        assert_eq!(height, Some(0));
    }
}
