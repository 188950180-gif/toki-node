//! 自愈系统
//! 
//! 自动检测和修复系统故障

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use async_trait::async_trait;
use tracing::{info, warn, error};

/// 组件类型
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Component {
    /// 数据库
    Database,
    /// 网络
    Network,
    /// 共识
    Consensus,
    /// 挖矿
    Mining,
    /// API 服务
    Api,
    /// 存储层
    Storage,
    /// 交易池
    TxPool,
    /// AI 服务
    AiService,
}

/// 错误类型
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ErrorType {
    /// 连接失败
    ConnectionFailed,
    /// 超时
    Timeout,
    /// 内存不足
    OutOfMemory,
    /// 磁盘空间不足
    DiskFull,
    /// 数据损坏
    DataCorruption,
    /// 进程崩溃
    ProcessCrash,
    /// 网络分区
    NetworkPartition,
    /// 共识失败
    ConsensusFailure,
    /// 同步失败
    SyncFailed,
    /// 自定义错误
    Custom(String),
}

/// 恢复策略
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// 重启组件
    Restart,
    /// 回滚状态
    Rollback,
    /// 重建数据
    Rebuild,
    /// 清理缓存
    ClearCache,
    /// 发送告警
    Alert,
    /// 等待恢复
    Wait(Duration),
    /// 自定义操作
    Custom(String),
}

/// 健康状态
#[derive(Clone, Debug)]
pub struct HealthStatus {
    /// 是否健康
    pub healthy: bool,
    /// 组件状态
    pub components: HashMap<Component, ComponentHealth>,
    /// 检测时间
    pub check_time: Instant,
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus {
            healthy: true,
            components: HashMap::new(),
            check_time: Instant::now(),
        }
    }
}

/// 组件健康状态
#[derive(Clone, Debug)]
pub struct ComponentHealth {
    /// 是否健康
    pub healthy: bool,
    /// 错误类型
    pub error: Option<ErrorType>,
    /// 错误消息
    pub message: Option<String>,
    /// 最后正常时间
    pub last_healthy: Option<Instant>,
}

/// 故障记录
#[derive(Clone, Debug)]
pub struct FailureRecord {
    /// 组件
    pub component: Component,
    /// 错误类型
    pub error_type: ErrorType,
    /// 错误消息
    pub message: String,
    /// 发生时间
    pub timestamp: Instant,
    /// 恢复策略
    pub recovery: RecoveryStrategy,
    /// 是否已恢复
    pub recovered: bool,
}

/// 健康检查器
#[async_trait]
pub trait HealthChecker: Send + Sync {
    /// 检查健康状态
    async fn check(&self) -> HealthStatus;
    
    /// 检查特定组件
    async fn check_component(&self, component: &Component) -> ComponentHealth;
}

/// 恢复执行器
#[async_trait]
pub trait RecoveryExecutor: Send + Sync {
    /// 执行恢复
    async fn execute(&self, component: &Component, strategy: &RecoveryStrategy) -> Result<()>;
}

/// 自愈系统配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelfHealingConfig {
    /// 检查间隔（秒）
    pub check_interval: u64,
    /// 最大恢复尝试次数
    pub max_recovery_attempts: u32,
    /// 恢复间隔（秒）
    pub recovery_interval: u64,
    /// 是否启用自动恢复
    pub auto_recovery: bool,
    /// 是否发送告警
    pub send_alerts: bool,
}

impl Default for SelfHealingConfig {
    fn default() -> Self {
        SelfHealingConfig {
            check_interval: 60,
            max_recovery_attempts: 3,
            recovery_interval: 30,
            auto_recovery: true,
            send_alerts: true,
        }
    }
}

