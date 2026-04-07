//! 自动节点发现与连接模块
//! 
//! 实现自动发现外部节点、建立连接、维护网络拓扑

use std::collections::{HashMap, HashSet};
use std::time::Instant;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use anyhow::Result;
use tracing::{info, warn, debug};

/// 节点 ID
pub type NodeId = String;

/// 节点地址
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeAddress {
    /// IP 地址
    pub ip: String,
    /// 端口
    pub port: u16,
    /// 协议
    pub protocol: String,
}

impl std::fmt::Display for NodeAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}://{}:{}", self.protocol, self.ip, self.port)
    }
}

/// 节点信息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInfo {
    /// 节点 ID
    pub id: NodeId,
    /// 节点地址
    pub address: NodeAddress,
    /// 节点版本
    pub version: String,
    /// 区块高度
    pub block_height: u64,
    /// 最后活跃时间
    pub last_seen: u64,
    /// 延迟（毫秒）
    pub latency: u64,
    /// 可靠性评分
    pub reliability: f64,
    /// 是否为种子节点
    pub is_seed: bool,
}

/// 节点发现配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// 种子节点列表
    pub seed_nodes: Vec<NodeAddress>,
    /// 最大连接数
    pub max_connections: usize,
    /// 最小连接数
    pub min_connections: usize,
    /// 发现间隔（秒）
    pub discovery_interval: u64,
    /// 心跳间隔（秒）
    pub heartbeat_interval: u64,
    /// 节点超时（秒）
    pub node_timeout: u64,
    /// 最大尝试次数
    pub max_attempts: u32,
    /// 启用自动发现
    pub auto_discovery: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        DiscoveryConfig {
            seed_nodes: vec![
                NodeAddress { ip: "182.254.176.30".to_string(), port: 30333, protocol: "toki".to_string() },
            ],
            max_connections: 100,
            min_connections: 10,
            discovery_interval: 60,
            heartbeat_interval: 30,
            node_timeout: 300,
            max_attempts: 3,
            auto_discovery: true,
        }
    }
}

/// 节点连接器
#[async_trait]
pub trait NodeConnector: Send + Sync {
    /// 连接节点
    async fn connect(&self, address: &NodeAddress) -> Result<NodeInfo>;
    
    /// 断开连接
    async fn disconnect(&self, node_id: &NodeId) -> Result<()>;
    
    /// 发送心跳
    async fn heartbeat(&self, node_id: &NodeId) -> Result<u64>;
    
    /// 获取节点列表
    async fn get_peers(&self, node_id: &NodeId) -> Result<Vec<NodeInfo>>;
}

/// 自动节点发现器
pub struct AutoNodeDiscovery {
    /// 配置
    config: DiscoveryConfig,
    /// 已知节点
    known_nodes: HashMap<NodeId, NodeInfo>,
    /// 活跃连接
    active_connections: HashSet<NodeId>,
    /// 连接器
    connector: Option<Box<dyn NodeConnector>>,
    /// 尝试计数
    attempt_counts: HashMap<NodeId, u32>,
    /// 上次发现时间
    last_discovery: Option<Instant>,
    /// 上次心跳时间
    last_heartbeat: Option<Instant>,
}

impl AutoNodeDiscovery {
    /// 创建新的自动发现器
    pub fn new(config: DiscoveryConfig) -> Self {
        AutoNodeDiscovery {
            config,
            known_nodes: HashMap::new(),
            active_connections: HashSet::new(),
            connector: None,
            attempt_counts: HashMap::new(),
            last_discovery: None,
            last_heartbeat: None,
        }
    }

    /// 设置连接器
    pub fn set_connector(&mut self, connector: Box<dyn NodeConnector>) {
        self.connector = Some(connector);
    }

