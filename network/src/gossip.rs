//! Gossip 消息广播模块

use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Gossip 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GossipMessage {
    /// 新区块广播
    NewBlock {
        height: u64,
        hash: Vec<u8>,
        proposer: Vec<u8>,
    },
    /// 新交易广播
    NewTransaction {
        tx_hash: Vec<u8>,
        tx_data: Vec<u8>,
    },
    /// 难度更新
    DifficultyUpdate {
        new_difficulty: u64,
        block_height: u64,
    },
    /// 节点状态
    NodeStatus {
        height: u64,
        peer_count: usize,
        is_syncing: bool,
    },
}

/// Gossip 管理器
pub struct GossipManager {
    /// 消息计数
    message_count: Arc<RwLock<u64>>,
    /// 订阅的主题
    topics: Vec<String>,
}

impl GossipManager {
    pub fn new() -> Self {
        GossipManager {
            message_count: Arc::new(RwLock::new(0)),
            topics: vec![
                "toki-blocks".to_string(),
                "toki-transactions".to_string(),
                "toki-status".to_string(),
            ],
        }
    }

    /// 获取主题列表
    pub fn topics(&self) -> &[String] {
        &self.topics
    }

    /// 序列化消息
    pub fn serialize(&self, msg: &GossipMessage) -> Vec<u8> {
        bincode::serialize(msg).unwrap_or_default()
    }

    /// 反序列化消息
    pub fn deserialize(&self, data: &[u8]) -> Option<GossipMessage> {
        bincode::deserialize(data).ok()
    }

    /// 记录消息
    pub fn record_message(&self) {
        let mut count = self.message_count.write();
        *count += 1;
    }

    /// 获取消息计数
    pub fn message_count(&self) -> u64 {
        *self.message_count.read()
    }
}

impl Default for GossipManager {
    fn default() -> Self {
        Self::new()
    }
}
