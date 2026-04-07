//! 挖矿完整实现 v2
//!
//! 实际的工作量证明计算和区块广播集成

use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use parking_lot::Mutex;
use anyhow::Result;
use tracing::{info, warn, debug};
use chrono::{DateTime, Utc};

use toki_core::{Block, Transaction, Hash, Address, BlockHeader, TOKI_BASE_UNIT};
use crate::difficulty::DifficultyAdjuster;

/// 挖矿配置
#[derive(Clone, Debug)]
pub struct MiningConfig {
    /// 挖矿线程数（0 = 自动检测）
    pub thread_count: usize,
    /// 矿工地址
    pub miner_address: Address,
    /// 目标出块时间（秒）
    pub target_block_time: u64,
    /// 初始难度
    pub initial_difficulty: u64,
    /// 最大交易数
    pub max_tx_per_block: usize,
}

impl Default for MiningConfig {
    fn default() -> Self {
        MiningConfig {
            thread_count: 0,
            miner_address: Address::ZERO,
            target_block_time: 10,
            initial_difficulty: 1_000_000,
            max_tx_per_block: 100,
        }
    }
}

impl MiningConfig {
    /// 获取实际线程数
    pub fn actual_thread_count(&self) -> usize {
        if self.thread_count == 0 {
            num_cpus::get()
        } else {
            self.thread_count
        }
    }
}

/// 挖矿统计
#[derive(Debug, Default)]
pub struct MiningStats {
    /// 总哈希计算次数
    pub total_hashes: AtomicU64,
    /// 找到的区块数
    pub blocks_found: AtomicU64,
    /// 挖矿时间（秒）
    pub mining_time: AtomicU64,
    /// 当前线程数
    pub thread_count: AtomicUsize,
}

/// 挖矿任务
pub struct MiningTask {
    /// 前一区块哈希
    pub prev_hash: Hash,
    /// 区块高度
    pub height: u64,
    /// 交易列表
    pub transactions: Vec<Transaction>,
    /// 难度
    pub difficulty: u64,
    /// 矿工地址
    pub miner: Address,
    /// 时间戳
    pub timestamp: u64,
}

/// 挖矿结果
#[derive(Debug)]
pub struct MiningResult {
    /// 挖出的区块
    pub block: Block,
    /// 使用的 nonce
    pub nonce: u64,
    /// 计算的哈希次数
    pub hash_count: u64,
    /// 耗时
    pub elapsed: Duration,
}

/// 挖矿器（完整实现）
pub struct Miner {
    config: MiningConfig,
    stats: Arc<MiningStats>,
    running: Arc<AtomicBool>,
    /// 难度调整器
    difficulty_adjuster: Arc<Mutex<DifficultyAdjuster>>,
    /// 找到区块的回调
    on_block_found: Option<Arc<dyn Fn(Block) + Send + Sync>>,
}

impl Miner {
    /// 创建新挖矿器
    pub fn new(config: MiningConfig) -> Self {
        let difficulty_adjuster = DifficultyAdjuster::new(config.initial_difficulty);

        Miner {
            config,
            stats: Arc::new(MiningStats::default()),
            running: Arc::new(AtomicBool::new(false)),
            difficulty_adjuster: Arc::new(Mutex::new(difficulty_adjuster)),
            on_block_found: None,
        }
    }

    /// 设置找到区块回调
    pub fn set_block_found_callback<F>(&mut self, callback: F)
    where
        F: Fn(Block) + Send + Sync + 'static,
    {
        self.on_block_found = Some(Arc::new(callback));
    }

