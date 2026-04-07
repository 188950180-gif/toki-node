//! 挖矿与网络广播集成
//!
//! 实现挖矿成功后自动广播区块到网络

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

use toki_core::{Block, Hash, Transaction};

/// 挖矿与广播事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MiningBroadcastEvent {
    /// 区块已挖出，准备广播
    BlockMined {
        block: Block,
        miner: String,
        timestamp: u64,
    },
    /// 区块广播成功
    BroadcastSuccess { block_hash: Hash, peer_count: usize },
    /// 区块广播失败
    BroadcastFailed { block_hash: Hash, error: String },
    /// 收到远程区块
    RemoteBlockReceived { block: Block, from_peer: String },
    /// 挖矿状态变更
    MiningStateChanged { running: bool, thread_count: usize },
}

/// 挖矿广播配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningBroadcastConfig {
    /// 挖矿线程数
    pub thread_count: usize,
    /// 目标出块时间（秒）
    pub target_block_time: u64,
    /// 初始难度
    pub initial_difficulty: u64,
    /// 最大广播重试次数
    pub max_broadcast_retries: u32,
    /// 广播超时（秒）
    pub broadcast_timeout: u64,
    /// 是否启用自动广播
    pub auto_broadcast: bool,
}

impl Default for MiningBroadcastConfig {
    fn default() -> Self {
        MiningBroadcastConfig {
            thread_count: 0, // 自动检测
            target_block_time: 10,
            initial_difficulty: 1_000_000,
            max_broadcast_retries: 3,
            broadcast_timeout: 5,
            auto_broadcast: true,
        }
    }
}

/// 挖矿广播统计
#[derive(Debug, Default)]
pub struct MiningBroadcastStats {
    /// 挖出的区块数
    pub blocks_mined: AtomicU64,
    /// 广播成功的区块数
    pub broadcast_success: AtomicU64,
    /// 广播失败的区块数
    pub broadcast_failed: AtomicU64,
    /// 收到的远程区块数
    pub remote_blocks_received: AtomicU64,
    /// 总哈希计算次数
    pub total_hashes: AtomicU64,
}

/// 挖矿广播集成器
pub struct MiningBroadcastIntegration {
    /// 配置
    config: MiningBroadcastConfig,
    /// 统计
    stats: Arc<MiningBroadcastStats>,
    /// 事件发送器
    event_sender: Sender<MiningBroadcastEvent>,
    /// 事件接收器
    event_receiver: Receiver<MiningBroadcastEvent>,
    /// 是否运行中
    running: Arc<AtomicBool>,
    /// 当前难度
    current_difficulty: Arc<AtomicU64>,
    /// 当前高度
    current_height: Arc<AtomicU64>,
}

impl MiningBroadcastIntegration {
    /// 创建新的挖矿广播集成器
    pub fn new(config: MiningBroadcastConfig) -> Self {
        let (sender, receiver) = channel();
        let thread_count = if config.thread_count == 0 {
            num_cpus::get()
        } else {
            config.thread_count
        };

        MiningBroadcastIntegration {
            config: MiningBroadcastConfig {
                thread_count,
                ..config
            },
            stats: Arc::new(MiningBroadcastStats::default()),
            event_sender: sender,
            event_receiver: receiver,
            running: Arc::new(AtomicBool::new(false)),
            current_difficulty: Arc::new(AtomicU64::new(config.initial_difficulty)),
            current_height: Arc::new(AtomicU64::new(0)),
        }
    }

