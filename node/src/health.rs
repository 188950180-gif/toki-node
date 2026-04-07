//! 健康检查

use serde::{Deserialize, Serialize};

/// 健康状态
#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    /// 是否健康
    pub healthy: bool,
    /// 区块高度
    pub height: u64,
    /// 连接数
    pub connections: usize,
    /// 内存使用（MB）
    pub memory_mb: u64,
    /// 同步状态
    pub sync_status: SyncStatus,
    /// 运行时间（秒）
    pub uptime_secs: u64,
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus {
            healthy: true,
            height: 0,
            connections: 0,
            memory_mb: 0,
            sync_status: SyncStatus::Synced,
            uptime_secs: 0,
        }
    }
}

/// 同步状态
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SyncStatus {
    /// 同步中
    Syncing,
    /// 已同步
    Synced,
    /// 落后
    Behind,
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncStatus::Syncing => write!(f, "syncing"),
            SyncStatus::Synced => write!(f, "synced"),
            SyncStatus::Behind => write!(f, "behind"),
        }
    }
}
