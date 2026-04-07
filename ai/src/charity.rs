//! 公益模块
//!
//! 区块链自主运行核心功能
//! - 公益资金管理
//! - 公益项目执行
//! - 透明度追踪

use anyhow::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

use toki_core::{Address, Hash, TOKI_BASE_UNIT};

/// 公益项目
#[derive(Debug, Clone)]
pub struct CharityProject {
    /// 项目 ID
    pub id: String,
    /// 项目名称
    pub name: String,
    /// 项目描述
    pub description: String,
    /// 目标金额
    pub target_amount: u64,
    /// 已筹集金额
    pub raised_amount: u64,
    /// 受益地址
    pub beneficiary_address: Address,
    /// 创建时间
    pub created_at: i64,
    /// 状态
    pub status: CharityStatus,
}

/// 公益项目状态
#[derive(Debug, Clone, PartialEq)]
pub enum CharityStatus {
    /// 筹集中
    Fundraising,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
}

/// 公益系统
pub struct CharitySystem {
    /// 公益资金池
    charity_pool: Arc<RwLock<u64>>,
    /// 公益项目
    projects: Arc<RwLock<HashMap<String, CharityProject>>>,
    /// 公益地址
    charity_address: Address,
}

impl CharitySystem {
    /// 创建新的公益系统
    pub fn new(charity_address: Address) -> Self {
        info!("创建公益系统");

        CharitySystem {
            charity_pool: Arc::new(RwLock::new(0)),
            projects: Arc::new(RwLock::new(HashMap::new())),
            charity_address,
        }
    }

    /// 添加公益资金
    pub fn add_funds(&self, amount: u64) -> Result<()> {
        let mut pool = self.charity_pool.write();
        *pool += amount;
        info!("公益资金增加: {}", amount);
        Ok(())
    }

    /// 创建公益项目
    pub fn create_project(&self, project: CharityProject) -> Result<()> {
        let mut projects = self.projects.write();

        if projects.contains_key(&project.id) {
            return Err(anyhow::anyhow!("项目已存在"));
        }

        info!(
            "创建公益项目: {} (目标: {})",
            project.name, project.target_amount
        );
        projects.insert(project.id.clone(), project);

        Ok(())
    }

    /// 向项目捐赠
    pub fn donate_to_project(&self, project_id: &str, amount: u64) -> Result<()> {
        let mut pool = self.charity_pool.write();

        if *pool < amount {
            return Err(anyhow::anyhow!("公益资金不足"));
        }

        *pool -= amount;

        let mut projects = self.projects.write();
        if let Some(project) = projects.get_mut(project_id) {
            project.raised_amount += amount;

            info!(
                "捐赠给项目 {}: {} (总计: {}/{})",
                project.name, amount, project.raised_amount, project.target_amount
            );

            // 检查是否达到目标
            if project.raised_amount >= project.target_amount {
                project.status = CharityStatus::Completed;
                info!("项目 {} 已完成!", project.name);
            }
        }

        Ok(())
    }

    /// 获取公益资金池余额
    pub fn get_pool_balance(&self) -> u64 {
        *self.charity_pool.read()
    }

    /// 获取项目信息
    pub fn get_project(&self, project_id: &str) -> Option<CharityProject> {
        self.projects.read().get(project_id).cloned()
    }

    /// 获取所有项目
    pub fn get_all_projects(&self) -> Vec<CharityProject> {
        self.projects.read().values().cloned().collect()
    }

    /// 获取公益活动统计
    pub fn get_stats(&self) -> CharityStats {
        let pool_balance = self.get_pool_balance();
        let projects = self.get_all_projects();

        let total_raised = projects.iter().map(|p| p.raised_amount).sum();

        let completed_projects = projects
            .iter()
            .filter(|p| p.status == CharityStatus::Completed)
            .count();

        CharityStats {
            pool_balance,
            total_raised,
            project_count: projects.len(),
            completed_projects,
        }
    }
}

/// 公益统计信息
#[derive(Debug, Clone)]
pub struct CharityStats {
    /// 公益资金池余额
    pub pool_balance: u64,
    /// 总筹集金额
    pub total_raised: u64,
    /// 项目总数
    pub project_count: usize,
    /// 已完成项目数
    pub completed_projects: usize,
}

impl Default for CharitySystem {
    fn default() -> Self {
        Self::new(Address::new([1u8; 32]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charity_system_creation() {
        let charity = CharitySystem::default();
        assert_eq!(charity.get_pool_balance(), 0);
    }

    #[test]
    fn test_add_funds() {
        let charity = CharitySystem::default();
        charity.add_funds(1000 * TOKI_BASE_UNIT).unwrap();

        assert_eq!(charity.get_pool_balance(), 1000 * TOKI_BASE_UNIT);
    }

    #[test]
    fn test_create_project() {
        let charity = CharitySystem::default();

        let project = CharityProject {
            id: "test_project".to_string(),
            name: "Test Project".to_string(),
            description: "Test Description".to_string(),
            target_amount: 10 * TOKI_BASE_UNIT,
            raised_amount: 0,
            beneficiary_address: Address::new([2u8; 32]),
            created_at: chrono::Utc::now().timestamp(),
            status: CharityStatus::Fundraising,
        };

        charity.create_project(project).unwrap();
        assert_eq!(charity.get_all_projects().len(), 1);
    }

    #[test]
    fn test_donate_to_project() {
        let charity = CharitySystem::default();

        charity.add_funds(100 * TOKI_BASE_UNIT).unwrap();

        let project = CharityProject {
            id: "test_project".to_string(),
            name: "Test Project".to_string(),
            description: "Test Description".to_string(),
            target_amount: 10 * TOKI_BASE_UNIT,
            raised_amount: 0,
            beneficiary_address: Address::new([2u8; 32]),
            created_at: chrono::Utc::now().timestamp(),
            status: CharityStatus::Fundraising,
        };

        charity.create_project(project.clone()).unwrap();
        charity
            .donate_to_project("test_project", 5 * TOKI_BASE_UNIT)
            .unwrap();

        let updated = charity.get_project("test_project").unwrap();
        assert_eq!(updated.raised_amount, 5 * TOKI_BASE_UNIT);
    }

    #[test]
    fn test_get_stats() {
        let charity = CharitySystem::default();
        let stats = charity.get_stats();

        assert_eq!(stats.pool_balance, 0);
        assert_eq!(stats.total_raised, 0);
        assert_eq!(stats.project_count, 0);
        assert_eq!(stats.completed_projects, 0);
    }
}
