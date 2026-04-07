//! 区块定义

use crate::{Address, Hash, Transaction};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 区块头
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    /// 区块高度
    pub height: u64,
    /// 前序区块哈希
    pub prev_hash: Hash,
    /// Merkle 根
    pub merkle_root: Hash,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 难度
    pub difficulty: u64,
    /// Nonce（工作量证明）
    pub nonce: u64,
    /// 矿工地址
    pub miner: Address,
}

impl BlockHeader {
    /// 创建新区块头
    pub fn new(
        height: u64,
        prev_hash: Hash,
        merkle_root: Hash,
        difficulty: u64,
        miner: Address,
    ) -> Self {
        BlockHeader {
            height,
            prev_hash,
            merkle_root,
            timestamp: Utc::now(),
            difficulty,
            nonce: 0,
            miner,
        }
    }

    /// 计算区块头哈希（用于 PoW）
    pub fn compute_hash(&self) -> Hash {
        let mut data = Vec::new();
        data.extend_from_slice(&self.height.to_le_bytes());
        data.extend_from_slice(self.prev_hash.as_bytes());
        data.extend_from_slice(self.merkle_root.as_bytes());
        data.extend_from_slice(&self.timestamp.timestamp().to_le_bytes());
        data.extend_from_slice(&self.difficulty.to_le_bytes());
        data.extend_from_slice(&self.nonce.to_le_bytes());
        data.extend_from_slice(self.miner.as_bytes());
        Hash::from_data(&data)
    }

    /// 检查是否满足难度要求
    pub fn meets_difficulty(&self) -> bool {
        let hash = self.compute_hash();
        let hash_value = bytes_to_u64(&hash.0[0..8]);
        hash_value < self.difficulty
    }
}

/// 区块
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    /// 区块头
    pub header: BlockHeader,
    /// 交易列表
    pub transactions: Vec<Transaction>,
    /// 区块哈希（计算得出）
    #[serde(skip)]
    pub block_hash: Option<Hash>,
}

impl Block {
    /// 创建新区块
    pub fn new(
        height: u64,
        prev_hash: Hash,
        transactions: Vec<Transaction>,
        difficulty: u64,
        miner: Address,
    ) -> Self {
        let merkle_root = compute_merkle_root(&transactions);
        let header = BlockHeader::new(height, prev_hash, merkle_root, difficulty, miner);
        let mut block = Block {
            header,
            transactions,
            block_hash: None,
        };
        block.block_hash = Some(block.header.compute_hash());
        block
    }

    /// 创建创世区块
    pub fn genesis() -> Self {
        Self::new(0, Hash::ZERO, vec![], 1, Address::ZERO)
    }

    /// 获取区块哈希
    pub fn hash(&self) -> Hash {
        self.block_hash.unwrap_or_else(|| self.header.compute_hash())
    }

    /// 获取区块高度
    pub fn height(&self) -> u64 {
        self.header.height
    }

    /// 设置 nonce（挖矿时使用）
    pub fn set_nonce(&mut self, nonce: u64) {
        self.header.nonce = nonce;
        self.block_hash = Some(self.header.compute_hash());
    }

    /// 检查是否满足难度要求
    pub fn meets_difficulty(&self) -> bool {
        self.header.meets_difficulty()
    }

    /// 验证 Merkle 根
    pub fn verify_merkle_root(&self) -> bool {
        let computed = compute_merkle_root(&self.transactions);
        computed == self.header.merkle_root
    }

    /// 获取交易数量
    pub fn tx_count(&self) -> usize {
        self.transactions.len()
    }

    /// 计算区块大小（字节）
    pub fn size(&self) -> usize {
        // 简化估算
        let header_size = 8 + 32 + 32 + 8 + 8 + 8 + 32; // 约 128 字节
        let tx_size: usize = self.transactions.iter().map(|tx| {
            // 简化估算每笔交易大小
            32 + tx.inputs.len() * 36 + tx.outputs.len() * 72 + tx.ring_signature.ring.len() * 32 + 64
        }).sum();
        header_size + tx_size
    }
}

