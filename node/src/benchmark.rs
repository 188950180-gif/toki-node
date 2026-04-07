//! 基准测试

use std::time::Instant;
use toki_core::*;

/// 基准测试结果
struct BenchmarkResult {
    name: String,
    iterations: u64,
    total_time_ms: u128,
    avg_time_us: f64,
    ops_per_sec: f64,
}

impl BenchmarkResult {
    fn print(&self) {
        println!(
            "{:<30} {:>8} iters  {:>8} ms  {:>8.2} μs/op  {:>10.0} ops/s",
            self.name, self.iterations, self.total_time_ms, self.avg_time_us, self.ops_per_sec
        );
    }
}

fn benchmark<F: Fn()>(name: &str, iterations: u64, f: F) -> BenchmarkResult {
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();

    let total_time_ms = elapsed.as_millis();
    let avg_time_us = (elapsed.as_nanos() as f64 / iterations as f64) / 1000.0;
    let ops_per_sec = if total_time_ms > 0 {
        (iterations as f64) / (total_time_ms as f64 / 1000.0)
    } else {
        0.0
    };

    BenchmarkResult {
        name: name.to_string(),
        iterations,
        total_time_ms,
        avg_time_us,
        ops_per_sec,
    }
}

fn main() {
    println!("\n=== Toki 基准测试 ===\n");
    println!(
        "{:<30} {:>10}       {:>10}  {:>12}  {:>12}",
        "测试名称", "迭代次数", "总时间", "平均时间", "吞吐量"
    );
    println!("{}", "-".repeat(80));

    // 哈希计算基准
    let data = vec![0u8; 1024]; // 1KB 数据
    benchmark("Hash::from_data (1KB)", 10000, || {
        Hash::from_data(&data);
    })
    .print();

    // 地址生成基准
    benchmark("Address::new", 100000, || {
        Address::new([1u8; 32]);
    })
    .print();

    // Base58 编码基准
    let addr = Address::new([1u8; 32]);
    benchmark("Address::to_base58", 100000, || {
        addr.to_base58();
    })
    .print();

    // 交易创建基准
    let addr = Address::new([1u8; 32]);
    benchmark("Transaction::new", 10000, || {
        let output = Output::new(addr.clone(), 100 * TOKI_BASE_UNIT);
        let ring_sig = RingSignature::new(vec![], vec![1u8; 64], Hash::ZERO);
        Transaction::new(vec![], vec![output], ring_sig, 1000);
    })
    .print();

    // 区块创建基准
    benchmark("Block::new (empty)", 10000, || {
        Block::new(0, Hash::ZERO, vec![], 1000, Address::ZERO);
    })
    .print();

    // 区块哈希基准
    let block = Block::new(0, Hash::ZERO, vec![], 1000, Address::ZERO);
    benchmark("Block::hash", 100000, || {
        block.hash();
    })
    .print();

    // Merkle 根计算基准
    let mut txs = Vec::new();
    for _ in 0..100 {
        let output = Output::new(addr.clone(), 100 * TOKI_BASE_UNIT);
        let ring_sig = RingSignature::new(vec![], vec![1u8; 64], Hash::ZERO);
        txs.push(Transaction::new(vec![], vec![output], ring_sig, 1000));
    }
    let block_with_txs = Block::new(0, Hash::ZERO, txs, 1000, Address::ZERO);
    benchmark("Block::merkle_root (100 txs)", 1000, || {
        block_with_txs.verify_merkle_root();
    })
    .print();

    // 创世区块创建基准
    let genesis = Block::genesis();
    benchmark("Block::hash (genesis)", 10000, || {
        genesis.hash();
    })
    .print();

    println!("{}", "-".repeat(80));
    println!("\n基准测试完成！\n");
}
