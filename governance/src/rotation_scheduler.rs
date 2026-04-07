//! 密钥轮换调度器
//!
//! 实现自动化的密钥轮换调度
//! 定期检查并执行密钥轮换

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};
use parking_lot::RwLock;
use tokio::sync::mpsc;
use anyhow::Result;
use tracing::{info, warn, debug, error};

use super::key_rotation::{KeyRotationManager, KeyRotationConfig};

/// 调度器配置
#[derive(Clone, Debug)]
pub struct SchedulerConfig {
    /// 检查间隔（秒）
    pub check_interval_secs: u64,
    /// 启用自动调度
    pub auto_schedule_enabled: bool,
    /// 最大重试次数
    pub max_retries: u32,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        SchedulerConfig {
            check_interval_secs: 3600, // 每小时检查一次
            auto_schedule_enabled: true,
            max_retries: 3,
        }
    }
}

/// 调度器事件
#[derive(Debug, Clone)]
pub enum SchedulerEvent {
    /// 开始检查
    CheckStarted,
    /// 需要轮换
    RotationRequired,
    /// 轮换成功
    RotationSuccess { count: u64 },
    /// 轮换失败
    RotationFailed { error: String },
    /// 跳过轮换
    RotationSkipped,
    /// 调度器停止
    Stopped,
}

/// 密钥轮换调度器
pub struct RotationScheduler {
    config: SchedulerConfig,
    rotation_manager: Arc<KeyRotationManager>,
    event_sender: mpsc::UnboundedSender<SchedulerEvent>,
    event_receiver: Option<mpsc::UnboundedReceiver<SchedulerEvent>>,
    running: Arc<AtomicBool>,
}

impl RotationScheduler {
    /// 创建新的调度器
    pub fn new(
        config: SchedulerConfig,
        rotation_manager: Arc<KeyRotationManager>,
    ) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        info!("创建密钥轮换调度器");
        info!("检查间隔: {} 秒", config.check_interval_secs);

        RotationScheduler {
            config,
            rotation_manager,
            event_sender,
            event_receiver: Some(event_receiver),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 启动调度器
    pub async fn start(&self) -> Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            warn!("调度器已在运行");
            return Ok(());
        }

        info!("启动密钥轮换调度器");

        let config = self.config.clone();
        let rotation_manager = self.rotation_manager.clone();
        let event_sender = self.event_sender.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(config.check_interval_secs));
            
            info!("调度器已启动，将在 {} 秒后首次检查", config.check_interval_secs);

            loop {
                interval.tick().await;

                if !running.load(Ordering::SeqCst) {
                    info!("调度器停止");
                    let _ = event_sender.send(SchedulerEvent::Stopped);
                    break;
                }

                Self::check_and_rotate(&rotation_manager, &event_sender, &config).await;
            }
        });

        Ok(())
    }

    /// 检查并执行轮换
    async fn check_and_rotate(
        rotation_manager: &Arc<KeyRotationManager>,
        event_sender: &mpsc::UnboundedSender<SchedulerEvent>,
        config: &SchedulerConfig,
    ) {
        let _ = event_sender.send(SchedulerEvent::CheckStarted);
        debug!("检查密钥轮换状态");

        // 检查是否需要轮换
        if !rotation_manager.should_rotate() {
            debug!("不需要轮换");
            let _ = event_sender.send(SchedulerEvent::RotationSkipped);
            return;
        }

        info!("触发密钥轮换");
        let _ = event_sender.send(SchedulerEvent::RotationRequired);

        // 执行轮换（带重试）
        let mut retries = 0;
        loop {
            match rotation_manager.rotate_key().await {
                Ok(_) => {
                    let state = rotation_manager.get_state();
                    info!("密钥轮换成功（第{}次）", state.rotation_count);
                    let _ = event_sender.send(SchedulerEvent::RotationSuccess {
                        count: state.rotation_count,
                    });
                    break;
                }
                Err(e) => {
                    retries += 1;
                    if retries >= config.max_retries {
                        error!("密钥轮换失败，已达最大重试次数: {}", e);
                        let _ = event_sender.send(SchedulerEvent::RotationFailed {
                            error: e.to_string(),
                        });
                        break;
                    } else {
                        warn!("密钥轮换失败，重试 {}/{}: {}", retries, config.max_retries, e);
                        tokio::time::sleep(Duration::from_secs(60)).await;
                    }
                }
            }
        }
    }

    /// 停止调度器
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        info!("调度器停止请求已发送");
    }

    /// 检查是否运行中
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// 获取事件接收器
    pub fn take_event_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<SchedulerEvent>> {
        self.event_receiver.take()
    }

    /// 手动触发检查
    pub async fn trigger_check(&self) -> Result<()> {
        info!("手动触发密钥轮换检查");
        Self::check_and_rotate(&self.rotation_manager, &self.event_sender, &self.config).await;
        Ok(())
    }

    /// 获取状态
    pub fn get_status(&self) -> SchedulerStatus {
        SchedulerStatus {
            is_running: self.is_running(),
            check_interval_secs: self.config.check_interval_secs,
            auto_schedule_enabled: self.config.auto_schedule_enabled,
            rotation_state: self.rotation_manager.get_state(),
        }
    }
}

/// 调度器状态
#[derive(Debug, Clone)]
pub struct SchedulerStatus {
    /// 是否运行中
    pub is_running: bool,
    /// 检查间隔
    pub check_interval_secs: u64,
    /// 是否启用自动调度
    pub auto_schedule_enabled: bool,
    /// 轮换状态
    pub rotation_state: super::key_rotation::KeyRotationState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let config = KeyRotationConfig::default();
        let manager = Arc::new(KeyRotationManager::new(config, vec![0u8; 32]));
        let scheduler = RotationScheduler::new(SchedulerConfig::default(), manager);
        
        assert!(!scheduler.is_running());
    }

    #[tokio::test]
    async fn test_scheduler_start_stop() {
        let config = KeyRotationConfig::default();
        let manager = Arc::new(KeyRotationManager::new(config, vec![0u8; 32]));
        let scheduler = RotationScheduler::new(SchedulerConfig::default(), manager);
        
        scheduler.start().await.unwrap();
        assert!(scheduler.is_running());
        
        scheduler.stop();
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(!scheduler.is_running());
    }
}
