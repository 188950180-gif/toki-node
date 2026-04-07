//! 压力测试
//!
//! 测试系统在高负载下的稳定性

use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::Semaphore;
use anyhow::Result;
use tracing::{info, warn, error};

use toki_core::{Block, Transaction, Address, TOKI_BASE_UNIT};
use toki_consensus::{TransactionPool, DifficultyAdjuster};
use toki_storage::Database;

/// 压力测试配置
#[derive(Clone)]
pub struct StressTestConfig {
    /// 并发数
    pub concurrency: usize,
    /// 每个并发的操作数
    pub operations_per_worker: usize,
    /// 超时时间（秒）
    pub timeout_secs: u64,
}

impl Default for StressTestConfig {
    fn default() -> Self {
        StressTestConfig {
            concurrency: 100,
            operations_per_worker: 1000,
            timeout_secs: 300,
        }
    }
}

/// 压力测试结果
pub struct StressTestResult {
    /// 总操作数
    pub total_operations: usize,
    /// 成功数
    pub success_count: usize,
    /// 失败数
    pub failure_count: usize,
    /// 总耗时（毫秒）
    pub total_duration_ms: u128,
    /// 平均耗时（微秒）
    pub avg_duration_us: u128,
    /// 吞吐量（操作/秒）
    pub throughput: f64,
}

/// 压力测试
pub struct StressTest {
    config: StressTestConfig,
    db: Arc<Database>,
    tx_pool: Arc<TransactionPool>,
}

impl StressTest {
    /// 创建新的压力测试
    pub fn new(config: StressTestConfig, db: Arc<Database>, tx_pool: Arc<TransactionPool>) -> Self {
        StressTest {
            config,
            db,
            tx_pool,
        }
    }

    /// 运行交易池压力测试
    pub async fn run_tx_pool_test(&self) -> Result<StressTestResult> {
        info!("开始交易池压力测试");
        info!("并发数: {}", self.config.concurrency);
        info!("每并发操作数: {}", self.config.operations_per_worker);

        let start = Instant::now();
        let semaphore = Arc::new(Semaphore::new(self.config.concurrency));
        let mut handles = vec![];
        let success_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let failure_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        for i in 0..self.config.concurrency {
            let permit = semaphore.clone();
            let tx_pool = self.tx_pool.clone();
            let success = success_count.clone();
            let failure = failure_count.clone();
            let ops = self.config.operations_per_worker;

            let handle = tokio::spawn(async move {
                let _permit = permit.acquire().await.unwrap();

                for j in 0..ops {
                    let tx = create_test_transaction(i, j);

                    match tx_pool.add_transaction(tx) {
                        Ok(_) => {
                            success.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        }
                        Err(_) => {
                            failure.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        }
                    }
                }
            });

            handles.push(handle);
        }

        // 等待所有任务完成
        for handle in handles {
            handle.await?;
        }

        let duration = start.elapsed();

        let total_operations = self.config.concurrency * self.config.operations_per_worker;
        let success_count = success_count.load(std::sync::atomic::Ordering::SeqCst);
        let failure_count = failure_count.load(std::sync::atomic::Ordering::SeqCst);
        let total_duration_ms = duration.as_millis();
        let avg_duration_us = duration.as_micros() / total_operations as u128;
        let throughput = total_operations as f64 / duration.as_secs_f64();

        info!("交易池压力测试完成");
        info!("总操作数: {}", total_operations);
        info!("成功: {} ({:.1}%)", success_count, success_count as f64 / total_operations as f64 * 100.0);
        info!("失败: {} ({:.1}%)", failure_count, failure_count as f64 / total_operations as f64 * 100.0);
        info!("总耗时: {:?}ms", total_duration_ms);
        info!("平均耗时: {}μs", avg_duration_us);
        info!("吞吐量: {:.2} ops/s", throughput);

        Ok(StressTestResult {
            total_operations,
            success_count,
            failure_count,
            total_duration_ms,
            avg_duration_us,
            throughput,
        })
    }

