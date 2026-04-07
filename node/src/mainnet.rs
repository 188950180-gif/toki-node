//! 主网部署配置
//!
//! 生产环境部署配置和脚本

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

/// 主网配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainnetConfig {
    /// 链 ID
    pub chain_id: u64,
    /// 数据目录
    pub data_dir: PathBuf,
    /// 网络端口
    pub network_port: u16,
    /// API 端口
    pub api_port: u16,
    /// 种子节点
    pub seed_nodes: Vec<SeedNode>,
    /// 出块时间（秒）
    pub block_time: u64,
    /// 初始难度
    pub initial_difficulty: u64,
    /// 最大连接数
    pub max_connections: usize,
    /// 是否启用挖矿
    pub enable_mining: bool,
    /// 是否启用 API
    pub enable_api: bool,
    /// 日志级别
    pub log_level: String,
    /// 备份配置
    pub backup: BackupConfig,
}

/// 种子节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedNode {
    /// 节点名称
    pub name: String,
    /// IP 地址
    pub ip: String,
    /// 端口
    pub port: u16,
}

impl SeedNode {
    /// 转换为多地址格式
    pub fn to_multiaddr(&self) -> String {
        format!("/ip4/{}/tcp/{}", self.ip, self.port)
    }
}

/// 备份配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// 是否启用自动备份
    pub enabled: bool,
    /// 备份目录
    pub backup_dir: PathBuf,
    /// 备份间隔（秒）
    pub interval: u64,
    /// 最大备份数
    pub max_count: usize,
}

impl Default for MainnetConfig {
    fn default() -> Self {
        MainnetConfig {
            chain_id: 1, // 主网链 ID
            data_dir: PathBuf::from("/opt/toki/data"),
            network_port: 30333,
            api_port: 8080,
            seed_nodes: vec![SeedNode {
                name: "seed-1".to_string(),
                ip: "182.254.176.30".to_string(),
                port: 30333,
            }],
            block_time: 10,
            initial_difficulty: 1_000_000,
            max_connections: 200,
            enable_mining: true,
            enable_api: true,
            log_level: "info".to_string(),
            backup: BackupConfig {
                enabled: true,
                backup_dir: PathBuf::from("/opt/toki/backups"),
                interval: 3600,
                max_count: 20,
            },
        }
    }
}

/// 主网部署器
pub struct MainnetDeployer {
    /// 配置
    config: MainnetConfig,
}

impl MainnetDeployer {
    /// 创建新的部署器
    pub fn new(config: MainnetConfig) -> Self {
        MainnetDeployer { config }
    }

    /// 部署主网
    pub fn deploy(&self) -> Result<MainnetDeploymentReport> {
        info!("开始主网部署...");

        let mut report = MainnetDeploymentReport::default();

        // 1. 创建目录结构
        self.create_directories()?;
        report.directories_created = true;
        info!("✓ 目录结构创建完成");

        // 2. 生成配置文件
        self.generate_configs()?;
        report.configs_generated = true;
        info!("✓ 配置文件生成完成");

        // 3. 生成 systemd 服务文件
        self.generate_systemd_service()?;
        report.service_created = true;
        info!("✓ systemd 服务文件生成完成");

        // 4. 生成启动脚本
        self.generate_scripts()?;
        report.scripts_created = true;
        info!("✓ 启动脚本生成完成");

        // 5. 初始化数据库
        self.init_database()?;
        report.database_initialized = true;
        info!("✓ 数据库初始化完成");

        // 6. 创建创世区块
        self.create_genesis()?;
        report.genesis_created = true;
        info!("✓ 创世区块创建完成");

        report.success = true;

        info!("主网部署完成！");
        Ok(report)
    }

    fn create_directories(&self) -> Result<()> {
        // 创建主目录
        let dirs: Vec<PathBuf> = vec![
            self.config.data_dir.clone(),
            self.config.backup.backup_dir.clone(),
            PathBuf::from("/var/log/toki"),
            PathBuf::from("/opt/toki/bin"),
            PathBuf::from("/opt/toki/config"),
        ];

        for dir in &dirs {
            if !dir.exists() {
                fs::create_dir_all(dir)?;
            }
        }

        // 子目录
        let subdirs = vec!["blocks", "transactions", "accounts", "state", "peers"];

        for subdir in subdirs {
            fs::create_dir_all(self.config.data_dir.join(subdir))?;
        }

        Ok(())
    }

    fn generate_configs(&self) -> Result<()> {
        // 节点配置
        let node_config = format!(
            r#"
# Toki 主网节点配置
# 生成时间: {}

chain_id = {}
data_dir = {:?}
backup_path = {:?}

[network]
listen_addr = "/ip4/0.0.0.0/tcp/{}"
bootstrap_peers = [{}]
max_connections = {}
enable_discovery = true
enable_mdns = false

[consensus]
enable_mining = {}
target_block_time = {}
difficulty_adjust_interval = 100
initial_difficulty = {}

[api]
listen_addr = "0.0.0.0:{}"
enable_cors = true
rate_limit = 100

[ai]
enable_auto_execute = true
enable_self_healing = true
check_interval = 60

[logging]
level = "{}"
file = "/var/log/toki/node.log"
max_size = 104857600  # 100MB
max_files = 10
"#,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            self.config.chain_id,
            self.config.data_dir,
            self.config.backup.backup_dir,
            self.config.network_port,
            self.config
                .seed_nodes
                .iter()
                .map(|s| format!("\"{}\"", s.to_multiaddr()))
                .collect::<Vec<_>>()
                .join(", "),
            self.config.max_connections,
            self.config.enable_mining,
            self.config.block_time,
            self.config.initial_difficulty,
            self.config.api_port,
            self.config.log_level,
        );

        fs::write("/opt/toki/config/node.toml", node_config)?;

        Ok(())
    }

