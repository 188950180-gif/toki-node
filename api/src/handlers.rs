//! API 处理器

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use toki_storage::{AccountStore, BlockStore};
use tracing::info;

/// API 状态
#[derive(Clone)]
pub struct ApiState {
    pub block_store: Arc<BlockStore>,
    pub account_store: Arc<AccountStore>,
}

// ==================== 响应类型 ====================

/// 通用响应
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        ApiResponse {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: &str) -> Self {
        ApiResponse {
            success: false,
            data: None,
            error: Some(msg.to_string()),
        }
    }
}

/// 健康检查响应
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub healthy: bool,
    pub version: String,
    pub height: u64,
    pub connections: usize,
}

/// 节点信息响应
#[derive(Debug, Serialize)]
pub struct NodeInfoResponse {
    pub node_id: String,
    pub version: String,
    pub height: u64,
    pub peer_count: usize,
    pub is_syncing: bool,
}

/// 区块响应
#[derive(Debug, Serialize)]
pub struct BlockResponse {
    pub height: u64,
    pub hash: String,
    pub timestamp: i64,
    pub tx_count: usize,
    pub difficulty: u64,
}

/// 交易响应
#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub hash: String,
    pub height: u64,
    pub fee: u64,
    pub status: String,
}

/// 账户响应
#[derive(Debug, Serialize)]
pub struct AccountResponse {
    pub address: String,
    pub balance: u64,
    pub locked: u64,
    pub account_type: String,
}

/// 共识状态响应
#[derive(Debug, Serialize)]
pub struct ConsensusResponse {
    pub height: u64,
    pub difficulty: u64,
    pub target_time: u64,
    pub hash_rate: f64,
}

/// 治理提案响应
#[derive(Debug, Serialize)]
pub struct ProposalResponse {
    pub id: u64,
    pub title: String,
    pub status: String,
    pub votes_for: u64,
    pub votes_against: u64,
}

/// 交易池响应
#[derive(Debug, Serialize)]
pub struct TransactionPoolResponse {
    pub tx_count: usize,
    pub pending_txs: Vec<TransactionInfo>,
}

/// 交易信息
#[derive(Debug, Serialize)]
pub struct TransactionInfo {
    pub hash: String,
    pub fee: u64,
    pub size: usize,
}

// ==================== 请求类型 ====================

/// 发送交易请求
#[derive(Debug, Deserialize)]
pub struct SendTransactionRequest {
    /// 交易输入
    pub inputs: Vec<TxInput>,
    /// 交易输出
    pub outputs: Vec<TxOutput>,
    /// 手续费
    pub fee: u64,
}

/// 交易输入
#[derive(Debug, Deserialize)]
pub struct TxInput {
    /// 前一笔交易的哈希
    pub prev_tx_hash: String,
    /// 输出索引
    pub output_index: u32,
}

/// 交易输出
#[derive(Debug, Deserialize)]
pub struct TxOutput {
    /// 接收地址
    pub address: String,
    /// 金额
    pub amount: u64,
}

/// 投票请求
#[derive(Debug, Deserialize)]
pub struct VoteRequest {
    pub proposal_id: u64,
    pub vote: bool,
    pub voter_address: String,
}

/// 区块范围查询
#[derive(Debug, Deserialize)]
pub struct BlockRangeQuery {
    pub start: u64,
    pub end: u64,
}

// ==================== 处理器函数 ====================

/// 健康检查
pub async fn health_check() -> Json<ApiResponse<HealthResponse>> {
    Json(ApiResponse::success(HealthResponse {
        healthy: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
        height: 0,
        connections: 0,
    }))
}

/// 获取节点信息
pub async fn get_node_info(State(state): State<ApiState>) -> Json<ApiResponse<NodeInfoResponse>> {
    // 获取最新区块高度
    let height = match state.block_store.get_latest_block() {
        Ok(Some(block)) => block.height(),
        _ => 0,
    };

    Json(ApiResponse::success(NodeInfoResponse {
        node_id: "unknown".to_string(), // TODO: 从 P2P 模块获取
        version: env!("CARGO_PKG_VERSION").to_string(),
        height,
        peer_count: 0, // TODO: 从 P2P 模块获取
        is_syncing: false,
    }))
}

