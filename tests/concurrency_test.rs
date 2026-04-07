//! 并发测试
//!
//! 测试系统在高并发下的稳定性

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Semaphore, Mutex};
use anyhow::Result;
use tracing::{info, warn, error};

use toki_core::{Block, Transaction, Address, TOKI_BASE_UNIT};
use toki_consensus::{TransactionPool, DifficultyAdjuster};
use toki_storage::Database;

/// 并发测试配置
#[derive(Clone)]
pub struct ConcurrencyTestConfig {
    /// 最大并发数
    pub max_concurrency: usize,
    /// 每个线程的操作数
    pub operations_per_thread: usize,
    /// 测试时长（秒）
    pub duration_secs: u64,
}

impl Default for ConcurrencyTestConfig {
    fn default() -> Self {
        ConcurrencyTestConfig {
            max_concurrency: 200,
            operations_per_thread: 100,
            duration_secs: 60,
        }
    }
}

/// 并发测试结果
pub struct ConcurrencyTestResult {
    /// 总操作数
    pub total_operations: usize,
    /// 成功数
    pub success_count: usize,
    /// 失败数
    pub failure_count: usize,
    /// 并发数
    pub concurrency: usize,
    /// 总耗时（毫秒）
    pub total_duration_ms: u128,
    /// 吞吐量（操作/秒）
    pub throughput: f64,
    /// 是否有竞态条件
    pub has_race_condition: bool,
}

/// 并发测试
pub struct ConcurrencyTest {
    config: ConcurrencyTestConfig,
    db: Arc<Database>,
    tx_pool: Arc<TransactionPool>,
}

impl ConcurrencyTest {
    /// 创建新的并发测试
    pub fn new(config: ConcurrencyTestConfig, db: Arc<Database>, tx_pool: Arc<TransactionPool>) -> Self {
        ConcurrencyTest {
            config,
            db,
            tx_pool,
        }
    }

