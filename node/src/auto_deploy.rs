//! 自动部署模块
//!
//! 实现区块链自动部署、初始化、配置管理

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

// 简化的目录复制函数
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// 部署环境
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DeploymentEnv {
    /// 开发环境
    Development,
    /// 测试网
    Testnet,
    /// 主网
    Mainnet,
}

/// 部署配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeploymentConfig {
    /// 部署环境
    pub env: DeploymentEnv,
    /// 数据目录
    pub data_dir: PathBuf,
    /// 配置目录
    pub config_dir: PathBuf,
    /// 日志目录
    pub log_dir: PathBuf,
    /// 备份目录
    pub backup_dir: PathBuf,
    /// 是否启用自动备份
    pub auto_backup: bool,
    /// 备份间隔（秒）
    pub backup_interval: u64,
    /// 最大备份数
    pub max_backups: usize,
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        DeploymentConfig {
            env: DeploymentEnv::Development,
            data_dir: PathBuf::from("./data"),
            config_dir: PathBuf::from("./config"),
            log_dir: PathBuf::from("./logs"),
            backup_dir: PathBuf::from("./backups"),
            auto_backup: true,
            backup_interval: 3600,
            max_backups: 10,
        }
    }
}

impl DeploymentConfig {
    /// 创建主网配置
    pub fn mainnet() -> Self {
        DeploymentConfig {
            env: DeploymentEnv::Mainnet,
            data_dir: PathBuf::from("/opt/toki/data"),
            config_dir: PathBuf::from("/opt/toki/config"),
            log_dir: PathBuf::from("/var/log/toki"),
            backup_dir: PathBuf::from("/opt/toki/backups"),
            auto_backup: true,
            backup_interval: 3600,
            max_backups: 20,
        }
    }

    /// 创建测试网配置
    pub fn testnet() -> Self {
        DeploymentConfig {
            env: DeploymentEnv::Testnet,
            data_dir: PathBuf::from("./testnet-data"),
            config_dir: PathBuf::from("./testnet-config"),
            log_dir: PathBuf::from("./testnet-logs"),
            backup_dir: PathBuf::from("./testnet-backups"),
            auto_backup: true,
            backup_interval: 1800,
            max_backups: 5,
        }
    }
}

/// 自动部署器
pub struct AutoDeployer {
    /// 部署配置
    config: DeploymentConfig,
    /// 是否已初始化
    initialized: bool,
}

impl AutoDeployer {
    /// 创建新的自动部署器
    pub fn new(config: DeploymentConfig) -> Self {
        AutoDeployer {
            config,
            initialized: false,
        }
    }

    /// 执行完整部署
    pub fn deploy(&mut self) -> Result<DeploymentResult> {
        info!("开始自动部署 [{:?}]", self.config.env);

        let mut result = DeploymentResult::default();

        // 1. 创建目录结构
        self.create_directories()?;
        result.directories_created = true;
        info!("✓ 目录结构创建完成");

        // 2. 生成配置文件
        self.generate_configs()?;
        result.configs_generated = true;
        info!("✓ 配置文件生成完成");

        // 3. 初始化数据库
        self.init_database()?;
        result.database_initialized = true;
        info!("✓ 数据库初始化完成");

        // 4. 创建创世区块
        self.create_genesis()?;
        result.genesis_created = true;
        info!("✓ 创世区块创建完成");

        // 5. 设置系统服务
        self.setup_service()?;
        result.service_setup = true;
        info!("✓ 系统服务设置完成");

        self.initialized = true;
        result.success = true;

        info!("自动部署完成！");
        Ok(result)
    }

    /// 创建目录结构
    fn create_directories(&self) -> Result<()> {
        let dirs = vec![
            &self.config.data_dir,
            &self.config.config_dir,
            &self.config.log_dir,
            &self.config.backup_dir,
        ];

        for dir in dirs {
            if !dir.exists() {
                fs::create_dir_all(dir)?;
                info!("创建目录: {:?}", dir);
            }
        }

        // 创建子目录
        let subdirs = vec![
            self.config.data_dir.join("blocks"),
            self.config.data_dir.join("transactions"),
            self.config.data_dir.join("accounts"),
            self.config.data_dir.join("state"),
        ];

        for subdir in subdirs {
            if !subdir.exists() {
                fs::create_dir_all(&subdir)?;
            }
        }

        Ok(())
    }

