//! 集成测试

use toki_core::*;

#[test]
fn test_full_blockchain_flow() {
    // 1. 创建创世区块
    let genesis = Block::genesis();
    assert_eq!(genesis.height(), 0);

    // 2. 创建链
    let mut prev_hash = genesis.hash();
    for height in 1..=5 {
        let block = Block::new(height, prev_hash, vec![], 1000, Address::ZERO);
        prev_hash = block.hash();

        // 验证区块
        assert_eq!(block.height(), height);
    }
}

#[test]
fn test_account_and_transaction_flow() {
    // 1. 创建账户
    let addr = Address::new([1u8; 32]);
    let account = Account::new(addr.clone(), AccountType::Personal);

    assert_eq!(account.account_type, AccountType::Personal);

    // 2. 创建交易
    let output = Output::new(addr, 100 * TOKI_BASE_UNIT);
    let ring_sig = RingSignature::new(vec![], vec![1u8; 64], Hash::ZERO);
    let tx = Transaction::new(vec![], vec![output], ring_sig, 1000);

    assert_eq!(tx.fee, 1000);
    assert_eq!(tx.outputs.len(), 1);
}

#[test]
fn test_hash_and_address_consistency() {
    // 哈希一致性
    let data = b"test data";
    let hash1 = Hash::from_data(data);
    let hash2 = Hash::from_data(data);
    assert_eq!(hash1, hash2);

    // 地址编码一致性
    let bytes = [1u8; 32];
    let addr = Address::new(bytes);
    let encoded = addr.to_base58();
    let decoded = Address::from_base58(&encoded);
    assert!(decoded.is_ok());
    assert_eq!(decoded.unwrap(), addr);
}

#[test]
fn test_all_account_types() {
    let addr = Address::new([1u8; 32]);

    let personal = Account::new(addr.clone(), AccountType::Personal);
    assert_eq!(personal.account_type, AccountType::Personal);

    let collective = Account::new(addr.clone(), AccountType::Collective);
    assert_eq!(collective.account_type, AccountType::Collective);

    let nation = Account::new(addr.clone(), AccountType::Nation);
    assert_eq!(nation.account_type, AccountType::Nation);

    let developer = Account::new(addr.clone(), AccountType::Developer { is_main: false });
    assert!(matches!(
        developer.account_type,
        AccountType::Developer { .. }
    ));

    let ai = Account::new(addr.clone(), AccountType::AIAggregate);
    assert_eq!(ai.account_type, AccountType::AIAggregate);
}

#[test]
fn test_all_regions() {
    let regions = vec![
        Region::US,
        Region::EU,
        Region::RU,
        Region::AS,
        Region::Other,
    ];

    for region in regions {
        let _ = region; // 验证所有变体存在
    }
}

#[test]
fn test_block_merkle_root() {
    // 空区块
    let empty_block = Block::new(0, Hash::ZERO, vec![], 1000, Address::ZERO);
    assert!(empty_block.verify_merkle_root());

    // 有交易的区块
    let addr = Address::new([1u8; 32]);
    let output = Output::new(addr, 100 * TOKI_BASE_UNIT);
    let ring_sig = RingSignature::new(vec![], vec![1u8; 64], Hash::ZERO);
    let tx = Transaction::new(vec![], vec![output], ring_sig, 1000);

    let tx_block = Block::new(0, Hash::ZERO, vec![tx], 1000, Address::ZERO);
    assert!(tx_block.verify_merkle_root());
}

#[test]
fn test_constants() {
    // 验证总量 (8_144_000_000 * 100_000 = 814_400_000_000_000)
    assert_eq!(TOTAL_SUPPLY, 814_400_000_000_000u64);

    // 验证池比例
    assert!((DISTRIBUTION_POOL_RATIO + RESERVE_POOL_RATIO - 1.0).abs() < f64::EPSILON);

    // 验证目标出块时间
    assert_eq!(TARGET_BLOCK_TIME_SECS, 10);
}
