//! 网络事件循环与区块同步
//! 
//! 实现节点间通信、区块同步、状态同步

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use async_trait::async_trait;
use tracing::{info, warn, debug};

use toki_core::{Block, Hash, Transaction};

/// 节点 ID
pub type PeerId = String;

/// 网络事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkEvent {
    /// 新节点连接
    PeerConnected(PeerId),
    /// 节点断开
    PeerDisconnected(PeerId),
    /// 收到区块
    BlockReceived {
        from: PeerId,
        block: Block,
    },
    /// 收到交易
    TransactionReceived {
        from: PeerId,
        transaction: Transaction,
    },
    /// 区块请求
    BlockRequest {
        from: PeerId,
        start_height: u64,
        count: u32,
    },
    /// 区块响应
    BlockResponse {
        to: PeerId,
        blocks: Vec<Block>,
    },
    /// 状态同步请求
    StateSyncRequest {
        from: PeerId,
        height: u64,
    },
    /// 心跳
    Heartbeat {
        from: PeerId,
        height: u64,
        hash: Hash,
    },
}

/// 同步状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SyncState {
    /// 空闲
    Idle,
    /// 同步中
    Syncing {
        target_height: u64,
        current_height: u64,
        from_peer: PeerId,
    },
    /// 已同步
    Synced {
        height: u64,
        hash: Hash,
    },
    /// 同步失败
    Failed {
        reason: String,
    },
}

/// 区块同步器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockSyncConfig {
    /// 最大并发同步数
    pub max_concurrent_syncs: usize,
    /// 每次请求的区块数
    pub blocks_per_request: u32,
    /// 同步超时（秒）
    pub sync_timeout: u64,
    /// 心跳间隔（秒）
    pub heartbeat_interval: u64,
    /// 最大待处理区块数
    pub max_pending_blocks: usize,
    /// 是否启用快速同步
    pub fast_sync: bool,
}

impl Default for BlockSyncConfig {
    fn default() -> Self {
        BlockSyncConfig {
            max_concurrent_syncs: 3,
            blocks_per_request: 100,
            sync_timeout: 30,
            heartbeat_interval: 10,
            max_pending_blocks: 1000,
            fast_sync: true,
        }
    }
}

/// 区块同步器
pub struct BlockSynchronizer {
    /// 配置
    config: BlockSyncConfig,
    /// 当前同步状态
    sync_state: SyncState,
    /// 已连接的节点
    peers: HashMap<PeerId, PeerInfo>,
    /// 待处理的区块
    pending_blocks: VecDeque<Block>,
    /// 已请求的区块高度
    requested_heights: HashSet<u64>,
    /// 本地最新高度
    local_height: u64,
    /// 本地最新哈希
    local_hash: Hash,
    /// 同步开始时间
    sync_start_time: Option<Instant>,
}

/// 节点信息
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// 节点 ID
    pub id: PeerId,
    /// 最新高度
    pub height: u64,
    /// 最新哈希
    pub hash: Hash,
    /// 最后心跳时间
    pub last_heartbeat: Instant,
    /// 延迟（毫秒）
    pub latency: u64,
    /// 是否可信
    pub trusted: bool,
}

impl BlockSynchronizer {
    /// 创建新的同步器
    pub fn new(config: BlockSyncConfig) -> Self {
        BlockSynchronizer {
            config,
            sync_state: SyncState::Idle,
            peers: HashMap::new(),
            pending_blocks: VecDeque::new(),
            requested_heights: HashSet::new(),
            local_height: 0,
            local_hash: Hash::ZERO,
            sync_start_time: None,
        }
    }

    /// 处理网络事件
    pub fn handle_event(&mut self, event: NetworkEvent) -> Result<Vec<NetworkAction>> {
        let mut actions = Vec::new();
        
        match event {
            NetworkEvent::PeerConnected(peer_id) => {
                self.on_peer_connected(&peer_id);
                actions.push(NetworkAction::RequestState { peer_id });
            }
            
            NetworkEvent::PeerDisconnected(peer_id) => {
                self.on_peer_disconnected(&peer_id);
            }
            
            NetworkEvent::BlockReceived { from, block } => {
                self.on_block_received(&from, &block)?;
                actions.push(NetworkAction::ProcessBlock { block });
            }
            
            NetworkEvent::BlockRequest { from, start_height, count } => {
                let blocks = self.get_blocks_for_sync(start_height, count);
                actions.push(NetworkAction::SendBlocks { to: from, blocks });
            }
            
            NetworkEvent::Heartbeat { from, height, hash } => {
                self.on_heartbeat(&from, height, hash);
            }
            
            NetworkEvent::StateSyncRequest { from, height } => {
                actions.push(NetworkAction::SendState { to: from, height });
            }
            
            _ => {}
        }
        
        Ok(actions)
    }

    /// 节点连接
    fn on_peer_connected(&mut self, peer_id: &PeerId) {
        info!("节点连接: {}", peer_id);
        
        self.peers.insert(peer_id.clone(), PeerInfo {
            id: peer_id.clone(),
            height: 0,
            hash: Hash::ZERO,
            last_heartbeat: Instant::now(),
            latency: 0,
            trusted: false,
        });
    }

    /// 节点断开
    fn on_peer_disconnected(&mut self, peer_id: &PeerId) {
        info!("节点断开: {}", peer_id);
        self.peers.remove(peer_id);
    }

