//! Toki 超主权数字货币平台 - 加密模块

pub mod ring_signature;
pub mod hash;
pub mod encryption;
pub mod random;
pub mod aes;

pub use ring_signature::*;
pub use hash::*;
pub use encryption::*;

/// 哈希函数（兼容旧接口）
pub fn hash(data: &[u8]) -> ([u8; 32],) {
    (HashUtil::hash(data),)
}
