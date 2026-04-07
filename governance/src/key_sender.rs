//! 密钥发送模块
//!
//! 实现密钥片段的安全发送机制

use std::time::Duration;
use anyhow::Result;
use tracing::{info, warn, error, debug};

use super::key_rotation::KeyFragment;

/// 发送配置
#[derive(Clone, Debug)]
pub struct SendConfig {
    /// 重试次数
    pub max_retries: u32,
    /// 超时时间（秒）
    pub timeout_secs: u64,
    /// 启用确认
    pub enable_confirmation: bool,
}

impl Default for SendConfig {
    fn default() -> Self {
        SendConfig {
            max_retries: 3,
            timeout_secs: 30,
            enable_confirmation: true,
        }
    }
}

/// 发送结果
#[derive(Debug, Clone)]
pub struct SendResult {
    /// 是否成功
    pub success: bool,
    /// 重试次数
    pub retries: u32,
    /// 耗时（毫秒）
    pub duration_ms: u64,
    /// 错误信息
    pub error_message: Option<String>,
}

/// 密钥发送器
pub struct KeySender {
    config: SendConfig,
}

impl KeySender {
    /// 创建新的发送器
    pub fn new(config: SendConfig) -> Self {
        info!("创建密钥发送器");
        KeySender { config }
    }

    /// 发送到手机号
    pub async fn send_to_phone(&self, phone: &str, fragment: &KeyFragment) -> Result<SendResult> {
        info!("发送密钥片段到手机号（已加密）");
        debug!("片段ID: {}", fragment.fragment_id);

        let start = std::time::Instant::now();
        let mut retries = 0;

        loop {
            match self.try_send_to_phone(phone, fragment).await {
                Ok(_) => {
                    let duration = start.elapsed().as_millis() as u64;
                    info!("✅ 发送到手机号成功");
                    
                    return Ok(SendResult {
                        success: true,
                        retries,
                        duration_ms: duration,
                        error_message: None,
                    });
                }
                Err(e) => {
                    retries += 1;
                    if retries >= self.config.max_retries {
                        error!("❌ 发送到手机号失败（重试{}次）: {}", self.config.max_retries, e);
                        let duration = start.elapsed().as_millis() as u64;
                        
                        return Ok(SendResult {
                            success: false,
                            retries,
                            duration_ms: duration,
                            error_message: Some(e.to_string()),
                        });
                    }
                    warn!("发送失败，重试 {}/{}: {}", retries, self.config.max_retries, e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    /// 发送到邮箱
    pub async fn send_to_email(&self, email: &str, fragment: &KeyFragment) -> Result<SendResult> {
        info!("发送密钥片段到邮箱（已加密）");
        debug!("片段ID: {}", fragment.fragment_id);

        let start = std::time::Instant::now();
        let mut retries = 0;

        loop {
            match self.try_send_to_email(email, fragment).await {
                Ok(_) => {
                    let duration = start.elapsed().as_millis() as u64;
                    info!("✅ 发送到邮箱成功");
                    
                    return Ok(SendResult {
                        success: true,
                        retries,
                        duration_ms: duration,
                        error_message: None,
                    });
                }
                Err(e) => {
                    retries += 1;
                    if retries >= self.config.max_retries {
                        error!("❌ 发送到邮箱失败（重试{}次）: {}", self.config.max_retries, e);
                        let duration = start.elapsed().as_millis() as u64;
                        
                        return Ok(SendResult {
                            success: false,
                            retries,
                            duration_ms: duration,
                            error_message: Some(e.to_string()),
                        });
                    }
                    warn!("发送失败，重试 {}/{}: {}", retries, self.config.max_retries, e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    /// 尝试发送到手机号
    async fn try_send_to_phone(&self, phone: &str, fragment: &KeyFragment) -> Result<()> {
        // 模拟发送逻辑
        // 实际实现应该使用短信网关 API
        
        debug!("模拟发送到手机号: {}", phone);
        debug!("片段数据长度: {} 字节", fragment.encrypted_data.len());
        
        // 模拟网络延迟
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // 模拟成功
        Ok(())
    }

    /// 尝试发送到邮箱
    async fn try_send_to_email(&self, email: &str, fragment: &KeyFragment) -> Result<()> {
        // 模拟发送逻辑
        // 实际实现应该使用 SMTP 或邮件 API
        
        debug!("模拟发送到邮箱: {}", email);
        debug!("片段数据长度: {} 字节", fragment.encrypted_data.len());
        
        // 模拟网络延迟
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // 模拟成功
        Ok(())
    }

    /// 批量发送片段
    pub async fn send_batch(&self, targets: &[(String, KeyFragment)]) -> Vec<SendResult> {
        let mut results = Vec::new();
        
        for (target, fragment) in targets {
            let result = if target.contains('@') {
                // 邮箱
                self.send_to_email(&target, fragment).await.unwrap_or_else(|e| SendResult {
                    success: false,
                    retries: 0,
                    duration_ms: 0,
                    error_message: Some(e.to_string()),
                })
            } else {
                // 手机号
                self.send_to_phone(&target, fragment).await.unwrap_or_else(|e| SendResult {
                    success: false,
                    retries: 0,
                    duration_ms: 0,
                    error_message: Some(e.to_string()),
                })
            };
            
            results.push(result);
        }
        
        results
    }
}

impl Default for KeySender {
    fn default() -> Self {
        Self::new(SendConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_key_sender_creation() {
        let sender = KeySender::new(SendConfig::default());
        assert_eq!(sender.config.max_retries, 3);
    }

    #[tokio::test]
    async fn test_send_to_phone() {
        let sender = KeySender::default();
        let fragment = KeyFragment {
            fragment_id: "test".to_string(),
            encrypted_data: vec![1u8; 32],
            created_at: 0,
            index: 0,
        };
        
        let result = sender.send_to_phone("13800138000", &fragment).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_send_to_email() {
        let sender = KeySender::default();
        let fragment = KeyFragment {
            fragment_id: "test".to_string(),
            encrypted_data: vec![1u8; 32],
            created_at: 0,
            index: 1,
        };
        
        let result = sender.send_to_email("test@example.com", &fragment).await.unwrap();
        assert!(result.success);
    }
}
