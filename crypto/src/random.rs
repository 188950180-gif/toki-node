//! 随机数生成工具

use anyhow::Result;

/// 填充随机字节
pub fn fill_random(bytes: &mut [u8]) -> Result<()> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.fill(bytes);
    Ok(())
}

/// 生成随机字节
pub fn random_bytes(len: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..len).map(|_| rng.gen()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_random() {
        let mut bytes = [0u8; 32];
        fill_random(&mut bytes).unwrap();
        // 检查不是全零
        assert!(bytes.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_random_bytes() {
        let bytes = random_bytes(32);
        assert_eq!(bytes.len(), 32);
        // 检查不是全零
        assert!(bytes.iter().any(|&b| b != 0));
    }
}
