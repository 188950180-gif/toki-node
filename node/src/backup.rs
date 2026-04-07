//! 数据备份和恢复机制
//!
//! 提供自动备份和手动备份功能

use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use tracing::{info, warn};

/// 备份配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackupConfig {
    /// 备份目录
    pub backup_dir: PathBuf,
    /// 备份间隔（小时）
    pub interval_hours: u64,
    /// 保留备份数量
    pub keep_count: usize,
    /// 是否压缩
    pub compress: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        BackupConfig {
            backup_dir: PathBuf::from("./backups"),
            interval_hours: 24,
            keep_count: 7,
            compress: true,
        }
    }
}

/// 备份元数据
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// 备份时间
    pub timestamp: DateTime<Utc>,
    /// 备份文件名
    pub filename: String,
    /// 数据目录大小（字节）
    pub size: u64,
    /// 区块高度
    pub block_height: u64,
    /// 是否压缩
    pub compressed: bool,
}

/// 备份管理器
pub struct BackupManager {
    /// 配置
    config: BackupConfig,
    /// 数据目录
    data_dir: PathBuf,
}

impl BackupManager {
    /// 创建新的备份管理器
    pub fn new(config: BackupConfig, data_dir: PathBuf) -> Self {
        BackupManager { config, data_dir }
    }

    /// 创建备份
    pub fn create_backup(&self, block_height: u64) -> Result<BackupMetadata> {
        info!("开始创建备份...");

        // 创建备份目录
        fs::create_dir_all(&self.config.backup_dir)
            .context("Failed to create backup directory")?;

        // 生成备份文件名
        let timestamp = Utc::now();
        let filename = format!(
            "backup-{}-{}.tar{}",
            timestamp.format("%Y%m%d-%H%M%S"),
            block_height,
            if self.config.compress { ".gz" } else { "" }
        );

        let backup_path = self.config.backup_dir.join(&filename);

        // 计算数据目录大小
        let size = self.calculate_dir_size(&self.data_dir)?;

        // 创建备份
        if self.config.compress {
            self.create_compressed_backup(&backup_path)?;
        } else {
            self.create_uncompressed_backup(&backup_path)?;
        }

        info!("备份创建成功: {}", filename);

        // 创建元数据
        let metadata = BackupMetadata {
            timestamp,
            filename,
            size,
            block_height,
            compressed: self.config.compress,
        };

        // 保存元数据
        self.save_metadata(&metadata)?;

        // 清理旧备份
        self.cleanup_old_backups()?;

        Ok(metadata)
    }

    /// 创建压缩备份
    fn create_compressed_backup(&self, backup_path: &Path) -> Result<()> {
        use std::process::Command;

        let output = Command::new("tar")
            .arg("-czf")
            .arg(backup_path)
            .arg("-C")
            .arg(&self.data_dir)
            .arg(".")
            .output()
            .context("Failed to create compressed backup")?;

        if !output.status.success() {
            warn!("备份命令失败: {}", String::from_utf8_lossy(&output.stderr));
            return Err(anyhow::anyhow!("Backup command failed"));
        }

        Ok(())
    }

    /// 创建未压缩备份
    fn create_uncompressed_backup(&self, backup_path: &Path) -> Result<()> {
        use std::process::Command;

        let output = Command::new("tar")
            .arg("-cf")
            .arg(backup_path)
            .arg("-C")
            .arg(&self.data_dir)
            .arg(".")
            .output()
            .context("Failed to create uncompressed backup")?;

        if !output.status.success() {
            warn!("备份命令失败: {}", String::from_utf8_lossy(&output.stderr));
            return Err(anyhow::anyhow!("Backup command failed"));
        }

        Ok(())
    }

