//! 加密工具（占位）

use x25519_dalek::{EphemeralSecret, PublicKey};

/// 端到端加密
pub struct E2EEncryption {
    public_key: PublicKey,
}

impl E2EEncryption {
    /// 生成新密钥对
    pub fn new() -> (Self, EphemeralSecret) {
        let secret = EphemeralSecret::random_from_rng(rand::thread_rng());
        let public = PublicKey::from(&secret);
        (E2EEncryption { public_key: public }, secret)
    }

    /// 获取公钥
    pub fn public_key(&self) -> &[u8] {
        self.public_key.as_bytes()
    }
}

impl Default for E2EEncryption {
    fn default() -> Self {
        let (enc, _) = Self::new();
        enc
    }
}

/// AES-256-GCM 加密
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Vec<u8> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Nonce};
    
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let nonce = Nonce::from_slice(&[0u8; 12]);
    cipher.encrypt(nonce, plaintext).unwrap_or_default()
}

/// AES-256-GCM 解密
pub fn decrypt(key: &[u8; 32], ciphertext: &[u8]) -> Vec<u8> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Nonce};
    
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let nonce = Nonce::from_slice(&[0u8; 12]);
    cipher.decrypt(nonce, ciphertext).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e2e_encryption_new() {
        let (enc, _) = E2EEncryption::new();
        assert_eq!(enc.public_key().len(), 32);
    }

    #[test]
    fn test_e2e_encryption_default() {
        let enc = E2EEncryption::default();
        assert_eq!(enc.public_key().len(), 32);
    }

    #[test]
    fn test_encrypt_decrypt() {
        let key = [1u8; 32];
        let plaintext = b"hello world";
        
        let ciphertext = encrypt(&key, plaintext);
        let decrypted = decrypt(&key, &ciphertext);
        
        assert_eq!(decrypted, plaintext.to_vec());
    }

    #[test]
    fn test_encrypt_produces_different_output() {
        let key = [1u8; 32];
        let plaintext = b"hello world";
        
        let ciphertext = encrypt(&key, plaintext);
        
        // 密文应该与明文不同
        assert_ne!(ciphertext.to_vec(), plaintext.to_vec());
    }

    #[test]
    fn test_encrypt_empty() {
        let key = [1u8; 32];
        let ciphertext = encrypt(&key, b"");
        assert!(!ciphertext.is_empty());
    }
}
