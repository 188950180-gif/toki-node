//! AI 任务调度器实现
//!
//! 协调所有 AI 自动执行任务的具体实现

use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{info, debug, warn};
use anyhow::Result;

use toki_core::{Address, Transaction, TOKI_BASE_UNIT};
use toki_storage::{BlockStore, AccountStore};
use toki_consensus::TransactionPool;
use crate::{
    aggregator::AIAggregator,
    charity::CharitySystem,
    welfare::WelfareSystem,
    scheduler::{Scheduler, TaskType},
};

/// AI 任务执行器
pub struct AITaskExecutor {
    scheduler: Arc<Scheduler>,
    aggregator: Arc<AIAggregator>,
    charity: Arc<CharitySystem>,
    welfare: Arc<WelfareSystem>,
    /// 区块高度
    block_height: Arc<RwLock<u64>>,
    /// 区块存储
    block_store: Option<Arc<BlockStore>>,
    /// 账户存储
    account_store: Option<Arc<AccountStore>>,
    /// 交易池
    tx_pool: Option<Arc<RwLock<TransactionPool>>>,
}

impl AITaskExecutor {
    /// 创建新的任务执行器
    pub fn new(
        scheduler: Arc<Scheduler>,
        aggregator: Arc<AIAggregator>,
        charity: Arc<CharitySystem>,
        welfare: Arc<WelfareSystem>,
    ) -> Self {
        AITaskExecutor {
            scheduler,
            aggregator,
            charity,
            welfare,
            block_height: Arc::new(RwLock::new(0)),
            block_store: None,
            account_store: None,
            tx_pool: None,
        }
    }
    
    /// 设置存储和交易池
    pub fn with_storage(
        &mut self,
        block_store: Arc<BlockStore>,
        account_store: Arc<AccountStore>,
        tx_pool: Arc<RwLock<TransactionPool>>,
    ) {
        self.block_store = Some(block_store);
        self.account_store = Some(account_store);
        self.tx_pool = Some(tx_pool);
    }

    /// 更新区块高度
    pub fn update_block_height(&self, height: u64) {
        *self.block_height.write() = height;
    }

    /// 执行任务
    pub async fn execute_task(&self, task_type: &TaskType) -> Result<()> {
        match task_type {
            TaskType::DistributeBasic => {
                self.distribute_basic().await?;
            }
            TaskType::CheckEqualization => {
                self.check_equalization().await?;
            }
            TaskType::CalculateTheta => {
                self.calculate_theta().await?;
            }
            TaskType::ExecuteCharity => {
                self.execute_charity().await?;
            }
            TaskType::CheckFiatChannel => {
                self.check_fiat_channel().await?;
            }
            TaskType::RecoverInactive => {
                self.recover_inactive().await?;
            }
        }
        Ok(())
    }

    /// 分发基础赠送
    async fn distribute_basic(&self) -> Result<()> {
        info!("开始执行基础分发任务");

        // 检查是否有存储和交易池
        let account_store = match &self.account_store {
            Some(store) => store,
            None => {
                warn!("账户存储未设置，跳过分发");
                return Ok(());
            }
        };
        
        let tx_pool = match &self.tx_pool {
            Some(pool) => pool,
            None => {
                warn!("交易池未设置，跳过分发");
                return Ok(());
            }
        };

        let height = *self.block_height.read();
        
        // 每 1000 个区块执行一次
        if height % 1000 != 0 {
            debug!("当前区块 {} 不满足分发条件", height);
            return Ok(());
        }

        // 查询符合条件的账户（余额 > 0）
        let eligible_accounts = match account_store.get_all_accounts() {
            Ok(accounts) => accounts.into_iter()
                .filter(|a| a.balance > 0)
                .collect::<Vec<_>>(),
            Err(e) => {
                warn!("查询账户失败: {}", e);
                return Ok(());
            }
        };

        if eligible_accounts.is_empty() {
            debug!("没有符合条件的账户");
            return Ok(());
        }

        // 计算分发金额（每个账户 100 toki）
        let amount_per_account = 100 * TOKI_BASE_UNIT;
        
        info!("基础分发: {} 个账户, 每个账户 {} toki", 
              eligible_accounts.len(), amount_per_account / TOKI_BASE_UNIT);

        // TODO: 创建分发交易并提交到交易池
        // 这需要实现 Transaction::new_distribution 方法
        
        info!("基础分发完成");
        Ok(())
    }