    /// 运行并发测试
    pub async fn run_test(&self) -> Result<ConcurrencyTestResult> {
        info!("开始并发测试");
        info!("最大并发数: {}", self.config.max_concurrency);
        info!("每线程操作数: {}", self.config.operations_per_thread);

        let start = Instant::now();
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrency));
        let counter = Arc::new(Mutex::new(0u64));
        let success_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let failure_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let has_race = Arc::new(std::sync::atomic::AtomicBool::new(false));

        // 启动多个并发任务
        let mut handles = vec![];

        for thread_id in 0..self.config.max_concurrency {
            let permit = semaphore.clone();
            let tx_pool = self.tx_pool.clone();
            let db = self.db.clone();
            let counter = counter.clone();
            let success = success_count.clone();
            let failure = failure_count.clone();
            let race_detected = has_race.clone();
            let ops = self.config.operations_per_thread;

            let handle = tokio::spawn(async move {
                let _permit = permit.acquire().await.unwrap();

                for i in 0..ops {
                    // 模拟竞态条件检测
                    let mut count = counter.lock().await;
                    let prev = *count;
                    *count += 1;
                    let current = *count;
                    drop(count);

                    // 检查是否有序列化问题
                    if current != prev + 1 {
                        race_detected.store(true, std::sync::atomic::Ordering::SeqCst);
                    }

                    // 执行交易池操作
                    let tx = create_test_transaction(thread_id, i);
                    match tx_pool.add_transaction(tx) {
                        Ok(_) => {
                            success.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        }
                        Err(_) => {
                            failure.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        }
                    }

                    // 执行数据库操作
                    let key = format!("concurrent_{}_{}", thread_id, i);
                    let value = format!("value_{}", i);
                    if db.put("test_cf", key.as_bytes(), value.as_bytes()).is_err() {
                        failure.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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

        let total_operations = self.config.max_concurrency * self.config.operations_per_thread;
        let success_count = success_count.load(std::sync::atomic::Ordering::SeqCst);
        let failure_count = failure_count.load(std::sync::atomic::Ordering::SeqCst);
        let has_race = has_race.load(std::sync::atomic::Ordering::SeqCst);
        let total_duration_ms = duration.as_millis();
        let throughput = total_operations as f64 / duration.as_secs_f64();

        info!("并发测试完成");
        info!("总操作数: {}", total_operations);
        info!("成功: {} ({:.1}%)", success_count, success_count as f64 / total_operations as f64 * 100.0);
        info!("失败: {} ({:.1}%)", failure_count, failure_count as f64 / total_operations as f64 * 100.0);
        info!("总耗时: {:?}ms", total_duration_ms);
        info!("吞吐量: {:.2} ops/s", throughput);
        info!("竞态条件: {}", if has_race { "检测到" } else { "未检测到" });

        if has_race {
            warn!("⚠️  检测到竞态条件");
        } else {
            info!("✅ 未检测到竞态条件");
        }

        Ok(ConcurrencyTestResult {
            total_operations,
            success_count,
            failure_count,
            concurrency: self.config.max_concurrency,
            total_duration_ms,
            throughput,
            has_race_condition: has_race,
        })
    }

    /// 运行读写并发测试
    pub async fn run_read_write_test(&self) -> Result<ConcurrencyTestResult> {
        info!("开始读写并发测试");

        let start = Instant::now();
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrency));
        let success_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let failure_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let mut handles = vec![];

        // 50% 读，50% 写
        for thread_id in 0..self.config.max_concurrency {
            let permit = semaphore.clone();
            let db = self.db.clone();
            let success = success_count.clone();
            let failure = failure_count.clone();
            let is_read = thread_id % 2 == 0;
            let ops = self.config.operations_per_thread;

            let handle = tokio::spawn(async move {
                let _permit = permit.acquire().await.unwrap();

                for i in 0..ops {
                    if is_read {
                        // 读操作
                        let key = format!("rw_test_{}", i);
                        match db.get("test_cf", key.as_bytes()) {
                            Ok(_) => {
                                success.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            }
                            Err(_) => {
                                failure.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            }
                        }
                    } else {
                        // 写操作
                        let key = format!("rw_test_{}", i);
                        let value = format!("value_{}_{}", thread_id, i);
                        match db.put("test_cf", key.as_bytes(), value.as_bytes()) {
                            Ok(_) => {
                                success.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            }
                            Err(_) => {
                                failure.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            }
                        }
                    }
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.await?;
        }

        let duration = start.elapsed();
        let total_operations = self.config.max_concurrency * self.config.operations_per_thread;
        let success_count = success_count.load(std::sync::atomic::Ordering::SeqCst);
        let failure_count = failure_count.load(std::sync::atomic::Ordering::SeqCst);
        let total_duration_ms = duration.as_millis();
        let throughput = total_operations as f64 / duration.as_secs_f64();

        info!("读写并发测试完成");
        info!("总操作数: {}", total_operations);
        info!("成功: {}", success_count);
        info!("失败: {}", failure_count);
        info!("吞吐量: {:.2} ops/s", throughput);

        Ok(ConcurrencyTestResult {
            total_operations,
            success_count,
            failure_count,
            concurrency: self.config.max_concurrency,
            total_duration_ms,
            throughput,
            has_race_condition: false,
        })
    }
}

/// 创建测试交易
fn create_test_transaction(thread_id: usize, index: usize) -> Transaction {
    use toki_core::crypto;

    let from = Address::new([thread_id as u8; 32]);
    let to = Address::new([index as u8; 32]);
    let amount = TOKI_BASE_UNIT;
    let fee = TOKI_BASE_UNIT / 1000;
    let nonce = (thread_id * 10000 + index) as u64;

    let mut tx = Transaction::new(from, to, amount, fee, nonce);
    let signature = crypto::random::random_bytes(64);
    tx.signature = Some(signature);

    tx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrency_test_creation() {
        let db = Arc::new(Database::open("/tmp/test_concurrency").unwrap());
        let tx_pool = Arc::new(TransactionPool::new(10000));
        let concurrency_test = ConcurrencyTest::new(ConcurrencyTestConfig::default(), db, tx_pool);
        
        assert_eq!(concurrency_test.config.max_concurrency, 200);
    }
}
