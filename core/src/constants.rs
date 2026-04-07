//! Toki 平台常量定义

/// 全球人口基准（81.44 亿）
pub const GLOBAL_POPULATION: u64 = 8_144_000_000;

/// 人均基准额度（100,000 toki）
pub const PER_CAPITA_AMOUNT: u64 = 100_000;

/// toki 总量（814.4 万亿）
pub const TOTAL_SUPPLY: u64 = GLOBAL_POPULATION * PER_CAPITA_AMOUNT;

/// DistributionPool 占比（80%）
pub const DISTRIBUTION_POOL_RATIO: f64 = 0.8;

/// ReservePool 占比（20%）
pub const RESERVE_POOL_RATIO: f64 = 0.2;

/// DistributionPool 初始金额
pub const DISTRIBUTION_POOL_INITIAL: u64 = (TOTAL_SUPPLY as f64 * DISTRIBUTION_POOL_RATIO) as u64;

/// ReservePool 初始金额
pub const RESERVE_POOL_INITIAL: u64 = (TOTAL_SUPPLY as f64 * RESERVE_POOL_RATIO) as u64;

/// 基础赠送金额（100,000 toki）
pub const BASIC_GIFT_AMOUNT: u64 = 100_000;

/// 基础赠送解锁天数（10 年 = 3650 天）
pub const BASIC_GIFT_UNLOCK_DAYS: u64 = 3650;

/// 注册时一次性配发金额（20 toki）
pub const INITIAL_DISTRIBUTION: u64 = 20;

/// 每月配发金额（10 toki）
pub const MONTHLY_DISTRIBUTION: u64 = 10;

/// 每月配发持续月数（12 个月）
pub const MONTHLY_DISTRIBUTION_MONTHS: u64 = 12;

/// 低保触发余额阈值（30 toki）
pub const WELFARE_THRESHOLD: u64 = 30;

/// 低保每日发放金额（15 toki）
pub const WELFARE_DAILY_AMOUNT: u64 = 15;

/// 低保停止余额阈值（100 toki）
pub const WELFARE_STOP_THRESHOLD: u64 = 100;

/// 纸币消失率阈值（90%）
pub const FIAT_DISAPPEAR_THRESHOLD: f64 = 0.9;

/// 个人账户余额上限（20 亿 toki）
pub const PERSONAL_BALANCE_LIMIT: u64 = 2_000_000_000;

/// 开发者总收款账户上限（1000 亿 toki）
pub const DEVELOPER_MAIN_ACCOUNT_LIMIT: u64 = 100_000_000_000;

/// 交易服务费率（1/100,000）
pub const TRANSACTION_FEE_RATE: f64 = 0.00001;

/// 交易服务费延迟启动天数（180天）
pub const FEE_DELAY_DAYS: u64 = 180;

/// 收费公告提前天数（15天）
pub const FEE_ANNOUNCEMENT_DAYS: u64 = 15;

/// 平权削减比例（20%）
pub const EQUALIZATION_RATE: f64 = 0.2;

/// 目标出块时间（10 秒）
pub const TARGET_BLOCK_TIME_SECS: u64 = 10;

/// 难度调整周期（2016 区块）
pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u64 = 2016;

/// 投票期天数（7 天）
pub const VOTING_PERIOD_DAYS: u64 = 7;

/// 投票通过阈值（50%）
pub const VOTE_PASS_THRESHOLD: f64 = 0.5;

/// 投票参与率阈值（30%）
pub const VOTE_PARTICIPATION_THRESHOLD: f64 = 0.3;

/// 法币通道关闭倒计时天数（365 天）
pub const FIAT_CHANNEL_COUNTDOWN_DAYS: u64 = 365;

/// 法币通道启动倒计时延迟天数（365 天）
pub const FIAT_CHANNEL_START_DELAY_DAYS: u64 = 365;

/// AI 归集账户数量
pub const AI_AGGREGATOR_ACCOUNT_COUNT: usize = 100;

/// 开发者账户数量
pub const DEVELOPER_ACCOUNT_COUNT: usize = 6;

/// 开发者超级账号数量
pub const SUPER_ACCOUNT_COUNT: usize = 5;

/// 公益执行每区域最大次数
pub const CHARITY_MAX_EXECUTIONS: u8 = 5;

/// 美国公益触发阈值（4000 亿美元）
pub const CHARITY_USD_THRESHOLD: u64 = 400_000_000_000;

/// 欧洲+俄罗斯公益触发阈值（4000 亿欧元）
pub const CHARITY_EUR_THRESHOLD: u64 = 400_000_000_000;

/// 个人兑换限额（1 亿 toki）
pub const PERSONAL_EXCHANGE_LIMIT: u64 = 100_000_000;

/// 集体兑换限额（5 亿 toki）
pub const COLLECTIVE_EXCHANGE_LIMIT: u64 = 500_000_000;

/// 国家兑换限额（100 亿 toki）
pub const NATION_EXCHANGE_LIMIT: u64 = 10_000_000_000;

/// 兑换奖励第一阶段（前 6 个月，5%）
pub const EXCHANGE_BONUS_PHASE1_MONTHS: u64 = 6;
pub const EXCHANGE_BONUS_PHASE1_RATE: f64 = 0.05;

/// 兑换奖励第二阶段（6-12 个月，2%）
pub const EXCHANGE_BONUS_PHASE2_MONTHS: u64 = 12;
pub const EXCHANGE_BONUS_PHASE2_RATE: f64 = 0.02;

/// 集体账户不活跃回收天数（90 天）
pub const COLLECTIVE_INACTIVE_DAYS: u64 = 90;

/// 国家账户超限处理天数（30 天）
pub const NATION_OVERLIMIT_DAYS: u64 = 30;

/// 开发者授权超时小时数（72 小时）
pub const DEV_AUTH_TIMEOUT_HOURS: u64 = 72;

/// 区块确认数（6 个区块）
pub const BLOCK_CONFIRMATIONS: u64 = 6;

/// toki 基本单位（1 toki = 10^8 基本单位）
pub const TOKI_BASE_UNIT: u64 = 100_000_000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_supply() {
        // 验证总量计算正确
        // 8_144_000_000 * 100_000 = 814_400_000_000_000 (814.4万亿)
        assert_eq!(TOTAL_SUPPLY, 814_400_000_000_000u64);
    }

    #[test]
    fn test_pool_ratios() {
        // 验证池比例总和为 1
        assert!((DISTRIBUTION_POOL_RATIO + RESERVE_POOL_RATIO - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pool_amounts() {
        // 验证池金额总和等于总量
        assert_eq!(DISTRIBUTION_POOL_INITIAL + RESERVE_POOL_INITIAL, TOTAL_SUPPLY);
    }
}
