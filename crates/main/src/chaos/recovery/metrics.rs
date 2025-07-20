//! Recovery metrics collection and analysis

use super::{CompletedRecoveryMetrics, RecoveryMetrics};
use crate::error::PrimalError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Metrics collector for recovery operations
#[derive(Debug)]
pub struct RecoveryMetricsCollector {
    /// Historical recovery metrics storage
    historical_metrics: Arc<RwLock<Vec<CompletedRecoveryMetrics>>>,
    /// Active recovery metrics by ID
    active_metrics: Arc<RwLock<HashMap<String, RecoveryMetrics>>>,
    /// Metrics aggregation cache
    aggregated_metrics: Arc<RwLock<AggregatedRecoveryMetrics>>,
}

/// Aggregated recovery metrics for analysis
#[derive(Debug, Clone)]
pub struct AggregatedRecoveryMetrics {
    /// Total number of recoveries performed
    pub total_recoveries: u32,
    /// Number of successful recoveries
    pub successful_recoveries: u32,
    /// Average recovery duration
    pub avg_recovery_duration_ms: u64,
    /// Most common recovery strategies
    pub common_strategies: HashMap<String, u32>,
    /// Average success rate
    pub avg_success_rate: f64,
    /// Average health improvement
    pub avg_health_improvement: f64,
    /// Recovery trend over time
    pub recovery_trends: RecoveryTrends,
}

/// Recovery trends analysis
#[derive(Debug, Clone)]
pub struct RecoveryTrends {
    /// Success rate trend (improving, stable, declining)
    pub success_rate_trend: TrendDirection,
    /// Duration trend (faster, stable, slower)
    pub duration_trend: TrendDirection,
    /// Frequency trend (more frequent, stable, less frequent)
    pub frequency_trend: TrendDirection,
    /// Health improvement trend
    pub health_improvement_trend: TrendDirection,
}

/// Trend direction enumeration
#[derive(Debug, Clone)]
pub enum TrendDirection {
    /// Metric is improving
    Improving,
    /// Metric is stable
    Stable,
    /// Metric is declining
    Declining,
    /// Not enough data to determine trend
    Unknown,
}

impl RecoveryMetricsCollector {
    /// Create a new recovery metrics collector
    pub fn new() -> Self {
        Self {
            historical_metrics: Arc::new(RwLock::new(Vec::new())),
            active_metrics: Arc::new(RwLock::new(HashMap::new())),
            aggregated_metrics: Arc::new(RwLock::new(AggregatedRecoveryMetrics::default())),
        }
    }

    /// Start tracking metrics for a recovery operation
    pub async fn start_tracking(&self, recovery_id: String, initial_metrics: RecoveryMetrics) {
        let mut active = self.active_metrics.write().await;
        active.insert(recovery_id, initial_metrics);
    }

    /// Update metrics for an active recovery
    pub async fn update_metrics(
        &self,
        recovery_id: &str,
        metrics: RecoveryMetrics,
    ) -> Result<(), PrimalError> {
        let mut active = self.active_metrics.write().await;
        if let Some(existing_metrics) = active.get_mut(recovery_id) {
            *existing_metrics = metrics;
            Ok(())
        } else {
            Err(PrimalError::NotFound(format!(
                "Recovery metrics not found for ID: {}",
                recovery_id
            )))
        }
    }

    /// Complete tracking and store historical metrics
    pub async fn store_completed_recovery(&self, completed_metrics: CompletedRecoveryMetrics) {
        // Store in historical metrics
        {
            let mut historical = self.historical_metrics.write().await;
            historical.push(completed_metrics.clone());
        }

        // Remove from active tracking
        {
            let mut active = self.active_metrics.write().await;
            active.remove(&completed_metrics.recovery_id);
        }

        // Update aggregated metrics
        self.update_aggregated_metrics().await;
    }

    /// Get historical recovery metrics
    pub async fn get_historical_metrics(&self) -> Vec<CompletedRecoveryMetrics> {
        self.historical_metrics.read().await.clone()
    }

