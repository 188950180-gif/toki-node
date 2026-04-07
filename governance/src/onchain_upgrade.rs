//! 链上自动升级模块
//! 
//! 通过治理提案实现链上自动升级

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::{info, warn};

use toki_core::{Hash, Address};

/// 升级提案 ID
pub type ProposalId = Hash;

/// 升级类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpgradeType {
    /// 共识协议升级
    ConsensusProtocol,
    /// 网络协议升级
    NetworkProtocol,
    /// 治理规则升级
    GovernanceRules,
    /// 经济模型升级
    EconomicModel,
    /// 客户端版本升级
    ClientVersion,
}

impl std::fmt::Display for UpgradeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpgradeType::ConsensusProtocol => write!(f, "ConsensusProtocol"),
            UpgradeType::NetworkProtocol => write!(f, "NetworkProtocol"),
            UpgradeType::GovernanceRules => write!(f, "GovernanceRules"),
            UpgradeType::EconomicModel => write!(f, "EconomicModel"),
            UpgradeType::ClientVersion => write!(f, "ClientVersion"),
        }
    }
}

/// 升级阶段
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpgradePhase {
    /// 提案中
    Proposed,
    /// 投票中
    Voting,
    /// 已通过
    Approved,
    /// 锁定期（等待执行）
    Locked,
    /// 执行中
    Executing,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
}

/// 链上升级提案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeProposal {
    /// 提案 ID
    pub id: ProposalId,
    /// 升级类型
    pub upgrade_type: UpgradeType,
    /// 升级阶段
    pub phase: UpgradePhase,
    /// 目标版本
    pub target_version: String,
    /// 当前版本
    pub current_version: String,
    /// 提案者
    pub proposer: Address,
    /// 提案时间
    pub propose_time: u64,
    /// 投票开始时间
    pub vote_start: u64,
    /// 投票结束时间
    pub vote_end: u64,
    /// 锁定期结束时间
    pub lock_end: u64,
    /// 执行时间
    pub execute_time: u64,
    /// 赞成票
    pub votes_for: u64,
    /// 反对票
    pub votes_against: u64,
    /// 弃权票
    pub votes_abstain: u64,
    /// 升级数据（如新协议参数）
    pub upgrade_data: Vec<u8>,
    /// 升级说明
    pub description: String,
    /// 激活高度
    pub activation_height: u64,
}

/// 升级投票
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeVote {
    /// 提案 ID
    pub proposal_id: ProposalId,
    /// 投票者
    pub voter: Address,
    /// 投票选择
    pub choice: VoteChoice,
    /// 投票权重
    pub weight: u64,
    /// 投票时间
    pub timestamp: u64,
}

/// 投票选择
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoteChoice {
    /// 赞成
    For,
    /// 反对
    Against,
    /// 弃权
    Abstain,
}

/// 链上升级配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnChainUpgradeConfig {
    /// 投票周期（区块数）
    pub voting_period: u64,
    /// 锁定周期（区块数）
    pub lock_period: u64,
    /// 通过阈值（百分比）
    pub pass_threshold: f64,
    /// 最小参与率
    pub min_participation: f64,
    /// 最小提案者质押
    pub min_proposer_stake: u64,
    /// 是否启用自动执行
    pub auto_execute: bool,
}

impl Default for OnChainUpgradeConfig {
    fn default() -> Self {
        OnChainUpgradeConfig {
            voting_period: 10080,    // 约 1 周（10秒出块）
            lock_period: 20160,      // 约 2 周
            pass_threshold: 0.667,   // 2/3 多数
            min_participation: 0.3,  // 30% 参与率
            min_proposer_stake: 1_000_000, // 100 万 toki
            auto_execute: true,
        }
    }
}

/// 链上升级管理器
pub struct OnChainUpgradeManager {
    /// 配置
    config: OnChainUpgradeConfig,
    /// 活跃提案
    proposals: HashMap<ProposalId, UpgradeProposal>,
    /// 投票记录
    votes: HashMap<ProposalId, Vec<UpgradeVote>>,
    /// 已执行升级
    executed_upgrades: Vec<ExecutedUpgrade>,
    /// 当前版本
    current_version: String,
    /// 当前区块高度
    current_height: u64,
}

