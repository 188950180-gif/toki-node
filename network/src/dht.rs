//! DHT 节点发现模块

use std::collections::HashMap;

use tracing::debug;

/// DHT 存储的记录类型
#[derive(Debug, Clone)]
pub enum DhtRecord {
    /// 节点地址
    NodeAddress(String),
    /// 区块哈希
    BlockHash(u64, Vec<u8>),
    /// 交易索引
    TransactionIndex(Vec<u8>, u64),
}

/// DHT 管理器
pub struct DhtManager {
    /// 本地缓存
    cache: HashMap<Vec<u8>, Vec<u8>>,
}

impl DhtManager {
    pub fn new() -> Self {
        DhtManager {
            cache: HashMap::new(),
        }
    }

    /// 存储记录
    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) {
        debug!(
            "DHT 存储: key={} bytes, value={} bytes",
            key.len(),
            value.len()
        );
        self.cache.insert(key, value);
    }

    /// 获取记录
    pub fn get(&self, key: &[u8]) -> Option<&Vec<u8>> {
        self.cache.get(key)
    }

    /// 删除记录
    pub fn remove(&mut self, key: &[u8]) {
        self.cache.remove(key);
    }

    /// 获取缓存大小
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

impl Default for DhtManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 创建区块高度到哈希的 DHT 键
pub fn block_hash_key(height: u64) -> Vec<u8> {
    format!("block:{}", height).into_bytes()
}

/// 创建交易 ID 到位置的 DHT 键
pub fn tx_index_key(tx_hash: &[u8]) -> Vec<u8> {
    let mut key = b"tx:".to_vec();
    key.extend_from_slice(tx_hash);
    key
}
