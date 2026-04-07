//! Toki 超主权数字货币平台 - 加密模块

pub mod aes;
pub mod encryption;
pub mod hash;
pub mod random;
pub mod ring_signature;

pub use encryption::*;
pub use hash::*;
pub use ring_signature::*;

/// 哈希函数（兼容旧接口）
pub fn hash(data: &[u8]) -> ([u8; 32],) {
    (HashUtil::hash(data),)
}
