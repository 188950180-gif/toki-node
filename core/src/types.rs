//! 基础类型定义

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// 哈希值（32 字节）
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    /// 零哈希
    pub const ZERO: Hash = Hash([0u8; 32]);

    /// 从字节数组创建
    pub fn new(bytes: [u8; 32]) -> Self {
        Hash(bytes)
    }

    /// 从数据计算哈希（使用 Blake3）
    pub fn from_data(data: &[u8]) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(data);
        let hash = hasher.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(hash.as_bytes());
        Hash(bytes)
    }

    /// 转换为字节数组
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// 转换为十六进制字符串
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// 从十六进制字符串解析
    pub fn from_hex(s: &str) -> Result<Self, TypesError> {
        let bytes = hex::decode(s).map_err(|_| TypesError::InvalidHex)?;
        if bytes.len() != 32 {
            return Err(TypesError::InvalidHashLength);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Hash(arr))
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash({})", self.to_hex())
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Default for Hash {
    fn default() -> Self {
        Self::ZERO
    }
}

/// 地址（Base58 编码）
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address(pub [u8; 32]);

impl Address {
    /// 零地址
    pub const ZERO: Address = Address([0u8; 32]);

    /// 从字节数组创建
    pub fn new(bytes: [u8; 32]) -> Self {
        Address(bytes)
    }

    /// 从公钥生成地址
    pub fn from_pubkey(pubkey: &[u8]) -> Self {
        let hash = Hash::from_data(pubkey);
        Address(hash.0)
    }

    /// 转换为字节数组
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// 转换为 Base58 字符串
    pub fn to_base58(&self) -> String {
        base58::ToBase58::to_base58(&self.0[..])
    }

    /// 从 Base58 字符串解析
    pub fn from_base58(s: &str) -> Result<Self, TypesError> {
        let bytes = base58::FromBase58::from_base58(s).map_err(|_| TypesError::InvalidBase58)?;
        if bytes.len() != 32 {
            return Err(TypesError::InvalidAddressLength);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Address(arr))
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Address({})", self.to_base58())
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_base58())
    }
}

impl Default for Address {
    fn default() -> Self {
        Self::ZERO
    }
}

impl FromStr for Address {
    type Err = TypesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Address::from_base58(s)
    }
}

/// 区域枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Region {
    /// 美国
    US,
    /// 欧洲
    EU,
    /// 俄罗斯
    RU,
    /// 亚洲
    AS,
    /// 其他
    Other,
}

impl Default for Region {
    fn default() -> Self {
        Region::Other
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Region::US => write!(f, "US"),
            Region::EU => write!(f, "EU"),
            Region::RU => write!(f, "RU"),
            Region::AS => write!(f, "AS"),
            Region::Other => write!(f, "Other"),
        }
    }
}

/// 法币类型
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FiatType {
    /// 美元
    USD,
    /// 欧元
    EUR,
    /// 人民币
    CNY,
    /// 日元
    JPY,
    /// 英镑
    GBP,
    /// 其他
    Other(String),
}

impl Default for FiatType {
    fn default() -> Self {
        FiatType::USD
    }
}

impl fmt::Display for FiatType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FiatType::USD => write!(f, "USD"),
            FiatType::EUR => write!(f, "EUR"),
            FiatType::CNY => write!(f, "CNY"),
            FiatType::JPY => write!(f, "JPY"),
            FiatType::GBP => write!(f, "GBP"),
            FiatType::Other(s) => write!(f, "{}", s),
        }
    }
}

/// 类型错误
#[derive(Debug, Error)]
pub enum TypesError {
    #[error("Invalid hex string")]
    InvalidHex,

    #[error("Invalid hash length")]
    InvalidHashLength,

    #[error("Invalid base58 string")]
    InvalidBase58,

    #[error("Invalid address length")]
    InvalidAddressLength,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_from_data() {
        let data = b"test data";
        let hash1 = Hash::from_data(data);
        let hash2 = Hash::from_data(data);
        assert_eq!(hash1, hash2);

        let data2 = b"different data";
        let hash3 = Hash::from_data(data2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_hex() {
        let hash = Hash::from_data(b"test");
        let hex = hash.to_hex();
        let hash2 = Hash::from_hex(&hex).unwrap();
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_address_base58() {
        let bytes = [1u8; 32];
        let addr = Address::new(bytes);
        let base58 = addr.to_base58();
        let addr2 = Address::from_base58(&base58).unwrap();
        assert_eq!(addr, addr2);
    }

    #[test]
    fn test_address_from_str() {
        let bytes = [2u8; 32];
        let addr = Address::new(bytes);
        let base58 = addr.to_base58();
        let addr2: Address = base58.parse().unwrap();
        assert_eq!(addr, addr2);
    }
}
