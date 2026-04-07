//! 交易服务费延迟功能测试

use toki_core::{Transaction, TOKI_BASE_UNIT, constants::{FEE_DELAY_DAYS, FEE_ANNOUNCEMENT_DAYS}};

#[test]
fn test_fee_delay_constants() {
    // 验证延迟天数
    assert_eq!(FEE_DELAY_DAYS, 180, "延迟天数应为180天");
    assert_eq!(FEE_ANNOUNCEMENT_DAYS, 15, "公告天数应为15天");
    println!("✅ 常量验证通过");
}

#[test]
fn test_calculate_fee_with_delay_free_period() {
    // 测试免费期（0-180天）
    let amount = 100_000 * TOKI_BASE_UNIT;
    let genesis_time = 1704067200u64; // 2024-01-01
    
    // 第0天 - 免费
    let current_time = genesis_time;
    let fee = Transaction::calculate_fee_with_delay(amount, genesis_time, current_time);
    assert_eq!(fee, 0, "第0天应免费");
    
    // 第90天 - 免费
    let current_time = genesis_time + 90 * 86400;
    let fee = Transaction::calculate_fee_with_delay(amount, genesis_time, current_time);
    assert_eq!(fee, 0, "第90天应免费");
    
    // 第179天 - 免费
    let current_time = genesis_time + 179 * 86400;
    let fee = Transaction::calculate_fee_with_delay(amount, genesis_time, current_time);
    assert_eq!(fee, 0, "第179天应免费");
    
    println!("✅ 免费期测试通过");
}

#[test]
fn test_calculate_fee_with_delay_fee_period() {
    // 测试收费期（180天后）
    let amount = 100_000 * TOKI_BASE_UNIT;
    let genesis_time = 1704067200u64; // 2024-01-01
    
    // 第180天 - 开始收费
    let current_time = genesis_time + 180 * 86400;
    let fee = Transaction::calculate_fee_with_delay(amount, genesis_time, current_time);
    assert!(fee > 0, "第180天应开始收费");
    
    // 第365天 - 正常收费
    let current_time = genesis_time + 365 * 86400;
    let fee = Transaction::calculate_fee_with_delay(amount, genesis_time, current_time);
    assert!(fee > 0, "第365天应正常收费");
    
    // 验证费用计算正确
    let expected_fee = Transaction::calculate_fee(amount);
    assert_eq!(fee, expected_fee, "收费期费用应等于正常费用");
    
    println!("✅ 收费期测试通过");
}

#[test]
fn test_fee_announcement_no_announcement() {
    // 测试无公告期（0-165天）
    let genesis_time = 1704067200u64; // 2024-01-01
    
    // 第0天 - 无公告
    let current_time = genesis_time;
    let announcement = Transaction::check_fee_announcement(genesis_time, current_time);
    assert!(announcement.is_none(), "第0天不应有公告");
    
    // 第100天 - 无公告
    let current_time = genesis_time + 100 * 86400;
    let announcement = Transaction::check_fee_announcement(genesis_time, current_time);
    assert!(announcement.is_none(), "第100天不应有公告");
    
    // 第164天 - 无公告
    let current_time = genesis_time + 164 * 86400;
    let announcement = Transaction::check_fee_announcement(genesis_time, current_time);
    assert!(announcement.is_none(), "第164天不应有公告");
    
    println!("✅ 无公告期测试通过");
}

#[test]
fn test_fee_announcement_announcement_period() {
    // 测试公告期（165-180天）
    let genesis_time = 1704067200u64; // 2024-01-01
    
    // 第165天 - 开始公告
    let current_time = genesis_time + 165 * 86400;
    let announcement = Transaction::check_fee_announcement(genesis_time, current_time);
    assert!(announcement.is_some(), "第165天应有公告");
    let ann = announcement.unwrap();
    assert_eq!(ann.days_remaining, 15, "第165天剩余15天");
    assert!(ann.is_announced, "应标记为已公告");
    
    // 第172天 - 公告
    let current_time = genesis_time + 172 * 86400;
    let announcement = Transaction::check_fee_announcement(genesis_time, current_time);
    assert!(announcement.is_some(), "第172天应有公告");
    let ann = announcement.unwrap();
    assert_eq!(ann.days_remaining, 8, "第172天剩余8天");
    
    // 第179天 - 公告
    let current_time = genesis_time + 179 * 86400;
    let announcement = Transaction::check_fee_announcement(genesis_time, current_time);
    assert!(announcement.is_some(), "第179天应有公告");
    let ann = announcement.unwrap();
    assert_eq!(ann.days_remaining, 1, "第179天剩余1天");
    
    println!("✅ 公告期测试通过");
}

#[test]
fn test_fee_announcement_after_fee_start() {
    // 测试收费开始后无公告
    let genesis_time = 1704067200u64; // 2024-01-01
    
    // 第180天 - 无公告
    let current_time = genesis_time + 180 * 86400;
    let announcement = Transaction::check_fee_announcement(genesis_time, current_time);
    assert!(announcement.is_none(), "第180天不应有公告");
    
    // 第365天 - 无公告
    let current_time = genesis_time + 365 * 86400;
    let announcement = Transaction::check_fee_announcement(genesis_time, current_time);
    assert!(announcement.is_none(), "第365天不应有公告");
    
    println!("✅ 收费后无公告测试通过");
}

#[test]
fn test_fee_calculation_consistency() {
    // 测试费用计算一致性
    let amount = 1_000_000 * TOKI_BASE_UNIT;
    let genesis_time = 1704067200u64;
    
    // 收费期的费用应等于正常费用
    let current_time = genesis_time + 200 * 86400;
    let fee_with_delay = Transaction::calculate_fee_with_delay(amount, genesis_time, current_time);
    let normal_fee = Transaction::calculate_fee(amount);
    assert_eq!(fee_with_delay, normal_fee, "收费期费用应与正常费用一致");
    
    println!("✅ 费用计算一致性测试通过");
}

#[test]
fn test_edge_cases() {
    // 测试边界情况
    let amount = 100 * TOKI_BASE_UNIT;
    let genesis_time = 1704067200u64;
    
    // 刚好180天
    let current_time = genesis_time + 180 * 86400;
    let fee = Transaction::calculate_fee_with_delay(amount, genesis_time, current_time);
    assert!(fee > 0, "刚好180天应收费");
    
    // 179天23小时59分 - 仍免费
    let current_time = genesis_time + 180 * 86400 - 1;
    let fee = Transaction::calculate_fee_with_delay(amount, genesis_time, current_time);
    assert_eq!(fee, 0, "179天23小时59分应免费");
    
    println!("✅ 边界情况测试通过");
}