    /// Get active recovery metrics
    pub async fn get_active_metrics(&self, recovery_id: &str) -> Option<RecoveryMetrics> {
        self.active_metrics.read().await.get(recovery_id).cloned()
    }

    /// Get aggregated metrics
    pub async fn get_aggregated_metrics(&self) -> AggregatedRecoveryMetrics {
        self.aggregated_metrics.read().await.clone()
    }

    /// Get recovery statistics
    pub async fn get_recovery_statistics(&self) -> RecoveryStatistics {
        let historical = self.historical_metrics.read().await;
        let total_recoveries = historical.len() as u32;

        if total_recoveries == 0 {
            return RecoveryStatistics::default();
        }

        let successful_recoveries = historical
            .iter()
            .filter(|m| m.success_rate >= 0.8) // Consider 80%+ as successful
            .count() as u32;

        let avg_duration = historical
            .iter()
            .map(|m| m.recovery_duration.as_millis() as u64)
            .sum::<u64>()
            / total_recoveries as u64;

        let avg_success_rate =
            historical.iter().map(|m| m.success_rate).sum::<f64>() / total_recoveries as f64;

        let avg_health_improvement =
            historical.iter().map(|m| m.health_improvement).sum::<f64>() / total_recoveries as f64;

        // Calculate strategy usage
        let mut strategy_usage = HashMap::new();
        for metrics in historical.iter() {
            for strategy in &metrics.strategies_used {
                let strategy_name = format!("{:?}", strategy);
                *strategy_usage.entry(strategy_name).or_insert(0) += 1;
            }
        }

        RecoveryStatistics {
            total_recoveries,
            successful_recoveries,
            success_rate: successful_recoveries as f64 / total_recoveries as f64,
            avg_recovery_duration_ms: avg_duration,
            avg_success_rate,
            avg_health_improvement,
            strategy_usage,
            recent_recoveries: historical.iter().rev().take(10).cloned().collect(),
        }
    }

    /// Update aggregated metrics based on historical data
    async fn update_aggregated_metrics(&self) {
        let historical = self.historical_metrics.read().await;
        let total_recoveries = historical.len() as u32;

        if total_recoveries == 0 {
            return;
        }

        let successful_recoveries =
            historical.iter().filter(|m| m.success_rate >= 0.8).count() as u32;

        let avg_recovery_duration_ms = historical
            .iter()
            .map(|m| m.recovery_duration.as_millis() as u64)
            .sum::<u64>()
            / total_recoveries as u64;

        let mut common_strategies = HashMap::new();
        for metrics in historical.iter() {
            for strategy in &metrics.strategies_used {
                let strategy_name = format!("{:?}", strategy);
                *common_strategies.entry(strategy_name).or_insert(0) += 1;
            }
        }

        let avg_success_rate =
            historical.iter().map(|m| m.success_rate).sum::<f64>() / total_recoveries as f64;

        let avg_health_improvement =
            historical.iter().map(|m| m.health_improvement).sum::<f64>() / total_recoveries as f64;

        let recovery_trends = self.calculate_trends(&historical).await;

        let aggregated = AggregatedRecoveryMetrics {
            total_recoveries,
            successful_recoveries,
            avg_recovery_duration_ms,
            common_strategies,
            avg_success_rate,
            avg_health_improvement,
            recovery_trends,
        };

        *self.aggregated_metrics.write().await = aggregated;
    }