/// 已执行升级
#[derive(Debug, Clone)]
pub struct ExecutedUpgrade {
    /// 提案 ID
    pub proposal_id: ProposalId,
    /// 升级类型
    pub upgrade_type: UpgradeType,
    /// 执行高度
    pub height: u64,
    /// 从版本
    pub from_version: String,
    /// 到版本
    pub to_version: String,
    /// 执行时间
    pub timestamp: u64,
}

impl OnChainUpgradeManager {
    /// 创建新的链上升级管理器
    pub fn new(config: OnChainUpgradeConfig, current_version: String) -> Self {
        OnChainUpgradeManager {
            config,
            proposals: HashMap::new(),
            votes: HashMap::new(),
            executed_upgrades: Vec::new(),
            current_version,
            current_height: 0,
        }
    }

    /// 提交升级提案
    pub fn propose(
        &mut self,
        upgrade_type: UpgradeType,
        target_version: String,
        proposer: Address,
        upgrade_data: Vec<u8>,
        description: String,
    ) -> Result<ProposalId> {
        // 检查版本
        if target_version <= self.current_version {
            return Err(anyhow::anyhow!("目标版本必须高于当前版本"));
        }
        
        // 检查是否已有相同类型的待处理升级
        for proposal in self.proposals.values() {
            if proposal.upgrade_type == upgrade_type 
                && proposal.phase != UpgradePhase::Completed 
                && proposal.phase != UpgradePhase::Cancelled {
                return Err(anyhow::anyhow!("已有相同类型的待处理升级"));
            }
        }
        
        let now = self.current_height;
        let upgrade_type_clone = upgrade_type.clone();
        let proposal = UpgradeProposal {
            id: Hash::from_data(&format!("{}{}{:?}", upgrade_type, target_version, now).as_bytes()),
            upgrade_type: upgrade_type_clone,
            phase: UpgradePhase::Proposed,
            target_version: target_version.clone(),
            current_version: self.current_version.clone(),
            proposer,
            propose_time: now,
            vote_start: now + 100, // 100 块后开始投票
            vote_end: now + 100 + self.config.voting_period,
            lock_end: now + 100 + self.config.voting_period + self.config.lock_period,
            execute_time: now + 100 + self.config.voting_period + self.config.lock_period + 100,
            votes_for: 0,
            votes_against: 0,
            votes_abstain: 0,
            upgrade_data,
            description,
            activation_height: 0,
        };
        
        let id = proposal.id;
        self.proposals.insert(id, proposal);
        self.votes.insert(id, Vec::new());
        
        info!("创建升级提案: {} -> {}, 类型: {}", 
            self.current_version, target_version, upgrade_type.clone());
        
        Ok(id)
    }

