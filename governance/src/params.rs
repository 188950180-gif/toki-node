//! 链上参数管理
//!
//! 管理可治理的链上参数

use serde::{Deserialize, Serialize};

/// 参数类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParamType {
    U64(u64),
    F64(f64),
    String(String),
    Bool(bool),
}

/// 链上参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainParams {
    /// 目标出块时间（秒）
    pub target_block_time: u64,
    /// 难度调整周期（区块数）
    pub difficulty_adjustment_interval: u64,
    /// 最小交易费
    pub min_tx_fee: u64,
    /// 交易费率
    pub tx_fee_rate: f64,
    /// 平权削减比例
    pub equalization_rate: f64,
    /// 投票期天数
    pub voting_period_days: u64,
    /// 投票通过阈值
    pub vote_pass_threshold: f64,
    /// 投票参与率阈值
    pub vote_participation_threshold: f64,
    /// 法币通道关闭倒计时天数
    pub fiat_channel_countdown_days: u64,
    /// 个人账户余额上限
    pub personal_balance_limit: u64,
}

impl Default for ChainParams {
    fn default() -> Self {
        ChainParams {
            target_block_time: 10,
            difficulty_adjustment_interval: 100,
            min_tx_fee: 1000,
            tx_fee_rate: 0.00001,
            equalization_rate: 0.2,
            voting_period_days: 7,
            vote_pass_threshold: 0.5,
            vote_participation_threshold: 0.3,
            fiat_channel_countdown_days: 365,
            personal_balance_limit: 2_000_000_000,
        }
    }
}

impl ChainParams {
    /// 获取参数值
    pub fn get(&self, name: &str) -> Option<ParamType> {
        match name {
            "target_block_time" => Some(ParamType::U64(self.target_block_time)),
            "difficulty_adjustment_interval" => {
                Some(ParamType::U64(self.difficulty_adjustment_interval))
            }
            "min_tx_fee" => Some(ParamType::U64(self.min_tx_fee)),
            "tx_fee_rate" => Some(ParamType::F64(self.tx_fee_rate)),
            "equalization_rate" => Some(ParamType::F64(self.equalization_rate)),
            "voting_period_days" => Some(ParamType::U64(self.voting_period_days)),
            "vote_pass_threshold" => Some(ParamType::F64(self.vote_pass_threshold)),
            "vote_participation_threshold" => {
                Some(ParamType::F64(self.vote_participation_threshold))
            }
            "fiat_channel_countdown_days" => Some(ParamType::U64(self.fiat_channel_countdown_days)),
            "personal_balance_limit" => Some(ParamType::U64(self.personal_balance_limit)),
            _ => None,
        }
    }

    /// 设置参数值
    pub fn set(&mut self, name: &str, value: ParamType) -> Result<(), String> {
        match name {
            "target_block_time" => {
                if let ParamType::U64(v) = value {
                    self.target_block_time = v;
                    Ok(())
                } else {
                    Err("参数类型错误".to_string())
                }
            }
            "difficulty_adjustment_interval" => {
                if let ParamType::U64(v) = value {
                    self.difficulty_adjustment_interval = v;
                    Ok(())
                } else {
                    Err("参数类型错误".to_string())
                }
            }
            "min_tx_fee" => {
                if let ParamType::U64(v) = value {
                    self.min_tx_fee = v;
                    Ok(())
                } else {
                    Err("参数类型错误".to_string())
                }
            }
            "tx_fee_rate" => {
                if let ParamType::F64(v) = value {
                    self.tx_fee_rate = v;
                    Ok(())
                } else {
                    Err("参数类型错误".to_string())
                }
            }
            "equalization_rate" => {
                if let ParamType::F64(v) = value {
                    self.equalization_rate = v;
                    Ok(())
                } else {
                    Err("参数类型错误".to_string())
                }
            }
            "voting_period_days" => {
                if let ParamType::U64(v) = value {
                    self.voting_period_days = v;
                    Ok(())
                } else {
                    Err("参数类型错误".to_string())
                }
            }
            "vote_pass_threshold" => {
                if let ParamType::F64(v) = value {
                    self.vote_pass_threshold = v;
                    Ok(())
                } else {
                    Err("参数类型错误".to_string())
                }
            }
            "vote_participation_threshold" => {
                if let ParamType::F64(v) = value {
                    self.vote_participation_threshold = v;
                    Ok(())
                } else {
                    Err("参数类型错误".to_string())
                }
            }
            "fiat_channel_countdown_days" => {
                if let ParamType::U64(v) = value {
                    self.fiat_channel_countdown_days = v;
                    Ok(())
                } else {
                    Err("参数类型错误".to_string())
                }
            }
            "personal_balance_limit" => {
                if let ParamType::U64(v) = value {
                    self.personal_balance_limit = v;
                    Ok(())
                } else {
                    Err("参数类型错误".to_string())
                }
            }
            _ => Err(format!("未知参数: {}", name)),
        }
    }

    /// 验证参数值
    pub fn validate(&self) -> Result<(), String> {
        if self.target_block_time == 0 {
            return Err("target_block_time 不能为 0".to_string());
        }
        if self.tx_fee_rate < 0.0 || self.tx_fee_rate > 1.0 {
            return Err("tx_fee_rate 必须在 0-1 之间".to_string());
        }
        if self.vote_pass_threshold < 0.0 || self.vote_pass_threshold > 1.0 {
            return Err("vote_pass_threshold 必须在 0-1 之间".to_string());
        }
        Ok(())
    }
}

/// 参数变更记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamChange {
    pub param_name: String,
    pub old_value: ParamType,
    pub new_value: ParamType,
    pub block_height: u64,
    pub proposal_id: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let params = ChainParams::default();
        assert_eq!(params.target_block_time, 10);
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_get_set_param() {
        let mut params = ChainParams::default();

        // 获取参数
        let value = params.get("target_block_time");
        assert!(matches!(value, Some(ParamType::U64(10))));

        // 设置参数
        let result = params.set("target_block_time", ParamType::U64(20));
        assert!(result.is_ok());
        assert_eq!(params.target_block_time, 20);
    }

    #[test]
    fn test_invalid_param() {
        let mut params = ChainParams::default();

        // 设置错误类型
        let result = params.set("target_block_time", ParamType::F64(1.0));
        assert!(result.is_err());

        // 设置未知参数
        let result = params.set("unknown", ParamType::U64(1));
        assert!(result.is_err());
    }
}