    /// 收到区块
    fn on_block_received(&mut self, from: &PeerId, block: &Block) -> Result<()> {
        let height = block.header.height;
        
        debug!("收到区块: 高度={}, 来自={}", height, from);
        
        // 移除已请求标记
        self.requested_heights.remove(&height);
        
        // 添加到待处理队列
        if self.pending_blocks.len() < self.config.max_pending_blocks {
            self.pending_blocks.push_back(block.clone());
        }
        
        // 更新同步状态
        if let SyncState::Syncing { current_height, .. } = &mut self.sync_state {
            if height > *current_height {
                *current_height = height;
            }
        }
        
        Ok(())
    }

    /// 处理心跳
    fn on_heartbeat(&mut self, peer_id: &PeerId, height: u64, hash: Hash) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.height = height;
            peer.hash = hash;
            peer.last_heartbeat = Instant::now();
        }
        
        // 检查是否需要同步
        if height > self.local_height {
            self.check_sync_needed();
        }
    }

    /// 检查是否需要同步
    fn check_sync_needed(&mut self) {
        if !matches!(self.sync_state, SyncState::Idle) {
            return;
        }
        
        // 找到最高节点
        let best_peer = self.peers.values()
            .max_by_key(|p| p.height);
        
        if let Some(peer) = best_peer {
            if peer.height > self.local_height {
                info!("开始同步: 本地={}, 远程={}", self.local_height, peer.height);
                
                self.sync_state = SyncState::Syncing {
                    target_height: peer.height,
                    current_height: self.local_height,
                    from_peer: peer.id.clone(),
                };
                
                self.sync_start_time = Some(Instant::now());
            }
        }
    }

    /// 获取同步区块
    fn get_blocks_for_sync(&self, start_height: u64, count: u32) -> Vec<Block> {
        // TODO: 实际从存储获取
        Vec::new()
    }

    /// 开始同步
    pub fn start_sync(&mut self) -> Result<Vec<NetworkAction>> {
        let mut actions = Vec::new();
        
        // 找到最佳同步源
        let best_peer = self.peers.values()
            .filter(|p| p.height > self.local_height)
            .max_by_key(|p| p.height);
        
        if let Some(peer) = best_peer {
            let start_height = self.local_height + 1;
            
            info!("请求同步: {} -> {}, 从高度 {}", peer.id, peer.height, start_height);
            
            actions.push(NetworkAction::RequestBlocks {
                to: peer.id.clone(),
                start_height,
                count: self.config.blocks_per_request,
            });
            
            // 标记已请求
            for h in start_height..start_height + self.config.blocks_per_request as u64 {
                self.requested_heights.insert(h);
            }
        }
        
        Ok(actions)
    }

    /// 更新本地状态
    pub fn update_local_state(&mut self, height: u64, hash: Hash) {
        self.local_height = height;
        self.local_hash = hash;
        
        // 检查同步是否完成
        if let SyncState::Syncing { target_height, .. } = &self.sync_state {
            if height >= *target_height {
                info!("同步完成: 高度={}", height);
                self.sync_state = SyncState::Synced { height, hash };
                self.sync_start_time = None;
            }
        }
    }

    /// 获取同步状态
    pub fn state(&self) -> &SyncState {
        &self.sync_state
    }

    /// 获取待处理区块数
    pub fn pending_count(&self) -> usize {
        self.pending_blocks.len()
    }

    /// 获取下一个待处理区块
    pub fn pop_pending_block(&mut self) -> Option<Block> {
        self.pending_blocks.pop_front()
    }

    /// 获取节点数
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// 获取同步进度
    pub fn sync_progress(&self) -> f64 {
        match &self.sync_state {
            SyncState::Syncing { target_height, current_height, .. } => {
                if *target_height > 0 {
                    *current_height as f64 / *target_height as f64
                } else {
                    0.0
                }
            }
            SyncState::Synced { height, .. } => {
                if self.local_height > 0 {
                    *height as f64 / self.local_height as f64
                } else {
                    1.0
                }
            }
            _ => 0.0,
        }
    }
}

/// 网络动作
#[derive(Debug, Clone)]
pub enum NetworkAction {
    /// 请求状态
    RequestState { peer_id: PeerId },
    /// 发送区块
    SendBlocks { to: PeerId, blocks: Vec<Block> },
    /// 发送状态
    SendState { to: PeerId, height: u64 },
    /// 请求区块
    RequestBlocks { to: PeerId, start_height: u64, count: u32 },
    /// 处理区块
    ProcessBlock { block: Block },
    /// 广播区块
    BroadcastBlock { block: Block },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_config_default() {
        let config = BlockSyncConfig::default();
        assert_eq!(config.blocks_per_request, 100);
    }

    #[test]
    fn test_synchronizer_creation() {
        let config = BlockSyncConfig::default();
        let syncer = BlockSynchronizer::new(config);
        
        assert!(matches!(syncer.state(), SyncState::Idle));
        assert_eq!(syncer.peer_count(), 0);
    }

    #[test]
    fn test_peer_connected() {
        let config = BlockSyncConfig::default();
        let mut syncer = BlockSynchronizer::new(config);
        
        let actions = syncer.handle_event(NetworkEvent::PeerConnected("peer1".to_string())).unwrap();
        
        assert_eq!(syncer.peer_count(), 1);
        assert_eq!(actions.len(), 1);
    }
}
