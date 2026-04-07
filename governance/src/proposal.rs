//! 治理提案模块
//!
//! 实现链上治理提案的创建、查询和管理

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;

use toki_core::{Address, Hash};

/// 提案类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    /// 参数调整
    ParameterChange {
        param_name: String,
        old_value: String,
        new_value: String,
    },
    /// 功能升级
    FeatureUpgrade {
        feature_id: String,
        description: String,
    },
    /// 公益执行
    CharityExecution { region: String, amount: u64 },
    /// 开发者建议
    DeveloperSuggestion { content: String },
}

/// 提案状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    /// 待投票
    Pending,
    /// 投票中
    Voting,
    /// 已通过
    Passed,
    /// 已拒绝
    Rejected,
    /// 已执行
    Executed,
    /// 已取消
    Cancelled,
}

/// 治理提案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// 提案 ID
    pub id: u64,
    /// 提案类型
    pub proposal_type: ProposalType,
    /// 提案标题
    pub title: String,
    /// 提案描述
    pub description: String,
    /// 提案者地址
    pub proposer: Address,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 投票开始时间
    pub voting_start: DateTime<Utc>,
    /// 投票结束时间
    pub voting_end: DateTime<Utc>,
    /// 当前状态
    pub status: ProposalStatus,
    /// 赞成票数
    pub votes_for: u64,
    /// 反对票数
    pub votes_against: u64,
    /// 弃权票数
    pub votes_abstain: u64,
    /// 投票者列表
    pub voters: Vec<Address>,
}

impl Proposal {
    /// 创建新提案
    pub fn new(
        id: u64,
        proposal_type: ProposalType,
        title: String,
        description: String,
        proposer: Address,
        voting_period_days: u64,
    ) -> Self {
        let now = Utc::now();
        let voting_end = now + chrono::Duration::days(voting_period_days as i64);

        Proposal {
            id,
            proposal_type,
            title,
            description,
            proposer,
            created_at: now,
            voting_start: now,
            voting_end,
            status: ProposalStatus::Pending,
            votes_for: 0,
            votes_against: 0,
            votes_abstain: 0,
            voters: Vec::new(),
        }
    }

    /// 开始投票
    pub fn start_voting(&mut self) {
        if self.status == ProposalStatus::Pending {
            self.status = ProposalStatus::Voting;
            self.voting_start = Utc::now();
        }
    }

    /// 检查是否可以投票
    pub fn can_vote(&self) -> bool {
        self.status == ProposalStatus::Voting && Utc::now() < self.voting_end
    }

    /// 检查是否已投票
    pub fn has_voted(&self, voter: &Address) -> bool {
        self.voters.contains(voter)
    }

    /// 投票
    pub fn vote(&mut self, voter: Address, support: bool) -> Result<(), String> {
        if !self.can_vote() {
            return Err("提案不在投票期".to_string());
        }

        if self.has_voted(&voter) {
            return Err("已经投过票".to_string());
        }

        if support {
            self.votes_for += 1;
        } else {
            self.votes_against += 1;
        }

        self.voters.push(voter);
        Ok(())
    }

    /// 结束投票
    pub fn end_voting(&mut self, pass_threshold: f64, participation_threshold: f64) {
        if self.status != ProposalStatus::Voting {
            return;
        }

        let total_votes = self.votes_for + self.votes_against + self.votes_abstain;

        // 检查参与率
        // TODO: 需要知道总账户数
        let participation_rate = 1.0; // 暂时假设满足

        if participation_rate < participation_threshold {
            self.status = ProposalStatus::Rejected;
            return;
        }

        // 检查通过率
        let pass_rate = if total_votes > 0 {
            self.votes_for as f64 / total_votes as f64
        } else {
            0.0
        };

        if pass_rate >= pass_threshold {
            self.status = ProposalStatus::Passed;
        } else {
            self.status = ProposalStatus::Rejected;
        }
    }

    /// 计算提案哈希
    pub fn hash(&self) -> Hash {
        let data = format!(
            "{}{}{}{}{}",
            self.id,
            self.title,
            self.description,
            self.created_at.timestamp(),
            self.proposer.to_base58(),
        );
        Hash::from_data(data.as_bytes())
    }
}

/// 提案管理器
pub struct ProposalManager {
    /// 提案列表
    proposals: Vec<Proposal>,
    /// 下一个提案 ID
    next_id: u64,
    /// 投票期天数
    voting_period_days: u64,
    /// 通过阈值
    pass_threshold: f64,
    /// 参与率阈值
    participation_threshold: f64,
}

impl ProposalManager {
    pub fn new(voting_period_days: u64, pass_threshold: f64, participation_threshold: f64) -> Self {
        ProposalManager {
            proposals: Vec::new(),
            next_id: 1,
            voting_period_days,
            pass_threshold,
            participation_threshold,
        }
    }

    /// 创建提案
    pub fn create_proposal(
        &mut self,
        proposal_type: ProposalType,
        title: String,
        description: String,
        proposer: Address,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let proposal = Proposal::new(
            id,
            proposal_type,
            title.clone(),
            description,
            proposer,
            self.voting_period_days,
        );

        self.proposals.push(proposal);
        info!("创建提案: {} - {}", id, title);

        id
    }

    /// 获取提案
    pub fn get_proposal(&self, id: u64) -> Option<&Proposal> {
        self.proposals.iter().find(|p| p.id == id)
    }

    /// 获取活跃提案
    pub fn get_active_proposals(&self) -> Vec<&Proposal> {
        self.proposals
            .iter()
            .filter(|p| p.status == ProposalStatus::Voting || p.status == ProposalStatus::Pending)
            .collect()
    }

    /// 投票
    pub fn vote(&mut self, proposal_id: u64, voter: Address, support: bool) -> Result<(), String> {
        let proposal = self.proposals.iter_mut().find(|p| p.id == proposal_id);

        match proposal {
            Some(p) => p.vote(voter, support),
            None => Err("提案不存在".to_string()),
        }
    }

    /// 更新提案状态
    pub fn update_proposals(&mut self) {
        for proposal in &mut self.proposals {
            if proposal.status == ProposalStatus::Voting && Utc::now() >= proposal.voting_end {
                proposal.end_voting(self.pass_threshold, self.participation_threshold);
            }
        }
    }
}

impl Default for ProposalManager {
    fn default() -> Self {
        Self::new(7, 0.5, 0.3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proposal_creation() {
        let proposer = Address::new([1u8; 32]);
        let proposal = Proposal::new(
            1,
            ProposalType::ParameterChange {
                param_name: "difficulty".to_string(),
                old_value: "1000000".to_string(),
                new_value: "2000000".to_string(),
            },
            "调整难度".to_string(),
            "提高网络难度".to_string(),
            proposer,
            7,
        );

        assert_eq!(proposal.id, 1);
        assert_eq!(proposal.status, ProposalStatus::Pending);
    }

    #[test]
    fn test_voting() {
        let mut manager = ProposalManager::default();
        let proposer = Address::new([1u8; 32]);

        let id = manager.create_proposal(
            ProposalType::ParameterChange {
                param_name: "test".to_string(),
                old_value: "1".to_string(),
                new_value: "2".to_string(),
            },
            "测试提案".to_string(),
            "测试描述".to_string(),
            proposer.clone(),
        );

        // 开始投票
        let proposal = manager.proposals.iter_mut().find(|p| p.id == id).unwrap();
        proposal.start_voting();

        // 投票
        let voter = Address::new([2u8; 32]);
        let result = manager.vote(id, voter, true);
        assert!(result.is_ok());
    }
}
