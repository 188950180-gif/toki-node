//! 投票模块
//! 
//! 实现投票权和投票记录

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use toki_core::{Address, Hash};

/// 投票选项
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoteOption {
    /// 赞成
    For,
    /// 反对
    Against,
    /// 弃权
    Abstain,
}

/// 投票记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// 提案 ID
    pub proposal_id: u64,
    /// 投票者地址
    pub voter: Address,
    /// 投票选项
    pub option: VoteOption,
    /// 投票时间
    pub timestamp: DateTime<Utc>,
    /// 投票权重（基于余额）
    pub weight: u64,
}

impl Vote {
    pub fn new(proposal_id: u64, voter: Address, option: VoteOption, weight: u64) -> Self {
        Vote {
            proposal_id,
            voter,
            option,
            timestamp: Utc::now(),
            weight,
        }
    }

    /// 计算投票哈希
    pub fn hash(&self) -> Hash {
        let data = format!(
            "{}{}{}{}",
            self.proposal_id,
            self.voter.to_base58(),
            match self.option {
                VoteOption::For => "for",
                VoteOption::Against => "against",
                VoteOption::Abstain => "abstain",
            },
            self.timestamp.timestamp(),
        );
        Hash::from_data(data.as_bytes())
    }
}

/// 投票结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteResult {
    pub proposal_id: u64,
    pub total_votes: u64,
    pub votes_for: u64,
    pub votes_against: u64,
    pub votes_abstain: u64,
    pub weighted_for: u64,
    pub weighted_against: u64,
}

impl VoteResult {
    pub fn new(proposal_id: u64) -> Self {
        VoteResult {
            proposal_id,
            total_votes: 0,
            votes_for: 0,
            votes_against: 0,
            votes_abstain: 0,
            weighted_for: 0,
            weighted_against: 0,
        }
    }

    /// 添加投票
    pub fn add_vote(&mut self, vote: &Vote) {
        self.total_votes += 1;
        
        match vote.option {
            VoteOption::For => {
                self.votes_for += 1;
                self.weighted_for += vote.weight;
            }
            VoteOption::Against => {
                self.votes_against += 1;
                self.weighted_against += vote.weight;
            }
            VoteOption::Abstain => {
                self.votes_abstain += 1;
            }
        }
    }

    /// 计算赞成率
    pub fn pass_rate(&self) -> f64 {
        if self.total_votes == 0 {
            return 0.0;
        }
        self.votes_for as f64 / self.total_votes as f64
    }

    /// 计算加权赞成率
    pub fn weighted_pass_rate(&self) -> f64 {
        let total_weight = self.weighted_for + self.weighted_against;
        if total_weight == 0 {
            return 0.0;
        }
        self.weighted_for as f64 / total_weight as f64
    }
}

/// 投票权计算器
pub struct VotingPowerCalculator;

impl VotingPowerCalculator {
    /// 计算投票权（基于余额）
    pub fn calculate(balance: u64) -> u64 {
        // 简单实现：1 toki = 1 票
        // 可以添加更复杂的计算逻辑
        balance
    }

    /// 计算账户类型的额外权重
    pub fn account_type_bonus(account_type: &toki_core::AccountType) -> f64 {
        match account_type {
            toki_core::AccountType::Nation => 2.0,
            toki_core::AccountType::Collective => 1.5,
            _ => 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vote_creation() {
        let voter = Address::new([1u8; 32]);
        let vote = Vote::new(1, voter, VoteOption::For, 1000);
        
        assert_eq!(vote.proposal_id, 1);
        assert_eq!(vote.option, VoteOption::For);
    }

    #[test]
    fn test_vote_result() {
        let mut result = VoteResult::new(1);
        
        let voter1 = Address::new([1u8; 32]);
        let voter2 = Address::new([2u8; 32]);
        
        result.add_vote(&Vote::new(1, voter1, VoteOption::For, 100));
        result.add_vote(&Vote::new(1, voter2, VoteOption::Against, 50));
        
        assert_eq!(result.total_votes, 2);
        assert_eq!(result.votes_for, 1);
        assert_eq!(result.votes_against, 1);
        assert_eq!(result.weighted_for, 100);
        assert_eq!(result.weighted_against, 50);
    }

    #[test]
    fn test_pass_rate() {
        let mut result = VoteResult::new(1);
        
        result.votes_for = 60;
        result.votes_against = 30;
        result.votes_abstain = 10;
        result.total_votes = 100;
        
        let rate = result.pass_rate();
        assert!((rate - 0.6).abs() < 0.001);
    }
}
