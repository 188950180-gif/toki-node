//! Sled 数据库封装（替代 RocksDB）

use crate::StorageError;
use parking_lot::RwLock;
use sled::Db;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};

/// 数据库封装
pub struct Database {
    /// Sled 实例
    db: Arc<Db>,
    /// 写锁（用于批量写入）
    write_lock: RwLock<()>,
}

impl Database {
    /// 打开或创建数据库
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let path = path.as_ref();
        info!("Opening database at {:?}", path);

        // 确保目录存在
        std::fs::create_dir_all(path)?;

        // 打开数据库
        let db = sled::open(path)?;

        Ok(Database {
            db: Arc::new(db),
            write_lock: RwLock::new(()),
        })
    }

    /// 获取值
    pub fn get(&self, cf: &str, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let tree = self.db.open_tree(cf)?;
        let value = tree.get(key)?;
        Ok(value.map(|v| v.to_vec()))
    }

    /// 设置值
    pub fn put(&self, cf: &str, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        let _lock = self.write_lock.read();
        let tree = self.db.open_tree(cf)?;
        tree.insert(key, value)?;
        Ok(())
    }

    /// 删除值
    pub fn delete(&self, cf: &str, key: &[u8]) -> Result<(), StorageError> {
        let _lock = self.write_lock.read();
        let tree = self.db.open_tree(cf)?;
        tree.remove(key)?;
        Ok(())
    }

    /// 检查键是否存在
    pub fn exists(&self, cf: &str, key: &[u8]) -> Result<bool, StorageError> {
        let tree = self.db.open_tree(cf)?;
        let value = tree.get(key)?;
        Ok(value.is_some())
    }

    /// 批量写入
    pub fn write_batch(&self, writes: Vec<WriteOp>) -> Result<(), StorageError> {
        let _lock = self.write_lock.write();

        for op in writes {
            let tree = self.db.open_tree(&op.cf)?;
            match op.op_type {
                WriteType::Put => {
                    tree.insert(&op.key, op.value.as_slice())?;
                }
                WriteType::Delete => {
                    tree.remove(&op.key)?;
                }
            }
        }

        self.db.flush()?;
        Ok(())
    }

    /// 获取迭代器
    pub fn iterator(&self, cf: &str, mode: IteratorMode) -> Result<DBIterator, StorageError> {
        let tree = self.db.open_tree(cf)?;
        let iter: sled::Iter = match mode {
            IteratorMode::Start => tree.iter(),
            IteratorMode::End => tree.iter(),
            IteratorMode::From(key, _dir) => tree.range(key..),
        };
        Ok(DBIterator(iter))
    }

    /// 获取前缀迭代器
    pub fn prefix_iterator(&self, cf: &str, prefix: &[u8]) -> Result<DBIterator, StorageError> {
        let tree = self.db.open_tree(cf)?;
        let iter = tree.scan_prefix(prefix);
        Ok(DBIterator(iter))
    }

    /// 获取内部 DB 引用
    pub fn inner(&self) -> &Db {
        &self.db
    }

    /// 刷新数据到磁盘
    pub fn flush(&self) -> Result<(), StorageError> {
        self.db.flush()?;
        Ok(())
    }

    /// 压缩数据库
    pub fn compact(&self) -> Result<(), StorageError> {
        // sled 的 compact 在 Db 上
        (*self.db).flush()?;
        Ok(())
    }

    /// 创建快照
    pub fn snapshot(&self) -> Result<Snapshot, StorageError> {
        Ok(Snapshot {
            db: Arc::clone(&self.db),
        })
    }

    /// 从快照恢复
    pub fn restore_from_snapshot(&self, _snapshot: &Snapshot) -> Result<(), StorageError> {
        // Sled 不支持快照恢复，需要手动实现
        warn!("Sled 不支持快照恢复，需要手动实现");
        Ok(())
    }
}

/// 数据库快照
pub struct Snapshot {
    db: Arc<Db>,
}

impl Snapshot {
    /// 获取值
    pub fn get(&self, cf: &str, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let tree = self.db.open_tree(cf)?;
        let value = tree.get(key)?;
        Ok(value.map(|v| v.to_vec()))
    }

    /// 获取迭代器
    pub fn iterator(&self, cf: &str) -> Result<DBIterator, StorageError> {
        let tree = self.db.open_tree(cf)?;
        Ok(DBIterator(tree.iter()))
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Database {
            db: Arc::clone(&self.db),
            write_lock: RwLock::new(()),
        }
    }
}

/// 写操作类型
#[derive(Clone, Debug)]
pub enum WriteType {
    Put,
    Delete,
}

/// 写操作
#[derive(Clone, Debug)]
pub struct WriteOp {
    pub cf: String,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub op_type: WriteType,
}

impl WriteOp {
    /// 创建 Put 操作
    pub fn put(cf: &str, key: Vec<u8>, value: Vec<u8>) -> Self {
        WriteOp {
            cf: cf.to_string(),
            key,
            value,
            op_type: WriteType::Put,
        }
    }

    /// 创建 Delete 操作
    pub fn delete(cf: &str, key: Vec<u8>) -> Self {
        WriteOp {
            cf: cf.to_string(),
            key,
            value: Vec::new(),
            op_type: WriteType::Delete,
        }
    }
}

/// 迭代器模式
#[derive(Clone, Debug)]
pub enum IteratorMode {
    Start,
    End,
    From(Vec<u8>, bool), // bool: true = forward, false = reverse
}

/// 数据库迭代器包装
pub struct DBIterator(sled::Iter);

impl DBIterator {
    /// 转换为迭代器
    pub fn into_iter(self) -> impl Iterator<Item = (Box<[u8]>, Box<[u8]>)> {
        self.0.filter_map(|r| r.ok()).map(|(k, v)| {
            let k: Box<[u8]> = k.to_vec().into_boxed_slice();
            let v: Box<[u8]> = v.to_vec().into_boxed_slice();
            (k, v)
        })
    }
}

// 列族常量（sled 使用 tree，概念类似）
pub const CF_BLOCKS: &str = "blocks";
pub const CF_TRANSACTIONS: &str = "transactions";
pub const CF_ACCOUNTS: &str = "accounts";
pub const CF_UTXO: &str = "utxo";
pub const CF_KEY_IMAGES: &str = "key_images";
pub const CF_PROPOSALS: &str = "proposals";
pub const CF_EXCHANGES: &str = "exchanges";
pub const CF_PARAMS: &str = "params";
pub const CF_METADATA: &str = "metadata";

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_database_open() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();

        // 测试基本读写
        db.put(CF_METADATA, b"test_key", b"test_value").unwrap();
        let value = db.get(CF_METADATA, b"test_key").unwrap();
        assert_eq!(value, Some(b"test_value".to_vec()));

        // 测试存在性检查
        assert!(db.exists(CF_METADATA, b"test_key").unwrap());
        assert!(!db.exists(CF_METADATA, b"nonexistent").unwrap());

        // 测试删除
        db.delete(CF_METADATA, b"test_key").unwrap();
        let value = db.get(CF_METADATA, b"test_key").unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn test_write_batch() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();

        let writes = vec![
            WriteOp::put(CF_METADATA, b"key1".to_vec(), b"value1".to_vec()),
            WriteOp::put(CF_METADATA, b"key2".to_vec(), b"value2".to_vec()),
            WriteOp::delete(CF_METADATA, b"key1".to_vec()),
        ];

        db.write_batch(writes).unwrap();

        assert!(!db.exists(CF_METADATA, b"key1").unwrap());
        assert!(db.exists(CF_METADATA, b"key2").unwrap());
    }
}
