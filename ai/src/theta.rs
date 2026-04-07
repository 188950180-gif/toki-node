//! 物资价值指数计算模块
//! 
//! 计算 theta 系数，实现 toki 与物资的锚定

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use toki_core::TOTAL_SUPPLY;

/// 物资价值数据
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialValueData {
    /// 全球 GDP（美元）
    pub global_gdp: u64,
    /// 消费者价格指数
    pub consumer_price_index: f64,
    /// 全球人口
    pub global_population: u64,
    /// 数据时间
    pub timestamp: DateTime<Utc>,
}

/// Theta 计算器
pub struct ThetaCalculator {
    /// toki 总量
    total_supply: u64,
    /// 历史数据
    history: Vec<MaterialValueData>,
}

impl ThetaCalculator {
    pub fn new() -> Self {
        ThetaCalculator {
            total_supply: TOTAL_SUPPLY,
            history: Vec::new(),
        }
    }

    /// 计算物资价值总值
    pub fn calculate_material_value(&self, data: &MaterialValueData) -> u64 {
        // 物资价值 = GDP * CPI / 100
        let gdp = data.global_gdp as f64;
        let cpi = data.consumer_price_index / 100.0;
        (gdp * cpi) as u64
    }

    /// 计算 theta 系数
    pub fn calculate_theta(&self, data: &MaterialValueData) -> f64 {
        let material_value = self.calculate_material_value(data);
        
        if material_value == 0 {
            return 1.0;
        }
        
        // theta = toki 总量 / 物资价值
        self.total_supply as f64 / material_value as f64
    }

    /// 计算调节量
    pub fn calculate_adjustment(&self, data: &MaterialValueData) -> AdjustmentResult {
        let material_value = self.calculate_material_value(data);
        let theta = self.calculate_theta(data);
        
        // 计算当前 toki 价值
        let current_value = (self.total_supply as f64 / theta) as u64;
        
        // 判断是否需要调节
        let adjustment_type = if current_value > material_value {
            AdjustmentType::Deflation
        } else if current_value < material_value {
            AdjustmentType::Inflation
        } else {
            AdjustmentType::None
        };
        
        let adjustment_amount = if current_value > material_value {
            current_value - material_value
        } else {
            material_value - current_value
        };
        
        AdjustmentResult {
            theta,
            material_value,
            current_value,
            adjustment_type,
            adjustment_amount,
        }
    }

    /// 添加历史数据
    pub fn add_data(&mut self, data: MaterialValueData) {
        self.history.push(data);
        // 保留最近 365 天的数据
        if self.history.len() > 365 {
            self.history.remove(0);
        }
    }

    /// 获取平均 theta
    pub fn average_theta(&self) -> f64 {
        if self.history.is_empty() {
            return 1.0;
        }
        
        let sum: f64 = self.history.iter()
            .map(|d| self.calculate_theta(d))
            .sum();
        
        sum / self.history.len() as f64
    }
}

impl Default for ThetaCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// 调节类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AdjustmentType {
    /// 无需调节
    None,
    /// 通缩（释放调节池）
    Deflation,
    /// 通胀（回收 toki）
    Inflation,
}

/// 调节结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustmentResult {
    /// theta 系数
    pub theta: f64,
    /// 物资价值
    pub material_value: u64,
    /// 当前价值
    pub current_value: u64,
    /// 调节类型
    pub adjustment_type: AdjustmentType,
    /// 调节量
    pub adjustment_amount: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_value_calculation() {
        let calc = ThetaCalculator::new();
        let data = MaterialValueData {
            global_gdp: 100_000_000_000_000, // 100 万亿美元
            consumer_price_index: 100.0,
            global_population: 8_000_000_000,
            timestamp: Utc::now(),
        };
        
        let value = calc.calculate_material_value(&data);
        assert_eq!(value, 100_000_000_000_000);
    }

    #[test]
    fn test_theta_calculation() {
        let calc = ThetaCalculator::new();
        let data = MaterialValueData {
            global_gdp: 814_400_000_000_000, // 814.4 万亿美元
            consumer_price_index: 100.0,
            global_population: 8_144_000_000,
            timestamp: Utc::now(),
        };
        
        let theta = calc.calculate_theta(&data);
        // theta 应该接近 1.0
        assert!(theta > 0.9 && theta < 1.1);
    }

    #[test]
    fn test_adjustment() {
        let calc = ThetaCalculator::new();
        let data = MaterialValueData {
            global_gdp: 100_000_000_000_000,
            consumer_price_index: 100.0,
            global_population: 8_000_000_000,
            timestamp: Utc::now(),
        };
        
        let result = calc.calculate_adjustment(&data);
        assert!(result.theta > 0.0);
    }
}
