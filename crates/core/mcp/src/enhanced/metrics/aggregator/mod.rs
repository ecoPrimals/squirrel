// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Metrics Aggregator
//!
//! This module processes and aggregates metrics from the unified collector,
//! providing statistical analysis, trend detection, and performance insights.

pub mod performance;
pub mod statistics;
pub mod analysis;
pub mod trends;

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Mutex};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{debug, info, warn, error, instrument};

use crate::error::{Result, types::MCPError};
use super::collector::{UnifiedMetrics, MetricValue};
use super::MetricsConfig;

pub use performance::*;
pub use statistics::*;
pub use analysis::*;
pub use trends::*;

/// Metrics aggregator for processing collected metrics
#[derive(Debug)]
pub struct MetricsAggregator {
    /// Raw metrics history
    raw_metrics_history: Arc<RwLock<VecDeque<UnifiedMetrics>>>,
    
    /// Current aggregated metrics
    current_aggregation: Arc<RwLock<AggregatedMetrics>>,
    
    /// Historical aggregations
    historical_aggregations: Arc<RwLock<VecDeque<AggregatedMetrics>>>,
    
    /// Trend analyzer
    trend_analyzer: Arc<TrendAnalyzer>,
    
    /// Performance analyzer
    performance_analyzer: Arc<PerformanceAnalyzer>,
    
    /// Configuration
    config: MetricsConfig,
    
    /// Aggregator state
    state: Arc<RwLock<AggregatorState>>,
}

/// Aggregated metrics with statistical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    /// Aggregation timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Aggregation time window
    pub time_window: Duration,
    
    /// Total samples processed
    pub sample_count: usize,
    
    /// Metric values (key-value pairs)
    pub metric_values: HashMap<String, f64>,
    
    /// Statistical summaries for each metric
    pub metric_statistics: HashMap<String, MetricStatistics>,
    
    /// Overall performance metrics
    pub overall_performance: OverallPerformance,
    
    /// Per-component performance breakdown
    pub component_performance: HashMap<String, ComponentPerformance>,
    
    /// Resource utilization metrics
    pub resource_utilization: ResourceUtilization,
    
    /// Error analysis
    pub error_analysis: ErrorAnalysis,
    
    /// Throughput analysis
    pub throughput_analysis: ThroughputAnalysis,
    
    /// Latency analysis
    pub latency_analysis: LatencyAnalysis,
    
    /// Trend information
    pub trends: HashMap<String, TrendInfo>,
    
    /// Aggregation metadata
    pub metadata: AggregationMetadata,
}

/// Statistical summary for a metric
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetricStatistics {
    /// Minimum value
    pub min: f64,
    
    /// Maximum value  
    pub max: f64,
    
    /// Average value
    pub mean: f64,
    
    /// Standard deviation
    pub std_dev: f64,
    
    /// Median value
    pub median: f64,
    
    /// 95th percentile
    pub p95: f64,
    
    /// 99th percentile
    pub p99: f64,
    
    /// Sample count
    pub count: usize,
}

/// Aggregation metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AggregationMetadata {
    /// Aggregation start time
    pub start_time: DateTime<Utc>,
    
    /// Aggregation end time
    pub end_time: DateTime<Utc>,
    
    /// Processing duration
    pub processing_duration: Duration,
    
    /// Data quality score (0.0 to 1.0)
    pub quality_score: f64,
    
    /// Number of missing data points
    pub missing_data_points: usize,
    
    /// Aggregation version
    pub version: String,
    
    /// Additional metadata
    pub additional: HashMap<String, String>,
}

/// Aggregator state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AggregatorState {
    /// Current status
    pub status: AggregatorStatus,
    
    /// Total metrics processed
    pub total_metrics_processed: u64,
    
    /// Processing performance metrics
    pub processing_performance: ProcessingPerformance,
    
    /// Last aggregation timestamp
    pub last_aggregation: Option<DateTime<Utc>>,
    
    /// Error count
    pub error_count: u64,
    
    /// Warning count
    pub warning_count: u64,
}

/// Aggregator status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum AggregatorStatus {
    /// Aggregator is initializing
    #[default]
    Initializing,
    /// Aggregator is active
    Active,
    /// Aggregator is paused
    Paused,
    /// Aggregator has failed
    Failed,
}

/// Processing performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcessingPerformance {
    /// Average processing time per batch
    pub avg_processing_time: Duration,
    
    /// Processing throughput (metrics per second)
    pub throughput: f64,
    
    /// Memory usage for processing
    pub memory_usage_mb: f64,
}