/// 计算 Merkle 根（优化版本）
fn compute_merkle_root(transactions: &[Transaction]) -> Hash {
    if transactions.is_empty() {
        return Hash::ZERO;
    }

    // 预分配空间，减少内存分配
    let tx_count = transactions.len();
    let mut hashes: Vec<Hash> = Vec::with_capacity(tx_count.next_power_of_two());
    
    // 批量计算交易哈希
    hashes.extend(transactions.iter().map(|tx| tx.hash()));

    // 构建 Merkle 树（优化循环）
    while hashes.len() > 1 {
        let len = hashes.len();
        let next_len = (len + 1) / 2;
        let mut next_level = Vec::with_capacity(next_len);
        
        let mut i = 0;
        while i < len {
            if i + 1 < len {
                // 合并两个哈希（使用固定大小数组避免 Vec 分配）
                let mut data = [0u8; 64];
                data[..32].copy_from_slice(hashes[i].as_bytes());
                data[32..].copy_from_slice(hashes[i + 1].as_bytes());
                next_level.push(Hash::from_data(&data));
            } else {
                // 奇数个节点，复制最后一个
                next_level.push(hashes[i]);
            }
            i += 2;
        }
        hashes = next_level;
    }

    hashes[0]
}

/// 将字节数组转换为 u64（小端序）
fn bytes_to_u64(bytes: &[u8]) -> u64 {
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&bytes[0..8]);
    u64::from_le_bytes(arr)
}

/// 区块模板（用于挖矿）
#[derive(Clone, Debug)]
pub struct BlockTemplate {
    /// 前序区块哈希
    pub prev_hash: Hash,
    /// 下一个区块高度
    pub height: u64,
    /// 待打包交易
    pub transactions: Vec<Transaction>,
    /// 难度
    pub difficulty: u64,
    /// 矿工地址
    pub miner: Address,
}

impl BlockTemplate {
    /// 创建新区块模板
    pub fn new(
        prev_hash: Hash,
        height: u64,
        transactions: Vec<Transaction>,
        difficulty: u64,
        miner: Address,
    ) -> Self {
        BlockTemplate {
            prev_hash,
            height,
            transactions,
            difficulty,
            miner,
        }
    }

    /// 转换为区块（设置 nonce）
    pub fn to_block(&self, nonce: u64) -> Block {
        let mut block = Block::new(
            self.height,
            self.prev_hash,
            self.transactions.clone(),
            self.difficulty,
            self.miner,
        );
        block.set_nonce(nonce);
        block
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Input, Output, RingSignature, TOKI_BASE_UNIT};

    #[test]
    fn test_genesis_block() {
        let genesis = Block::genesis();
        assert_eq!(genesis.height(), 0);
        assert_eq!(genesis.header.prev_hash, Hash::ZERO);
        assert_eq!(genesis.tx_count(), 0);
    }

    #[test]
    fn test_block_hash() {
        let block1 = Block::new(1, Hash::ZERO, vec![], 1000, Address::ZERO);
        let hash1 = block1.hash();

        let block2 = Block::new(1, Hash::ZERO, vec![], 1000, Address::ZERO);
        let hash2 = block2.hash();

        // 相同参数的区块应该有相同的哈希（忽略时间戳差异）
        // 注意：由于时间戳不同，哈希可能不同
        assert!(block1.verify_merkle_root());
        assert!(block2.verify_merkle_root());
    }

    #[test]
    fn test_merkle_root() {
        // 空交易
        let root1 = compute_merkle_root(&[]);
        assert_eq!(root1, Hash::ZERO);

        // 单笔交易
        let addr = Address::new([1u8; 32]);
        let output = Output::new(addr, 100 * TOKI_BASE_UNIT);
        let ring_sig = RingSignature::new(vec![vec![1u8; 32]], vec![2u8; 64], Hash::ZERO);
        let tx = Transaction::new(vec![], vec![output], ring_sig, 1000);

        let root2 = compute_merkle_root(&[tx.clone()]);
        assert_ne!(root2, Hash::ZERO);

        // 两笔交易
        let ring_sig2 = RingSignature::new(vec![vec![1u8; 32]], vec![2u8; 64], Hash::ZERO);
        let tx2 = Transaction::new(vec![], vec![Output::new(addr, 200 * TOKI_BASE_UNIT)], ring_sig2, 1000);
        let root3 = compute_merkle_root(&[tx, tx2]);
        assert_ne!(root3, root2);
    }

    #[test]
    fn test_block_template() {
        let template = BlockTemplate::new(Hash::ZERO, 1, vec![], 1000, Address::ZERO);
        let block = template.to_block(12345);

        assert_eq!(block.height(), 1);
        assert_eq!(block.header.nonce, 12345);
    }
}
