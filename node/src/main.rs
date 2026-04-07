//! Toki 超主权数字货币平台 - 节点主程序

mod auto_deploy;
mod auto_upgrade;
mod cli;
mod config;
mod health;
mod logging;
mod mainnet;
mod metrics;
mod node;
mod testnet;

use anyhow::Result;
use tracing::info;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // 解析命令行参数
    let cli = Cli::parse_args();

    match cli.command {
        Commands::Start {
            config: config_path,
            data_dir,
            listen,
            bootstrap,
            mining,
            miner_address,
            api,
            log_level,
        } => {
            run_start(
                config_path,
                data_dir,
                listen,
                bootstrap,
                mining,
                miner_address,
                api,
                log_level,
            )
            .await?;
        }

        Commands::Genesis { output, chain } => {
            run_genesis(output, chain)?;
        }

        Commands::Status { node } => {
            run_status(node).await?;
        }

        Commands::Block {
            height,
            latest,
            node,
        } => {
            run_block(height, latest, node).await?;
        }

        Commands::Account { address, node } => {
            run_account(address, node).await?;
        }

        Commands::Send {
            to,
            amount,
            fee,
            keyfile,
            node,
        } => {
            run_send(to, amount, fee, keyfile, node).await?;
        }

        Commands::Keygen { output } => {
            run_keygen(output)?;
        }

        Commands::Validate { file } => {
            run_validate(file)?;
        }
    }

    Ok(())
}

/// 启动节点
async fn run_start(
    config_path: String,
    data_dir: String,
    listen: String,
    bootstrap: Vec<String>,
    mining: bool,
    miner_address: Option<String>,
    api: String,
    log_level: String,
) -> Result<()> {
    // 初始化日志
    logging::init_logging(logging::LogConfig {
        level: log_level,
        ..Default::default()
    });

    info!("========================================");
    info!("  Toki 超主权数字货币节点");
    info!("  版本: {}", env!("CARGO_PKG_VERSION"));
    info!("========================================");
    info!("");
    info!("配置文件: {}", config_path);
    info!("数据目录: {}", data_dir);
    info!("网络监听: {}", listen);
    info!("API 监听: {}", api);
    info!("启用挖矿: {}", mining);

    if !bootstrap.is_empty() {
        info!("种子节点: {:?}", bootstrap);
    }

    // 加载配置
    let mut config = config::load_config(&config_path)?;

    // 命令行参数覆盖配置文件
    config.data_dir = data_dir;
    config.network.listen_addr = listen;
    config.network.bootstrap_peers = bootstrap;
    config.consensus.enable_mining = mining;
    config.api.listen_addr = api;

    if let Some(addr) = miner_address {
        config.consensus.miner_address = addr;
    }

    // 创建并启动节点
    let node = node::Node::new(config).await?;
    node.run().await?;

    Ok(())
}

/// 生成创世区块
fn run_genesis(output: String, chain: String) -> Result<()> {
    use toki_core::{Block, GenesisConfig};

    println!("生成创世配置...");

    let genesis_config = match chain.as_str() {
        "mainnet" => GenesisConfig::mainnet(),
        "testnet" => GenesisConfig::testnet(),
        _ => {
            return Err(anyhow::anyhow!("未知链类型: {}", chain));
        }
    };

    // 验证配置
    genesis_config
        .validate()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // 生成创世区块
    let genesis_block = Block::genesis();

    // 保存到文件
    let json = serde_json::to_string_pretty(&genesis_config)?;
    std::fs::write(&output, &json)?;

    println!("创世配置已保存到: {}", output);
    println!("链 ID: {}", genesis_config.chain_id);
    println!("创世区块哈希: {}", genesis_block.hash());

    Ok(())
}

/// 查询节点状态
async fn run_status(node: String) -> Result<()> {
    let url = format!("{}/health", node);
    let resp = reqwest::get(&url).await?;
    let text = resp.text().await?;
    println!("{}", text);
    Ok(())
}

/// 查询区块
async fn run_block(height: Option<u64>, latest: bool, node: String) -> Result<()> {
    let url = if latest {
        format!("{}/api/v1/block/latest", node)
    } else if let Some(h) = height {
        format!("{}/api/v1/block/{}", node, h)
    } else {
        format!("{}/api/v1/block/latest", node)
    };

    let resp = reqwest::get(&url).await?;
    let text = resp.text().await?;
    println!("{}", text);
    Ok(())
}

/// 查询账户
async fn run_account(address: String, node: String) -> Result<()> {
    let url = format!("{}/api/v1/account/{}", node, address);
    let resp = reqwest::get(&url).await?;
    let text = resp.text().await?;
    println!("{}", text);
    Ok(())
}

/// 发送交易
async fn run_send(to: String, amount: u64, fee: u64, keyfile: String, node: String) -> Result<()> {
    println!("发送交易:");
    println!("  接收地址: {}", to);
    println!("  金额: {} toki", amount);
    println!("  交易费: {}", fee);
    println!("  密钥文件: {}", keyfile);
    println!("  节点: {}", node);

    // TODO: 实现交易签名和发送

    Ok(())
}

/// 生成密钥对
fn run_keygen(output: String) -> Result<()> {
    use toki_core::Hash;

    println!("生成密钥对...");

    // 生成随机私钥（32 字节）
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rand::Rng::fill(&mut rng, &mut bytes);
    let private_key = Hash::new(bytes);

    // 保存到文件
    let key_data = serde_json::json!({
        "private_key": private_key.to_hex(),
        "created_at": chrono::Utc::now().to_rfc3339(),
    });

    let json = serde_json::to_string_pretty(&key_data)?;
    std::fs::write(&output, &json)?;

    println!("密钥对已保存到: {}", output);
    println!("私钥: {}", private_key.to_hex());

    Ok(())
}

/// 验证区块
fn run_validate(file: String) -> Result<()> {
    use toki_core::Block;

    println!("验证区块文件: {}", file);

    let content = std::fs::read_to_string(&file)?;
    let block: Block = serde_json::from_str(&content)?;

    println!("区块高度: {}", block.height());
    println!("区块哈希: {}", block.hash());
    println!("交易数量: {}", block.tx_count());
    println!("Merkle 根验证: {}", block.verify_merkle_root());

    if block.verify_merkle_root() {
        println!("✓ 区块验证通过");
    } else {
        println!("✗ 区块验证失败");
    }

    Ok(())
}
