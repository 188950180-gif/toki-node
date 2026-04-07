//! 测试网配置与验证
//!
//! 提供测试网部署、验证和监控功能

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// 测试网配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestnetConfig {
    /// 网络名称
    pub name: String,
    /// 链 ID
    pub chain_id: u64,
    /// 数据目录
    pub data_dir: PathBuf,
    /// 网络端口
    pub network_port: u16,
    /// API 端口
    pub api_port: u16,
    /// 种子节点
    pub seed_nodes: Vec<String>,
    /// 出块时间（秒）
    pub block_time: u64,
    /// 初始难度
    pub initial_difficulty: u64,
    /// 测试账户
    pub test_accounts: Vec<TestAccount>,
    /// 是否启用挖矿
    pub enable_mining: bool,
    /// 是否启用 API
    pub enable_api: bool,
}

/// 测试账户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestAccount {
    /// 账户名称
    pub name: String,
    /// 地址
    pub address: String,
    /// 初始余额
    pub initial_balance: u64,
}

impl Default for TestnetConfig {
    fn default() -> Self {
        TestnetConfig {
            name: "toki-testnet".to_string(),
            chain_id: 1000,
            data_dir: PathBuf::from("./testnet-data"),
            network_port: 30334,
            api_port: 8081,
            seed_nodes: vec![],
            block_time: 5,               // 测试网更快
            initial_difficulty: 100_000, // 测试网更简单
            test_accounts: vec![
                TestAccount {
                    name: "alice".to_string(),
                    address: "toki1alice0000000000000000000000000000000".to_string(),
                    initial_balance: 1_000_000_000_000, // 100 万 toki
                },
                TestAccount {
                    name: "bob".to_string(),
                    address: "toki1bob00000000000000000000000000000000000".to_string(),
                    initial_balance: 1_000_000_000_000,
                },
            ],
            enable_mining: true,
            enable_api: true,
        }
    }
}

/// 测试网验证器
pub struct TestnetValidator {
    /// 配置
    config: TestnetConfig,
    /// 验证结果
    results: Vec<ValidationResult>,
}

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// 检查项名称
    pub name: String,
    /// 是否通过
    pub passed: bool,
    /// 消息
    pub message: String,
    /// 耗时（毫秒）
    pub duration_ms: u64,
}

impl TestnetValidator {
    /// 创建新的验证器
    pub fn new(config: TestnetConfig) -> Self {
        TestnetValidator {
            config,
            results: Vec::new(),
        }
    }

    /// 运行所有验证
    pub fn validate_all(&mut self) -> Result<ValidationReport> {
        info!("开始测试网验证...");

        self.results.clear();

        // 1. 检查配置
        self.validate_config()?;

        // 2. 检查数据目录
        self.validate_data_dir()?;

        // 3. 检查网络配置
        self.validate_network()?;

        // 4. 检查创世区块
        self.validate_genesis()?;

        // 5. 检查账户
        self.validate_accounts()?;

        // 6. 检查共识参数
        self.validate_consensus()?;

        // 生成报告
        let report = self.generate_report();

        info!("验证完成: {}/{} 通过", report.passed, report.total);

        Ok(report)
    }

    /// 验证配置
    fn validate_config(&mut self) -> Result<()> {
        let start = std::time::Instant::now();

        let passed =
            !self.config.name.is_empty() && self.config.chain_id > 0 && self.config.block_time > 0;

        self.results.push(ValidationResult {
            name: "配置验证".to_string(),
            passed,
            message: if passed {
                "配置有效".to_string()
            } else {
                "配置无效".to_string()
            },
            duration_ms: start.elapsed().as_millis() as u64,
        });

        Ok(())
    }

    /// 验证数据目录
    fn validate_data_dir(&mut self) -> Result<()> {
        let start = std::time::Instant::now();

        let passed = if !self.config.data_dir.exists() {
            fs::create_dir_all(&self.config.data_dir).is_ok()
        } else {
            true
        };

        self.results.push(ValidationResult {
            name: "数据目录验证".to_string(),
            passed,
            message: format!("数据目录: {:?}", self.config.data_dir),
            duration_ms: start.elapsed().as_millis() as u64,
        });

        Ok(())
    }

    /// 验证网络配置
    fn validate_network(&mut self) -> Result<()> {
        let start = std::time::Instant::now();

        let passed = self.config.network_port > 0
            && self.config.api_port > 0
            && self.config.network_port != self.config.api_port;

        self.results.push(ValidationResult {
            name: "网络配置验证".to_string(),
            passed,
            message: format!(
                "网络端口: {}, API端口: {}",
                self.config.network_port, self.config.api_port
            ),
            duration_ms: start.elapsed().as_millis() as u64,
        });

        Ok(())
    }

    /// 验证创世区块
    fn validate_genesis(&mut self) -> Result<()> {
        let start = std::time::Instant::now();

        let genesis_path = self.config.data_dir.join("genesis.json");
        let passed = if !genesis_path.exists() {
            // 创建创世区块
            let genesis = serde_json::json!({
                "chain_id": self.config.chain_id,
                "name": &self.config.name,
                "timestamp": 0,
                "height": 0,
                "transactions": [],
                "accounts": &self.config.test_accounts,
            });

            fs::write(&genesis_path, serde_json::to_string_pretty(&genesis)?)?;
            true
        } else {
            true
        };

        self.results.push(ValidationResult {
            name: "创世区块验证".to_string(),
            passed,
            message: format!("创世区块: {:?}", genesis_path),
            duration_ms: start.elapsed().as_millis() as u64,
        });

        Ok(())
    }