    fn generate_systemd_service(&self) -> Result<()> {
        let service = format!(
            r#"[Unit]
Description=Toki Blockchain Node
Documentation=https://toki.network/docs
After=network.target network-online.target
Wants=network-online.target

[Service]
Type=simple
User=toki
Group=toki
WorkingDirectory=/opt/toki
ExecStart=/opt/toki/bin/toki-node start --config /opt/toki/config/node.toml
ExecReload=/bin/kill -HUP $MAINPID
Restart=always
RestartSec=10
TimeoutStartSec=300
TimeoutStopSec=300

# 资源限制
LimitNOFILE=65536
LimitNPROC=65536

# 安全加固
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/toki /var/log/toki

# 环境变量
Environment="RUST_LOG={}"
Environment="TOKI_CHAIN_ID={}"

[Install]
WantedBy=multi-user.target
"#,
            self.config.log_level, self.config.chain_id,
        );

        fs::write("/opt/toki/config/toki-node.service", service)?;

        Ok(())
    }

    fn generate_scripts(&self) -> Result<()> {
        // 启动脚本
        let start_script = r#"#!/bin/bash
# Toki 节点启动脚本

set -e

echo "启动 Toki 节点..."

# 检查是否已运行
if systemctl is-active --quiet toki-node; then
    echo "节点已在运行"
    exit 0
fi

# 启动服务
sudo systemctl start toki-node
sudo systemctl enable toki-node

echo "节点已启动"
"#;
        fs::write("/opt/toki/bin/start.sh", start_script)?;

        // 停止脚本
        let stop_script = r#"#!/bin/bash
# Toki 节点停止脚本

set -e

echo "停止 Toki 节点..."

sudo systemctl stop toki-node

echo "节点已停止"
"#;
        fs::write("/opt/toki/bin/stop.sh", stop_script)?;

        // 状态检查脚本
        let status_script = r#"#!/bin/bash
# Toki 节点状态检查

echo "=== Toki 节点状态 ==="
echo ""

# 服务状态
echo "服务状态:"
sudo systemctl status toki-node --no-pager
echo ""

# 资源使用
echo "资源使用:"
ps aux | grep toki-node | grep -v grep
echo ""

# 网络连接
echo "网络连接:"
ss -tlnp | grep 30333 || echo "网络端口未监听"
echo ""

# 日志
echo "最近日志:"
sudo journalctl -u toki-node -n 20 --no-pager
"#;
        fs::write("/opt/toki/bin/status.sh", status_script)?;

        Ok(())
    }

    fn init_database(&self) -> Result<()> {
        let db_path = self.config.data_dir.join("db");
        fs::create_dir_all(db_path)?;
        Ok(())
    }

    fn create_genesis(&self) -> Result<()> {
        let genesis = serde_json::json!({
            "chain_id": self.config.chain_id,
            "name": "toki-mainnet",
            "timestamp": 0,
            "height": 0,
            "transactions": [],
            "state": {
                "total_supply": 814400000000000000_u64,
                "accounts": [],
            },
            "config": {
                "block_time": self.config.block_time,
                "initial_difficulty": self.config.initial_difficulty,
            },
        });

        fs::write(
            self.config.data_dir.join("genesis.json"),
            serde_json::to_string_pretty(&genesis)?,
        )?;

        Ok(())
    }

    /// 生成部署指令
    pub fn generate_instructions(&self) -> String {
        format!(
            r#"# Toki 主网部署指令

## 1. 复制文件到服务器
scp -r /opt/toki user@182.254.176.30:/opt/

## 2. 安装服务
sudo cp /opt/toki/config/toki-node.service /etc/systemd/system/
sudo systemctl daemon-reload

## 3. 启动节点
sudo systemctl start toki-node
sudo systemctl enable toki-node

## 4. 检查状态
sudo systemctl status toki-node
sudo journalctl -u toki-node -f

## 5. 防火墙配置
sudo ufw allow {}/tcp  # 网络端口
sudo ufw allow {}/tcp  # API 端口

## 6. 监控
# Prometheus 指标: http://182.254.176.30:{}/metrics
# API 端点: http://182.254.176.30:{}/api/v1
"#,
            self.config.network_port,
            self.config.api_port,
            self.config.api_port,
            self.config.api_port,
        )
    }
}

/// 主网部署报告
#[derive(Debug, Default)]
pub struct MainnetDeploymentReport {
    pub success: bool,
    pub directories_created: bool,
    pub configs_generated: bool,
    pub service_created: bool,
    pub scripts_created: bool,
    pub database_initialized: bool,
    pub genesis_created: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mainnet_config_default() {
        let config = MainnetConfig::default();
        assert_eq!(config.chain_id, 1);
        assert_eq!(config.block_time, 10);
    }

    #[test]
    fn test_seed_node() {
        let seed = SeedNode {
            name: "test".to_string(),
            ip: "192.168.1.1".to_string(),
            port: 30333,
        };

        assert_eq!(seed.to_multiaddr(), "/ip4/192.168.1.1/tcp/30333");
    }
}