    /// 生成配置文件
    fn generate_configs(&self) -> Result<()> {
        // 生成节点配置
        let node_config = self.generate_node_config();
        let config_path = self.config.config_dir.join("node.toml");
        fs::write(&config_path, &node_config)?;
        info!("生成节点配置: {:?}", config_path);

        // 生成网络配置
        let network_config = self.generate_network_config();
        let network_path = self.config.config_dir.join("network.toml");
        fs::write(&network_path, &network_config)?;
        info!("生成网络配置: {:?}", network_path);

        // 生成共识配置
        let consensus_config = self.generate_consensus_config();
        let consensus_path = self.config.config_dir.join("consensus.toml");
        fs::write(&consensus_path, &consensus_config)?;
        info!("生成共识配置: {:?}", consensus_path);

        Ok(())
    }

    /// 生成节点配置
    fn generate_node_config(&self) -> String {
        match self.config.env {
            DeploymentEnv::Mainnet => r#"
# Toki 主网节点配置
data_dir = "/opt/toki/data"
backup_path = "/opt/toki/backups"

[network]
listen_addr = "/ip4/0.0.0.0/tcp/30333"
bootstrap_peers = ["/ip4/182.254.176.30/tcp/30333"]
max_connections = 200
enable_discovery = true

[consensus]
enable_mining = true
target_block_time = 10
difficulty_adjust_interval = 100

[api]
listen_addr = "0.0.0.0:8080"
enable_cors = true

[ai]
enable_auto_execute = true
enable_self_healing = true
check_interval = 60

[logging]
level = "info"
file = "/var/log/toki/node.log"
"#
            .to_string(),
            DeploymentEnv::Testnet => r#"
# Toki 测试网节点配置
data_dir = "./testnet-data"
backup_path = "./testnet-backups"

[network]
listen_addr = "/ip4/0.0.0.0/tcp/30334"
bootstrap_peers = []
max_connections = 50
enable_discovery = true

[consensus]
enable_mining = true
target_block_time = 5
difficulty_adjust_interval = 50

[api]
listen_addr = "0.0.0.0:8081"
enable_cors = true

[ai]
enable_auto_execute = true
enable_self_healing = true
check_interval = 30

[logging]
level = "debug"
file = "./testnet-logs/node.log"
"#
            .to_string(),
            DeploymentEnv::Development => r#"
# Toki 开发环境配置
data_dir = "./data"
backup_path = "./backups"

[network]
listen_addr = "/ip4/127.0.0.1/tcp/30333"
bootstrap_peers = []
max_connections = 10
enable_discovery = false

[consensus]
enable_mining = true
target_block_time = 3
difficulty_adjust_interval = 10

[api]
listen_addr = "127.0.0.1:8080"
enable_cors = true

[ai]
enable_auto_execute = false
enable_self_healing = false
check_interval = 10

[logging]
level = "debug"
file = "./logs/node.log"
"#
            .to_string(),
        }
    }

    /// 生成网络配置
    fn generate_network_config(&self) -> String {
        r#"
# Toki 网络配置
[discovery]
auto_discovery = true
discovery_interval = 60
heartbeat_interval = 30
node_timeout = 300

[protocol]
version = "0.1.0"
min_version = "0.1.0"

[security]
enable_encryption = true
max_message_size = 10485760
"#
        .to_string()
    }

    /// 生成共识配置
    fn generate_consensus_config(&self) -> String {
        r#"
# Toki 共识配置
[mining]
thread_count = 0  # 0 = auto detect
initial_difficulty = 1000000
max_difficulty = 1000000000000
min_difficulty = 1000

[difficulty]
adjustment_interval = 100
target_block_time = 10
max_adjustment_rate = 4.0

[reward]
base_reward = 100000000000  # 100 toki
halving_interval = 210000
"#
        .to_string()
    }

    /// 初始化数据库
    fn init_database(&self) -> Result<()> {
        let db_path = self.config.data_dir.join("db");
        if !db_path.exists() {
            fs::create_dir_all(&db_path)?;
        }
        Ok(())
    }

