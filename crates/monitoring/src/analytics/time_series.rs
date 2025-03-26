//! Time series analysis for the analytics module.
//!
//! This module provides functionality for analyzing time-series data,
//! including aggregation, sampling, and statistical analysis.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration};

use crate::analytics::storage::{AnalyticsStorage, StorageError};
use crate::analytics::AnalyticsError;

/// A single data point in a time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// The ID of the component this data point belongs to
    pub component_id: String,
    
    /// The name of the metric this data point belongs to
    pub metric_name: String,
    
    /// The value of the data point
    pub value: f64,
    
    /// The timestamp of the data point in milliseconds since the Unix epoch
    pub timestamp: i64,
}

/// A time window for time series analysis
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TimeWindow {
    /// Last n minutes
    Minutes(u32),
    
    /// Last n hours
    Hours(u32),
    
    /// Last n days
    Days(u32),
    
    /// Last n weeks
    Weeks(u32),
    
    /// Last n months
    Months(u32),
    
    /// Custom time range in milliseconds
    Custom {
        /// Start time in milliseconds since the Unix epoch
        start: i64,
        
        /// End time in milliseconds since the Unix epoch
        end: i64,
    },
}

impl TimeWindow {
    /// Convert the time window to a start and end timestamp
    pub fn to_timestamps(&self) -> (i64, i64) {
        let now = Utc::now().timestamp_millis();
        
        match self {
            TimeWindow::Minutes(n) => {
                let duration = Duration::minutes(*n as i64);
                let start = now - duration.num_milliseconds();
                (start, now)
            },
            TimeWindow::Hours(n) => {
                let duration = Duration::hours(*n as i64);
                let start = now - duration.num_milliseconds();
                (start, now)
            },
            TimeWindow::Days(n) => {
                let duration = Duration::days(*n as i64);
                let start = now - duration.num_milliseconds();
                (start, now)
            },
            TimeWindow::Weeks(n) => {
                let duration = Duration::weeks(*n as i64);
                let start = now - duration.num_milliseconds();
                (start, now)
            },
            TimeWindow::Months(n) => {
                // Approximate a month as 30 days
                let duration = Duration::days(30 * (*n as i64));
                let start = now - duration.num_milliseconds();
                (start, now)
            },
            TimeWindow::Custom { start, end } => (*start, *end),
        }
    }
}

/// Method for aggregating time series data
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AggregationMethod {
    /// Mean of values in the time window
    Mean,
    
    /// Median of values in the time window
    Median,
    
    /// Maximum value in the time window
    Max,
    
    /// Minimum value in the time window
    Min,
    
    /// Sum of values in the time window
    Sum,
    
    /// Count of values in the time window
    Count,
    
    /// Standard deviation of values in the time window
    StdDev,
    
    /// Percentile of values in the time window
    Percentile(f64),
}

/// Configuration for time series analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesConfig {
    /// Maximum data points to return in a result
    pub max_data_points: usize,
    
    /// Whether to interpolate missing data points
    pub interpolate_missing: bool,
    
    /// Default aggregation method
    pub default_aggregation: AggregationMethod,
    
    /// Maximum time window for analysis in milliseconds
    pub max_time_window: i64,
}

impl Default for TimeSeriesConfig {
    fn default() -> Self {
        Self {
            max_data_points: 1000,
            interpolate_missing: true,
            default_aggregation: AggregationMethod::Mean,
            max_time_window: 30 * 24 * 60 * 60 * 1000, // 30 days in milliseconds
        }
    }
}

/// Result of a time series analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// The component ID
    pub component_id: String,
    
    /// The metric name
    pub metric_name: String,
    
    /// The time window
    pub window: TimeWindow,
    
    /// The aggregation method
    pub method: AggregationMethod,
    
    /// The aggregated data points
    pub data: Vec<DataPoint>,
    
    /// Statistics about the data
    pub statistics: Statistics,
    
    /// The timestamp when the analysis was performed
    pub timestamp: i64,
}

