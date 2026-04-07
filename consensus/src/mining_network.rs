//! 挖矿与网络完整集成
//!
//! 实现挖矿成功后自动广播到 P2P 网络

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

use toki_core::{Block, Hash, Transaction};

/// 网络广播接口
pub trait NetworkBroadcaster: Send + Sync {
    /// 广播区块到网络
    fn broadcast_block(&self, block: &Block) -> Result<BroadcastResult>;

    /// 广播交易到网络
    fn broadcast_transaction(&self, tx: &Transaction) -> Result<BroadcastResult>;

    /// 获取连接的节点数
    fn peer_count(&self) -> usize;
}

/// 广播结果
#[derive(Debug, Clone)]
pub struct BroadcastResult {
    /// 成功广播的节点数
    pub success_count: usize,
    /// 失败的节点数
    pub failed_count: usize,
    /// 总耗时（毫秒）
    pub duration_ms: u64,
}

/// 挖矿网络集成事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MiningNetworkEvent {
    /// 挖矿开始
    MiningStarted {
        thread_count: usize,
        difficulty: u64,
    },
    /// 区块挖出
    BlockMined {
        height: u64,
        hash: Hash,
        miner: String,
        nonce: u64,
        timestamp: u64,
    },
    /// 广播开始
    BroadcastStarted { block_hash: Hash, peer_count: usize },
    /// 广播完成
    BroadcastCompleted {
        block_hash: Hash,
        success: usize,
        failed: usize,
        duration_ms: u64,
    },
    /// 区块被远程节点接受
    BlockAccepted { block_hash: Hash, by_peer: String },
    /// 挖矿停止
    MiningStopped,
    /// 错误
    Error { message: String },
}

/// 挖矿网络集成配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningNetworkConfig {
    /// 挖矿线程数
    pub thread_count: usize,
    /// 目标出块时间
    pub target_block_time: u64,
    /// 初始难度
    pub initial_difficulty: u64,
    /// 最大广播重试
    pub max_broadcast_retries: u32,
    /// 广播超时
    pub broadcast_timeout_ms: u64,
    /// 是否自动广播
    pub auto_broadcast: bool,
    /// 是否验证广播结果
    pub verify_broadcast: bool,
    /// 最小确认节点数
    pub min_confirmations: usize,
}

impl Default for MiningNetworkConfig {
    fn default() -> Self {
        MiningNetworkConfig {
            thread_count: 0,
            target_block_time: 10,
            initial_difficulty: 1_000_000,
            max_broadcast_retries: 3,
            broadcast_timeout_ms: 5000,
            auto_broadcast: true,
            verify_broadcast: true,
            min_confirmations: 3,
        }
    }
}

/// 挖矿网络集成器
pub struct MiningNetworkIntegration {
    /// 配置
    config: MiningNetworkConfig,
    /// 网络广播器
    broadcaster: Option<Arc<dyn NetworkBroadcaster>>,
    /// 事件发送器
    event_sender: Sender<MiningNetworkEvent>,
    /// 事件接收器
    event_receiver: Receiver<MiningNetworkEvent>,
    /// 运行状态
    running: Arc<AtomicBool>,
    /// 当前难度
    current_difficulty: Arc<AtomicU64>,
    /// 当前高度
    current_height: Arc<AtomicU64>,
    /// 挖出的区块数
    blocks_mined: Arc<AtomicU64>,
    /// 广播成功数
    broadcast_success: Arc<AtomicU64>,
    /// 广播失败数
    broadcast_failed: Arc<AtomicU64>,
}

impl MiningNetworkIntegration {
    /// 创建新的集成器
    pub fn new(config: MiningNetworkConfig) -> Self {
        let (sender, receiver) = channel();
        let threads = if config.thread_count == 0 {
            num_cpus::get()
        } else {
            config.thread_count
        };

        MiningNetworkIntegration {
            config: MiningNetworkConfig {
                thread_count: threads,
                ..config
            },
            broadcaster: None,
            event_sender: sender,
            event_receiver: receiver,
            running: Arc::new(AtomicBool::new(false)),
            current_difficulty: Arc::new(AtomicU64::new(config.initial_difficulty)),
            current_height: Arc::new(AtomicU64::new(0)),
            blocks_mined: Arc::new(AtomicU64::new(0)),
            broadcast_success: Arc::new(AtomicU64::new(0)),
            broadcast_failed: Arc::new(AtomicU64::new(0)),
        }
    }

    /// 设置网络广播器
    pub fn set_broadcaster(&mut self, broadcaster: Arc<dyn NetworkBroadcaster>) {
        self.broadcaster = Some(broadcaster);
    }

