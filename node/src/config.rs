//! 节点配置

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 节点配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeConfig {
    /// 数据目录
    pub data_dir: String,
    /// 备份路径
    pub backup_path: PathBuf,
    /// 网络配置
    pub network: NetworkConfig,
    /// 共识配置
    pub consensus: ConsensusConfig,
    /// API 配置
    pub api: ApiConfig,
    /// 日志级别
    pub log_level: String,
}

/// 网络配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// P2P 监听地址
    pub listen_addr: String,
    /// 种子节点
    pub bootstrap_peers: Vec<String>,
    /// 最大连接数
    pub max_connections: usize,
    /// 启用 UPnP
    pub enable_upnp: bool,
    /// 启用节点发现
    pub enable_discovery: bool,
    /// 启用 P2P 网络
    pub enable_p2p: bool,
}

/// 共识配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// 是否启用挖矿
    pub enable_mining: bool,
    /// 矿工地址
    pub miner_address: String,
    /// 目标出块时间
    pub target_block_time: u64,
}

/// API 配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiConfig {
    /// API 监听地址
    pub listen_addr: String,
    /// 启用 WebSocket
    pub enable_ws: bool,
}

impl Default for NodeConfig {
    fn default() -> Self {
        NodeConfig {
            data_dir: "./data".to_string(),
            backup_path: PathBuf::from("./backups"),
            network: NetworkConfig::default(),
            consensus: ConsensusConfig::default(),
            api: ApiConfig::default(),
            log_level: "info".to_string(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig {
            listen_addr: "/ip4/0.0.0.0/tcp/30333".to_string(),
            bootstrap_peers: vec![],
            max_connections: 100,
            enable_upnp: false,
            enable_discovery: true,
            enable_p2p: true,
        }
    }
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        ConsensusConfig {
            enable_mining: false,
            miner_address: String::new(),
            target_block_time: 10,
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        ApiConfig {
            listen_addr: "0.0.0.0:8080".to_string(),
            enable_ws: true,
        }
    }
}

/// 加载配置
pub fn load_config(path: &str) -> anyhow::Result<NodeConfig> {
    if std::path::Path::new(path).exists() {
        let content = std::fs::read_to_string(path)?;
        let config: NodeConfig = toml::from_str(&content)?;
        Ok(config)
    } else {
        // 使用默认配置
        Ok(NodeConfig::default())
    }
}