    /// Calculate recovery trends from historical data
    async fn calculate_trends(&self, historical: &[CompletedRecoveryMetrics]) -> RecoveryTrends {
        if historical.len() < 3 {
            return RecoveryTrends::default();
        }

        // Simple trend calculation - compare recent half with older half
        let mid_point = historical.len() / 2;
        let (older_half, recent_half) = historical.split_at(mid_point);

        // Calculate success rate trends
        let older_success_rate =
            older_half.iter().map(|m| m.success_rate).sum::<f64>() / older_half.len() as f64;

        let recent_success_rate =
            recent_half.iter().map(|m| m.success_rate).sum::<f64>() / recent_half.len() as f64;

        let success_rate_trend = match (recent_success_rate - older_success_rate) {
            diff if diff > 0.05 => TrendDirection::Improving,
            diff if diff < -0.05 => TrendDirection::Declining,
            _ => TrendDirection::Stable,
        };

        // Calculate duration trends
        let older_avg_duration = older_half
            .iter()
            .map(|m| m.recovery_duration.as_millis() as u64)
            .sum::<u64>()
            / older_half.len() as u64;

        let recent_avg_duration = recent_half
            .iter()
            .map(|m| m.recovery_duration.as_millis() as u64)
            .sum::<u64>()
            / recent_half.len() as u64;

        let duration_trend = match recent_avg_duration.cmp(&older_avg_duration) {
            std::cmp::Ordering::Less => TrendDirection::Improving, // Faster is better
            std::cmp::Ordering::Greater => TrendDirection::Declining,
            std::cmp::Ordering::Equal => TrendDirection::Stable,
        };

        // Calculate health improvement trends
        let older_health_improvement =
            older_half.iter().map(|m| m.health_improvement).sum::<f64>() / older_half.len() as f64;

        let recent_health_improvement = recent_half
            .iter()
            .map(|m| m.health_improvement)
            .sum::<f64>()
            / recent_half.len() as f64;

        let health_improvement_trend = match (recent_health_improvement - older_health_improvement)
        {
            diff if diff > 0.05 => TrendDirection::Improving,
            diff if diff < -0.05 => TrendDirection::Declining,
            _ => TrendDirection::Stable,
        };

        RecoveryTrends {
            success_rate_trend,
            duration_trend,
            frequency_trend: TrendDirection::Stable, // TODO: Calculate based on timestamps
            health_improvement_trend,
        }
    }

    /// Clear all metrics (useful for testing)
    pub async fn clear_all_metrics(&self) {
        self.historical_metrics.write().await.clear();
        self.active_metrics.write().await.clear();
        *self.aggregated_metrics.write().await = AggregatedRecoveryMetrics::default();
    }
}

/// Recovery statistics summary
#[derive(Debug, Clone)]
pub struct RecoveryStatistics {
    /// Total number of recoveries
    pub total_recoveries: u32,
    /// Number of successful recoveries
    pub successful_recoveries: u32,
    /// Overall success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Average recovery duration in milliseconds
    pub avg_recovery_duration_ms: u64,
    /// Average validation success rate
    pub avg_success_rate: f64,
    /// Average health improvement score
    pub avg_health_improvement: f64,
    /// Strategy usage frequency
    pub strategy_usage: HashMap<String, u32>,
    /// Recent recoveries for trend analysis
    pub recent_recoveries: Vec<CompletedRecoveryMetrics>,
}

impl Default for AggregatedRecoveryMetrics {
    fn default() -> Self {
        Self {
            total_recoveries: 0,
            successful_recoveries: 0,
            avg_recovery_duration_ms: 0,
            common_strategies: HashMap::new(),
            avg_success_rate: 0.0,
            avg_health_improvement: 0.0,
            recovery_trends: RecoveryTrends::default(),
        }
    }
}

impl Default for RecoveryTrends {
    fn default() -> Self {
        Self {
            success_rate_trend: TrendDirection::Unknown,
            duration_trend: TrendDirection::Unknown,
            frequency_trend: TrendDirection::Unknown,
            health_improvement_trend: TrendDirection::Unknown,
        }
    }
}

impl Default for RecoveryStatistics {
    fn default() -> Self {
        Self {
            total_recoveries: 0,
            successful_recoveries: 0,
            success_rate: 0.0,
            avg_recovery_duration_ms: 0,
            avg_success_rate: 0.0,
            avg_health_improvement: 0.0,
            strategy_usage: HashMap::new(),
            recent_recoveries: Vec::new(),
        }
    }
}

impl Default for RecoveryMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
