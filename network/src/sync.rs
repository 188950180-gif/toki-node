//! 区块同步模块

use std::collections::VecDeque;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

use toki_core::Block;
use toki_storage::BlockStore;

/// 同步状态
#[derive(Debug, Clone, PartialEq)]
pub enum SyncStatus {
    /// 未同步
    NotSynced,
    /// 同步中
    Syncing {
        /// 目标高度
        target_height: u64,
        /// 当前进度
        current: u64,
    },
    /// 已同步
    Synced,
}

/// 区块同步器
#[allow(dead_code)]
pub struct BlockSyncer {
    /// 本地区块存储
    block_store: Arc<BlockStore>,
    /// 本地最新高度
    local_height: u64,
    /// 目标高度
    target_height: u64,
    /// 同步状态
    status: SyncStatus,
    /// 待请求区块队列
    pending_blocks: VecDeque<u64>,
    /// 区块请求发送器
    request_sender: mpsc::Sender<SyncRequest>,
    /// 区块响应接收器
    response_receiver: mpsc::Receiver<SyncResponse>,
    /// 最大并发请求数
    max_concurrent_requests: usize,
    /// 当前请求数
    current_requests: usize,
}

/// 同步请求
#[derive(Debug, Clone)]
pub enum SyncRequest {
    /// 请求区块高度范围
    GetBlocks {
        start: u64,
        end: u64,
        request_id: u64,
    },
    /// 请求单个区块
    GetBlock(u64),
    /// 请求最新高度
    GetHeight,
}

/// 同步响应
#[derive(Debug, Clone)]
pub enum SyncResponse {
    /// 区块列表
    Blocks { request_id: u64, blocks: Vec<Block> },
    /// 单个区块
    Block(Option<Block>),
    /// 最新高度
    Height(u64),
}

impl BlockSyncer {
    /// 创建同步器
    pub fn new(
        block_store: Arc<BlockStore>,
        request_sender: mpsc::Sender<SyncRequest>,
        response_receiver: mpsc::Receiver<SyncResponse>,
    ) -> Self {
        let local_height = block_store.get_latest_height().ok().flatten().unwrap_or(0);

        BlockSyncer {
            block_store,
            local_height,
            target_height: 0,
            status: SyncStatus::NotSynced,
            pending_blocks: VecDeque::new(),
            request_sender,
            response_receiver,
            max_concurrent_requests: 10,
            current_requests: 0,
        }
    }

    /// 获取同步状态
    pub fn status(&self) -> &SyncStatus {
        &self.status
    }

    /// 获取本地高度
    pub fn local_height(&self) -> u64 {
        self.local_height
    }

    /// 获取目标高度
    pub fn target_height(&self) -> u64 {
        self.target_height
    }

    /// 获取同步进度
    pub fn progress(&self) -> f64 {
        if self.target_height == 0 || self.local_height >= self.target_height {
            return 1.0;
        }
        self.local_height as f64 / self.target_height as f64
    }

    /// 开始同步
    pub async fn start_sync(&mut self, target_height: u64) {
        if matches!(self.status, SyncStatus::Syncing { .. }) {
            warn!("同步已在进行中");
            return;
        }

        if target_height <= self.local_height {
            info!("已同步到最新高度: {}", self.local_height);
            self.status = SyncStatus::Synced;
            return;
        }

        self.target_height = target_height;
        self.status = SyncStatus::Syncing {
            target_height,
            current: self.local_height,
        };

        info!(
            "开始同步: 本地高度 {}, 目标高度 {}",
            self.local_height, target_height
        );

        // 构建待请求区块队列
        self.pending_blocks.clear();
        for height in (self.local_height + 1)..=target_height {
            self.pending_blocks.push_back(height);
        }

        // 发送批量请求
        self.send_batch_requests().await;
    }

