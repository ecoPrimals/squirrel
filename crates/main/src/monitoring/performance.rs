// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! # Performance Tracking Module
//!
//! This module provides comprehensive performance tracking and analysis for the Squirrel AI ecosystem.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info};

use super::PerformanceSummary;
use crate::error::PrimalError;

/// Performance tracking system
pub struct PerformanceTracker {
    /// Performance metrics storage
    metrics: Arc<RwLock<HashMap<String, PerformanceMetric>>>,
    /// Performance history
    history: Arc<RwLock<VecDeque<PerformanceSnapshot>>>,
    /// Performance baselines
    baselines: Arc<RwLock<HashMap<String, PerformanceBaseline>>>,
    /// Configuration
    config: PerformanceConfig,
}

/// Performance metric data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    /// Metric name
    pub name: String,
    /// Current value
    pub current_value: f64,
    /// Average value
    pub average_value: f64,
    /// Minimum value
    pub min_value: f64,
    /// Maximum value
    pub max_value: f64,
    /// Standard deviation
    pub std_deviation: f64,
    /// Sample count
    pub sample_count: u64,
    /// Last update timestamp
    pub last_update: DateTime<Utc>,
    /// Trend direction
    pub trend: TrendDirection,
}

/// Performance snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    /// Snapshot timestamp
    pub timestamp: DateTime<Utc>,
    /// System performance summary
    pub summary: PerformanceSummary,
    /// Component performance metrics
    pub component_metrics: HashMap<String, HashMap<String, f64>>,
    /// Resource utilization
    pub resource_utilization: ResourceUtilization,
}

/// Performance baseline for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    /// Baseline name
    pub name: String,
    /// Baseline values
    pub values: HashMap<String, f64>,
    /// Baseline timestamp
    pub timestamp: DateTime<Utc>,
    /// Baseline description
    pub description: String,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    /// CPU utilization percentage
    pub cpu_percent: f64,
    /// Memory utilization percentage
    pub memory_percent: f64,
    /// Disk I/O utilization percentage
    pub disk_io_percent: f64,
    /// Network I/O utilization percentage
    pub network_io_percent: f64,
    /// Active threads
    pub active_threads: u32,
    /// File descriptors used
    pub file_descriptors: u32,
    /// Active connections
    pub active_connections: u32,
}

/// Performance trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
    Unknown,
}

/// Performance tracker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum history size
    pub max_history_size: usize,
    /// Sampling window size
    pub sampling_window: Duration,
    /// Trend analysis window
    pub trend_window: Duration,
    /// Performance thresholds
    pub thresholds: HashMap<String, PerformanceThreshold>,
}

/// Performance threshold definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThreshold {
    /// Threshold name
    pub name: String,
    /// Warning threshold
    pub warning_threshold: f64,
    /// Critical threshold
    pub critical_threshold: f64,
    /// Comparison direction (above/below)
    pub direction: ThresholdDirection,
}

