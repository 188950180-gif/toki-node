//! 自动升级模块
//! 
//! 实现区块链自动升级、版本管理、热更新

use std::path::{Path, PathBuf};
use std::fs;
use std::time::Instant;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use async_trait::async_trait;
use tracing::{info, warn, error};

/// 版本号
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version {
    /// 主版本号
    pub major: u32,
    /// 次版本号
    pub minor: u32,
    /// 修订号
    pub patch: u32,
}

impl Version {
    /// 创建新版本
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Version { major, minor, patch }
    }

    /// 从字符串解析
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<_> = s.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        Some(Version {
            major: parts[0].parse().ok()?,
            minor: parts[1].parse().ok()?,
            patch: parts[2].parse().ok()?,
        })
    }

    /// 转换为字符串
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    /// 检查是否兼容
    pub fn is_compatible(&self, other: &Version) -> bool {
        // 主版本号相同才兼容
        self.major == other.major
    }

    /// 检查是否需要升级
    pub fn needs_upgrade(&self, latest: &Version) -> bool {
        self < latest
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Default for Version {
    fn default() -> Self {
        Version::new(0, 1, 0)
    }
}

/// 升级信息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpgradeInfo {
    /// 版本
    pub version: Version,
    /// 发布时间
    pub release_time: u64,
    /// 下载地址
    pub download_url: String,
    /// 校验和
    pub checksum: String,
    /// 更新说明
    pub changelog: String,
    /// 是否为强制更新
    pub mandatory: bool,
    /// 最低兼容版本
    pub min_compatible: Version,
}

/// 升级配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpgradeConfig {
    /// 当前版本
    pub current_version: Version,
    /// 检查更新间隔（秒）
    pub check_interval: u64,
    /// 是否启用自动升级
    pub auto_upgrade: bool,
    /// 是否启用热更新
    pub hot_reload: bool,
    /// 备份当前版本
    pub backup_before_upgrade: bool,
    /// 升级服务器地址
    pub upgrade_server: String,
    /// 下载目录
    pub download_dir: PathBuf,
    /// 最大回滚次数
    pub max_rollbacks: usize,
}

impl Default for UpgradeConfig {
    fn default() -> Self {
        UpgradeConfig {
            current_version: Version::new(0, 1, 0),
            check_interval: 3600,
            auto_upgrade: true,
            hot_reload: true,
            backup_before_upgrade: true,
            upgrade_server: "https://upgrade.toki.network".to_string(),
            download_dir: PathBuf::from("./upgrades"),
            max_rollbacks: 5,
        }
    }
}

/// 版本检查器
#[async_trait]
pub trait VersionChecker: Send + Sync {
    /// 检查最新版本
    async fn check_latest(&self) -> Result<Option<UpgradeInfo>>;
    
    /// 下载升级包
    async fn download(&self, info: &UpgradeInfo) -> Result<PathBuf>;
    
    /// 验证升级包
    async fn verify(&self, path: &Path, info: &UpgradeInfo) -> Result<bool>;
}

/// 升级状态
#[derive(Debug, Clone, PartialEq)]
pub enum UpgradeState {
    /// 空闲
    Idle,
    /// 检查更新中
    Checking,
    /// 下载中
    Downloading,
    /// 验证中
    Verifying,
    /// 安装中
    Installing,
    /// 回滚中
    RollingBack,
    /// 需要重启
    NeedRestart,
}

/// 自动升级器
pub struct AutoUpgrader {
    /// 配置
    config: UpgradeConfig,
    /// 版本检查器
    checker: Option<Box<dyn VersionChecker>>,
    /// 当前状态
    state: UpgradeState,
    /// 升级历史
    upgrade_history: Vec<UpgradeRecord>,
    /// 上次检查时间
    last_check: Option<Instant>,
    /// 待安装的升级包
    pending_upgrade: Option<PathBuf>,
}

/// 升级记录
#[derive(Debug, Clone)]
pub struct UpgradeRecord {
    /// 从版本
    pub from_version: Version,
    /// 到版本
    pub to_version: Version,
    /// 升级时间
    pub timestamp: u64,
    /// 是否成功
    pub success: bool,
    /// 备注
    pub note: String,
}