/// Statistics about time series data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    /// The mean value
    pub mean: f64,
    
    /// The median value
    pub median: f64,
    
    /// The minimum value
    pub min: f64,
    
    /// The maximum value
    pub max: f64,
    
    /// The standard deviation
    pub std_dev: f64,
    
    /// The 95th percentile
    pub percentile_95: f64,
    
    /// The number of data points
    pub count: usize,
    
    /// The sum of values
    pub sum: f64,
}

/// Time series analyzer for analyzing time-series data
pub struct TimeSeriesAnalyzer {
    /// Configuration for time series analysis
    config: TimeSeriesConfig,
    
    /// Storage for time series data
    storage: Arc<RwLock<AnalyticsStorage>>,
}

impl TimeSeriesAnalyzer {
    /// Create a new time series analyzer with the given configuration and storage
    pub fn new(config: TimeSeriesConfig, storage: Arc<RwLock<AnalyticsStorage>>) -> Result<Self, AnalyticsError> {
        Ok(Self {
            config,
            storage,
        })
    }
    
    /// Analyze time series data for a specific component and metric
    pub async fn analyze(&self, component_id: &str, metric_name: &str, 
        window: TimeWindow, method: AggregationMethod) 
        -> Result<AnalysisResult, AnalyticsError> 
    {
        // Get the data
        let data = self.get_data(component_id, metric_name, window).await?;
        
        // Calculate statistics
        let statistics = self.calculate_statistics(&data)?;
        
        // Create the result
        let result = AnalysisResult {
            component_id: component_id.to_string(),
            metric_name: metric_name.to_string(),
            window,
            method,
            data,
            statistics,
            timestamp: Utc::now().timestamp_millis(),
        };
        
        Ok(result)
    }
    
    /// Get time series data for a specific component and metric
    pub async fn get_data(&self, component_id: &str, metric_name: &str, window: TimeWindow) 
        -> Result<Vec<DataPoint>, AnalyticsError> 
    {
        let (start, end) = window.to_timestamps();
        
        // Check if the time window is too large
        if end - start > self.config.max_time_window {
            return Err(AnalyticsError::ConfigError(
                format!("Time window too large: {} ms (max: {} ms)", end - start, self.config.max_time_window)
            ));
        }
        
        // Get data from storage
        let storage = self.storage.read().await;
        let data = storage.get_data_points(component_id, metric_name, start, end).await
            .map_err(|e| match e {
                StorageError::IoError(e) => AnalyticsError::IoError(e),
                StorageError::SerializationError(e) => AnalyticsError::SerializationError(e),
                StorageError::NotFound(e) => AnalyticsError::AnalysisError(e),
                StorageError::Other(e) => AnalyticsError::StorageError(e),
            })?;
        
        // If there's too much data, downsample it
        let data = if data.len() > self.config.max_data_points {
            self.downsample(data, self.config.max_data_points)
        } else {
            data
        };
        
        // If interpolation is enabled, interpolate missing data points
        let data = if self.config.interpolate_missing {
            self.interpolate(data, start, end)
        } else {
            data
        };
        
        Ok(data)
    }
    
    /// Calculate statistics for time series data
    fn calculate_statistics(&self, data: &[DataPoint]) -> Result<Statistics, AnalyticsError> {
        if data.is_empty() {
            return Err(AnalyticsError::AnalysisError("No data to calculate statistics".to_string()));
        }
        
        let count = data.len();
        let sum: f64 = data.iter().map(|dp| dp.value).sum();
        let mean = sum / count as f64;
        
        // Calculate standard deviation
        let variance = data.iter()
            .map(|dp| (dp.value - mean).powi(2))
            .sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();
        
        // Sort values for median and percentile calculations
        let mut values: Vec<f64> = data.iter().map(|dp| dp.value).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let min = values.first().copied().unwrap_or(0.0);
        let max = values.last().copied().unwrap_or(0.0);
        
        // Calculate median
        let median = if count % 2 == 0 {
            (values[count / 2 - 1] + values[count / 2]) / 2.0
        } else {
            values[count / 2]
        };
        
        // Calculate 95th percentile
        let p95_index = (0.95 * count as f64).floor() as usize;
        let percentile_95 = values.get(p95_index).copied().unwrap_or(0.0);
        
        Ok(Statistics {
            mean,
            median,
            min,
            max,
            std_dev,
            percentile_95,
            count,
            sum,
        })
    }
    