/// 自愈系统
pub struct SelfHealingSystem {
    /// 配置
    config: SelfHealingConfig,
    /// 健康检查器
    health_checker: Option<Box<dyn HealthChecker>>,
    /// 恢复执行器
    recovery_executor: Option<Box<dyn RecoveryExecutor>>,
    /// 恢复策略映射
    recovery_strategies: HashMap<ErrorType, RecoveryStrategy>,
    /// 故障历史
    failure_history: Vec<FailureRecord>,
    /// 恢复尝试计数
    recovery_attempts: HashMap<Component, u32>,
    /// 上次检查时间
    last_check: Option<Instant>,
}

impl SelfHealingSystem {
    /// 创建新的自愈系统
    pub fn new() -> Self {
        let mut recovery_strategies = HashMap::new();
        
        // 默认恢复策略
        recovery_strategies.insert(ErrorType::ConnectionFailed, RecoveryStrategy::Restart);
        recovery_strategies.insert(ErrorType::Timeout, RecoveryStrategy::Wait(Duration::from_secs(10)));
        recovery_strategies.insert(ErrorType::OutOfMemory, RecoveryStrategy::ClearCache);
        recovery_strategies.insert(ErrorType::DiskFull, RecoveryStrategy::Alert);
        recovery_strategies.insert(ErrorType::DataCorruption, RecoveryStrategy::Rebuild);
        recovery_strategies.insert(ErrorType::ProcessCrash, RecoveryStrategy::Restart);
        recovery_strategies.insert(ErrorType::NetworkPartition, RecoveryStrategy::Wait(Duration::from_secs(30)));
        recovery_strategies.insert(ErrorType::ConsensusFailure, RecoveryStrategy::Rollback);
        recovery_strategies.insert(ErrorType::SyncFailed, RecoveryStrategy::Restart);
        
        SelfHealingSystem {
            config: SelfHealingConfig::default(),
            health_checker: None,
            recovery_executor: None,
            recovery_strategies,
            failure_history: Vec::new(),
            recovery_attempts: HashMap::new(),
            last_check: None,
        }
    }

    /// 使用配置创建
    pub fn with_config(config: SelfHealingConfig) -> Self {
        let mut system = Self::new();
        system.config = config;
        system
    }

    /// 设置健康检查器
    pub fn set_health_checker(&mut self, checker: Box<dyn HealthChecker>) {
        self.health_checker = Some(checker);
    }

    /// 设置恢复执行器
    pub fn set_recovery_executor(&mut self, executor: Box<dyn RecoveryExecutor>) {
        self.recovery_executor = Some(executor);
    }

    /// 添加恢复策略
    pub fn add_recovery_strategy(&mut self, error_type: ErrorType, strategy: RecoveryStrategy) {
        self.recovery_strategies.insert(error_type, strategy);
    }

