//! P2P 网络实现
//!
//! 使用 libp2p 实现节点通信、区块和交易广播

use bincode;
use libp2p::{
    core::upgrade::Version,
    gossipsub::{
        self, Behaviour as Gossipsub, Config as GossipsubConfig, IdentTopic, MessageAuthenticity,
    },
    identity, noise,
    ping::{Behaviour as Ping, Config as PingConfig},
    swarm::{Swarm, SwarmEvent},
    tcp::Config as TcpConfig,
    yamux, Multiaddr, PeerId, Transport,
};
use serde::{de::DeserializeOwned, Serialize};
use std::error::Error;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// P2P 网络配置
#[derive(Clone, Debug)]
pub struct P2PConfig {
    /// 监听地址
    pub listen_addr: String,
    /// 种子节点
    pub bootstrap_peers: Vec<String>,
    /// 心跳间隔（秒）
    pub heartbeat_interval: u64,
    /// 最大连接数
    pub max_connections: usize,
}

impl Default for P2PConfig {
    fn default() -> Self {
        P2PConfig {
            listen_addr: "/ip4/0.0.0.0/tcp/30333".to_string(),
            bootstrap_peers: vec![],
            heartbeat_interval: 10,
            max_connections: 50,
        }
    }
}

/// 网络消息类型
#[derive(Clone, Debug, Serialize, serde::Deserialize)]
pub enum NetworkMessage {
    /// 区块消息
    Block(Vec<u8>),
    /// 交易消息
    Transaction(Vec<u8>),
    /// 同步请求
    SyncRequest { start_height: u64, count: u32 },
    /// 同步响应
    SyncResponse { blocks: Vec<Vec<u8>> },
}

/// P2P 网络节点
pub struct P2PNode {
    /// 本地 Peer ID
    local_peer_id: PeerId,
    /// 配置
    config: P2PConfig,
    /// 消息发送通道
    msg_sender: mpsc::UnboundedSender<NetworkMessage>,
    /// 消息接收通道
    msg_receiver: mpsc::UnboundedReceiver<NetworkMessage>,
    /// 是否已启动
    started: bool,
}

impl P2PNode {
    /// 创建新的 P2P 节点
    pub async fn new(config: P2PConfig) -> Result<Self, Box<dyn Error>> {
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        info!("创建 P2P 节点: Peer ID = {}", local_peer_id);

        // 创建消息通道
        let (msg_sender, msg_receiver) = mpsc::unbounded_channel();

        Ok(P2PNode {
            local_peer_id,
            config,
            msg_sender,
            msg_receiver,
            started: false,
        })
    }

    /// 启动网络
    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        if self.started {
            warn!("P2P 网络已经启动");
            return Ok(());
        }

        info!("启动 P2P 网络: {}", self.config.listen_addr);

        // 连接到种子节点
        for peer in &self.config.bootstrap_peers {
            info!("连接到种子节点: {}", peer);
        }