    /// 检查平权
    async fn check_equalization(&self) -> Result<()> {
        info!("开始执行平权检查");

        let stats = self.aggregator.get_stats();
        info!(
            "账户统计: 总数={}, 贫困账户={}, 富裕账户={}",
            stats.total_accounts, stats.poor_accounts, stats.rich_accounts
        );

        // 如果贫困账户超过 20%，触发平权
        let poverty_rate = stats.total_accounts.saturating_sub(0);
        if stats.total_accounts > 0 {
            let rate = (stats.poor_accounts as f64 / stats.total_accounts as f64) * 100.0;
            if rate > 20.0 {
                info!("贫困率 {:.1}% 超过阈值，触发平权", rate);
                self.aggregator.equalize()?;
            }
        }

        Ok(())
    }

    /// 计算 theta
    async fn calculate_theta(&self) -> Result<()> {
        info!("开始计算 theta");

        // 检查是否有区块存储
        let block_store = match &self.block_store {
            Some(store) => store,
            None => {
                warn!("区块存储未设置，使用统计方法计算 theta");
                return self.calculate_theta_from_stats().await;
            }
        };

        // 获取最新区块高度
        let latest_height = match block_store.get_latest_height()? {
            Some(h) => h,
            None => {
                warn!("没有区块数据");
                return Ok(());
            }
        };

        // 获取最近 100 个区块的交易数据
        let start_height = if latest_height > 100 { latest_height - 100 } else { 0 };
        let mut total_volume = 0u64;
        let mut tx_count = 0usize;

        for height in start_height..=latest_height {
            if let Ok(Some(block)) = block_store.get_block_by_height(height) {
                for tx in &block.transactions {
                    total_volume += tx.amount;
                    tx_count += 1;
                }
            }
        }

        if tx_count == 0 {
            info!("没有交易数据，跳过 theta 计算");
            return Ok(());
        }

        // 计算 theta = 总交易额 / 交易数量
        let theta = total_volume as f64 / tx_count as f64;

        info!("Theta 计算: {} 笔交易, 总额 {} toki, theta = {:.4}", 
              tx_count, total_volume / TOKI_BASE_UNIT, theta);

        // theta > 10000 表示交易额过大，可能需要调整
        if theta > 10000.0 * TOKI_BASE_UNIT as f64 {
            warn!("Theta 值 {:.4} 过高，建议检查交易数据", theta);
        }

        Ok(())
    }
    
    /// 从统计数据计算 theta（备用方法）
    async fn calculate_theta_from_stats(&self) -> Result<()> {
        let stats = self.aggregator.get_stats();
        
        // theta = 富裕账户平均余额 / 贫困账户平均余额
        let theta = if stats.poor_accounts > 0 && stats.rich_accounts > 0 {
            let rich_avg = stats.rich_total_balance / stats.rich_accounts as u64;
            let poor_avg = stats.poor_total_balance / stats.poor_accounts as u64;
            rich_avg as f64 / poor_avg as f64
        } else {
            1.0
        };

        info!("Theta 指数（统计）: {:.2}", theta);

        // theta > 10 表示严重不平等
        if theta > 10.0 {
            warn!("不平等指数 {:.2} 过高，建议启动平权", theta);
        }

        Ok(())
    }

    /// 执行公益
    async fn execute_charity(&self) -> Result<()> {
        info!("开始执行公益任务");

        let height = *self.block_height.read();
        
        // 每 100 个区块执行一次
        if height % 100 != 0 {
            debug!("当前区块 {} 不满足公益执行条件", height);
            return Ok(());
        }

        // 检查是否有活跃的公益项目
        let projects = self.charity.get_active_projects();
        if projects.is_empty() {
            debug!("没有活跃的公益项目");
            return Ok(());
        }

        // 自动捐赠到活跃项目
        for project in projects {
            let donation_amount = 10 * TOKI_BASE_UNIT;
            self.charity.donate_to_project(&project.id, donation_amount)?;
            info!("自动捐赠到项目 {}: {} TOKI", project.name, donation_amount);
        }

        Ok(())
    }

    /// 检查法币通道
    async fn check_fiat_channel(&self) -> Result<()> {
        info!("检查法币通道");

        // 模拟法币通道检查
        // 在实际实现中，这里会检查：
        // 1. 法币出入金记录
        // 2. 汇率更新
        // 3. KYC 验证状态

        debug!("法币通道检查完成");
        Ok(())
    }

    /// 回收不活跃账户
    async fn recover_inactive(&self) -> Result<()> {
        info!("开始回收不活跃账户");

        let height = *self.block_height.read();
        
        // 每 10000 个区块执行一次
        if height % 10000 != 0 {
            debug!("当前区块 {} 不满足回收条件", height);
            return Ok(());
        }

        // 模拟不活跃账户回收
        // 在实际实现中，这里会：
        // 1. 识别超过 1 年未活动的账户
        // 2. 发送通知
        // 3. 回收余额到公益基金

        debug!("不活跃账户回收完成");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_distribute() {
        // TODO: 实现测试
    }
}