    /// 创建创世区块
    fn create_genesis(&self) -> Result<()> {
        let genesis_path = self.config.data_dir.join("genesis.json");
        if !genesis_path.exists() {
            let genesis = r#"{
  "version": "0.1.0",
  "timestamp": 0,
  "height": 0,
  "transactions": [],
  "state": {
    "total_supply": 814400000000000000,
    "accounts": []
  }
}"#;
            fs::write(&genesis_path, genesis)?;
        }
        Ok(())
    }

    /// 设置系统服务
    fn setup_service(&self) -> Result<()> {
        if self.config.env == DeploymentEnv::Mainnet {
            let service_path = self.config.config_dir.join("toki-node.service");
            let service = r#"[Unit]
Description=Toki Blockchain Node
After=network.target

[Service]
Type=simple
User=toki
Group=toki
WorkingDirectory=/opt/toki
ExecStart=/opt/toki/bin/toki-node start --config /opt/toki/config/node.toml
Restart=always
RestartSec=10
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
"#;
            fs::write(&service_path, service)?;
            info!("生成 systemd 服务文件");
        }
        Ok(())
    }

    /// 检查部署状态
    pub fn check_status(&self) -> DeploymentStatus {
        DeploymentStatus {
            initialized: self.initialized,
            data_dir_exists: self.config.data_dir.exists(),
            config_dir_exists: self.config.config_dir.exists(),
            database_exists: self.config.data_dir.join("db").exists(),
            genesis_exists: self.config.data_dir.join("genesis.json").exists(),
        }
    }

    /// 执行备份
    pub fn backup(&self) -> Result<PathBuf> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("backup_{}", timestamp);
        let backup_path = self.config.backup_dir.join(&backup_name);

        fs::create_dir_all(&backup_path)?;

        // 备份数据目录
        let data_backup = backup_path.join("data");
        if self.config.data_dir.exists() {
            copy_dir_all(&self.config.data_dir, &data_backup).ok();
        }

        // 备份配置目录
        let config_backup = backup_path.join("config");
        if self.config.config_dir.exists() {
            copy_dir_all(&self.config.config_dir, &config_backup).ok();
        }

        info!("备份完成: {:?}", backup_path);
        Ok(backup_path)
    }

    /// 清理旧备份
    pub fn cleanup_old_backups(&self) -> Result<()> {
        let mut backups: Vec<_> = fs::read_dir(&self.config.backup_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("backup_"))
            .collect();

        backups.sort_by_key(|e| e.file_name());

        // 保留最新的 max_backups 个
        let to_remove = backups.len().saturating_sub(self.config.max_backups);
        for entry in backups.into_iter().take(to_remove) {
            fs::remove_dir_all(entry.path())?;
            info!("删除旧备份: {:?}", entry.path());
        }

        Ok(())
    }
}

/// 部署结果
#[derive(Debug, Default)]
pub struct DeploymentResult {
    /// 是否成功
    pub success: bool,
    /// 目录已创建
    pub directories_created: bool,
    /// 配置已生成
    pub configs_generated: bool,
    /// 数据库已初始化
    pub database_initialized: bool,
    /// 创世区块已创建
    pub genesis_created: bool,
    /// 服务已设置
    pub service_setup: bool,
}

/// 部署状态
#[derive(Debug)]
pub struct DeploymentStatus {
    /// 是否已初始化
    pub initialized: bool,
    /// 数据目录存在
    pub data_dir_exists: bool,
    /// 配置目录存在
    pub config_dir_exists: bool,
    /// 数据库存在
    pub database_exists: bool,
    /// 创世区块存在
    pub genesis_exists: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deployment_config_default() {
        let config = DeploymentConfig::default();
        assert_eq!(config.env, DeploymentEnv::Development);
    }

    #[test]
    fn test_deployment_config_mainnet() {
        let config = DeploymentConfig::mainnet();
        assert_eq!(config.env, DeploymentEnv::Mainnet);
        assert!(config.data_dir.to_str().unwrap().contains("/opt/toki"));
    }

    #[test]
    fn test_deployment_config_testnet() {
        let config = DeploymentConfig::testnet();
        assert_eq!(config.env, DeploymentEnv::Testnet);
    }

    #[test]
    fn test_auto_deployer_creation() {
        let deployer = AutoDeployer::new(DeploymentConfig::default());
        assert!(!deployer.initialized);
    }
}
