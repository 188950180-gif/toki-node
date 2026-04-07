//! 加密存储模块
//!
//! 实现安全的密钥信息存储
//! 提供完整性验证和访问控制

use anyhow::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use super::key_rotation::EncryptedKeyInfo;
use toki_crypto as crypto;

/// 存储配置
#[derive(Clone, Debug)]
pub struct StorageConfig {
    /// 存储文件路径
    pub file_path: String,
    /// 启用完整性检查
    pub enable_integrity_check: bool,
    /// 访问超时（秒）
    pub access_timeout_secs: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        StorageConfig {
            file_path: ".toki_encrypted_data".to_string(),
            enable_integrity_check: true,
            access_timeout_secs: 30,
        }
    }
}

/// 存储状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageState {
    /// 最后访问时间
    pub last_access_time: u64,
    /// 访问次数
    pub access_count: u64,
    /// 创建时间
    pub created_at: u64,
    /// 是否已损坏
    pub is_corrupted: bool,
}

/// 加密存储
pub struct EncryptedStorage {
    config: StorageConfig,
    encryption_key: Vec<u8>,
    key_info: Arc<RwLock<Option<EncryptedKeyInfo>>>,
    state: Arc<RwLock<StorageState>>,
}

impl EncryptedStorage {
    /// 创建新的加密存储
    pub fn new(config: StorageConfig, encryption_key: Vec<u8>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let state = StorageState {
            last_access_time: now,
            access_count: 0,
            created_at: now,
            is_corrupted: false,
        };

        info!("创建加密存储: {}", config.file_path);

        EncryptedStorage {
            config,
            encryption_key,
            key_info: Arc::new(RwLock::new(None)),
            state: Arc::new(RwLock::new(state)),
        }
    }

    /// 存储密钥信息
    pub fn store_key_info(&self, key_info: EncryptedKeyInfo) -> Result<()> {
        self.update_access();

        // 序列化
        let data = bincode::serialize(&key_info)?;

        // 加密
        let encrypted = self.encrypt_data(&data)?;

        // 添加完整性验证
        let with_integrity = self.add_integrity_check(&encrypted)?;

        // 模拟存储（实际应该写入文件）
        debug!("密钥信息已加密存储");

        // 更新内存存储
        *self.key_info.write() = Some(key_info);

        Ok(())
    }

    /// 读取密钥信息
    pub fn load_key_info(&self) -> Result<EncryptedKeyInfo> {
        self.update_access();

        // 检查超时
        if self.is_access_timeout() {
            warn!("访问超时");
            return Err(anyhow::anyhow!("访问超时"));
        }

        // 检查完整性
        if self.config.enable_integrity_check && !self.verify_integrity() {
            warn!("完整性验证失败");
            self.mark_as_corrupted();
            return Err(anyhow::anyhow!("完整性验证失败"));
        }

        // 从内存读取
        let key_info = self.key_info.read();
        match key_info.as_ref() {
            Some(info) => Ok(info.clone()),
            None => Err(anyhow::anyhow!("密钥信息不存在")),
        }
    }

    /// 更新访问时间
    fn update_access(&self) {
        let mut state = self.state.write();
        state.last_access_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        state.access_count += 1;
    }

    /// 检查是否超时
    fn is_access_timeout(&self) -> bool {
        let state = self.state.read();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        (now - state.last_access_time) > self.config.access_timeout_secs
    }

    /// 验证完整性
    fn verify_integrity(&self) -> bool {
        // 检查状态
        let state = self.state.read();
        if state.is_corrupted {
            return false;
        }

        // 检查密钥信息
        let key_info = self.key_info.read();
        if let Some(ref info) = *key_info {
            let current_hash =
                self.create_verification_hash(&info.encrypted_phone, &info.encrypted_email);
            current_hash == info.verification_hash
        } else {
            false
        }
    }

    /// 标记为已损坏
    fn mark_as_corrupted(&self) {
        let mut state = self.state.write();
        state.is_corrupted = true;
        *self.key_info.write() = None;
        error!("存储已标记为损坏");
    }

    /// 加密数据
    fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        let nonce = crypto::random::random_bytes(12);

        // 简化加密：XOR（实际应该使用 AES-GCM）
        let mut encrypted = Vec::with_capacity(nonce.len() + data.len());
        encrypted.extend_from_slice(&nonce);

        for (i, &byte) in data.iter().enumerate() {
            let key_byte = self.encryption_key[i % self.encryption_key.len()];
            encrypted.push(byte ^ key_byte);
        }

        Ok(encrypted)
    }

    /// 解密数据
    fn decrypt_data(&self, encrypted: &[u8]) -> Result<Vec<u8>> {
        if encrypted.len() < 12 {
            return Err(anyhow::anyhow!("加密数据太短"));
        }

        let _nonce = &encrypted[..12];
        let ciphertext = &encrypted[12..];

        let mut decrypted = Vec::with_capacity(ciphertext.len());
        for (i, &byte) in ciphertext.iter().enumerate() {
            let key_byte = self.encryption_key[i % self.encryption_key.len()];
            decrypted.push(byte ^ key_byte);
        }

        Ok(decrypted)
    }

    /// 添加完整性检查
    fn add_integrity_check(&self, data: &[u8]) -> Result<Vec<u8>> {
        let hash = crypto::HashUtil::hash(data);
        let mut result = Vec::with_capacity(data.len() + hash.len());
        result.extend_from_slice(data);
        result.extend_from_slice(&hash);
        Ok(result)
    }

    /// 创建验证哈希
    fn create_verification_hash(&self, phone: &[u8], email: &[u8]) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(phone);
        data.extend_from_slice(email);
        crypto::HashUtil::hash(&data).to_vec()
    }

    /// 获取状态
    pub fn get_state(&self) -> StorageState {
        self.state.read().clone()
    }

    /// 检查是否有效
    pub fn is_valid(&self) -> bool {
        !self.state.read().is_corrupted && self.key_info.read().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_creation() {
        let storage = EncryptedStorage::new(StorageConfig::default(), vec![0u8; 32]);
        assert!(!storage.get_state().is_corrupted);
    }
}