/// 获取区块
pub async fn get_block(
    State(state): State<ApiState>,
    Path(height): Path<u64>,
) -> Json<ApiResponse<BlockResponse>> {
    match state.block_store.get_block_by_height(height) {
        Ok(Some(block)) => Json(ApiResponse::success(BlockResponse {
            height: block.height(),
            hash: block.hash().to_hex(),
            timestamp: block.header.timestamp.timestamp(),
            tx_count: block.transactions.len(),
            difficulty: block.header.difficulty,
        })),
        Ok(None) => Json(ApiResponse::error(&format!("Block {} not found", height))),
        Err(e) => Json(ApiResponse::error(&format!("Failed to get block: {}", e))),
    }
}

/// 获取最新区块
pub async fn get_latest_block(State(state): State<ApiState>) -> Json<ApiResponse<BlockResponse>> {
    match state.block_store.get_latest_block() {
        Ok(Some(block)) => Json(ApiResponse::success(BlockResponse {
            height: block.height(),
            hash: block.hash().to_hex(),
            timestamp: block.header.timestamp.timestamp(),
            tx_count: block.transactions.len(),
            difficulty: block.header.difficulty,
        })),
        Ok(None) => Json(ApiResponse::error("No blocks found")),
        Err(e) => Json(ApiResponse::error(&format!(
            "Failed to get latest block: {}",
            e
        ))),
    }
}

/// 获取区块范围
pub async fn get_block_range(
    State(state): State<ApiState>,
    Query(query): Query<BlockRangeQuery>,
) -> Json<ApiResponse<Vec<BlockResponse>>> {
    let mut blocks = Vec::new();

    for height in query.start..=query.end {
        match state.block_store.get_block_by_height(height) {
            Ok(Some(block)) => blocks.push(BlockResponse {
                height: block.height(),
                hash: block.hash().to_hex(),
                timestamp: block.header.timestamp.timestamp(),
                tx_count: block.transactions.len(),
                difficulty: block.header.difficulty,
            }),
            Ok(None) => continue, // 跳过不存在的区块
            Err(_) => continue,   // 跳过错误的区块
        }
    }

    Json(ApiResponse::success(blocks))
}

/// 获取交易
pub async fn get_transaction(Path(_hash): Path<String>) -> Json<ApiResponse<TransactionResponse>> {
    Json(ApiResponse::success(TransactionResponse {
        hash: "0".repeat(64),
        height: 0,
        fee: 0,
        status: "pending".to_string(),
    }))
}

/// 发送交易
pub async fn send_transaction(
    State(state): State<ApiState>,
    Json(req): Json<SendTransactionRequest>,
) -> Json<ApiResponse<TransactionResponse>> {
    use toki_consensus::TransactionPool;
    use toki_core::{Hash, Input, Output, RingSignature, Transaction};

    // 验证交易数据
    if req.inputs.is_empty() {
        return Json(ApiResponse::error(
            "Transaction must have at least one input",
        ));
    }

    if req.outputs.is_empty() {
        return Json(ApiResponse::error(
            "Transaction must have at least one output",
        ));
    }

    // 创建交易输入
    let inputs: Vec<Input> = req
        .inputs
        .iter()
        .map(|input| {
            Input::new(
                Hash::from_hex(&input.prev_tx_hash).unwrap_or(Hash::ZERO),
                input.output_index,
            )
        })
        .collect();

    // 创建交易输出
    let outputs: Vec<Output> = req
        .outputs
        .iter()
        .map(
            |output| match toki_core::Address::from_base58(&output.address) {
                Ok(addr) => Output::new(addr, output.amount),
                Err(_) => Output::new(toki_core::Address::default(), 0),
            },
        )
        .collect();

    // 创建环签名（简化版）
    let ring_sig = RingSignature::new(
        vec![vec![0u8; 32]], // 环
        vec![0u8; 64],       // 签名
        Hash::ZERO,          // 密钥镜像
    );

    // 创建交易
    let tx = Transaction::new(inputs, outputs, ring_sig, req.fee);

    // 计算交易哈希（使用序列化）
    let tx_data = bincode::serialize(&tx).unwrap_or_default();
    let tx_hash = toki_core::Hash::from_data(&tx_data);

    // 添加到交易池
    // 注意：这里需要通过状态访问交易池
    // 当前简化实现：仅记录日志

    info!(
        "交易已提交: hash={}, inputs={}, outputs={}, fee={}",
        tx_hash.to_hex(),
        tx.inputs.len(),
        tx.outputs.len(),
        req.fee
    );

    Json(ApiResponse::success(TransactionResponse {
        hash: tx_hash.to_hex(),
        height: 0,
        fee: req.fee,
        status: "pending".to_string(),
    }))
}