/// Threshold comparison direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThresholdDirection {
    Above,
    Below,
}

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceTracker {
    /// Create a new performance tracker
    #[must_use]
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(VecDeque::new())),
            baselines: Arc::new(RwLock::new(HashMap::new())),
            config: PerformanceConfig::default(),
        }
    }

    /// Track performance metrics
    pub async fn track_performance(&self) -> Result<(), PrimalError> {
        debug!("Tracking performance metrics");

        // Collect current performance data
        let summary = self.collect_performance_summary().await?;
        let component_metrics = self.collect_component_performance().await?;
        let resource_utilization = self.collect_resource_utilization().await?;

        // Update metrics
        self.update_metrics(&summary, &component_metrics).await?;

        // Create performance snapshot
        self.create_performance_snapshot(summary, component_metrics, resource_utilization)
            .await?;

        debug!("Performance tracking completed");
        Ok(())
    }

    /// Get performance summary
    pub async fn get_summary(&self) -> Result<PerformanceSummary, PrimalError> {
        self.collect_performance_summary().await
    }

    /// Get performance metric by name
    pub async fn get_metric(&self, name: &str) -> Result<PerformanceMetric, PrimalError> {
        let metrics = self.metrics.read().await;

        if let Some(metric) = metrics.get(name) {
            Ok(metric.clone())
        } else {
            Err(PrimalError::NotFoundError(format!(
                "Performance metric '{name}' not found"
            )))
        }
    }

    /// Get all performance metrics
    pub async fn get_all_metrics(&self) -> Result<HashMap<String, PerformanceMetric>, PrimalError> {
        let metrics = self.metrics.read().await;
        Ok(metrics.clone())
    }

    /// Set performance baseline
    pub async fn set_baseline(&self, name: &str, description: &str) -> Result<(), PrimalError> {
        let summary = self.collect_performance_summary().await?;
        let component_metrics = self.collect_component_performance().await?;

        let mut baseline_values = HashMap::new();

        // Add summary metrics to baseline
        baseline_values.insert("cpu_usage".to_string(), summary.cpu_usage);
        baseline_values.insert("memory_usage".to_string(), summary.memory_usage);
        baseline_values.insert("network_io".to_string(), summary.network_io);
        baseline_values.insert("disk_io".to_string(), summary.disk_io);
        baseline_values.insert("avg_response_time".to_string(), summary.avg_response_time);
        baseline_values.insert(
            "requests_per_second".to_string(),
            summary.requests_per_second,
        );
        baseline_values.insert("error_rate".to_string(), summary.error_rate);

        // Add component metrics to baseline
        for (component, metrics) in component_metrics {
            for (metric_name, value) in metrics {
                let key = format!("{component}_{metric_name}");
                baseline_values.insert(key, value);
            }
        }

        let baseline = PerformanceBaseline {
            name: name.to_string(),
            values: baseline_values,
            timestamp: Utc::now(),
            description: description.to_string(),
        };

        let mut baselines = self.baselines.write().await;
        baselines.insert(name.to_string(), baseline);

        info!("Performance baseline '{}' created", name);
        Ok(())
    }

    /// Compare current performance against baseline
    pub async fn compare_to_baseline(
        &self,
        baseline_name: &str,
    ) -> Result<HashMap<String, f64>, PrimalError> {
        let baselines = self.baselines.read().await;

        if let Some(baseline) = baselines.get(baseline_name) {
            let current_summary = self.collect_performance_summary().await?;
            let current_component_metrics = self.collect_component_performance().await?;

            let mut comparison = HashMap::new();

            // Compare summary metrics
            comparison.insert(
                "cpu_usage".to_string(),
                current_summary.cpu_usage - baseline.values.get("cpu_usage").unwrap_or(&0.0),
            );
            comparison.insert(
                "memory_usage".to_string(),
                current_summary.memory_usage - baseline.values.get("memory_usage").unwrap_or(&0.0),
            );
            comparison.insert(
                "avg_response_time".to_string(),
                current_summary.avg_response_time
                    - baseline.values.get("avg_response_time").unwrap_or(&0.0),
            );
            comparison.insert(
                "requests_per_second".to_string(),
                current_summary.requests_per_second
                    - baseline.values.get("requests_per_second").unwrap_or(&0.0),
            );
            comparison.insert(
                "error_rate".to_string(),
                current_summary.error_rate - baseline.values.get("error_rate").unwrap_or(&0.0),
            );

            // Compare component metrics
            for (component, metrics) in current_component_metrics {
                for (metric_name, current_value) in metrics {
                    let baseline_key = format!("{component}_{metric_name}");
                    if let Some(baseline_value) = baseline.values.get(&baseline_key) {
                        let key = format!("{component}_{metric_name}");
                        comparison.insert(key, current_value - baseline_value);
                    }
                }
            }

            Ok(comparison)
        } else {
            Err(PrimalError::NotFoundError(format!(
                "Baseline '{baseline_name}' not found"
            )))
        }
    }

    /// Get performance trends
    pub async fn get_trends(&self) -> Result<HashMap<String, TrendDirection>, PrimalError> {
        let metrics = self.metrics.read().await;
        let mut trends = HashMap::new();

        for (name, metric) in metrics.iter() {
            trends.insert(name.clone(), metric.trend.clone());
        }

        Ok(trends)
    }

    /// Collect performance summary
    async fn collect_performance_summary(&self) -> Result<PerformanceSummary, PrimalError> {
        // In a real implementation, these would come from actual system monitoring
        // For now, we'll use simulated values with some variation

        let base_time = Utc::now();
        let time_factor = (base_time.timestamp_millis() % 10000) as f64 / 10000.0;

        Ok(PerformanceSummary {
            cpu_usage: 25.0 + (time_factor * 20.0),
            memory_usage: 40.0 + (time_factor * 15.0),
            network_io: 1024.0 * (50.0 + time_factor * 30.0),
            disk_io: 1024.0 * (20.0 + time_factor * 10.0),
            avg_response_time: 120.0 + (time_factor * 50.0),
            requests_per_second: 45.0 + (time_factor * 25.0),
            error_rate: 0.5 + (time_factor * 1.0),
        })
    }

    /// Collect component performance metrics
    async fn collect_component_performance(
        &self,
    ) -> Result<HashMap<String, HashMap<String, f64>>, PrimalError> {
        let mut component_metrics = HashMap::new();

        // Internal components and capability domains
        let components = vec![
            "ai_intelligence",
            "mcp_integration",
            "context_state",
            "agent_deployment",
            "network",  // Network/orchestration capability domain
            "compute",  // Compute capability domain
            "storage",  // Storage capability domain
            "security", // Security capability domain
        ];

        for component in components {
            let metrics = self
                .collect_component_specific_performance(component)
                .await?;
            component_metrics.insert(component.to_string(), metrics);
        }

        Ok(component_metrics)
    }

    /// Collect performance metrics for a specific component
    async fn collect_component_specific_performance(
        &self,
        component: &str,
    ) -> Result<HashMap<String, f64>, PrimalError> {
        let mut metrics = HashMap::new();
        let time_factor = (Utc::now().timestamp_millis() % 5000) as f64 / 5000.0;

        match component {
            "ai_intelligence" => {
                metrics.insert("processing_time".to_string(), 150.0 + time_factor * 50.0);
                metrics.insert("throughput".to_string(), 42.0 + time_factor * 10.0);
                metrics.insert("memory_usage".to_string(), 256.0 + time_factor * 100.0);
                metrics.insert("cpu_usage".to_string(), 35.0 + time_factor * 15.0);
            }
            "mcp_integration" => {
                metrics.insert("message_latency".to_string(), 25.0 + time_factor * 10.0);
                metrics.insert("throughput".to_string(), 128.0 + time_factor * 30.0);
                metrics.insert("connection_pool_usage".to_string(), 0.6 + time_factor * 0.3);
                metrics.insert("protocol_overhead".to_string(), 12.0 + time_factor * 5.0);
            }
            "context_state" => {
                metrics.insert("retrieval_time".to_string(), 15.0 + time_factor * 8.0);
                metrics.insert("cache_hit_rate".to_string(), 0.85 + time_factor * 0.1);
                metrics.insert("storage_usage".to_string(), 1024.0 + time_factor * 200.0);
                metrics.insert("session_count".to_string(), 8.0 + time_factor * 4.0);
            }
            "agent_deployment" => {
                metrics.insert("deployment_time".to_string(), 30.0 + time_factor * 15.0);
                metrics.insert("success_rate".to_string(), 0.95 + time_factor * 0.04);
                metrics.insert("resource_usage".to_string(), 0.7 + time_factor * 0.2);
                metrics.insert("agent_count".to_string(), 12.0 + time_factor * 5.0);
            }
            // Capability-domain metrics (agnostic -- no primal names)
            // These zero-initialize; actual values come from the discovered provider's
            // metrics endpoint at runtime.
            "network" => {
                metrics.insert("orchestration_latency".to_string(), 0.0);
                metrics.insert("service_discovery_time".to_string(), 0.0);
                metrics.insert("load_balancer_efficiency".to_string(), 0.0);
                metrics.insert("health_check_time".to_string(), 0.0);
            }
            "compute" => {
                metrics.insert("job_processing_time".to_string(), 0.0);
                metrics.insert("queue_depth".to_string(), 0.0);
                metrics.insert("cpu_utilization".to_string(), 0.0);
                metrics.insert("throughput".to_string(), 0.0);
            }
            "storage" => {
                metrics.insert("storage_latency".to_string(), 0.0);
                metrics.insert("io_throughput".to_string(), 0.0);
                metrics.insert("storage_efficiency".to_string(), 0.0);
                metrics.insert("backup_time".to_string(), 0.0);
            }
            "security" => {
                metrics.insert("auth_latency".to_string(), 0.0);
                metrics.insert("auth_success_rate".to_string(), 0.0);
                metrics.insert("token_processing_time".to_string(), 0.0);
                metrics.insert("security_overhead".to_string(), 0.0);
            }
            _ => {
                metrics.insert("response_time".to_string(), 100.0 + time_factor * 20.0);
                metrics.insert("availability".to_string(), 0.99 + time_factor * 0.009);
            }
        }

        Ok(metrics)
    }

    /// Collect resource utilization metrics
    async fn collect_resource_utilization(&self) -> Result<ResourceUtilization, PrimalError> {
        let time_factor = (Utc::now().timestamp_millis() % 8000) as f64 / 8000.0;

        Ok(ResourceUtilization {
            cpu_percent: 35.0 + time_factor * 20.0,
            memory_percent: 45.0 + time_factor * 15.0,
            disk_io_percent: 25.0 + time_factor * 10.0,
            network_io_percent: 15.0 + time_factor * 8.0,
            active_threads: (150.0 + time_factor * 50.0) as u32,
            file_descriptors: (800.0 + time_factor * 200.0) as u32,
            active_connections: (25.0 + time_factor * 10.0) as u32,
        })
    }

    /// Update performance metrics
    async fn update_metrics(
        &self,
        summary: &PerformanceSummary,
        component_metrics: &HashMap<String, HashMap<String, f64>>,
    ) -> Result<(), PrimalError> {
        let mut metrics = self.metrics.write().await;

        // Update summary metrics
        self.update_metric(&mut metrics, "cpu_usage", summary.cpu_usage)
            .await;
        self.update_metric(&mut metrics, "memory_usage", summary.memory_usage)
            .await;
        self.update_metric(&mut metrics, "network_io", summary.network_io)
            .await;
        self.update_metric(&mut metrics, "disk_io", summary.disk_io)
            .await;
        self.update_metric(&mut metrics, "avg_response_time", summary.avg_response_time)
            .await;
        self.update_metric(
            &mut metrics,
            "requests_per_second",
            summary.requests_per_second,
        )
        .await;
        self.update_metric(&mut metrics, "error_rate", summary.error_rate)
            .await;

        // Update component metrics
        for (component, comp_metrics) in component_metrics {
            for (metric_name, value) in comp_metrics {
                let key = format!("{component}_{metric_name}");
                self.update_metric(&mut metrics, &key, *value).await;
            }
        }

        Ok(())
    }

    /// Update a specific metric
    async fn update_metric(
        &self,
        metrics: &mut HashMap<String, PerformanceMetric>,
        name: &str,
        value: f64,
    ) {
        if let Some(metric) = metrics.get_mut(name) {
            // Update existing metric
            metric.current_value = value;
            metric.sample_count += 1;

            // Update min/max
            if value < metric.min_value {
                metric.min_value = value;
            }
            if value > metric.max_value {
                metric.max_value = value;
            }

            // Update average (simple moving average)
            metric.average_value = (metric.average_value * (metric.sample_count - 1) as f64
                + value)
                / metric.sample_count as f64;

            // Update trend (simplified)
            if value > metric.average_value * 1.1 {
                metric.trend = TrendDirection::Degrading;
            } else if value < metric.average_value * 0.9 {
                metric.trend = TrendDirection::Improving;
            } else {
                metric.trend = TrendDirection::Stable;
            }

            metric.last_update = Utc::now();
        } else {
            // Create new metric
            metrics.insert(
                name.to_string(),
                PerformanceMetric {
                    name: name.to_string(),
                    current_value: value,
                    average_value: value,
                    min_value: value,
                    max_value: value,
                    std_deviation: 0.0,
                    sample_count: 1,
                    last_update: Utc::now(),
                    trend: TrendDirection::Unknown,
                },
            );
        }
    }

    /// Create performance snapshot
    async fn create_performance_snapshot(
        &self,
        summary: PerformanceSummary,
        component_metrics: HashMap<String, HashMap<String, f64>>,
        resource_utilization: ResourceUtilization,
    ) -> Result<(), PrimalError> {
        let snapshot = PerformanceSnapshot {
            timestamp: Utc::now(),
            summary,
            component_metrics,
            resource_utilization,
        };

        let mut history = self.history.write().await;
        history.push_back(snapshot);

        // Limit history size
        if history.len() > self.config.max_history_size {
            history.pop_front();
        }

        Ok(())
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_history_size: 1000,
            sampling_window: Duration::from_secs(60),
            trend_window: Duration::from_secs(300),
            thresholds: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_tracker_creation() {
        let tracker = PerformanceTracker::new();
        assert!(tracker.metrics.read().await.is_empty());
        assert!(tracker.history.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_performance_tracking() {
        let tracker = PerformanceTracker::new();

        let result = tracker.track_performance().await;
        assert!(result.is_ok());

        let metrics = tracker.metrics.read().await;
        assert!(!metrics.is_empty());
        assert!(metrics.contains_key("cpu_usage"));
        assert!(metrics.contains_key("memory_usage"));

        let history = tracker.history.read().await;
        assert_eq!(history.len(), 1);
    }

    #[tokio::test]
    async fn test_performance_summary() {
        let tracker = PerformanceTracker::new();

        let summary = tracker.get_summary().await.unwrap();
        assert!(summary.cpu_usage > 0.0);
        assert!(summary.memory_usage > 0.0);
        assert!(summary.avg_response_time > 0.0);
    }

    #[tokio::test]
    async fn test_baseline_creation() {
        let tracker = PerformanceTracker::new();

        let result = tracker.set_baseline("test_baseline", "Test baseline").await;
        assert!(result.is_ok());

        let baselines = tracker.baselines.read().await;
        assert!(baselines.contains_key("test_baseline"));
    }

    #[tokio::test]
    async fn test_baseline_comparison() {
        let tracker = PerformanceTracker::new();

        // Create a baseline
        tracker
            .set_baseline("test_baseline", "Test baseline")
            .await
            .unwrap();

        // Compare to baseline (no sleep needed -- values come from current snapshot)
        let comparison = tracker.compare_to_baseline("test_baseline").await.unwrap();
        assert!(!comparison.is_empty());
        assert!(comparison.contains_key("cpu_usage"));
    }

    #[tokio::test]
    async fn test_metric_retrieval() {
        let tracker = PerformanceTracker::new();

        // Track performance to create metrics
        tracker.track_performance().await.unwrap();

        let metric = tracker.get_metric("cpu_usage").await.unwrap();
        assert_eq!(metric.name, "cpu_usage");
        assert!(metric.current_value > 0.0);
        assert!(metric.sample_count > 0);
    }
}