    /// Downsample time series data to a maximum number of data points
    fn downsample(&self, data: Vec<DataPoint>, max_points: usize) -> Vec<DataPoint> {
        if data.len() <= max_points {
            return data;
        }
        
        let step = data.len() / max_points;
        let mut result = Vec::with_capacity(max_points);
        
        for i in (0..data.len()).step_by(step) {
            result.push(data[i].clone());
            
            if result.len() >= max_points {
                break;
            }
        }
        
        // Ensure we include the last data point
        if !data.is_empty() && (result.is_empty() || result.last().unwrap().timestamp < data.last().unwrap().timestamp) {
            result.push(data.last().unwrap().clone());
        }
        
        result
    }
    
    /// Interpolate missing data points in time series data
    fn interpolate(&self, data: Vec<DataPoint>, start: i64, end: i64) -> Vec<DataPoint> {
        if data.is_empty() {
            return data;
        }
        
        // If there's only one data point, return it without interpolation
        if data.len() == 1 {
            return data;
        }
        
        // Calculate the average interval between data points
        let total_time = end - start;
        let avg_interval = total_time / self.config.max_data_points as i64;
        
        let mut result = Vec::with_capacity(self.config.max_data_points);
        let mut current_time = start;
        
        // Add interpolated data points
        let mut i = 0;
        while current_time < end && result.len() < self.config.max_data_points {
            // Find the two data points to interpolate between
            while i + 1 < data.len() && data[i + 1].timestamp < current_time {
                i += 1;
            }
            
            if i + 1 >= data.len() {
                // We're past the last data point, stop interpolating
                break;
            }
            
            let dp1 = &data[i];
            let dp2 = &data[i + 1];
            
            // Calculate the interpolated value
            let t = (current_time - dp1.timestamp) as f64 / (dp2.timestamp - dp1.timestamp) as f64;
            let value = dp1.value + t * (dp2.value - dp1.value);
            
            // Add the interpolated data point
            result.push(DataPoint {
                component_id: dp1.component_id.clone(),
                metric_name: dp1.metric_name.clone(),
                value,
                timestamp: current_time,
            });
            
            // Move to the next time point
            current_time += avg_interval;
        }
        
        // Ensure we include the last data point
        if !data.is_empty() && (result.is_empty() || result.last().unwrap().timestamp < data.last().unwrap().timestamp) {
            result.push(data.last().unwrap().clone());
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::RwLock;
    use std::sync::Arc;
    use crate::analytics::storage::{AnalyticsStorage, StorageConfig};
    
    async fn create_test_analyzer() -> TimeSeriesAnalyzer {
        let storage_config = StorageConfig::default();
        let storage = Arc::new(RwLock::new(
            AnalyticsStorage::new(storage_config).await.unwrap()
        ));
        
        let config = TimeSeriesConfig::default();
        TimeSeriesAnalyzer::new(config, storage).unwrap()
    }
    
    #[tokio::test]
    async fn test_time_window_to_timestamps() {
        let window = TimeWindow::Minutes(10);
        let (start, end) = window.to_timestamps();
        assert!(end - start <= 10 * 60 * 1000 + 1); // Allow for a small difference due to timing
    }
    
    #[tokio::test]
    async fn test_downsample() {
        let analyzer = create_test_analyzer().await;
        
        // Create test data
        let mut data = Vec::new();
        for i in 0..1000 {
            data.push(DataPoint {
                component_id: "test".to_string(),
                metric_name: "test".to_string(),
                value: i as f64,
                timestamp: i as i64,
            });
        }
        
        let downsampled = analyzer.downsample(data, 100);
        assert!(downsampled.len() <= 100);
    }
} 