    /// 启动挖矿
    pub fn start(&self) -> Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            warn!("挖矿已在运行");
            return Ok(());
        }

        info!("启动挖矿网络集成，线程数: {}", self.config.thread_count);

        let _ = self.event_sender.send(MiningNetworkEvent::MiningStarted {
            thread_count: self.config.thread_count,
            difficulty: self.current_difficulty.load(Ordering::SeqCst),
        });

        Ok(())
    }

    /// 停止挖矿
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        info!("停止挖矿");
        let _ = self.event_sender.send(MiningNetworkEvent::MiningStopped);
    }

    /// 处理挖出的区块
    pub fn on_block_mined(&self, block: &Block, miner: &str) -> Result<bool> {
        let block_hash = block.hash();
        let height = block.header.height;

        info!("🎉 区块挖出! 高度={}, 哈希={}", height, block_hash);

        // 更新统计
        self.blocks_mined.fetch_add(1, Ordering::SeqCst);
        self.current_height.store(height, Ordering::SeqCst);

        // 发送事件
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let _ = self.event_sender.send(MiningNetworkEvent::BlockMined {
            height,
            hash: block_hash,
            miner: miner.to_string(),
            nonce: block.header.nonce,
            timestamp,
        });

        // 自动广播
        if self.config.auto_broadcast {
            self.broadcast_block_with_retry(block)
        } else {
            Ok(true)
        }
    }

    /// 广播区块（带重试）
    fn broadcast_block_with_retry(&self, block: &Block) -> Result<bool> {
        let block_hash = block.hash();

        if let Some(ref broadcaster) = self.broadcaster {
            let peer_count = broadcaster.peer_count();

            if peer_count == 0 {
                warn!("没有连接的节点，无法广播区块");
                return Ok(false);
            }

            let _ = self
                .event_sender
                .send(MiningNetworkEvent::BroadcastStarted {
                    block_hash,
                    peer_count,
                });

            // 重试广播
            for attempt in 0..self.config.max_broadcast_retries {
                debug!(
                    "广播区块 {} (尝试 {}/{})",
                    block_hash,
                    attempt + 1,
                    self.config.max_broadcast_retries
                );

                match broadcaster.broadcast_block(block) {
                    Ok(result) => {
                        if result.success_count > 0 {
                            self.broadcast_success.fetch_add(1, Ordering::SeqCst);

                            let _ =
                                self.event_sender
                                    .send(MiningNetworkEvent::BroadcastCompleted {
                                        block_hash,
                                        success: result.success_count,
                                        failed: result.failed_count,
                                        duration_ms: result.duration_ms,
                                    });

                            info!(
                                "✅ 区块广播成功: {} -> {} 节点",
                                block_hash, result.success_count
                            );
                            return Ok(true);
                        }
                    }
                    Err(e) => {
                        warn!("广播失败 (尝试 {}): {}", attempt + 1, e);
                    }
                }

                // 等待后重试
                std::thread::sleep(Duration::from_millis(100 * (attempt + 1) as u64));
            }

            // 所有重试都失败
            self.broadcast_failed.fetch_add(1, Ordering::SeqCst);
            error!("❌ 区块广播失败: {}", block_hash);

            let _ = self.event_sender.send(MiningNetworkEvent::Error {
                message: format!("广播失败: {}", block_hash),
            });

            Ok(false)
        } else {
            warn!("未设置网络广播器");
            Ok(false)
        }
    }

    /// 处理远程节点接受区块
    pub fn on_block_accepted(&self, block_hash: Hash, by_peer: &str) {
        debug!("区块 {} 被 {} 接受", block_hash, by_peer);

        let _ = self.event_sender.send(MiningNetworkEvent::BlockAccepted {
            block_hash,
            by_peer: by_peer.to_string(),
        });
    }

    /// 获取事件接收器
    pub fn events(&self) -> &Receiver<MiningNetworkEvent> {
        &self.event_receiver
    }

    /// 获取统计信息
    pub fn stats(&self) -> MiningNetworkStats {
        MiningNetworkStats {
            is_running: self.running.load(Ordering::SeqCst),
            current_height: self.current_height.load(Ordering::SeqCst),
            current_difficulty: self.current_difficulty.load(Ordering::SeqCst),
            blocks_mined: self.blocks_mined.load(Ordering::SeqCst),
            broadcast_success: self.broadcast_success.load(Ordering::SeqCst),
            broadcast_failed: self.broadcast_failed.load(Ordering::SeqCst),
            peer_count: self
                .broadcaster
                .as_ref()
                .map(|b| b.peer_count())
                .unwrap_or(0),
        }
    }

    /// 更新难度
    pub fn update_difficulty(&self, new_difficulty: u64) {
        self.current_difficulty
            .store(new_difficulty, Ordering::SeqCst);
        info!("难度更新: {}", new_difficulty);
    }
}

/// 挖矿网络统计
#[derive(Debug, Clone)]
pub struct MiningNetworkStats {
    pub is_running: bool,
    pub current_height: u64,
    pub current_difficulty: u64,
    pub blocks_mined: u64,
    pub broadcast_success: u64,
    pub broadcast_failed: u64,
    pub peer_count: usize,
}

/// 默认网络广播器（用于测试）
pub struct DefaultBroadcaster {
    peer_count: usize,
}

impl DefaultBroadcaster {
    pub fn new(peer_count: usize) -> Self {
        DefaultBroadcaster { peer_count }
    }
}

impl NetworkBroadcaster for DefaultBroadcaster {
    fn broadcast_block(&self, _block: &Block) -> Result<BroadcastResult> {
        Ok(BroadcastResult {
            success_count: self.peer_count,
            failed_count: 0,
            duration_ms: 10,
        })
    }

    fn broadcast_transaction(&self, _tx: &Transaction) -> Result<BroadcastResult> {
        Ok(BroadcastResult {
            success_count: self.peer_count,
            failed_count: 0,
            duration_ms: 5,
        })
    }

    fn peer_count(&self) -> usize {
        self.peer_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mining_network_integration() {
        let config = MiningNetworkConfig::default();
        let integration = MiningNetworkIntegration::new(config);

        assert!(!integration.stats().is_running);
        assert!(integration.config.thread_count > 0);
    }

    #[test]
    fn test_start_stop() {
        let config = MiningNetworkConfig::default();
        let integration = MiningNetworkIntegration::new(config);

        integration.start().unwrap();
        assert!(integration.stats().is_running);

        integration.stop();
        assert!(!integration.stats().is_running);
    }

    #[test]
    fn test_default_broadcaster() {
        let broadcaster = DefaultBroadcaster::new(10);
        assert_eq!(broadcaster.peer_count(), 10);
    }
}
