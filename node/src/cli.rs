//! 命令行界面

use clap::{Parser, Subcommand};

/// Toki 超主权数字货币节点
#[derive(Parser, Debug)]
#[command(name = "toki-node")]
#[command(about = "Toki 超主权数字货币节点", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// 子命令
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 启动节点
    Start {
        /// 配置文件路径
        #[arg(short, long, default_value = "config.toml")]
        config: String,

        /// 数据目录
        #[arg(short, long, default_value = "./data")]
        data_dir: String,

        /// 网络监听地址
        #[arg(short, long, default_value = "/ip4/0.0.0.0/tcp/30333")]
        listen: String,

        /// 种子节点地址（可多次指定）
        #[arg(short, long)]
        bootstrap: Vec<String>,

        /// 启用挖矿
        #[arg(short, long, default_value = "true")]
        mining: bool,

        /// 矿工地址
        #[arg(long)]
        miner_address: Option<String>,

        /// API 监听地址
        #[arg(long, default_value = "0.0.0.0:8080")]
        api: String,

        /// 日志级别
        #[arg(short, long, default_value = "info")]
        log_level: String,
    },

    /// 生成创世区块
    Genesis {
        /// 输出文件路径
        #[arg(short, long, default_value = "genesis.json")]
        output: String,

        /// 链类型 (mainnet/testnet)
        #[arg(short, long, default_value = "testnet")]
        chain: String,
    },

    /// 查询区块链状态
    Status {
        /// 节点 API 地址
        #[arg(short, long, default_value = "http://localhost:8080")]
        node: String,
    },

    /// 查询区块
    Block {
        /// 区块高度或哈希
        #[arg(short, long)]
        height: Option<u64>,

        /// 获取最新区块
        #[arg(short, long)]
        latest: bool,

        /// 节点 API 地址
        #[arg(short, long, default_value = "http://localhost:8080")]
        node: String,
    },

    /// 查询账户
    Account {
        /// 账户地址
        address: String,

        /// 节点 API 地址
        #[arg(short, long, default_value = "http://localhost:8080")]
        node: String,
    },

    /// 发送交易
    Send {
        /// 接收地址
        #[arg(short, long)]
        to: String,

        /// 金额 (toki)
        #[arg(short, long)]
        amount: u64,

        /// 交易费
        #[arg(long, default_value = "1000")]
        fee: u64,

        /// 发送方私钥文件
        #[arg(short, long)]
        keyfile: String,

        /// 节点 API 地址
        #[arg(short, long, default_value = "http://localhost:8080")]
        node: String,
    },

    /// 生成密钥对
    Keygen {
        /// 输出文件路径
        #[arg(short, long, default_value = "key.json")]
        output: String,
    },

    /// 验证区块
    Validate {
        /// 区块文件路径
        #[arg(short, long)]
        file: String,
    },
}

impl Cli {
    /// 解析命令行参数
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