    /// 恢复备份
    pub fn restore_backup(&self, filename: &str) -> Result<()> {
        info!("开始恢复备份: {}", filename);

        let backup_path = self.config.backup_dir.join(filename);

        if !backup_path.exists() {
            return Err(anyhow::anyhow!("Backup file not found: {}", filename));
        }

        // 清空数据目录
        if self.data_dir.exists() {
            fs::remove_dir_all(&self.data_dir)
                .context("Failed to clear data directory")?;
        }

        fs::create_dir_all(&self.data_dir)
            .context("Failed to create data directory")?;

        // 恢复备份
        if filename.ends_with(".gz") {
            self.restore_compressed_backup(&backup_path)?;
        } else {
            self.restore_uncompressed_backup(&backup_path)?;
        }

        info!("备份恢复成功");

        Ok(())
    }

    /// 恢复压缩备份
    fn restore_compressed_backup(&self, backup_path: &Path) -> Result<()> {
        use std::process::Command;

        let output = Command::new("tar")
            .arg("-xzf")
            .arg(backup_path)
            .arg("-C")
            .arg(&self.data_dir)
            .output()
            .context("Failed to restore compressed backup")?;

        if !output.status.success() {
            warn!("恢复命令失败: {}", String::from_utf8_lossy(&output.stderr));
            return Err(anyhow::anyhow!("Restore command failed"));
        }

        Ok(())
    }

    /// 恢复未压缩备份
    fn restore_uncompressed_backup(&self, backup_path: &Path) -> Result<()> {
        use std::process::Command;

        let output = Command::new("tar")
            .arg("-xf")
            .arg(backup_path)
            .arg("-C")
            .arg(&self.data_dir)
            .output()
            .context("Failed to restore uncompressed backup")?;

        if !output.status.success() {
            warn!("恢复命令失败: {}", String::from_utf8_lossy(&output.stderr));
            return Err(anyhow::anyhow!("Restore command failed"));
        }

        Ok(())
    }

    /// 列出所有备份
    pub fn list_backups(&self) -> Result<Vec<BackupMetadata>> {
        let mut backups = Vec::new();

        for entry in fs::read_dir(&self.config.backup_dir)
            .context("Failed to read backup directory")?
        {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(metadata) = serde_json::from_str::<BackupMetadata>(&content) {
                        backups.push(metadata);
                    }
                }
            }
        }

        // 按时间排序
        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(backups)
    }

    /// 清理旧备份
    fn cleanup_old_backups(&self) -> Result<()> {
        let backups = self.list_backups()?;

        if backups.len() > self.config.keep_count {
            for backup in backups.iter().skip(self.config.keep_count) {
                let backup_path = self.config.backup_dir.join(&backup.filename);
                let metadata_path = self.config.backup_dir
                    .join(format!("{}.json", backup.filename));

                fs::remove_file(&backup_path).ok();
                fs::remove_file(&metadata_path).ok();

                info!("删除旧备份: {}", backup.filename);
            }
        }

        Ok(())
    }

    /// 保存元数据
    fn save_metadata(&self, metadata: &BackupMetadata) -> Result<()> {
        let metadata_path = self.config.backup_dir
            .join(format!("{}.json", metadata.filename));

        let content = serde_json::to_string_pretty(metadata)
            .context("Failed to serialize metadata")?;

        fs::write(&metadata_path, content)
            .context("Failed to write metadata")?;

        Ok(())
    }

    /// 计算目录大小
    fn calculate_dir_size(&self, path: &Path) -> Result<u64> {
        let mut total_size = 0;

        for entry in fs::read_dir(path).context("Failed to read directory")? {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if metadata.is_dir() {
                total_size += self.calculate_dir_size(&entry.path())?;
            } else {
                total_size += metadata.len();
            }
        }

        Ok(total_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_backup_creation() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("data");
        let backup_dir = temp_dir.path().join("backups");

        fs::create_dir_all(&data_dir).unwrap();
        fs::write(data_dir.join("test.txt"), "test data").unwrap();

        let config = BackupConfig {
            backup_dir,
            keep_count: 5,
            compress: false,
            ..Default::default()
        };

        let manager = BackupManager::new(config, data_dir.clone());
        let metadata = manager.create_backup(100).unwrap();

        assert!(metadata.block_height == 100);
        assert!(!metadata.compressed);
    }
}
