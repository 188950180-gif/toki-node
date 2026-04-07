//! P2P 网络配置和消息定义

use serde::{Deserialize, Serialize};

/// 网络配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// 监听地址
    pub listen_addr: String,
    /// 种子节点列表
    pub bootstrap_peers: Vec<String>,
    /// 最大连接数
    pub max_connections: usize,
    /// 启用 mDNS 本地发现
    pub enable_mdns: bool,
    /// 连接超时（秒）
    pub connection_timeout: u64,
    /// 心跳间隔（秒）
    pub heartbeat_interval: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig {
            listen_addr: "/ip4/0.0.0.0/tcp/30333".to_string(),
            bootstrap_peers: vec![],
            max_connections: 100,
            enable_mdns: true,
            connection_timeout: 30,
            heartbeat_interval: 10,
        }
    }
}

/// 网络消息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// 区块消息
    Block(BlockMessage),
    /// 交易消息
    Transaction(TransactionMessage),
    /// 同步请求
    SyncRequest(SyncRequest),
    /// 同步响应
    SyncResponse(SyncResponse),
}

/// 区块消息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockMessage {
    pub height: u64,
    pub hash: String,
    pub data: Vec<u8>,
}

/// 交易消息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionMessage {
    pub hash: String,
    pub data: Vec<u8>,
}

/// 同步请求
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncRequest {
    pub start_height: u64,
    pub end_height: u64,
}

/// 同步响应
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub blocks: Vec<BlockMessage>,
}