    /// 运行数据库压力测试
    pub async fn run_database_test(&self) -> Result<StressTestResult> {
        info!("开始数据库压力测试");
        info!("并发数: {}", self.config.concurrency);
        info!("每并发操作数: {}", self.config.operations_per_worker);

        let start = Instant::now();
        let semaphore = Arc::new(Semaphore::new(self.config.concurrency));
        let mut handles = vec![];
        let success_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let failure_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        for i in 0..self.config.concurrency {
            let permit = semaphore.clone();
            let db = self.db.clone();
            let success = success_count.clone();
            let failure = failure_count.clone();
            let ops = self.config.operations_per_worker;

            let handle = tokio::spawn(async move {
                let _permit = permit.acquire().await.unwrap();

                for j in 0..ops {
                    let key = format!("key_{}_{}", i, j);
                    let value = format!("value_{}", j);

                    match db.put("test_cf", key.as_bytes(), value.as_bytes()) {
                        Ok(_) => {
                            success.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        }
                        Err(_) => {
                            failure.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        }
                    }
                }
            });

            handles.push(handle);
        }

        // 等待所有任务完成
        for handle in handles {
            handle.await?;
        }

        let duration = start.elapsed();

        let total_operations = self.config.concurrency * self.config.operations_per_worker;
        let success_count = success_count.load(std::sync::atomic::Ordering::SeqCst);
        let failure_count = failure_count.load(std::sync::atomic::Ordering::SeqCst);
        let total_duration_ms = duration.as_millis();
        let avg_duration_us = duration.as_micros() / total_operations as u128;
        let throughput = total_operations as f64 / duration.as_secs_f64();

        info!("数据库压力测试完成");
        info!("总操作数: {}", total_operations);
        info!("成功: {} ({:.1}%)", success_count, success_count as f64 / total_operations as f64 * 100.0);
        info!("失败: {} ({:.1}%)", failure_count, failure_count as f64 / total_operations as f64 * 100.0);
        info!("总耗时: {:?}ms", total_duration_ms);
        info!("平均耗时: {}μs", avg_duration_us);
        info!("吞吐量: {:.2} ops/s", throughput);

        Ok(StressTestResult {
            total_operations,
            success_count,
            failure_count,
            total_duration_ms,
            avg_duration_us,
            throughput,
        })
    }

    /// 运行完整压力测试
    pub async fn run_full_test(&self) -> Result<Vec<(String, StressTestResult)>> {
        info!("开始完整压力测试");

        let mut results = vec![];

        // 交易池测试
        let tx_result = self.run_tx_pool_test().await?;
        results.push(("Transaction Pool".to_string(), tx_result));

        // 数据库测试
        let db_result = self.run_database_test().await?;
        results.push(("Database".to_string(), db_result));

        info!("完整压力测试完成");

        Ok(results)
    }
}

/// 创建测试交易
fn create_test_transaction(worker_id: usize, index: usize) -> Transaction {
    use toki_core::crypto;

    let from = Address::new([worker_id as u8; 32]);
    let to = Address::new([index as u8; 32]);
    let amount = TOKI_BASE_UNIT;
    let fee = TOKI_BASE_UNIT / 1000;
    let nonce = (worker_id * 10000 + index) as u64;

    let mut tx = Transaction::new(from, to, amount, fee, nonce);
    
    // 签名（简化）
    let signature = crypto::random::random_bytes(64);
    tx.signature = Some(signature);

    tx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stress_test_creation() {
        let db = Arc::new(Database::open("/tmp/test_stress").unwrap());
        let tx_pool = Arc::new(TransactionPool::new(10000));
        let stress_test = StressTest::new(StressTestConfig::default(), db, tx_pool);
        
        // 验证创建成功
        assert_eq!(stress_test.config.concurrency, 100);
    }
}