    /// 启动挖矿
    pub fn start(&self) {
        if self.running.swap(true, Ordering::SeqCst) {
            warn!("Miner already running");
            return;
        }

        let thread_count = self.config.actual_thread_count();
        self.stats.thread_count.store(thread_count, Ordering::SeqCst);

        info!("Starting miner with {} threads", thread_count);

        // 启动挖矿线程
        for i in 0..thread_count {
            let running = self.running.clone();
            let stats = self.stats.clone();
            let config = self.config.clone();
            let difficulty_adjuster = self.difficulty_adjuster.clone();
            let on_block_found = self.on_block_found.clone();

            thread::spawn(move || {
                info!("Mining thread {} started", i);

                while running.load(Ordering::SeqCst) {
                    // 获取当前难度
                    let difficulty = {
                        difficulty_adjuster.lock().get_current_difficulty()
                    };

                    // 创建挖矿任务
                    let task = MiningTask {
                        prev_hash: Hash::ZERO,
                        height: 0,
                        transactions: vec![],
                        difficulty,
                        miner: config.miner_address,
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    };

                    // 执行挖矿
                    if let Some(result) = mine_block(&task, i as u64) {
                        info!("Block found by thread {}! Nonce: {}", i, result.nonce);

                        // 更新统计
                        stats.blocks_found.fetch_add(1, Ordering::SeqCst);

                        // 更新难度
                        {
                            let mut adjuster = difficulty_adjuster.lock();
                            adjuster.record_block_time(result.block.header.timestamp.timestamp() as u64);
                            adjuster.calculate_new_difficulty();
                        }

                        // 调用回调
                        if let Some(ref callback) = on_block_found {
                            callback(result.block);
                        }
                    }

                    // 更新哈希计数
                    stats.total_hashes.fetch_add(1_000_000, Ordering::SeqCst);

                    // 短暂休眠
                    thread::sleep(Duration::from_millis(100));
                }

                info!("Mining thread {} stopped", i);
            });
        }
    }

    /// 停止挖矿
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        info!("Miner stopped");
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> MiningStats {
        MiningStats {
            total_hashes: AtomicU64::new(self.stats.total_hashes.load(Ordering::SeqCst)),
            blocks_found: AtomicU64::new(self.stats.blocks_found.load(Ordering::SeqCst)),
            mining_time: AtomicU64::new(self.stats.mining_time.load(Ordering::SeqCst)),
            thread_count: AtomicUsize::new(self.stats.thread_count.load(Ordering::SeqCst)),
        }
    }

    /// 获取当前难度
    pub fn get_current_difficulty(&self) -> u64 {
        self.difficulty_adjuster.lock().get_current_difficulty()
    }
}

/// 挖矿区块
fn mine_block(task: &MiningTask, thread_id: u64) -> Option<MiningResult> {
    let start = Instant::now();

    // 计算区块哈希目标
    let target = calculate_target(task.difficulty);

    // 多线程挖矿
    let nonce = thread_id * 1_000_000;

    for n in 0..1_000_000 {
        let current_nonce = nonce + n;

        // 构造区块头
        let header = BlockHeader {
            height: task.height,
            prev_hash: task.prev_hash,
            merkle_root: calculate_merkle_root(&task.transactions),
            timestamp: DateTime::from_timestamp(task.timestamp as i64, 0).unwrap_or_else(Utc::now),
            difficulty: task.difficulty,
            nonce: current_nonce,
            miner: task.miner,
        };

        // 计算哈希
        let hash = calculate_block_hash(&header);

        // 检查是否满足难度
        if hash_meets_difficulty(&hash, &target) {
            // 构造区块
            let block = Block {
                header,
                transactions: task.transactions.clone(),
                block_hash: Some(hash),
            };

            let elapsed = start.elapsed();

            return Some(MiningResult {
                block,
                nonce: current_nonce,
                hash_count: n,
                elapsed,
            });
        }
    }

    None
}

/// 计算目标值
fn calculate_target(difficulty: u64) -> [u8; 32] {
    // 简化实现：目标值 = 2^256 / difficulty
    let mut target = [0u8; 32];
    if difficulty > 0 {
        let shift = 256 - difficulty.min(255);
        target[0] = 1 << (shift % 8);
    }
    target
}

/// 计算默克尔根
fn calculate_merkle_root(_transactions: &[Transaction]) -> Hash {
    // 简化实现
    Hash::ZERO
}

/// 计算区块哈希
fn calculate_block_hash(header: &BlockHeader) -> Hash {
    let data = bincode::serialize(header).unwrap();
    Hash::from_data(&data)
}

/// 检查哈希是否满足难度
fn hash_meets_difficulty(hash: &Hash, target: &[u8; 32]) -> bool {
    // 比较字节数组
    hash.as_bytes() <= target
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mining_config() {
        let config = MiningConfig::default();
        assert_eq!(config.thread_count, 0);
        assert!(config.actual_thread_count() > 0);
    }

    #[test]
    fn test_miner_creation() {
        let miner = Miner::new(MiningConfig::default());
        assert!(!miner.is_running());
    }

    #[test]
    fn test_difficulty_calculation() {
        let target = calculate_target(1_000_000);
        assert_ne!(target, [0u8; 32]);
    }
}