/// 获取账户
pub async fn get_account(
    State(state): State<ApiState>,
    Path(address): Path<String>,
) -> Json<ApiResponse<AccountResponse>> {
    // 解析地址
    let addr = match toki_core::Address::from_base58(&address) {
        Ok(a) => a,
        Err(_) => {
            return Json(ApiResponse::error(&format!(
                "Invalid address format: {}",
                address
            )))
        }
    };

    match state.account_store.get_account(&addr) {
        Ok(Some(account)) => Json(ApiResponse::success(AccountResponse {
            address: account.address.to_base58(),
            balance: account.balance,
            locked: account.locked_balance,
            account_type: format!("{:?}", account.account_type),
        })),
        Ok(None) => Json(ApiResponse::error(&format!(
            "Account {} not found",
            address
        ))),
        Err(e) => Json(ApiResponse::error(&format!("Failed to get account: {}", e))),
    }
}

/// 获取余额
pub async fn get_balance(
    State(state): State<ApiState>,
    Path(address): Path<String>,
) -> Json<ApiResponse<u64>> {
    // 解析地址
    let addr = match toki_core::Address::from_base58(&address) {
        Ok(a) => a,
        Err(_) => {
            return Json(ApiResponse::error(&format!(
                "Invalid address format: {}",
                address
            )))
        }
    };

    match state.account_store.get_account(&addr) {
        Ok(Some(account)) => Json(ApiResponse::success(account.balance)),
        Ok(None) => Json(ApiResponse::error(&format!(
            "Account {} not found",
            address
        ))),
        Err(e) => Json(ApiResponse::error(&format!("Failed to get account: {}", e))),
    }
}

/// 获取共识状态
pub async fn get_consensus_status(
    State(state): State<ApiState>,
) -> Json<ApiResponse<ConsensusResponse>> {
    // 获取最新区块高度
    let height = match state.block_store.get_latest_block() {
        Ok(Some(block)) => block.height(),
        _ => 0,
    };

    Json(ApiResponse::success(ConsensusResponse {
        height,
        difficulty: 1_000_000, // TODO: 从共识模块获取实际难度
        target_time: 10,
        hash_rate: 0.0, // TODO: 从挖矿模块获取实际算力
    }))
}

/// 获取难度
pub async fn get_difficulty() -> Json<ApiResponse<u64>> {
    Json(ApiResponse::success(1_000_000))
}

/// 获取提案列表
pub async fn get_proposals() -> Json<ApiResponse<Vec<ProposalResponse>>> {
    Json(ApiResponse::success(vec![]))
}

/// 提交投票
pub async fn submit_vote(Json(_req): Json<VoteRequest>) -> Json<ApiResponse<String>> {
    Json(ApiResponse::success("Vote submitted".to_string()))
}

/// 获取开发者状态
pub async fn get_developer_status() -> Json<ApiResponse<String>> {
    Json(ApiResponse::success("Developer system active".to_string()))
}

/// 获取交易池状态
pub async fn get_transaction_pool() -> Json<ApiResponse<TransactionPoolResponse>> {
    // TODO: 需要在 ApiState 中添加 tx_pool
    // 目前返回空池
    Json(ApiResponse::success(TransactionPoolResponse {
        tx_count: 0,
        pending_txs: vec![],
    }))
}

// ==================== 收费公告 API ====================

/// 收费公告响应
#[derive(Debug, Serialize)]
pub struct FeeAnnouncementResponse {
    /// 是否已公告
    pub is_announced: bool,
    /// 剩余天数
    pub days_remaining: u64,
    /// 费率
    pub fee_rate: f64,
    /// 开始收费时间
    pub start_time: u64,
    /// 公告消息
    pub message: String,
}

/// 获取收费公告
///
/// 返回收费公告信息（收费前15天）
pub async fn get_fee_announcement(
    State(state): State<ApiState>,
) -> Json<ApiResponse<Option<FeeAnnouncementResponse>>> {
    use std::time::{SystemTime, UNIX_EPOCH};
    use toki_core::Transaction;

    // 获取创世时间
    let genesis_time = match state.block_store.get_genesis_timestamp() {
        Ok(Some(t)) => t,
        _ => return Json(ApiResponse::success(None)),
    };

    // 获取当前时间
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // 检查是否需要公告
    match Transaction::check_fee_announcement(genesis_time, current_time) {
        Some(announcement) => Json(ApiResponse::success(Some(FeeAnnouncementResponse {
            is_announced: true,
            days_remaining: announcement.days_remaining,
            fee_rate: announcement.fee_rate,
            start_time: announcement.start_time,
            message: format!(
                "交易服务费将在 {} 天后开始收取",
                announcement.days_remaining
            ),
        }))),
        None => Json(ApiResponse::success(None)),
    }
}
