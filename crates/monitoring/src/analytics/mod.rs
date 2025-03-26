//! Analytics module for the monitoring system
//!
//! This module provides advanced analytics capabilities including:
//! - Time series analysis
//! - Trend detection
//! - Pattern recognition
//! - Predictive analytics
//! - Visualization generation

pub mod time_series;
pub mod trend_detection;
pub mod pattern_recognition;
pub mod predictive;
pub mod storage;
pub mod visualization;

// Re-exports for common types
pub use time_series::TimeSeriesAnalyzer;
pub use trend_detection::TrendDetector;
pub use pattern_recognition::PatternRecognizer;
pub use predictive::PredictiveAnalyzer;
pub use storage::AnalyticsStorage;
pub use visualization::VisualizationGenerator;

use std::fmt::Debug;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use chrono::{DateTime, Utc};
use anyhow::Result;
use tracing::{info, error, debug};

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    /// Time series configuration
    pub time_series: time_series::TimeSeriesConfig,
    /// Trend detection configuration
    pub trend_detection: trend_detection::TrendDetectionConfig,
    /// Pattern recognition configuration
    pub pattern_recognition: pattern_recognition::PatternRecognitionConfig,
    /// Predictive analytics configuration
    pub predictive: predictive::PredictiveConfig,
    /// Storage configuration
    pub storage: storage::StorageConfig,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            time_series: time_series::TimeSeriesConfig::default(),
            trend_detection: trend_detection::TrendDetectionConfig::default(),
            pattern_recognition: pattern_recognition::PatternRecognitionConfig::default(),
            predictive: predictive::PredictiveConfig::default(),
            storage: storage::StorageConfig::default(),
        }
    }
}

/// Analytics data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsDataPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Value
    pub value: f64,
    /// Metadata
    pub metadata: Option<Value>,
}

/// Analytics time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsTimeSeries {
    /// Series name
    pub name: String,
    /// Data points
    pub data: Vec<AnalyticsDataPoint>,
    /// Metadata
    pub metadata: Option<Value>,
}

/// Analytics result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsResult {
    /// Result ID
    pub id: String,
    /// Analysis type
    pub analysis_type: String,
    /// Start time
    pub start_time: DateTime<Utc>,
    /// End time
    pub end_time: DateTime<Utc>,
    /// Result data
    pub data: Value,
    /// Metadata
    pub metadata: Option<Value>,
}

/// Analytics errors
#[derive(Debug, Error)]
pub enum AnalyticsError {
    /// Time series error
    #[error("Time series error: {0}")]
    TimeSeriesError(String),
    
    /// Trend detection error
    #[error("Trend detection error: {0}")]
    TrendDetectionError(String),
    
    /// Pattern recognition error
    #[error("Pattern recognition error: {0}")]
    PatternRecognitionError(String),
    
    /// Predictive analytics error
    #[error("Predictive analytics error: {0}")]
    PredictiveError(String),
    
    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),
    
    /// Visualization error
    #[error("Visualization error: {0}")]
    VisualizationError(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Create an analytics service
pub fn create_analytics_service(config: AnalyticsConfig) -> Result<AnalyticsService> {
    let time_series_analyzer = std::sync::Arc::new(TimeSeriesAnalyzer::with_config(config.time_series.clone()));
    let trend_detector = std::sync::Arc::new(TrendDetector::with_config(config.trend_detection.clone()));
    let pattern_recognizer = std::sync::Arc::new(PatternRecognizer::with_config(config.pattern_recognition.clone()));
    let predictive_analyzer = std::sync::Arc::new(PredictiveAnalyzer::with_config(config.predictive.clone()));
    let visualization_generator = std::sync::Arc::new(VisualizationGenerator::new());
    let storage = std::sync::Arc::new(AnalyticsStorage::with_config(config.storage.clone()));
    
    let service = AnalyticsService {
        config,
        time_series_analyzer,
        trend_detector,
        pattern_recognizer,
        predictive_analyzer,
        visualization_generator,
        storage,
    };
    
    Ok(service)
}

/// Analytics service
#[derive(Debug)]
pub struct AnalyticsService {
    /// Configuration
    config: AnalyticsConfig,
    /// Time series analyzer
    time_series_analyzer: std::sync::Arc<TimeSeriesAnalyzer>,
    /// Trend detector
    trend_detector: std::sync::Arc<TrendDetector>,
    /// Pattern recognizer
    pattern_recognizer: std::sync::Arc<PatternRecognizer>,
    /// Predictive analyzer
    predictive_analyzer: std::sync::Arc<PredictiveAnalyzer>,
    /// Visualization generator
    visualization_generator: std::sync::Arc<VisualizationGenerator>,
    /// Storage
    storage: std::sync::Arc<AnalyticsStorage>,
}

impl AnalyticsService {
    /// Create a new analytics service with default configuration
    pub fn new() -> Self {
        create_analytics_service(AnalyticsConfig::default()).unwrap()
    }
    
    /// Analyze time series data
    pub async fn analyze_time_series(&self, data: Value) -> Result<Value> {
        self.time_series_analyzer.analyze(data).await
    }
    
    /// Detect trends in data
    pub async fn detect_trends(&self, data: Value) -> Result<Value> {
        self.trend_detector.detect_trends(data).await
    }
    
    /// Recognize patterns in data
    pub async fn recognize_patterns(&self, data: Value) -> Result<Value> {
        self.pattern_recognizer.recognize_patterns(data).await
    }
    
    /// Generate predictions
    pub async fn predict(&self, data: Value) -> Result<Value> {
        self.predictive_analyzer.predict(data).await
    }
    
    /// Generate visualization data
    pub async fn generate_visualizations(&self, data: Value) -> Result<Value> {
        self.visualization_generator.generate_visualizations(data).await
    }
    
    /// Store an analytics result
    pub async fn store_result(&self, result: AnalyticsResult) -> Result<()> {
        self.storage.store_result(result).await
    }
    
    /// Get an analytics result by ID
    pub async fn get_result(&self, id: &str) -> Result<Option<AnalyticsResult>> {
        self.storage.get_result(id).await
    }
}

impl Default for AnalyticsService {
    fn default() -> Self {
        Self::new()
    }
} 