    /// 投票
    pub fn vote(
        &mut self,
        proposal_id: &ProposalId,
        voter: Address,
        choice: VoteChoice,
        weight: u64,
    ) -> Result<()> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| anyhow::anyhow!("提案不存在"))?;
        
        // 检查投票阶段
        if proposal.phase != UpgradePhase::Voting {
            return Err(anyhow::anyhow!("不在投票阶段"));
        }
        
        // 检查投票时间
        if self.current_height < proposal.vote_start 
            || self.current_height > proposal.vote_end {
            return Err(anyhow::anyhow!("不在投票时间窗口"));
        }
        
        // 记录投票
        let vote = UpgradeVote {
            proposal_id: *proposal_id,
            voter,
            choice: choice.clone(),
            weight,
            timestamp: self.current_height,
        };
        
        // 更新计票
        match choice {
            VoteChoice::For => proposal.votes_for += weight,
            VoteChoice::Against => proposal.votes_against += weight,
            VoteChoice::Abstain => proposal.votes_abstain += weight,
        }
        
        self.votes.get_mut(proposal_id).unwrap().push(vote);
        
        Ok(())
    }

    /// 更新区块高度
    pub fn update_height(&mut self, height: u64) -> Result<Vec<UpgradeAction>> {
        self.current_height = height;
        let mut actions = Vec::new();
        
        for proposal in self.proposals.values_mut() {
            match proposal.phase {
                UpgradePhase::Proposed => {
                    if height >= proposal.vote_start {
                        proposal.phase = UpgradePhase::Voting;
                        info!("升级提案进入投票阶段: {}", proposal.id);
                    }
                }
                UpgradePhase::Voting => {
                    if height >= proposal.vote_end {
                        // 检查是否通过
                        let total = proposal.votes_for + proposal.votes_against + proposal.votes_abstain;
                        let participation = total as f64 / 1_000_000_000_000.0; // 假设总质押
                        
                        if participation >= self.config.min_participation {
                            let approval = proposal.votes_for as f64 / total as f64;
                            
                            if approval >= self.config.pass_threshold {
                                proposal.phase = UpgradePhase::Approved;
                                info!("升级提案通过: {}", proposal.id);
                            } else {
                                proposal.phase = UpgradePhase::Cancelled;
                                warn!("升级提案未通过: {}", proposal.id);
                            }
                        } else {
                            proposal.phase = UpgradePhase::Cancelled;
                            warn!("升级提案参与率不足: {}", proposal.id);
                        }
                    }
                }
                UpgradePhase::Approved => {
                    if height >= proposal.lock_end {
                        proposal.phase = UpgradePhase::Locked;
                        proposal.activation_height = height + 100;
                        info!("升级提案进入锁定期: {}", proposal.id);
                    }
                }
                UpgradePhase::Locked => {
                    if height >= proposal.execute_time {
                        proposal.phase = UpgradePhase::Executing;
                        info!("升级提案开始执行: {}", proposal.id);
                        
                        actions.push(UpgradeAction::Execute {
                            proposal_id: proposal.id,
                            upgrade_type: proposal.upgrade_type.clone(),
                            target_version: proposal.target_version.clone(),
                            upgrade_data: proposal.upgrade_data.clone(),
                        });
                    }
                }
                UpgradePhase::Executing => {
                    // 执行完成
                    proposal.phase = UpgradePhase::Completed;
                    self.current_version = proposal.target_version.clone();
                    
                    self.executed_upgrades.push(ExecutedUpgrade {
                        proposal_id: proposal.id,
                        upgrade_type: proposal.upgrade_type.clone(),
                        height,
                        from_version: proposal.current_version.clone(),
                        to_version: proposal.target_version.clone(),
                        timestamp: height,
                    });
                    
                    info!("升级完成: {} -> {}", proposal.current_version, proposal.target_version);
                    actions.push(UpgradeAction::Completed {
                        proposal_id: proposal.id,
                        new_version: proposal.target_version.clone(),
                    });
                }
                _ => {}
            }
        }
        
        Ok(actions)
    }

    /// 获取当前版本
    pub fn current_version(&self) -> &str {
        &self.current_version
    }

    /// 获取活跃提案
    pub fn active_proposals(&self) -> Vec<&UpgradeProposal> {
        self.proposals.values()
            .filter(|p| p.phase != UpgradePhase::Completed 
                && p.phase != UpgradePhase::Cancelled)
            .collect()
    }

    /// 获取已执行升级
    pub fn executed_upgrades(&self) -> &[ExecutedUpgrade] {
        &self.executed_upgrades
    }
}

/// 升级动作
#[derive(Debug, Clone)]
pub enum UpgradeAction {
    /// 执行升级
    Execute {
        proposal_id: ProposalId,
        upgrade_type: UpgradeType,
        target_version: String,
        upgrade_data: Vec<u8>,
    },
    /// 升级完成
    Completed {
        proposal_id: ProposalId,
        new_version: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upgrade_config_default() {
        let config = OnChainUpgradeConfig::default();
        assert_eq!(config.voting_period, 10080);
        assert_eq!(config.pass_threshold, 0.667);
    }

    #[test]
    fn test_manager_creation() {
        let config = OnChainUpgradeConfig::default();
        let manager = OnChainUpgradeManager::new(config, "0.1.0".to_string());
        assert_eq!(manager.current_version(), "0.1.0");
    }
}
