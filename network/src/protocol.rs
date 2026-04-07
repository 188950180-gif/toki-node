//! 网络协议定义

use serde::{Deserialize, Serialize};

/// 协议版本
pub const PROTOCOL_VERSION: &str = "toki/0.1.0";

/// 网络消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolMessage {
    /// 握手请求
    Handshake {
        version: String,
        height: u64,
        node_id: String,
    },
    /// 握手响应
    HandshakeResponse {
        accepted: bool,
        height: u64,
    },
    /// 区块请求
    BlockRequest {
        start_height: u64,
        count: u32,
    },
    /// 区块响应
    BlockResponse {
        blocks: Vec<Vec<u8>>,
    },
    /// 交易广播
    TransactionBroadcast {
        tx_data: Vec<u8>,
    },
    /// 状态查询
    StatusQuery,
    /// 状态响应
    StatusResponse {
        height: u64,
        difficulty: u64,
        peer_count: u32,
    },
}

/// 协议编解码
pub struct ProtocolCodec;

impl ProtocolCodec {
    /// 编码消息
    pub fn encode(msg: &ProtocolMessage) -> Vec<u8> {
        bincode::serialize(msg).unwrap_or_default()
    }

    /// 解码消息
    pub fn decode(data: &[u8]) -> Option<ProtocolMessage> {
        bincode::deserialize(data).ok()
    }
}
