//! 节点实现

use crate::config::NodeConfig;
use anyhow::Result;
use std::sync::Arc;
use toki_api::{ApiConfig, ApiServer, ApiState};
use toki_consensus::{Miner, MiningConfig, TransactionPool, TxPoolConfig};
use toki_core::Block;
use toki_network::{P2PConfig, P2PNode};
use toki_storage::{AccountStore, BlockStore, Database};
use tokio::signal;
use tracing::info;

/// 节点
#[allow(dead_code)]
pub struct Node {
    config: NodeConfig,
    db: Arc<Database>,
    pub block_store: Arc<BlockStore>,
    pub account_store: Arc<AccountStore>,
    miner: Option<Miner>,
    tx_pool: TransactionPool,
    p2p: Option<P2PNode>,
}

impl Node {
    /// 创建新节点
    pub async fn new(config: NodeConfig) -> Result<Self> {
        // 确保数据目录存在
        let data_path = std::path::Path::new(&config.data_dir);
        std::fs::create_dir_all(data_path)?;
        std::fs::create_dir_all(&config.backup_path)?;

        // 打开数据库
        let db = Database::open(data_path)?;
        let db = Arc::new(db);

        // 创建存储
        let block_store = Arc::new(BlockStore::new(Arc::clone(&db)));
        let account_store = Arc::new(AccountStore::new(Arc::clone(&db)));

        // 检查是否需要初始化创世区块
        if block_store.get_latest_height()?.is_none() {
            info!("初始化创世区块...");
            let genesis = Block::genesis();
            block_store.save_block(&genesis)?;
            info!("创世区块已创建: {}", genesis.hash());
        }

        // 创建交易池
        let tx_pool = TransactionPool::new(TxPoolConfig::default());

        // 创建挖矿器（如果启用）
        let miner = if config.consensus.enable_mining {
            let miner_config = MiningConfig::default();
            Some(Miner::new(miner_config))
        } else {
            None
        };

        // 创建 P2P 网络
        let p2p = if config.network.enable_p2p {
            let p2p_config = P2PConfig {
                listen_addr: config.network.listen_addr.clone(),
                bootstrap_peers: config.network.bootstrap_peers.clone(),
                ..Default::default()
            };
            match P2PNode::new(p2p_config).await {
                Ok(node) => Some(node),
                Err(e) => {
                    info!("P2P 网络创建失败: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Node {
            config,
            db,
            block_store,
            account_store,
            miner,
            tx_pool,
            p2p,
        })
    }

    /// 运行节点
    pub async fn run(&self) -> Result<()> {
        info!("");
        info!("节点已启动");
        info!("数据目录: {}", self.config.data_dir);
        info!("网络监听: {}", self.config.network.listen_addr);
        info!("API 监听: {}", self.config.api.listen_addr);
        info!("挖矿: {}", self.config.consensus.enable_mining);

        // 获取当前区块高度
        let height = self.block_store.get_latest_height()?;
        info!("当前区块高度: {:?}", height.unwrap_or(0));

        // 显示连接信息
        if !self.config.network.bootstrap_peers.is_empty() {
            info!("种子节点: {:?}", self.config.network.bootstrap_peers);
        } else {
            info!("作为种子节点运行");
        }

        // 启动挖矿（如果启用）
        if let Some(ref miner) = self.miner {
            info!("启动挖矿...");
            miner.start();
        }

        // 显示交易池状态
        info!("交易池大小: {}", self.tx_pool.tx_count());

        // 启动 API 服务器
        let api_state = ApiState {
            block_store: Arc::clone(&self.block_store),
            account_store: Arc::clone(&self.account_store),
        };
        let api_config = ApiConfig {
            listen_addr: self.config.api.listen_addr.clone(),
            enable_cors: true,
        };
        let api_server = ApiServer::new(api_config, api_state);

        // 在后台启动 API 服务器
        tokio::spawn(async move {
            api_server.run().await;
        });

        info!("");
        info!("按 Ctrl+C 停止节点");
        info!("");

        // 等待关闭信号
        self.wait_for_shutdown().await?;

        info!("节点正在关闭...");

        // 停止挖矿
        if let Some(ref miner) = self.miner {
            miner.stop();
        }

        // 刷新数据库
        self.db.flush()?;

        info!("节点已关闭");

        Ok(())
    }

    /// 等待关闭信号
    async fn wait_for_shutdown(&self) -> Result<()> {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }

        Ok(())
    }
}