    /// 初始化种子节点连接
    pub async fn initialize(&mut self) -> Result<Vec<NodeInfo>> {
        info!("初始化种子节点连接...");
        
        let mut connected = Vec::new();
        
        // 收集种子节点地址
        let seed_addresses: Vec<_> = self.config.seed_nodes.clone();
        
        for seed in seed_addresses {
            match self.connect_to_node(&seed).await {
                Ok(node) => {
                    info!("成功连接种子节点: {} ({})", node.id, seed);
                    connected.push(node);
                }
                Err(e) => {
                    warn!("连接种子节点失败: {} - {}", seed, e);
                }
            }
        }
        
        if connected.is_empty() {
            warn!("未能连接任何种子节点，作为独立节点运行");
        }
        
        Ok(connected)
    }

    /// 连接到节点
    async fn connect_to_node(&mut self, address: &NodeAddress) -> Result<NodeInfo> {
        if let Some(ref connector) = self.connector {
            let node = connector.connect(address).await?;
            self.known_nodes.insert(node.id.clone(), node.clone());
            self.active_connections.insert(node.id.clone());
            Ok(node)
        } else {
            Err(anyhow::anyhow!("未设置连接器"))
        }
    }

    /// 自动发现新节点
    pub async fn discover_nodes(&mut self) -> Result<Vec<NodeInfo>> {
        if !self.config.auto_discovery {
            return Ok(Vec::new());
        }
        
        info!("开始自动发现节点...");
        
        let mut discovered = Vec::new();
        
        // 从已连接节点获取对等节点列表
        for node_id in self.active_connections.clone() {
            if let Some(ref connector) = self.connector {
                match connector.get_peers(&node_id).await {
                    Ok(peers) => {
                        for peer in peers {
                            if !self.known_nodes.contains_key(&peer.id) {
                                info!("发现新节点: {} at {}", peer.id, peer.address);
                                self.known_nodes.insert(peer.id.clone(), peer.clone());
                                discovered.push(peer);
                            }
                        }
                    }
                    Err(e) => {
                        debug!("从节点 {} 获取对等节点失败: {}", node_id, e);
                    }
                }
            }
        }
        
        self.last_discovery = Some(Instant::now());
        info!("发现 {} 个新节点", discovered.len());
        
        Ok(discovered)
    }

    /// 维护连接
    pub async fn maintain_connections(&mut self) -> Result<()> {
        // 检查是否需要更多连接
        if self.active_connections.len() < self.config.min_connections {
            info!("连接数不足，尝试建立新连接...");
            self.connect_to_more_nodes().await?;
        }
        
        // 检查是否需要减少连接
        if self.active_connections.len() > self.config.max_connections {
            self.disconnect_worst_nodes().await?;
        }
        
        // 移除超时节点
        self.remove_timeout_nodes();
        
        Ok(())
    }

    /// 连接更多节点
    async fn connect_to_more_nodes(&mut self) -> Result<()> {
        let needed = self.config.min_connections - self.active_connections.len();
        let mut connected = 0;
        
        // 收集候选节点地址
        let candidates: Vec<_> = self.known_nodes.values()
            .filter(|n| !self.active_connections.contains(&n.id))
            .map(|n| (n.id.clone(), n.address.clone(), n.reliability))
            .collect();
        
        let mut sorted_candidates = candidates;
        sorted_candidates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        
        for (node_id, node_addr, _) in sorted_candidates {
            if connected >= needed {
                break;
            }
            
            let attempts = self.attempt_counts.get(&node_id).copied().unwrap_or(0);
            if attempts >= self.config.max_attempts {
                continue;
            }
            
            match self.connect_to_node(&node_addr).await {
                Ok(_) => {
                    connected += 1;
                    self.attempt_counts.remove(&node_id);
                }
                Err(e) => {
                    warn!("连接节点 {} 失败: {}", node_id, e);
                    self.attempt_counts.insert(node_id, attempts + 1);
                }
            }
        }
        
        Ok(())
    }

    /// 断开最差节点
    async fn disconnect_worst_nodes(&mut self) -> Result<()> {
        let excess = self.active_connections.len() - self.config.max_connections;
        
        // 按可靠性排序（升序）
        let mut nodes: Vec<_> = self.active_connections.iter()
            .filter_map(|id| self.known_nodes.get(id))
            .collect();
        nodes.sort_by(|a, b| a.reliability.partial_cmp(&b.reliability).unwrap());
        
        for node in nodes.iter().take(excess) {
            if let Some(ref connector) = self.connector {
                connector.disconnect(&node.id).await?;
                self.active_connections.remove(&node.id);
                info!("断开低可靠性节点: {}", node.id);
            }
        }
        
        Ok(())
    }