    /// 验证账户
    fn validate_accounts(&mut self) -> Result<()> {
        let start = std::time::Instant::now();

        let passed = !self.config.test_accounts.is_empty();

        self.results.push(ValidationResult {
            name: "账户验证".to_string(),
            passed,
            message: format!("测试账户数: {}", self.config.test_accounts.len()),
            duration_ms: start.elapsed().as_millis() as u64,
        });

        Ok(())
    }

    /// 验证共识参数
    fn validate_consensus(&mut self) -> Result<()> {
        let start = std::time::Instant::now();

        let passed = self.config.block_time >= 3
            && self.config.block_time <= 60
            && self.config.initial_difficulty > 0;

        self.results.push(ValidationResult {
            name: "共识参数验证".to_string(),
            passed,
            message: format!(
                "出块时间: {}秒, 初始难度: {}",
                self.config.block_time, self.config.initial_difficulty
            ),
            duration_ms: start.elapsed().as_millis() as u64,
        });

        Ok(())
    }

    /// 生成报告
    fn generate_report(&self) -> ValidationReport {
        let passed = self.results.iter().filter(|r| r.passed).count();
        let failed = self.results.len() - passed;

        ValidationReport {
            total: self.results.len(),
            passed,
            failed,
            results: self.results.clone(),
            all_passed: failed == 0,
        }
    }
}

/// 验证报告
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// 总检查项
    pub total: usize,
    /// 通过数
    pub passed: usize,
    /// 失败数
    pub failed: usize,
    /// 详细结果
    pub results: Vec<ValidationResult>,
    /// 是否全部通过
    pub all_passed: bool,
}

/// 测试网部署器
pub struct TestnetDeployer {
    /// 配置
    config: TestnetConfig,
}

impl TestnetDeployer {
    /// 创建新的部署器
    pub fn new(config: TestnetConfig) -> Self {
        TestnetDeployer { config }
    }

    /// 部署测试网
    pub fn deploy(&self) -> Result<DeploymentReport> {
        info!("部署测试网: {}", self.config.name);

        let mut report = DeploymentReport::default();

        // 1. 创建目录结构
        self.create_directories()?;
        report.directories_created = true;

        // 2. 生成配置文件
        self.generate_configs()?;
        report.configs_generated = true;

        // 3. 初始化数据库
        self.init_database()?;
        report.database_initialized = true;

        // 4. 创建创世区块
        self.create_genesis()?;
        report.genesis_created = true;

        // 5. 初始化测试账户
        self.init_test_accounts()?;
        report.accounts_initialized = true;

        report.success = true;

        info!("测试网部署完成");
        Ok(report)
    }

    fn create_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.config.data_dir)?;
        fs::create_dir_all(self.config.data_dir.join("blocks"))?;
        fs::create_dir_all(self.config.data_dir.join("transactions"))?;
        fs::create_dir_all(self.config.data_dir.join("logs"))?;
        Ok(())
    }

    fn generate_configs(&self) -> Result<()> {
        let config_content = format!(
            r#"
# {} 配置
chain_id = {}
data_dir = {:?}

[network]
port = {}
seed_nodes = {:?}

[consensus]
block_time = {}
initial_difficulty = {}
enable_mining = {}

[api]
port = {}
enable = {}
"#,
            self.config.name,
            self.config.chain_id,
            self.config.data_dir,
            self.config.network_port,
            self.config.seed_nodes,
            self.config.block_time,
            self.config.initial_difficulty,
            self.config.enable_mining,
            self.config.api_port,
            self.config.enable_api,
        );

        fs::write(self.config.data_dir.join("config.toml"), config_content)?;
        Ok(())
    }

    fn init_database(&self) -> Result<()> {
        let db_path = self.config.data_dir.join("db");
        fs::create_dir_all(db_path)?;
        Ok(())
    }

    fn create_genesis(&self) -> Result<()> {
        let genesis = serde_json::json!({
            "chain_id": self.config.chain_id,
            "name": &self.config.name,
            "timestamp": 0,
            "height": 0,
            "transactions": [],
            "accounts": &self.config.test_accounts,
        });

        fs::write(
            self.config.data_dir.join("genesis.json"),
            serde_json::to_string_pretty(&genesis)?,
        )?;
        Ok(())
    }

    fn init_test_accounts(&self) -> Result<()> {
        let accounts_path = self.config.data_dir.join("accounts.json");
        fs::write(
            accounts_path,
            serde_json::to_string_pretty(&self.config.test_accounts)?,
        )?;
        Ok(())
    }
}

/// 部署报告
#[derive(Debug, Default)]
pub struct DeploymentReport {
    pub success: bool,
    pub directories_created: bool,
    pub configs_generated: bool,
    pub database_initialized: bool,
    pub genesis_created: bool,
    pub accounts_initialized: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_testnet_config_default() {
        let config = TestnetConfig::default();
        assert_eq!(config.chain_id, 1000);
        assert_eq!(config.block_time, 5);
    }

    #[test]
    fn test_validator() {
        let config = TestnetConfig::default();
        let mut validator = TestnetValidator::new(config);
        let report = validator.validate_all().unwrap();

        assert!(report.all_passed);
    }
}
