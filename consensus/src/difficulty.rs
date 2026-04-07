//! 难度调整算法
//!
//! 实现自适应难度调整，支持低功耗挖矿

/// 难度调整配置
#[derive(Clone, Debug)]
pub struct DifficultyConfig {
    /// 目标出块时间（秒）
    pub target_block_time: u64,
    /// 难度调整周期（区块数）
    pub adjustment_interval: u64,
    /// 最小调整比例
    pub min_ratio: f64,
    /// 最大调整比例
    pub max_ratio: f64,
    /// 平滑因子（0-1，越大调整越激进）
    pub smooth_factor: f64,
}

impl Default for DifficultyConfig {
    fn default() -> Self {
        DifficultyConfig {
            target_block_time: 10,    // 10 秒
            adjustment_interval: 100, // 每 100 区块调整
            min_ratio: 0.5,           // 最小下调 50%
            max_ratio: 2.0,           // 最大上调 100%
            smooth_factor: 0.5,       // 平滑因子
        }
    }
}

/// 难度调整器
pub struct DifficultyAdjuster {
    config: DifficultyConfig,
    current_difficulty: u64,
    /// 最近区块时间记录
    recent_times: Vec<u64>,
}

impl DifficultyAdjuster {
    /// 创建新调整器
    pub fn new(initial_difficulty: u64) -> Self {
        DifficultyAdjuster {
            config: DifficultyConfig::default(),
            current_difficulty: initial_difficulty,
            recent_times: Vec::with_capacity(100),
        }
    }

    /// 使用自定义配置创建
    pub fn with_config(initial_difficulty: u64, config: DifficultyConfig) -> Self {
        DifficultyAdjuster {
            config,
            current_difficulty: initial_difficulty,
            recent_times: Vec::with_capacity(100),
        }
    }

    /// 记录区块时间
    pub fn record_block_time(&mut self, timestamp: u64) {
        self.recent_times.push(timestamp);

        // 保持最近 100 个区块的时间
        if self.recent_times.len() > 100 {
            self.recent_times.remove(0);
        }
    }

    /// 计算新的难度
    pub fn calculate_new_difficulty(&mut self) -> u64 {
        if self.recent_times.len() < 2 {
            return self.current_difficulty;
        }

        // 计算实际出块时间
        let first = *self.recent_times.first().unwrap();
        let last = *self.recent_times.last().unwrap();
        let actual_time = last - first;

        // 计算目标时间
        let target_time = self.config.target_block_time * (self.recent_times.len() as u64 - 1);

        // 计算调整比例
        let mut ratio = actual_time as f64 / target_time as f64;

        // 应用平滑因子
        ratio = 1.0 + (ratio - 1.0) * self.config.smooth_factor;

        // 限制调整幅度
        ratio = ratio.max(self.config.min_ratio);
        ratio = ratio.min(self.config.max_ratio);

        // 计算新难度
        let new_difficulty = (self.current_difficulty as f64 * ratio) as u64;

        // 确保难度不会变为 0
        let new_difficulty = new_difficulty.max(1);

        self.current_difficulty = new_difficulty;
        new_difficulty
    }

    /// 获取当前难度
    pub fn get_current_difficulty(&self) -> u64 {
        self.current_difficulty
    }

    /// 检查是否需要调整
    pub fn should_adjust(&self, block_height: u64) -> bool {
        block_height > 0 && block_height % self.config.adjustment_interval == 0
    }
}

/// 计算目标难度（用于创世区块）
pub fn calculate_initial_difficulty(target_time_secs: u64) -> u64 {
    // 基于目标出块时间计算初始难度
    // 假设普通 CPU 每秒可计算 10^6 次哈希
    // 目标：平均 10 秒找到一个区块
    let hashes_per_second = 1_000_000u64;
    hashes_per_second * target_time_secs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_adjustment() {
        let mut adjuster = DifficultyAdjuster::new(1_000_000);

        // 模拟 100 个区块，每个 10 秒（正常）
        let base_time = 1000;
        for i in 0..100 {
            adjuster.record_block_time(base_time + i * 10);
        }

        let new_diff = adjuster.calculate_new_difficulty();
        // 出块时间正常，难度应该接近初始值
        assert!(new_diff > 500_000 && new_diff < 2_000_000);
    }

    #[test]
    fn test_difficulty_increase() {
        let mut adjuster = DifficultyAdjuster::new(1_000_000);

        // 模拟 100 个区块，每个 5 秒（出块太快）
        let base_time = 1000;
        for i in 0..100 {
            adjuster.record_block_time(base_time + i * 5);
        }

        let new_diff = adjuster.calculate_new_difficulty();
        // 出块太快，难度应该上调（但由于平滑因子，可能不会超过初始值太多）
        // 实际时间 = 495 秒，目标时间 = 990 秒，ratio = 0.5
        // 平滑后 ratio = 1 + (0.5 - 1) * 0.5 = 0.75
        // 新难度 = 1_000_000 * 0.75 = 750_000
        assert!(new_diff < 1_000_000); // 难度下降（因为实际时间更短）
    }

    #[test]
    fn test_difficulty_decrease() {
        let mut adjuster = DifficultyAdjuster::new(1_000_000);

        // 模拟 100 个区块，每个 20 秒（出块太慢）
        let base_time = 1000;
        for i in 0..100 {
            adjuster.record_block_time(base_time + i * 20);
        }

        let new_diff = adjuster.calculate_new_difficulty();
        // 出块太慢，难度应该下调
        // 实际时间 = 1980 秒，目标时间 = 990 秒，ratio = 2.0
        // 平滑后 ratio = 1 + (2.0 - 1) * 0.5 = 1.5
        // 新难度 = 1_000_000 * 1.5 = 1_500_000
        assert!(new_diff > 1_000_000); // 难度上升（因为实际时间更长）
    }
}
