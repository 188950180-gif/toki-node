//! Toki 平台示例代码
//! 
//! 演示如何使用 Toki SDK 进行基本操作

use toki_core::*;

fn main() {
    println!("\n=== Toki 平台示例 ===\n");
    
    // 示例 1: 创建账户
    println!("1. 创建账户");
    let alice_addr = Address::new([1u8; 32]);
    let bob_addr = Address::new([2u8; 32]);
    
    let alice = Account::new(alice_addr.clone(), AccountType::Personal);
    let bob = Account::new(bob_addr.clone(), AccountType::Personal);
    
    println!("   Alice 地址: {}...", &alice_addr.to_base58()[..20]);
    println!("   Bob 地址: {}...", &bob_addr.to_base58()[..20]);
    println!("   Alice 账户类型: {:?}", alice.account_type);
    println!("   Bob 账户类型: {:?}", bob.account_type);
    
    // 示例 2: 创建交易
    println!("\n2. 创建交易");
    let output = Output::new(bob_addr.clone(), 100 * TOKI_BASE_UNIT);
    let ring_sig = RingSignature::new(vec![], vec![1u8; 64], Hash::ZERO);
    let tx = Transaction::new(vec![], vec![output], ring_sig, 1000);
    
    println!("   交易哈希: {}...", &tx.hash().to_hex()[..20]);
    println!("   输出数量: {} toki", tx.outputs[0].amount / TOKI_BASE_UNIT);
    println!("   交易费: {} μtoki", tx.fee);
    
    // 示例 3: 创建区块
    println!("\n3. 创建区块");
    let genesis = Block::genesis();
    println!("   创世区块高度: {}", genesis.height());
    println!("   创世区块哈希: {}...", &genesis.hash().to_hex()[..20]);
    
    let block1 = Block::new(1, genesis.hash(), vec![tx], 1000, alice_addr);
    println!("   区块 1 高度: {}", block1.height());
    println!("   区块 1 交易数: {}", block1.tx_count());
    println!("   区块 1 哈希: {}...", &block1.hash().to_hex()[..20]);
    
    // 示例 4: 哈希计算
    println!("\n4. 哈希计算");
    let data = b"Hello, Toki!";
    let hash = Hash::from_data(data);
    println!("   数据: {:?}", std::str::from_utf8(data).unwrap());
    println!("   哈希: {}", hash.to_hex());
    
    // 示例 5: 地址编码
    println!("\n5. 地址编码");
    let encoded = alice_addr.to_base58();
    let decoded = Address::from_base58(&encoded).unwrap();
    println!("   原始地址: {}...", &encoded[..20]);
    println!("   编码长度: {} 字节", encoded.len());
    println!("   解码成功: {}", decoded == alice_addr);
    
    // 示例 6: 账户类型
    println!("\n6. 账户类型");
    let types = vec![
        ("个人账户", AccountType::Personal),
        ("集体账户", AccountType::Collective),
        ("国家账户", AccountType::Nation),
        ("开发者账户", AccountType::Developer { is_main: true }),
        ("AI归集账户", AccountType::AIAggregate),
    ];
    
    for (name, account_type) in types {
        let acc = Account::new(alice_addr.clone(), account_type);
        println!("   {}: {:?}", name, acc.account_type);
    }
    
    // 示例 7: 区域
    println!("\n7. 区域");
    let regions = vec![
        ("美国", Region::US),
        ("欧洲", Region::EU),
        ("俄罗斯", Region::RU),
        ("亚洲", Region::AS),
        ("其他", Region::Other),
    ];
    
    for (name, region) in regions {
        println!("   {}: {:?}", name, region);
    }
    
    // 示例 8: 常量
    println!("\n8. 系统常量");
    println!("   总供应量: {} toki", TOTAL_SUPPLY / TOKI_BASE_UNIT);
    println!("   基本单位: {} μtoki", TOKI_BASE_UNIT);
    println!("   目标出块时间: {} 秒", TARGET_BLOCK_TIME_SECS);
    println!("   分发池比例: {:.2}%", DISTRIBUTION_POOL_RATIO * 100.0);
    println!("   储备池比例: {:.2}%", RESERVE_POOL_RATIO * 100.0);
    
    println!("\n=== 示例完成 ===\n");
}
