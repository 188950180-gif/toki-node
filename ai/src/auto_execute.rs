//! 自动执行引擎
//!
//! 条件触发自动执行系统

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

use toki_core::Address;

/// 自动执行规则 ID
pub type RuleId = u64;

/// 触发条件
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Condition {
    /// 区块高度达到
    BlockHeight(u64),
    /// 时间戳到达
    Timestamp(u64),
    /// 账户余额超过阈值
    BalanceExceeds { address: Address, threshold: u64 },
    /// 交易池大小超过
    TxPoolExceeds(u64),
    /// 网络连接数低于
    PeerCountBelow(u64),
    /// 内存使用率超过
    MemoryUsageExceeds(f64),
    /// CPU 使用率超过
    CpuUsageExceeds(f64),
    /// 自定义条件表达式
    Custom(String),
}

/// 执行动作
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Action {
    /// 发送交易
    SendTransaction {
        to: Address,
        amount: u64,
        data: Vec<u8>,
    },
    /// 调整系统参数
    AdjustParam { key: String, value: String },
    /// 触发治理提案
    TriggerProposal { title: String, content: String },
    /// 执行平权削减
    ExecuteEqualization,
    /// 执行基础派发
    ExecuteDistribution,
    /// 清理缓存
    ClearCache,
    /// 压缩数据库
    CompactDatabase,
    /// 发送告警
    SendAlert { level: AlertLevel, message: String },
    /// 暂停挖矿
    PauseMining,
    /// 恢复挖矿
    ResumeMining,
}

/// 告警级别
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

/// 自动执行规则
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutoExecutionRule {
    /// 规则 ID
    pub id: RuleId,
    /// 规则名称
    pub name: String,
    /// 触发条件
    pub condition: Condition,
    /// 执行动作
    pub action: Action,
    /// 优先级（越高越优先）
    pub priority: u32,
    /// 是否启用
    pub enabled: bool,
    /// 最大执行次数（0 表示无限制）
    pub max_executions: u32,
    /// 已执行次数
    pub execution_count: u32,
    /// 冷却时间（秒）
    pub cooldown: u64,
    /// 上次执行时间
    pub last_execution: u64,
}

impl AutoExecutionRule {
    /// 创建新规则
    pub fn new(id: RuleId, name: &str, condition: Condition, action: Action) -> Self {
        AutoExecutionRule {
            id,
            name: name.to_string(),
            condition,
            action,
            priority: 100,
            enabled: true,
            max_executions: 0,
            execution_count: 0,
            cooldown: 60,
            last_execution: 0,
        }
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// 设置冷却时间
    pub fn with_cooldown(mut self, cooldown: u64) -> Self {
        self.cooldown = cooldown;
        self
    }

    /// 设置最大执行次数
    pub fn with_max_executions(mut self, max: u32) -> Self {
        self.max_executions = max;
        self
    }

    /// 检查是否可以执行
    pub fn can_execute(&self, current_time: u64) -> bool {
        if !self.enabled {
            return false;
        }

        if self.max_executions > 0 && self.execution_count >= self.max_executions {
            return false;
        }

        if current_time < self.last_execution + self.cooldown {
            return false;
        }

        true
    }
}

/// 区块链状态
#[derive(Clone, Debug, Default)]
pub struct BlockchainState {
    /// 当前区块高度
    pub block_height: u64,
    /// 当前时间戳
    pub timestamp: u64,
    /// 交易池大小
    pub tx_pool_size: u64,
    /// 连接节点数
    pub peer_count: usize,
    /// 内存使用率
    pub memory_usage: f64,
    /// CPU 使用率
    pub cpu_usage: f64,
    /// 账户余额映射
    pub balances: HashMap<Address, u64>,
}

/// 动作执行器
#[async_trait]
pub trait ActionExecutor: Send + Sync {
    /// 执行动作
    async fn execute(&self, action: &Action) -> Result<()>;
}

/// 自动执行引擎
pub struct AutoExecutionEngine {
    /// 执行规则
    rules: HashMap<RuleId, AutoExecutionRule>,
    /// 下一个规则 ID
    next_rule_id: RuleId,
    /// 执行器
    executor: Option<Box<dyn ActionExecutor>>,
    /// 执行历史
    execution_history: Vec<ExecutionRecord>,
}

/// 执行记录
#[derive(Clone, Debug)]
pub struct ExecutionRecord {
    /// 规则 ID
    pub rule_id: RuleId,
    /// 规则名称
    pub rule_name: String,
    /// 执行时间
    pub timestamp: u64,
    /// 执行的动作
    pub action: Action,
    /// 是否成功
    pub success: bool,
}

impl AutoExecutionEngine {
    /// 创建新的自动执行引擎
    pub fn new() -> Self {
        AutoExecutionEngine {
            rules: HashMap::new(),
            next_rule_id: 1,
            executor: None,
            execution_history: Vec::new(),
        }
    }

    /// 设置执行器
    pub fn set_executor(&mut self, executor: Box<dyn ActionExecutor>) {
        self.executor = Some(executor);
    }

    /// 添加规则
    pub fn add_rule(&mut self, mut rule: AutoExecutionRule) -> RuleId {
        let id = self.next_rule_id;
        rule.id = id;
        self.rules.insert(id, rule);
        self.next_rule_id += 1;
        info!("添加自动执行规则: ID={}", id);
        id
    }

    /// 移除规则
    pub fn remove_rule(&mut self, id: RuleId) -> Option<AutoExecutionRule> {
        self.rules.remove(&id)
    }