impl AutoUpgrader {
    /// 创建新的自动升级器
    pub fn new(config: UpgradeConfig) -> Self {
        AutoUpgrader {
            config,
            checker: None,
            state: UpgradeState::Idle,
            upgrade_history: Vec::new(),
            last_check: None,
            pending_upgrade: None,
        }
    }

    /// 设置版本检查器
    pub fn set_checker(&mut self, checker: Box<dyn VersionChecker>) {
        self.checker = Some(checker);
    }

    /// 获取当前版本
    pub fn current_version(&self) -> &Version {
        &self.config.current_version
    }

    /// 获取当前状态
    pub fn state(&self) -> &UpgradeState {
        &self.state
    }

    /// 检查更新
    pub async fn check_for_updates(&mut self) -> Result<Option<UpgradeInfo>> {
        if self.state != UpgradeState::Idle {
            return Ok(None);
        }
        
        self.state = UpgradeState::Checking;
        info!("检查更新...");
        
        let result = if let Some(ref checker) = self.checker {
            checker.check_latest().await
        } else {
            Ok(None)
        };
        
        self.state = UpgradeState::Idle;
        self.last_check = Some(Instant::now());
        
        result
    }

    /// 执行升级
    pub async fn upgrade(&mut self, info: &UpgradeInfo) -> Result<bool> {
        if self.state != UpgradeState::Idle {
            return Ok(false);
        }
        
        // 检查是否需要升级
        if !self.config.current_version.needs_upgrade(&info.version) {
            info!("已是最新版本，无需升级");
            return Ok(true);
        }
        
        // 检查兼容性
        if !self.config.current_version.is_compatible(&info.min_compatible) {
            error!("版本不兼容，无法升级");
            return Ok(false);
        }
        
        info!("开始升级: {} -> {}", self.config.current_version, info.version);
        
        // 备份当前版本
        if self.config.backup_before_upgrade {
            self.backup_current_version()?;
        }
        
        // 下载升级包
        self.state = UpgradeState::Downloading;
        let package_path = if let Some(ref checker) = self.checker {
            checker.download(info).await?
        } else {
            return Err(anyhow::anyhow!("未设置版本检查器"));
        };
        
        // 验证升级包
        self.state = UpgradeState::Verifying;
        let valid = if let Some(ref checker) = self.checker {
            checker.verify(&package_path, info).await?
        } else {
            false
        };
        
        if !valid {
            error!("升级包验证失败");
            self.state = UpgradeState::Idle;
            return Ok(false);
        }
        
        // 安装升级
        self.state = UpgradeState::Installing;
        let success = self.install_upgrade(&package_path, info).await?;
        
        // 记录升级历史
        let record = UpgradeRecord {
            from_version: self.config.current_version.clone(),
            to_version: info.version.clone(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            success,
            note: if success { "升级成功".to_string() } else { "升级失败".to_string() },
        };
        self.upgrade_history.push(record);
        
        if success {
            self.config.current_version = info.version.clone();
            self.state = UpgradeState::NeedRestart;
            info!("升级完成，需要重启");
        } else {
            self.state = UpgradeState::Idle;
            // 回滚
            self.rollback().await?;
        }
        
        Ok(success)
    }

    /// 备份当前版本
    fn backup_current_version(&self) -> Result<()> {
        let backup_dir = self.config.download_dir.join("backup");
        fs::create_dir_all(&backup_dir)?;
        
        let version_str = self.config.current_version.to_string();
        let backup_path = backup_dir.join(format!("v{}", version_str));
        
        // 备份可执行文件
        let exe_path = std::env::current_exe()?;
        if exe_path.exists() {
            fs::copy(&exe_path, backup_path.join("toki-node"))?;
        }
        
        info!("已备份当前版本: v{}", version_str);
        Ok(())
    }

    /// 安装升级
    async fn install_upgrade(&mut self, package_path: &Path, info: &UpgradeInfo) -> Result<bool> {
        info!("安装升级包: {:?}", package_path);
        
        // 解压升级包
        let extract_dir = self.config.download_dir.join("extracted");
        if extract_dir.exists() {
            fs::remove_dir_all(&extract_dir)?;
        }
        fs::create_dir_all(&extract_dir)?;
        
        // TODO: 实现解压逻辑
        // 这里简化处理，假设升级包已解压
        
        // 替换可执行文件
        let new_exe = extract_dir.join("toki-node");
        if new_exe.exists() {
            let exe_path = std::env::current_exe()?;
            fs::copy(&new_exe, &exe_path)?;
            info!("已替换可执行文件");
        }
        
        // 更新配置文件
        self.update_configs(info)?;
        
        Ok(true)
    }

    /// 更新配置文件
    fn update_configs(&self, info: &UpgradeInfo) -> Result<()> {
        // 更新版本配置
        let version_file = self.config.download_dir.join("version.txt");
        fs::write(&version_file, info.version.to_string())?;
        
        Ok(())
    }

    /// 回滚到上一版本
    pub async fn rollback(&mut self) -> Result<bool> {
        if self.state != UpgradeState::Idle && self.state != UpgradeState::NeedRestart {
            return Ok(false);
        }
        
        self.state = UpgradeState::RollingBack;
        info!("开始回滚...");
        
        let backup_dir = self.config.download_dir.join("backup");
        if !backup_dir.exists() {
            warn!("没有可用的备份");
            self.state = UpgradeState::Idle;
            return Ok(false);
        }
        
        // 找到最新的备份
        let mut backups: Vec<_> = fs::read_dir(&backup_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("v"))
            .collect();
        
        backups.sort_by_key(|e| e.file_name());
        
        if let Some(latest) = backups.last() {
            let backup_path = latest.path();
            let backup_exe = backup_path.join("toki-node");
            
            if backup_exe.exists() {
                let exe_path = std::env::current_exe()?;
                fs::copy(&backup_exe, &exe_path)?;
                
                // 从备份目录名解析版本
                let version_str = latest.file_name().to_string_lossy().to_string();
                if version_str.len() > 1 {
                    if let Some(version) = Version::parse(&version_str[1..]) {
                        self.config.current_version = version;
                    }
                }
                
                info!("回滚完成");
                self.state = UpgradeState::NeedRestart;
                return Ok(true);
            }
        }
        
        self.state = UpgradeState::Idle;
        Ok(false)
    }

    /// 热更新模块
    pub async fn hot_reload(&mut self, module: &str) -> Result<bool> {
        if !self.config.hot_reload {
            return Ok(false);
        }
        
        info!("热更新模块: {}", module);
        
        // TODO: 实现热更新逻辑
        // 1. 下载新模块
        // 2. 验证模块
        // 3. 替换模块
        // 4. 重新加载
        
        Ok(true)
    }

    /// 获取升级历史
    pub fn get_history(&self) -> &[UpgradeRecord] {
        &self.upgrade_history
    }

    /// 是否需要重启
    pub fn needs_restart(&self) -> bool {
        self.state == UpgradeState::NeedRestart
    }

    /// 确认重启完成
    pub fn confirm_restart(&mut self) {
        if self.state == UpgradeState::NeedRestart {
            self.state = UpgradeState::Idle;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parse() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_version_compare() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(1, 1, 0);
        let v3 = Version::new(2, 0, 0);
        
        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1.is_compatible(&v2));
        assert!(!v1.is_compatible(&v3));
    }

    #[test]
    fn test_version_needs_upgrade() {
        let current = Version::new(1, 0, 0);
        let latest = Version::new(1, 1, 0);
        
        assert!(current.needs_upgrade(&latest));
        assert!(!latest.needs_upgrade(&current));
    }

    #[test]
    fn test_auto_upgrader_creation() {
        let upgrader = AutoUpgrader::new(UpgradeConfig::default());
        assert_eq!(upgrader.state(), &UpgradeState::Idle);
        assert_eq!(upgrader.current_version(), &Version::new(0, 1, 0));
    }
}
