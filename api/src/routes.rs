//! API 路由定义

use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::{self, ApiState};

/// 创建 API 路由
pub fn create_routes(state: ApiState) -> Router {
    Router::new()
        // 健康检查
        .route("/health", get(handlers::health_check))
        // 节点信息
        .route("/api/v1/node/info", get(handlers::get_node_info))
        // 区块相关
        .route("/api/v1/block/:height", get(handlers::get_block))
        .route("/api/v1/block/latest", get(handlers::get_latest_block))
        .route("/api/v1/block/range", get(handlers::get_block_range))
        // 交易相关
        .route("/api/v1/transaction/:hash", get(handlers::get_transaction))
        .route("/api/v1/transaction/send", post(handlers::send_transaction))
        // 账户相关
        .route("/api/v1/account/:address", get(handlers::get_account))
        .route(
            "/api/v1/account/balance/:address",
            get(handlers::get_balance),
        )
        // 共识相关
        .route(
            "/api/v1/consensus/status",
            get(handlers::get_consensus_status),
        )
        .route(
            "/api/v1/consensus/difficulty",
            get(handlers::get_difficulty),
        )
        // 治理相关
        .route("/api/v1/governance/proposals", get(handlers::get_proposals))
        .route("/api/v1/governance/vote", post(handlers::submit_vote))
        // 开发者相关
        .route(
            "/api/v1/developer/status",
            get(handlers::get_developer_status),
        )
        // 交易池相关
        .route(
            "/api/v1/transaction-pool",
            get(handlers::get_transaction_pool),
        )
        // 收费公告相关
        .route(
            "/api/v1/fee/announcement",
            get(handlers::get_fee_announcement),
        )
        .with_state(state)
}