    /// 发送批量请求
    async fn send_batch_requests(&mut self) {
        while self.current_requests < self.max_concurrent_requests
            && !self.pending_blocks.is_empty()
        {
            // 取出一批区块高度
            let start = self.pending_blocks.front().copied().unwrap_or(0);
            let mut end = start;
            let mut count = 0;

            while count < 100 && !self.pending_blocks.is_empty() {
                if let Some(h) = self.pending_blocks.pop_front() {
                    if h == end + 1 || count == 0 {
                        end = h;
                        count += 1;
                    } else {
                        self.pending_blocks.push_front(h);
                        break;
                    }
                }
            }

            if count > 0 {
                let request = SyncRequest::GetBlocks {
                    start,
                    end,
                    request_id: start,
                };

                if let Err(e) = self.request_sender.send(request).await {
                    warn!("发送同步请求失败: {}", e);
                } else {
                    self.current_requests += 1;
                }
            }
        }
    }

    /// 处理收到的区块
    pub async fn handle_blocks(&mut self, _request_id: u64, blocks: Vec<Block>) {
        self.current_requests = self.current_requests.saturating_sub(1);

        for block in blocks {
            let height = block.header.height;

            // 验证区块
            if !self.validate_block(&block) {
                warn!("区块 {} 验证失败", height);
                continue;
            }

            // 存储区块
            if let Err(e) = self.block_store.save_block(&block) {
                warn!("存储区块 {} 失败: {}", height, e);
                continue;
            }

            self.local_height = height;

            // 更新状态
            if let SyncStatus::Syncing { target_height, .. } = self.status {
                self.status = SyncStatus::Syncing {
                    target_height,
                    current: height,
                };
            }

            debug!("同步区块 {} 成功", height);
        }

        // 检查是否同步完成
        if self.pending_blocks.is_empty() && self.current_requests == 0 {
            if self.local_height >= self.target_height {
                self.status = SyncStatus::Synced;
                info!("同步完成，最新高度: {}", self.local_height);
            }
        } else {
            // 继续发送请求
            self.send_batch_requests().await;
        }
    }

    /// 验证区块
    fn validate_block(&self, block: &Block) -> bool {
        // 基本验证
        if block.header.height == 0 {
            return true; // 创世区块
        }

        // 验证前序区块哈希
        if let Ok(Some(prev_block)) = self
            .block_store
            .get_block_by_height(block.header.height - 1)
        {
            if block.header.prev_hash != prev_block.hash() {
                return false;
            }
        }

        // 验证难度
        if !block.meets_difficulty() {
            return false;
        }

        // 验证 Merkle 根
        if !block.verify_merkle_root() {
            return false;
        }

        true
    }

    /// 检查是否需要同步
    pub fn needs_sync(&self, remote_height: u64) -> bool {
        remote_height > self.local_height
    }

    /// 处理响应
    pub async fn process_response(&mut self, response: SyncResponse) {
        match response {
            SyncResponse::Blocks { request_id, blocks } => {
                self.handle_blocks(request_id, blocks).await;
            }
            SyncResponse::Height(height) => {
                if self.needs_sync(height) {
                    self.start_sync(height).await;
                }
            }
            _ => {}
        }
    }
}

/// 同步管理器
pub struct SyncManager {
    syncer: Arc<RwLock<BlockSyncer>>,
}

impl SyncManager {
    pub fn new(syncer: BlockSyncer) -> Self {
        SyncManager {
            syncer: Arc::new(RwLock::new(syncer)),
        }
    }

    /// 获取同步状态
    pub async fn get_status(&self) -> SyncStatus {
        self.syncer.read().await.status().clone()
    }

    /// 触发同步检查
    pub async fn check_sync(&self, remote_height: u64) {
        let mut syncer = self.syncer.write().await;
        if syncer.needs_sync(remote_height) {
            syncer.start_sync(remote_height).await;
        }
    }

    /// 获取同步进度
    pub async fn get_progress(&self) -> f64 {
        self.syncer.read().await.progress()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_status() {
        let status = SyncStatus::NotSynced;
        assert_eq!(status, SyncStatus::NotSynced);

        let status = SyncStatus::Syncing {
            target_height: 100,
            current: 50,
        };
        assert_eq!(
            status,
            SyncStatus::Syncing {
                target_height: 100,
                current: 50
            }
        );
    }
}