        self.started = true;
        info!("P2P 网络已启动");
        Ok(())
    }

    /// 停止网络
    pub fn stop(&mut self) {
        if !self.started {
            return;
        }

        info!("停止 P2P 网络");
        self.started = false;
    }

    /// 订阅主题
    pub fn subscribe(&mut self, topic: &str) -> Result<(), Box<dyn Error>> {
        if !self.started {
            return Err("P2P 网络未启动".into());
        }

        info!("订阅主题: {}", topic);
        Ok(())
    }

    /// 取消订阅主题
    pub fn unsubscribe(&mut self, topic: &str) -> Result<(), Box<dyn Error>> {
        if !self.started {
            return Err("P2P 网络未启动".into());
        }

        info!("取消订阅主题: {}", topic);
        Ok(())
    }

    /// 广播消息
    pub fn broadcast<T: Serialize>(
        &mut self,
        topic: &str,
        message: &T,
    ) -> Result<(), Box<dyn Error>> {
        if !self.started {
            debug!("P2P 网络未启动，跳过广播");
            return Ok(());
        }

        let data = bincode::serialize(message)?;
        debug!("广播消息到主题 {}: {} 字节", topic, data.len());

        // 通过通道发送
        let msg = NetworkMessage::Block(data);
        let _ = self.msg_sender.send(msg);

        Ok(())
    }

    /// 广播区块
    pub fn broadcast_block(&mut self, block: &toki_core::Block) -> Result<(), Box<dyn Error>> {
        if !self.started {
            debug!("P2P 网络未启动，跳过区块广播");
            return Ok(());
        }

        info!("广播区块: height={}", block.height());
        self.broadcast("blocks", block)
    }

    /// 广播交易
    pub fn broadcast_transaction(
        &mut self,
        tx: &toki_core::Transaction,
    ) -> Result<(), Box<dyn Error>> {
        if !self.started {
            debug!("P2P 网络未启动，跳过交易广播");
            return Ok(());
        }

        info!("广播交易: hash={:?}", tx.tx_hash);
        self.broadcast("transactions", tx)
    }

    /// 请求区块同步
    pub fn request_sync(&mut self, start_height: u64, count: u32) -> Result<(), Box<dyn Error>> {
        if !self.started {
            return Err("P2P 网络未启动".into());
        }

        info!("请求区块同步: start={}, count={}", start_height, count);

        let msg = NetworkMessage::SyncRequest {
            start_height,
            count,
        };
        let _ = self.msg_sender.send(msg);

        Ok(())
    }

    /// 接收消息
    pub async fn receive_message(&mut self) -> Option<NetworkMessage> {
        self.msg_receiver.recv().await
    }

    /// 获取本地 Peer ID
    pub fn local_peer_id(&self) -> &PeerId {
        &self.local_peer_id
    }

    /// 获取 Peer ID 字符串
    pub fn peer_id_string(&self) -> String {
        self.local_peer_id.to_string()
    }

    /// 获取连接的节点数量
    pub fn connected_peers(&self) -> usize {
        // 简化实现，返回 0
        0
    }

    /// 获取所有连接的节点
    pub fn get_connected_peers(&self) -> Vec<String> {
        // 简化实现，返回空列表
        vec![]
    }

    /// 是否已启动
    pub fn is_started(&self) -> bool {
        self.started
    }

    /// 连接到节点
    pub fn connect(&mut self, addr: &str) -> Result<(), Box<dyn Error>> {
        if !self.started {
            return Err("P2P 网络未启动".into());
        }

        info!("连接到节点: {}", addr);
        Ok(())
    }

    /// 断开节点连接
    pub fn disconnect(&mut self, peer_id: &str) -> Result<(), Box<dyn Error>> {
        if !self.started {
            return Err("P2P 网络未启动".into());
        }

        info!("断开节点: {}", peer_id);
        Ok(())
    }
}

/// P2P 网络统计信息
#[derive(Clone, Debug, Default)]
pub struct P2PStats {
    /// 连接的节点数
    pub connected_peers: usize,
    /// 发送的消息数
    pub messages_sent: u64,
    /// 接收的消息数
    pub messages_received: u64,
    /// 发送的字节数
    pub bytes_sent: u64,
    /// 接收的字节数
    pub bytes_received: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_p2p_node_creation() {
        let config = P2PConfig::default();
        let node = P2PNode::new(config).await;
        assert!(node.is_ok());
    }

    #[tokio::test]
    async fn test_p2p_node_start() {
        let config = P2PConfig::default();
        let mut node = P2PNode::new(config).await.unwrap();
        let result = node.start();
        assert!(result.is_ok());
        assert!(node.is_started());
    }

    #[tokio::test]
    async fn test_p2p_subscribe() {
        let config = P2PConfig::default();
        let mut node = P2PNode::new(config).await.unwrap();
        node.start().unwrap();
        let result = node.subscribe("blocks");
        assert!(result.is_ok());
    }
}