impl MetricsAggregator {
    /// Create a new metrics aggregator
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            raw_metrics_history: Arc::new(RwLock::new(VecDeque::new())),
            current_aggregation: Arc::new(RwLock::new(AggregatedMetrics::default())),
            historical_aggregations: Arc::new(RwLock::new(VecDeque::new())),
            trend_analyzer: Arc::new(TrendAnalyzer::new(TrendAnalysisConfig::default())),
            performance_analyzer: Arc::new(PerformanceAnalyzer::new(PerformanceAnalysisConfig::default())),
            config,
            state: Arc::new(RwLock::new(AggregatorState::default())),
        }
    }

    /// Process and aggregate metrics
    #[instrument(skip(self, metrics))]
    pub async fn aggregate_metrics(&self, metrics: UnifiedMetrics) -> Result<AggregatedMetrics> {
        let start_time = Instant::now();

        debug!("📊 Starting metrics aggregation for timestamp: {:?}", metrics.timestamp);

        // Store raw metrics
        {
            let mut history = self.raw_metrics_history.write().await;
            history.push_back(metrics.clone());
            
            // Keep only recent history
            while history.len() > self.config.max_history_size {
                history.pop_front();
            }
        }

        // Perform aggregation
        let aggregated = self.perform_aggregation(&metrics).await?;

        // Update current aggregation
        {
            let mut current = self.current_aggregation.write().await;
            *current = aggregated.clone();
        }

        // Store historical aggregation
        {
            let mut history = self.historical_aggregations.write().await;
            history.push_back(aggregated.clone());
            
            // Keep only recent history
            while history.len() > self.config.max_aggregation_history {
                history.pop_front();
            }
        }

        // Update processing state
        self.update_processing_state(start_time.elapsed()).await;

        info!("📊 Metrics aggregation completed in {:?}", start_time.elapsed());
        Ok(aggregated)
    }

    /// Perform the actual aggregation
    async fn perform_aggregation(&self, metrics: &UnifiedMetrics) -> Result<AggregatedMetrics> {
        let start_time = Utc::now();
        let processing_start = Instant::now();

        // Initialize aggregated metrics
        let mut aggregated = AggregatedMetrics {
            timestamp: metrics.timestamp,
            time_window: Duration::from_secs(60), // Default 1 minute window
            sample_count: 1,
            ..Default::default()
        };

        // Process metric values and calculate statistics
        for (metric_name, metric_value) in &metrics.metric_values {
            let value = self.extract_numeric_value(metric_value);
            aggregated.metric_values.insert(metric_name.clone(), value);
            
            // For now, use single value statistics
            aggregated.metric_statistics.insert(metric_name.clone(), MetricStatistics {
                min: value,
                max: value,
                mean: value,
                std_dev: 0.0,
                median: value,
                p95: value,
                p99: value,
                count: 1,
            });
        }

        // Perform performance analysis
        aggregated.overall_performance = self.performance_analyzer.analyze_overall_performance(metrics).await;
        aggregated.component_performance = self.performance_analyzer.analyze_component_performance(metrics).await;
        aggregated.resource_utilization = self.analyze_resource_utilization(metrics).await;

        // Perform error analysis
        aggregated.error_analysis = self.analyze_errors(metrics).await;

        // Perform throughput analysis
        aggregated.throughput_analysis = self.analyze_throughput(metrics).await;

        // Perform latency analysis
        aggregated.latency_analysis = self.analyze_latency(metrics).await;

        // Perform trend analysis
        aggregated.trends = self.trend_analyzer.analyze_trends(metrics).await;

        // Set metadata
        aggregated.metadata = AggregationMetadata {
            start_time,
            end_time: Utc::now(),
            processing_duration: processing_start.elapsed(),
            quality_score: 1.0, // Placeholder
            missing_data_points: 0,
            version: "1.0.0".to_string(),
            additional: HashMap::new(),
        };

        Ok(aggregated)
    }

    /// Extract numeric value from MetricValue
    fn extract_numeric_value(&self, value: &MetricValue) -> f64 {
        match value {
            MetricValue::Counter(v) => *v as f64,
            MetricValue::Gauge(v) => *v,
            MetricValue::Histogram { sum, count, .. } => if *count > 0 { *sum / (*count as f64) } else { 0.0 },
            MetricValue::Summary { sum, count, .. } => if *count > 0 { *sum / (*count as f64) } else { 0.0 },
        }
    }

    /// Analyze resource utilization
    async fn analyze_resource_utilization(&self, _metrics: &UnifiedMetrics) -> ResourceUtilization {
        // Placeholder implementation
        ResourceUtilization::default()
    }

    /// Analyze errors
    async fn analyze_errors(&self, _metrics: &UnifiedMetrics) -> ErrorAnalysis {
        // Placeholder implementation
        ErrorAnalysis::default()
    }

    /// Analyze throughput
    async fn analyze_throughput(&self, _metrics: &UnifiedMetrics) -> ThroughputAnalysis {
        // Placeholder implementation
        ThroughputAnalysis::default()
    }

    /// Analyze latency
    async fn analyze_latency(&self, _metrics: &UnifiedMetrics) -> LatencyAnalysis {
        // Placeholder implementation
        LatencyAnalysis::default()
    }

    /// Update processing state
    async fn update_processing_state(&self, processing_duration: Duration) {
        let mut state = self.state.write().await;
        state.status = AggregatorStatus::Active;
        state.total_metrics_processed += 1;
        state.last_aggregation = Some(Utc::now());
        state.processing_performance.avg_processing_time = processing_duration;
        state.processing_performance.throughput = 1.0 / processing_duration.as_secs_f64();
    }

    /// Get current aggregated metrics
    pub async fn get_current_aggregation(&self) -> AggregatedMetrics {
        self.current_aggregation.read().await.clone()
    }

    /// Get aggregator state
    pub async fn get_state(&self) -> AggregatorState {
        self.state.read().await.clone()
    }

    /// Get historical aggregations
    pub async fn get_historical_aggregations(&self, limit: Option<usize>) -> Vec<AggregatedMetrics> {
        let history = self.historical_aggregations.read().await;
        match limit {
            Some(n) => history.iter().rev().take(n).cloned().collect(),
            None => history.iter().cloned().collect(),
        }
    }
}

impl Default for AggregatedMetrics {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            time_window: Duration::from_secs(60),
            sample_count: 0,
            metric_values: HashMap::new(),
            metric_statistics: HashMap::new(),
            overall_performance: OverallPerformance::default(),
            component_performance: HashMap::new(),
            resource_utilization: ResourceUtilization::default(),
            error_analysis: ErrorAnalysis::default(),
            throughput_analysis: ThroughputAnalysis::default(),
            latency_analysis: LatencyAnalysis::default(),
            trends: HashMap::new(),
            metadata: AggregationMetadata::default(),
        }
    }
} 