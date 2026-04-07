//! 备份管理

use crate::StorageError;
use chrono::{DateTime, Local, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// 备份管理器
pub struct BackupManager {
    /// 备份目录
    backup_dir: PathBuf,
    /// 保留天数
    retention_days: u64,
}

impl BackupManager {
    /// 创建新备份管理器
    pub fn new<P: AsRef<Path>>(backup_dir: P, retention_days: u64) -> Self {
        BackupManager {
            backup_dir: backup_dir.as_ref().to_path_buf(),
            retention_days,
        }
    }

    /// 创建备份
    pub fn create_backup<P: AsRef<Path>>(&self, db_path: P) -> Result<PathBuf, StorageError> {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("backup_{}", timestamp);
        let backup_path = self.backup_dir.join(&backup_name);

        // 确保备份目录存在
        fs::create_dir_all(&self.backup_dir)?;

        info!("Creating backup at {:?}", backup_path);

        // 复制数据库目录
        self.copy_dir_all(db_path.as_ref(), &backup_path)?;

        // 记录备份元数据
        let metadata = BackupMetadata {
            timestamp: Utc::now(),
            path: backup_path.clone(),
        };
        let metadata_path = backup_path.join("backup_metadata.json");
        let metadata_json = serde_json::to_string_pretty(&metadata)?;
        fs::write(&metadata_path, metadata_json)?;

        info!("Backup created successfully");
        Ok(backup_path)
    }

    /// 恢复备份
    pub fn restore_backup<P: AsRef<Path>>(
        &self,
        backup_path: P,
        db_path: P,
    ) -> Result<(), StorageError> {
        info!("Restoring backup from {:?}", backup_path.as_ref());

        // 删除现有数据库
        if db_path.as_ref().exists() {
            fs::remove_dir_all(db_path.as_ref())?;
        }

        // 复制备份到数据库路径
        self.copy_dir_all(backup_path.as_ref(), db_path.as_ref())?;

        info!("Backup restored successfully");
        Ok(())
    }

    /// 清理过期备份
    pub fn cleanup_old_backups(&self) -> Result<Vec<PathBuf>, StorageError> {
        let mut removed = Vec::new();
        let now = Utc::now();

        // 遍历备份目录
        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            // 读取元数据
            let metadata_path = path.join("backup_metadata.json");
            if metadata_path.exists() {
                let metadata_json = fs::read_to_string(&metadata_path)?;
                if let Ok(metadata) = serde_json::from_str::<BackupMetadata>(&metadata_json) {
                    let age_days = (now - metadata.timestamp).num_days();
                    if age_days > self.retention_days as i64 {
                        info!("Removing old backup: {:?}", path);
                        fs::remove_dir_all(&path)?;
                        removed.push(path);
                    }
                }
            } else {
                // 没有元数据，尝试从目录名解析时间
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("backup_") {
                        warn!("Backup without metadata: {:?}", path);
                    }
                }
            }
        }

        Ok(removed)
    }

    /// 列出所有备份
    pub fn list_backups(&self) -> Result<Vec<BackupMetadata>, StorageError> {
        let mut backups = Vec::new();

        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let metadata_path = path.join("backup_metadata.json");
            if metadata_path.exists() {
                let metadata_json = fs::read_to_string(&metadata_path)?;
                if let Ok(metadata) = serde_json::from_str::<BackupMetadata>(&metadata_json) {
                    backups.push(metadata);
                }
            }
        }

        // 按时间排序（最新的在前）
        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(backups)
    }

    /// 获取最新备份
    pub fn get_latest_backup(&self) -> Result<Option<BackupMetadata>, StorageError> {
        let backups = self.list_backups()?;
        Ok(backups.into_iter().next())
    }

    /// 递归复制目录
    fn copy_dir_all<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        src: P,
        dst: Q,
    ) -> Result<(), StorageError> {
        let src = src.as_ref();
        let dst = dst.as_ref();

        fs::create_dir_all(dst)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if ty.is_dir() {
                self.copy_dir_all(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }

        Ok(())
    }
}

/// 备份元数据
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BackupMetadata {
    /// 备份时间
    pub timestamp: DateTime<Utc>,
    /// 备份路径
    pub path: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_backup_manager() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let db_dir = temp_dir.path().join("db");

        // 创建测试数据库目录
        fs::create_dir_all(&db_dir).unwrap();
        fs::write(db_dir.join("test.txt"), "test data").unwrap();

        let manager = BackupManager::new(&backup_dir, 7);

        // 创建备份
        let backup_path = manager.create_backup(&db_dir).unwrap();
        assert!(backup_path.exists());

        // 列出备份
        let backups = manager.list_backups().unwrap();
        assert_eq!(backups.len(), 1);

        // 获取最新备份
        let latest = manager.get_latest_backup().unwrap();
        assert!(latest.is_some());
    }
}
