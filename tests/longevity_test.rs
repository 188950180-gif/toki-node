//! 长时间运行测试
//!
//! 测试系统长时间运行的稳定性

use std::time::{Duration, Instant};
use std::sync::Arc;
use anyhow::Result;
use tracing::{info, warn, error};

use toki_core::{Block, Transaction, Address, TOKI_BASE_UNIT};
use toki_consensus::{TransactionPool, DifficultyAdjuster};
use toki_storage::Database;

/// 长时间测试配置
#[derive(Clone)]
pub struct LongevityTestConfig {
    /// 运行时长（秒）
    pub duration_secs: u64,
    /// 操作间隔（毫秒）
    pub operation_interval_ms: u64,
    /// 检查间隔（秒）
    pub check_interval_secs: u64,
}

impl Default for LongevityTestConfig {
    fn default() -> Self {
        LongevityTestConfig {
            duration_secs: 300, // 5分钟
            operation_interval_ms: 100,
            check_interval_secs: 10,
        }
    }
}

/// 长时间测试结果
pub struct LongevityTestResult {
    /// 总运行时长（秒）
    pub total_duration_secs: u64,
    /// 总操作数
    pub total_operations: usize,
    /// 成功数
    pub success_count: usize,
    /// 失败数
    pub failure_count: usize,
    /// 是否崩溃
    pub crashed: bool,
    /// 错误信息
    pub error_messages: Vec<String>,
}

/// 长时间测试
pub struct LongevityTest {
    config: LongevityTestConfig,
    db: Arc<Database>,
    tx_pool: Arc<TransactionPool>,
}

impl LongevityTest {
    /// 创建新的长时间测试
    pub fn new(config: LongevityTestConfig, db: Arc<Database>, tx_pool: Arc<TransactionPool>) -> Self {
        LongevityTest {
            config,
            db,
            tx_pool,
        }
    }

    /// 运行长时间测试
    pub async fn run_test(&self) -> Result<LongevityTestResult> {
        info!("开始长时间运行测试");
        info!("运行时长: {} 秒", self.config.duration_secs);
        info!("操作间隔: {} 毫秒", self.config.operation_interval_ms);
        info!("检查间隔: {} 秒", self.config.check_interval_secs);

        let start = Instant::now();
        let mut total_operations = 0usize;
        let mut success_count = 0usize;
        let mut failure_count = 0usize;
        let mut error_messages = vec![];
        let mut crashed = false;

        let operation_interval = Duration::from_millis(self.config.operation_interval_ms);
        let check_interval = Duration::from_secs(self.config.check_interval_secs);
        let total_duration = Duration::from_secs(self.config.duration_secs);

        let mut last_check = Instant::now();

        while start.elapsed() < total_duration {
            // 执行操作
            let tx = create_test_transaction(total_operations);
            match self.tx_pool.add_transaction(tx) {
                Ok(_) => {
                    success_count += 1;
                }
                Err(e) => {
                    failure_count += 1;
                    error_messages.push(format!("交易添加失败: {}", e));
                }
            }

            // 数据库操作
            let key = format!("longevity_{}", total_operations);
            let value = format!("value_{}", total_operations);
            if let Err(e) = self.db.put("test_cf", key.as_bytes(), value.as_bytes()) {
                failure_count += 1;
                error_messages.push(format!("数据库写入失败: {}", e));
            }

            total_operations += 1;

            // 定期检查
            if last_check.elapsed() >= check_interval {
                self.perform_health_check(&mut error_messages).await;
                last_check = Instant::now();

                // 打印进度
                let elapsed = start.elapsed().as_secs();
                let progress = (elapsed as f64 / self.config.duration_secs as f64) * 100.0;
                info!("进度: {:.1}% ({}/{}s), 操作数: {}, 成功: {}, 失败: {}",
                    progress, elapsed, self.config.duration_secs, total_operations, success_count, failure_count);
            }

            // 等待下一次操作
            tokio::time::sleep(operation_interval).await;
        }

        let total_duration_secs = start.elapsed().as_secs();

        info!("长时间运行测试完成");
        info!("总运行时长: {}s", total_duration_secs);
        info!("总操作数: {}", total_operations);
        info!("成功: {} ({:.1}%)", success_count, success_count as f64 / total_operations as f64 * 100.0);
        info!("失败: {} ({:.1}%)", failure_count, failure_count as f64 / total_operations as f64 * 100.0);

        if crashed {
            error!("❌ 系统崩溃");
        } else {
            info!("✅ 系统运行稳定");
        }

        Ok(LongevityTestResult {
            total_duration_secs,
            total_operations,
            success_count,
            failure_count,
            crashed,
            error_messages,
        })
    }

    /// 执行健康检查
    async fn perform_health_check(&self, error_messages: &mut Vec<String>) {
        // 检查数据库
        if let Err(e) = self.db.flush() {
            error_messages.push(format!("数据库刷新失败: {}", e));
        }

        // 检查交易池
        let tx_count = self.tx_pool.tx_count();
        if tx_count > 100000 {
            warn!("⚠️  交易池过大: {}", tx_count);
            error_messages.push(format!("交易池过大: {}", tx_count));
        }

        // 检查内存使用
        let memory_mb = get_memory_usage_mb();
        if memory_mb > 1000 {
            warn!("⚠️  内存使用过高: {} MB", memory_mb);
            error_messages.push(format!("内存使用过高: {} MB", memory_mb));
        } else {
            info!("内存使用: {} MB", memory_mb);
        }
    }
}

/// 获取内存使用量（MB）
fn get_memory_usage_mb() -> u64 {
    #[cfg(target_os = "windows")]
    {
        use std::mem;
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

/// 创建测试交易
fn create_test_transaction(index: usize) -> Transaction {
    use toki_core::crypto;

    let from = Address::new([index as u8; 32]);
    let to = Address::new([(index + 1) as u8; 32]);
    let amount = TOKI_BASE_UNIT;
    let fee = TOKI_BASE_UNIT / 1000;
    let nonce = index as u64;

    let mut tx = Transaction::new(from, to, amount, fee, nonce);
    let signature = crypto::random::random_bytes(64);
    tx.signature = Some(signature);

    tx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_longevity_test_creation() {
        let db = Arc::new(Database::open("/tmp/test_longevity").unwrap());
        let tx_pool = Arc::new(TransactionPool::new(10000));
        let longevity_test = LongevityTest::new(LongevityTestConfig::default(), db, tx_pool);
        
        assert_eq!(longevity_test.config.duration_secs, 300);
    }
}
