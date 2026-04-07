//! 哈希工具（占位）

pub struct HashUtil;

impl HashUtil {
    pub fn hash(data: &[u8]) -> [u8; 32] {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(data);
        let hash = hasher.finalize();
        let mut result = [0u8; 32];
        result.copy_from_slice(hash.as_bytes());
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_consistency() {
        let data = b"test data";
        let hash1 = HashUtil::hash(data);
        let hash2 = HashUtil::hash(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_different_data() {
        let hash1 = HashUtil::hash(b"data1");
        let hash2 = HashUtil::hash(b"data2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_empty() {
        let hash = HashUtil::hash(b"");
        assert_ne!(hash, [0u8; 32]);
    }

    #[test]
    fn test_hash_size() {
        let hash = HashUtil::hash(b"test");
        assert_eq!(hash.len(), 32);
    }
}