    /// 移除超时节点
    fn remove_timeout_nodes(&mut self) {
        let now = Instant::now().elapsed().as_secs();
        let timeout = self.config.node_timeout;
        
        self.known_nodes.retain(|_, node| {
            now - node.last_seen < timeout
        });
        
        self.active_connections.retain(|id| {
            self.known_nodes.contains_key(id)
        });
    }

    /// 发送心跳
    pub async fn send_heartbeats(&mut self) -> Result<()> {
        if let Some(ref connector) = self.connector {
            for node_id in self.active_connections.clone() {
                match connector.heartbeat(&node_id).await {
                    Ok(latency) => {
                        if let Some(node) = self.known_nodes.get_mut(&node_id) {
                            node.latency = latency;
                            node.last_seen = Instant::now().elapsed().as_secs();
                        }
                    }
                    Err(e) => {
                        debug!("心跳失败: {} - {}", node_id, e);
                    }
                }
            }
        }
        
        self.last_heartbeat = Some(Instant::now());
        Ok(())
    }

    /// 获取活跃连接数
    pub fn active_count(&self) -> usize {
        self.active_connections.len()
    }

    /// 获取已知节点数
    pub fn known_count(&self) -> usize {
        self.known_nodes.len()
    }

    /// 获取最佳节点
    pub fn get_best_nodes(&self, count: usize) -> Vec<&NodeInfo> {
        let mut nodes: Vec<_> = self.active_connections.iter()
            .filter_map(|id| self.known_nodes.get(id))
            .collect();
        nodes.sort_by(|a, b| b.reliability.partial_cmp(&a.reliability).unwrap());
        nodes.into_iter().take(count).collect()
    }

    /// 获取网络拓扑
    pub fn get_topology(&self) -> NetworkTopology {
        NetworkTopology {
            total_nodes: self.known_nodes.len(),
            active_connections: self.active_connections.len(),
            seed_nodes: self.known_nodes.values().filter(|n| n.is_seed).count(),
            avg_latency: self.calculate_avg_latency(),
            avg_reliability: self.calculate_avg_reliability(),
        }
    }

    fn calculate_avg_latency(&self) -> f64 {
        let active: Vec<_> = self.active_connections.iter()
            .filter_map(|id| self.known_nodes.get(id))
            .collect();
        
        if active.is_empty() {
            return 0.0;
        }
        
        active.iter().map(|n| n.latency as f64).sum::<f64>() / active.len() as f64
    }

    fn calculate_avg_reliability(&self) -> f64 {
        let active: Vec<_> = self.active_connections.iter()
            .filter_map(|id| self.known_nodes.get(id))
            .collect();
        
        if active.is_empty() {
            return 0.0;
        }
        
        active.iter().map(|n| n.reliability).sum::<f64>() / active.len() as f64
    }
}

/// 网络拓扑
#[derive(Debug, Clone)]
pub struct NetworkTopology {
    /// 总节点数
    pub total_nodes: usize,
    /// 活跃连接数
    pub active_connections: usize,
    /// 种子节点数
    pub seed_nodes: usize,
    /// 平均延迟
    pub avg_latency: f64,
    /// 平均可靠性
    pub avg_reliability: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();
        assert!(!config.seed_nodes.is_empty());
        assert!(config.auto_discovery);
    }

    #[test]
    fn test_node_address_display() {
        let addr = NodeAddress {
            ip: "192.168.1.1".to_string(),
            port: 30333,
            protocol: "toki".to_string(),
        };
        assert_eq!(format!("{}", addr), "toki://192.168.1.1:30333");
    }

    #[test]
    fn test_network_topology() {
        let discovery = AutoNodeDiscovery::new(DiscoveryConfig::default());
        let topology = discovery.get_topology();
        assert_eq!(topology.total_nodes, 0);
        assert_eq!(topology.active_connections, 0);
    }
}