    /// 启动挖矿与广播
    pub fn start(&self) -> Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            warn!("挖矿广播集成器已在运行");
            return Ok(());
        }

        info!("启动挖矿广播集成器，线程数: {}", self.config.thread_count);

        // 发送状态变更事件
        let _ = self
            .event_sender
            .send(MiningBroadcastEvent::MiningStateChanged {
                running: true,
                thread_count: self.config.thread_count,
            });

        Ok(())
    }

    /// 停止挖矿与广播
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        info!("停止挖矿广播集成器");

        let _ = self
            .event_sender
            .send(MiningBroadcastEvent::MiningStateChanged {
                running: false,
                thread_count: 0,
            });
    }

    /// 处理挖出的区块
    pub fn on_block_mined(&self, block: &Block, miner: &str) -> Result<()> {
        self.stats.blocks_mined.fetch_add(1, Ordering::SeqCst);

        info!(
            "区块挖出: 高度={}, 哈希={}",
            block.header.height,
            block.hash()
        );

        // 发送挖矿事件
        let _ = self.event_sender.send(MiningBroadcastEvent::BlockMined {
            block: block.clone(),
            miner: miner.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        // 自动广播
        if self.config.auto_broadcast {
            self.broadcast_block(block)?;
        }

        Ok(())
    }

    /// 广播区块到网络
    pub fn broadcast_block(&self, block: &Block) -> Result<()> {
        let block_hash = block.hash();
        info!("广播区块: {}", block_hash);

        // 模拟广播（实际实现需要连接 P2P 网络）
        let peer_count = self.simulate_broadcast(block)?;

        if peer_count > 0 {
            self.stats.broadcast_success.fetch_add(1, Ordering::SeqCst);
            let _ = self
                .event_sender
                .send(MiningBroadcastEvent::BroadcastSuccess {
                    block_hash,
                    peer_count,
                });
        } else {
            self.stats.broadcast_failed.fetch_add(1, Ordering::SeqCst);
            let _ = self
                .event_sender
                .send(MiningBroadcastEvent::BroadcastFailed {
                    block_hash,
                    error: "没有连接的节点".to_string(),
                });
        }

        Ok(())
    }

    /// 模拟广播（实际实现需要 P2P 网络）
    fn simulate_broadcast(&self, _block: &Block) -> Result<usize> {
        // TODO: 实际实现需要通过 P2P 网络广播
        // 这里返回模拟的节点数
        Ok(10)
    }

    /// 处理收到的远程区块
    pub fn on_remote_block_received(&self, block: &Block, from_peer: &str) -> Result<()> {
        self.stats
            .remote_blocks_received
            .fetch_add(1, Ordering::SeqCst);

        debug!(
            "收到远程区块: 高度={}, 来自={}",
            block.header.height, from_peer
        );

        let _ = self
            .event_sender
            .send(MiningBroadcastEvent::RemoteBlockReceived {
                block: block.clone(),
                from_peer: from_peer.to_string(),
            });

        Ok(())
    }

    /// 获取事件接收器
    pub fn events(&self) -> &Receiver<MiningBroadcastEvent> {
        &self.event_receiver
    }

    /// 获取统计信息
    pub fn stats(&self) -> MiningBroadcastStatsSnapshot {
        MiningBroadcastStatsSnapshot {
            blocks_mined: self.stats.blocks_mined.load(Ordering::SeqCst),
            broadcast_success: self.stats.broadcast_success.load(Ordering::SeqCst),
            broadcast_failed: self.stats.broadcast_failed.load(Ordering::SeqCst),
            remote_blocks_received: self.stats.remote_blocks_received.load(Ordering::SeqCst),
            total_hashes: self.stats.total_hashes.load(Ordering::SeqCst),
        }
    }

    /// 是否运行中
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// 获取当前难度
    pub fn current_difficulty(&self) -> u64 {
        self.current_difficulty.load(Ordering::SeqCst)
    }

    /// 获取当前高度
    pub fn current_height(&self) -> u64 {
        self.current_height.load(Ordering::SeqCst)
    }

    /// 更新难度
    pub fn update_difficulty(&self, new_difficulty: u64) {
        self.current_difficulty
            .store(new_difficulty, Ordering::SeqCst);
        info!("难度更新: {}", new_difficulty);
    }

    /// 更新高度
    pub fn update_height(&self, new_height: u64) {
        self.current_height.store(new_height, Ordering::SeqCst);
    }
}

/// 统计快照
#[derive(Debug, Clone)]
pub struct MiningBroadcastStatsSnapshot {
    pub blocks_mined: u64,
    pub broadcast_success: u64,
    pub broadcast_failed: u64,
    pub remote_blocks_received: u64,
    pub total_hashes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mining_broadcast_creation() {
        let config = MiningBroadcastConfig::default();
        let integration = MiningBroadcastIntegration::new(config);

        assert!(!integration.is_running());
        assert!(integration.config.thread_count > 0);
    }

    #[test]
    fn test_mining_broadcast_start_stop() {
        let config = MiningBroadcastConfig::default();
        let integration = MiningBroadcastIntegration::new(config);

        integration.start().unwrap();
        assert!(integration.is_running());

        integration.stop();
        assert!(!integration.is_running());
    }

    #[test]
    fn test_stats() {
        let config = MiningBroadcastConfig::default();
        let integration = MiningBroadcastIntegration::new(config);

        let stats = integration.stats();
        assert_eq!(stats.blocks_mined, 0);
        assert_eq!(stats.broadcast_success, 0);
    }
}