    /// 检测并修复问题
    pub async fn detect_and_heal(&mut self) -> Result<Vec<FailureRecord>> {
        let mut healed = Vec::new();
        
        if let Some(ref checker) = self.health_checker {
            let health = checker.check().await;
            self.last_check = Some(Instant::now());
            
            for (component, status) in &health.components {
                if !status.healthy {
                    if let Some(ref error_type) = status.error {
                        let message = status.message.clone().unwrap_or_default();
                        
                        warn!("检测到故障: {:?} - {:?}: {}", component, error_type, message);
                        
                        // 记录故障
                        let record = FailureRecord {
                            component: component.clone(),
                            error_type: error_type.clone(),
                            message: message.clone(),
                            timestamp: Instant::now(),
                            recovery: self.recovery_strategies.get(error_type)
                                .cloned()
                                .unwrap_or(RecoveryStrategy::Alert),
                            recovered: false,
                        };
                        
                        self.failure_history.push(record.clone());
                        
                        // 尝试恢复
                        if self.config.auto_recovery {
                            if let Some(record) = self.try_recover(component, error_type).await? {
                                healed.push(record);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(healed)
    }

    /// 尝试恢复
    async fn try_recover(&mut self, component: &Component, error_type: &ErrorType) -> Result<Option<FailureRecord>> {
        // 检查恢复尝试次数
        let attempts = self.recovery_attempts.get(component).copied().unwrap_or(0);
        if attempts >= self.config.max_recovery_attempts {
            error!("组件 {:?} 恢复失败次数过多，停止尝试", component);
            return Ok(None);
        }
        
        // 获取恢复策略
        let strategy = self.recovery_strategies.get(error_type)
            .cloned()
            .unwrap_or(RecoveryStrategy::Alert);
        
        info!("尝试恢复 {:?} 使用策略 {:?}", component, strategy);
        
        // 执行恢复
        if let Some(ref executor) = self.recovery_executor {
            match executor.execute(component, &strategy).await {
                Ok(()) => {
                    info!("组件 {:?} 恢复成功", component);
                    self.recovery_attempts.remove(component);
                    
                    // 更新故障记录
                    if let Some(record) = self.failure_history.last_mut() {
                        record.recovered = true;
                    }
                    
                    return Ok(self.failure_history.last().cloned());
                }
                Err(e) => {
                    warn!("组件 {:?} 恢复失败: {}", component, e);
                    self.recovery_attempts.insert(component.clone(), attempts + 1);
                }
            }
        }
        
        Ok(None)
    }

    /// 获取故障历史
    pub fn get_failure_history(&self) -> &[FailureRecord] {
        &self.failure_history
    }

    /// 清理历史
    pub fn clear_history(&mut self) {
        self.failure_history.clear();
        self.recovery_attempts.clear();
    }

    /// 获取健康状态摘要
    pub fn health_summary(&self) -> HealthSummary {
        let total_failures = self.failure_history.len();
        let recovered = self.failure_history.iter().filter(|r| r.recovered).count();
        
        HealthSummary {
            total_failures,
            recovered,
            pending: total_failures - recovered,
            last_check: self.last_check,
        }
    }
}

impl Default for SelfHealingSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// 健康状态摘要
#[derive(Debug)]
pub struct HealthSummary {
    /// 总故障数
    pub total_failures: usize,
    /// 已恢复数
    pub recovered: usize,
    /// 待处理数
    pub pending: usize,
    /// 上次检查时间
    pub last_check: Option<Instant>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_healing_creation() {
        let system = SelfHealingSystem::new();
        assert!(system.health_checker.is_none());
        assert!(system.recovery_executor.is_none());
    }

    #[test]
    fn test_recovery_strategies() {
        let system = SelfHealingSystem::new();
        
        // 检查默认策略
        assert!(system.recovery_strategies.contains_key(&ErrorType::ConnectionFailed));
        assert!(system.recovery_strategies.contains_key(&ErrorType::OutOfMemory));
        assert!(system.recovery_strategies.contains_key(&ErrorType::DataCorruption));
    }

    #[test]
    fn test_add_recovery_strategy() {
        let mut system = SelfHealingSystem::new();
        
        system.add_recovery_strategy(
            ErrorType::Custom("test".to_string()),
            RecoveryStrategy::Restart,
        );
        
        assert!(system.recovery_strategies.contains_key(&ErrorType::Custom("test".to_string())));
    }

    #[test]
    fn test_health_summary() {
        let mut system = SelfHealingSystem::new();
        
        // 添加一些故障记录
        system.failure_history.push(FailureRecord {
            component: Component::Database,
            error_type: ErrorType::ConnectionFailed,
            message: "test".to_string(),
            timestamp: Instant::now(),
            recovery: RecoveryStrategy::Restart,
            recovered: true,
        });
        
        system.failure_history.push(FailureRecord {
            component: Component::Network,
            error_type: ErrorType::Timeout,
            message: "test".to_string(),
            timestamp: Instant::now(),
            recovery: RecoveryStrategy::Wait(Duration::from_secs(10)),
            recovered: false,
        });
        
        let summary = system.health_summary();
        assert_eq!(summary.total_failures, 2);
        assert_eq!(summary.recovered, 1);
        assert_eq!(summary.pending, 1);
    }
}
