//! 环签名实现（占位）

pub struct RingSignature;

impl RingSignature {
    pub fn new() -> Self {
        RingSignature
    }
}

impl Default for RingSignature {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_signature_new() {
        let sig = RingSignature::new();
        assert!(matches!(sig, RingSignature));
    }

    #[test]
    fn test_ring_signature_default() {
        let sig = RingSignature::default();
        assert!(matches!(sig, RingSignature));
    }
}