    /// 启用规则
    pub fn enable_rule(&mut self, id: RuleId) {
        if let Some(rule) = self.rules.get_mut(&id) {
            rule.enabled = true;
        }
    }

    /// 禁用规则
    pub fn disable_rule(&mut self, id: RuleId) {
        if let Some(rule) = self.rules.get_mut(&id) {
            rule.enabled = false;
        }
    }

    /// 获取所有规则
    pub fn get_rules(&self) -> Vec<&AutoExecutionRule> {
        self.rules.values().collect()
    }

    /// 评估条件
    pub fn evaluate_condition(&self, condition: &Condition, state: &BlockchainState) -> bool {
        match condition {
            Condition::BlockHeight(height) => state.block_height >= *height,
            Condition::Timestamp(ts) => state.timestamp >= *ts,
            Condition::BalanceExceeds { address, threshold } => {
                state.balances.get(address).copied().unwrap_or(0) >= *threshold
            }
            Condition::TxPoolExceeds(size) => state.tx_pool_size >= *size,
            Condition::PeerCountBelow(count) => state.peer_count < *count as usize,
            Condition::MemoryUsageExceeds(rate) => state.memory_usage >= *rate,
            Condition::CpuUsageExceeds(rate) => state.cpu_usage >= *rate,
            Condition::Custom(expr) => self.evaluate_custom(expr, state),
        }
    }

    /// 评估自定义条件
    fn evaluate_custom(&self, _expr: &str, _state: &BlockchainState) -> bool {
        // TODO: 实现自定义表达式解析
        false
    }

    /// 检查并执行规则
    pub async fn check_and_execute(
        &mut self,
        state: &BlockchainState,
    ) -> Result<Vec<ExecutionRecord>> {
        let mut executed = Vec::new();

        // 收集需要执行的规则 ID
        let mut to_execute: Vec<RuleId> = Vec::new();
        for (id, rule) in &self.rules {
            if rule.can_execute(state.timestamp) && self.evaluate_condition(&rule.condition, state)
            {
                to_execute.push(*id);
            }
        }

        // 按优先级排序
        to_execute.sort_by(|a, b| {
            let pa = self.rules.get(a).map(|r| r.priority).unwrap_or(0);
            let pb = self.rules.get(b).map(|r| r.priority).unwrap_or(0);
            pb.cmp(&pa)
        });

        // 执行规则
        for id in to_execute {
            if let Some(rule) = self.rules.get_mut(&id) {
                let success = if let Some(ref executor) = self.executor {
                    executor.execute(&rule.action).await.is_ok()
                } else {
                    info!("规则 {} 触发，但无执行器", rule.name);
                    true
                };

                rule.execution_count += 1;
                rule.last_execution = state.timestamp;

                let record = ExecutionRecord {
                    rule_id: rule.id,
                    rule_name: rule.name.clone(),
                    timestamp: state.timestamp,
                    action: rule.action.clone(),
                    success,
                };

                info!("执行规则: {} -> {:?}", rule.name, rule.action);
                executed.push(record.clone());
                self.execution_history.push(record);
            }
        }

        Ok(executed)
    }

    /// 获取执行历史
    pub fn get_history(&self) -> &[ExecutionRecord] {
        &self.execution_history
    }

    /// 清理历史
    pub fn clear_history(&mut self) {
        self.execution_history.clear();
    }
}

impl Default for AutoExecutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_creation() {
        let rule = AutoExecutionRule::new(
            1,
            "test_rule",
            Condition::BlockHeight(100),
            Action::ClearCache,
        );

        assert_eq!(rule.id, 1);
        assert_eq!(rule.name, "test_rule");
        assert!(rule.enabled);
    }

    #[test]
    fn test_rule_can_execute() {
        let rule =
            AutoExecutionRule::new(1, "test", Condition::BlockHeight(100), Action::ClearCache)
                .with_cooldown(10);

        // 应该可以执行
        assert!(rule.can_execute(100));

        // 冷却期内不能执行
        let rule =
            AutoExecutionRule::new(1, "test", Condition::BlockHeight(100), Action::ClearCache)
                .with_cooldown(10);

        let mut rule = rule;
        rule.last_execution = 100;
        assert!(!rule.can_execute(105)); // 冷却期内
        assert!(rule.can_execute(111)); // 冷却期后
    }

    #[test]
    fn test_evaluate_condition() {
        let engine = AutoExecutionEngine::new();

        let state = BlockchainState {
            block_height: 150,
            timestamp: 1000,
            tx_pool_size: 500,
            peer_count: 10,
            memory_usage: 0.7,
            cpu_usage: 0.5,
            balances: HashMap::new(),
        };

        // 测试区块高度条件
        assert!(engine.evaluate_condition(&Condition::BlockHeight(100), &state));
        assert!(!engine.evaluate_condition(&Condition::BlockHeight(200), &state));

        // 测试交易池条件
        assert!(engine.evaluate_condition(&Condition::TxPoolExceeds(400), &state));
        assert!(!engine.evaluate_condition(&Condition::TxPoolExceeds(600), &state));

        // 测试节点数条件
        assert!(engine.evaluate_condition(&Condition::PeerCountBelow(15), &state));
        assert!(!engine.evaluate_condition(&Condition::PeerCountBelow(5), &state));
    }

    #[test]
    fn test_add_rule() {
        let mut engine = AutoExecutionEngine::new();

        let rule =
            AutoExecutionRule::new(0, "test", Condition::BlockHeight(100), Action::ClearCache);

        let id = engine.add_rule(rule);
        assert_eq!(id, 1);
        assert_eq!(engine.rules.len(), 1);
    }
}
