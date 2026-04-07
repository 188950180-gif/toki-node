//! AES 加密工具

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm as Aes256GcmInner, Nonce};

/// AES-256-GCM 加密器
pub struct Aes256Gcm {
    cipher: Aes256GcmInner,
}

impl Aes256Gcm {
    /// 创建新的加密器
    pub fn new(key: &[u8; 32]) -> Self {
        Aes256Gcm {
            cipher: Aes256GcmInner::new_from_slice(key).unwrap(),
        }
    }

    /// 加密
    pub fn encrypt(&self, nonce: &[u8; 12], plaintext: &[u8]) -> Vec<u8> {
        let nonce = Nonce::from_slice(nonce);
        self.cipher.encrypt(nonce, plaintext).unwrap_or_else(|_| Vec::new())
    }

    /// 解密
    pub fn decrypt(&self, nonce: &[u8; 12], ciphertext: &[u8]) -> Vec<u8> {
        let nonce = Nonce::from_slice(nonce);
        self.cipher.decrypt(nonce, ciphertext).unwrap_or_else(|_| Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = [1u8; 32];
        let cipher = Aes256Gcm::new(&key);
        let nonce = [0u8; 12];
        let plaintext = b"hello world";

        let ciphertext = cipher.encrypt(&nonce, plaintext);
        let decrypted = cipher.decrypt(&nonce, &ciphertext);

        assert_eq!(decrypted, plaintext.to_vec());
    }
}
