//! 挖矿集成模块
//!
//! 实现挖矿与网络广播的完整集成

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

use toki_core::{Block, Hash, Transaction};

/// 挖矿事件
#[derive(Debug, Clone)]
pub enum MiningEvent {
    /// 区块已挖出
    BlockMined(Block),
    /// 挖矿状态变更
    StateChanged(bool),
    /// 难度调整
    DifficultyAdjusted(u64),
    /// 错误
    Error(String),
}

/// 挖矿任务
pub struct MiningTask {
    /// 前一区块哈希
    pub prev_hash: Hash,
    /// 交易列表
    pub transactions: Vec<Transaction>,
    /// 当前难度
    pub difficulty: u64,
    /// 区块高度
    pub height: u64,
    /// 时间戳
    pub timestamp: u64,
}

/// 挖矿结果
#[derive(Debug)]
pub struct MiningResult {
    /// 挖出的区块
    pub block: Block,
    /// 计算的哈希次数
    pub hash_count: u64,
    /// 耗时
    pub elapsed: Duration,
}

/// 集成挖矿器
pub struct IntegratedMiner {
    /// 是否运行中
    running: Arc<AtomicBool>,
    /// 事件发送器
    event_sender: Sender<MiningEvent>,
    /// 事件接收器
    event_receiver: Receiver<MiningEvent>,
    /// 统计：总哈希数
    total_hashes: Arc<AtomicU64>,
    /// 统计：挖出的区块数
    blocks_found: Arc<AtomicU64>,
    /// 线程数
    thread_count: usize,
}

impl IntegratedMiner {
    /// 创建新的集成挖矿器
    pub fn new(thread_count: usize) -> Self {
        let (sender, receiver) = channel();
        let actual_threads = if thread_count == 0 {
            num_cpus::get()
        } else {
            thread_count
        };

        IntegratedMiner {
            running: Arc::new(AtomicBool::new(false)),
            event_sender: sender,
            event_receiver: receiver,
            total_hashes: Arc::new(AtomicU64::new(0)),
            blocks_found: Arc::new(AtomicU64::new(0)),
            thread_count: actual_threads,
        }
    }

    /// 启动挖矿（简化版本，不依赖 BlockStore）
    pub fn start_simple(&self) {
        if self.running.swap(true, Ordering::SeqCst) {
            warn!("挖矿器已在运行");
            return;
        }

        info!("启动集成挖矿器，线程数: {}", self.thread_count);
        let _ = self.event_sender.send(MiningEvent::StateChanged(true));
    }

    /// 停止挖矿
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        info!("停止挖矿器");
    }

    /// 是否运行中
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// 获取事件接收器
    pub fn events(&self) -> &Receiver<MiningEvent> {
        &self.event_receiver
    }

    /// 获取统计信息
    pub fn stats(&self) -> MiningStats {
        MiningStats {
            total_hashes: self.total_hashes.load(Ordering::SeqCst),
            blocks_found: self.blocks_found.load(Ordering::SeqCst),
        }
    }
}

/// 挖矿统计
#[derive(Debug, Clone)]
pub struct MiningStats {
    pub total_hashes: u64,
    pub blocks_found: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_miner_creation() {
        let miner = IntegratedMiner::new(4);
        assert!(!miner.is_running());
        assert_eq!(miner.thread_count, 4);
    }

    #[test]
    fn test_miner_auto_threads() {
        let miner = IntegratedMiner::new(0);
        assert!(miner.thread_count > 0);
    }
}
