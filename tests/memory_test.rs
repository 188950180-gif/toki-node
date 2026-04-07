//! 内存泄漏测试
//!
//! 检查系统是否存在内存泄漏

use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::Semaphore;
use anyhow::Result;
use tracing::{info, warn};

use toki_core::{Block, Transaction, Address, TOKI_BASE_UNIT};
use toki_consensus::TransactionPool;
use toki_storage::Database;

/// 内存测试配置
#[derive(Clone)]
pub struct MemoryTestConfig {
    /// 迭代次数
    pub iterations: usize,
    /// 每次迭代的操作数
    pub operations_per_iteration: usize,
    /// 内存阈值（MB）
    pub memory_threshold_mb: u64,
}

impl Default for MemoryTestConfig {
    fn default() -> Self {
        MemoryTestConfig {
            iterations: 100,
            operations_per_iteration: 1000,
            memory_threshold_mb: 100,
        }
    }
}

/// 内存测试结果
pub struct MemoryTestResult {
    /// 初始内存（MB）
    pub initial_memory_mb: u64,
    /// 最终内存（MB）
    pub final_memory_mb: u64,
    /// 内存增长（MB）
    pub memory_growth_mb: i64,
    /// 是否超过阈值
    pub exceeded_threshold: bool,
    /// 总操作数
    pub total_operations: usize,
}

/// 内存测试
pub struct MemoryTest {
    config: MemoryTestConfig,
    db: Arc<Database>,
    tx_pool: Arc<TransactionPool>,
}

impl MemoryTest {
    /// 创建新的内存测试
    pub fn new(config: MemoryTestConfig, db: Arc<Database>, tx_pool: Arc<TransactionPool>) -> Self {
        MemoryTest {
            config,
            db,
            tx_pool,
        }
    }

    /// 获取当前内存使用量（MB）
    fn get_memory_usage_mb() -> u64 {
        // 简化实现：使用系统调用获取内存
        // 实际应该使用平台特定的 API
        #[cfg(target_os = "windows")]
        {
            // Windows: 使用 GetProcessMemoryInfo
            use std::mem;
            use std::ptr;
            use winapi::um::psapi::GetProcessMemoryInfo;
            use winapi::um::psapi::PROCESS_MEMORY_COUNTERS;
            use winapi::um::processthreadsapi::GetCurrentProcess;

            unsafe {
                let mut pmc: PROCESS_MEMORY_COUNTERS = mem::zeroed();
                pmc.cb = mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;

                if GetProcessMemoryInfo(
                    GetCurrentProcess(),
                    &mut pmc,
                    mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
                ) != 0
                {
                    return pmc.WorkingSetSize / 1024 / 1024;
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Linux/macOS: 读取 /proc/self/status
            if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            if let Ok(kb) = parts[1].parse::<u64>() {
                                return kb / 1024;
                            }
                        }
                    }
                }
            }
        }

        0
    }

    /// 运行内存泄漏测试
    pub async fn run_test(&self) -> Result<MemoryTestResult> {
        info!("开始内存泄漏测试");
        info!("迭代次数: {}", self.config.iterations);
        info!("每次迭代操作数: {}", self.config.operations_per_iteration);

        // 记录初始内存
        let initial_memory = Self::get_memory_usage_mb();
        info!("初始内存: {} MB", initial_memory);

        let start = Instant::now();

        // 执行多次迭代
        for iteration in 0..self.config.iterations {
            // 执行操作
            for i in 0..self.config.operations_per_iteration {
                let tx = create_test_transaction(iteration, i);
                self.tx_pool.add_transaction(tx).ok();
            }

            // 定期清理
            if iteration % 10 == 0 {
                self.tx_pool.cleanup();
                self.db.flush().ok();
            }

            // 打印进度
            if iteration % 20 == 0 {
                let current_memory = Self::get_memory_usage_mb();
                info!("迭代 {}/{} - 当前内存: {} MB", iteration, self.config.iterations, current_memory);
            }
        }

        let duration = start.elapsed();

        // 记录最终内存
        let final_memory = Self::get_memory_usage_mb();
        info!("最终内存: {} MB", final_memory);

        let memory_growth = final_memory as i64 - initial_memory as i64;
        let exceeded_threshold = memory_growth as u64 > self.config.memory_threshold_mb;
        let total_operations = self.config.iterations * self.config.operations_per_iteration;

        info!("内存增长: {} MB", memory_growth);
        info!("总耗时: {:?}", duration);
        info!("总操作数: {}", total_operations);

        if exceeded_threshold {
            warn!("⚠️  内存增长超过阈值: {} MB > {} MB", memory_growth, self.config.memory_threshold_mb);
        } else {
            info!("✅ 内存增长在正常范围内");
        }

        Ok(MemoryTestResult {
            initial_memory_mb: initial_memory,
            final_memory_mb: final_memory,
            memory_growth_mb: memory_growth,
            exceeded_threshold,
            total_operations,
        })
    }
}

/// 创建测试交易
fn create_test_transaction(iteration: usize, index: usize) -> Transaction {
    use toki_core::crypto;

    let from = Address::new([iteration as u8; 32]);
    let to = Address::new([index as u8; 32]);
    let amount = TOKI_BASE_UNIT;
    let fee = TOKI_BASE_UNIT / 1000;
    let nonce = (iteration * 10000 + index) as u64;

    let mut tx = Transaction::new(from, to, amount, fee, nonce);
    let signature = crypto::random::random_bytes(64);
    tx.signature = Some(signature);

    tx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_test_creation() {
        let db = Arc::new(Database::open("/tmp/test_memory").unwrap());
        let tx_pool = Arc::new(TransactionPool::new(10000));
        let memory_test = MemoryTest::new(MemoryTestConfig::default(), db, tx_pool);
        
        assert_eq!(memory_test.config.iterations, 100);
    }
}
