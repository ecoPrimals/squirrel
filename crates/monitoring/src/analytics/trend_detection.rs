//! Trend detection module for the analytics system.
//!
//! This module provides functionality for analyzing time series data
//! to detect trends and patterns over time.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::Utc;

use crate::analytics::time_series::{TimeSeriesAnalyzer, TimeWindow, DataPoint};
use crate::analytics::AnalyticsError;

/// Type of trend detected in the data
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TrendType {
    /// Increasing trend (values are going up)
    Increasing,
    
    /// Decreasing trend (values are going down)
    Decreasing,
    
    /// Stable trend (values are not changing significantly)
    Stable,
    
    /// Volatile trend (values are changing rapidly)
    Volatile,
    
    /// Cyclical trend (values show repeating patterns)
    Cyclical,
    
    /// Spike (sudden increase followed by a return to normal)
    Spike,
    
    /// Drop (sudden decrease followed by a return to normal)
    Drop,
    
    /// Anomaly (value is significantly different from the expected range)
    Anomaly,
}

/// Configuration for trend detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendDetectionConfig {
    /// Minimum number of data points required for trend detection
    pub min_data_points: usize,
    
    /// Threshold for determining if a change is significant (0.0 - 1.0)
    pub significance_threshold: f64,
    
    /// Number of time windows to analyze for trend detection
    pub analysis_windows: Vec<TimeWindow>,
    
    /// Threshold for volatility detection
    pub volatility_threshold: f64,
    
    /// Threshold for anomaly detection (standard deviations from mean)
    pub anomaly_threshold: f64,
    
    /// Threshold for spike/drop detection (percent change)
    pub spike_threshold: f64,
}

impl Default for TrendDetectionConfig {
    fn default() -> Self {
        Self {
            min_data_points: 10,
            significance_threshold: 0.05,
            analysis_windows: vec![
                TimeWindow::Minutes(5),
                TimeWindow::Minutes(15),
                TimeWindow::Hours(1),
                TimeWindow::Hours(24),
            ],
            volatility_threshold: 0.15,
            anomaly_threshold: 3.0,
            spike_threshold: 0.25,
        }
    }
}

/// A detected trend in the data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trend {
    /// The component ID
    pub component_id: String,
    
    /// The metric name
    pub metric_name: String,
    
    /// The type of trend detected
    pub trend_type: TrendType,
    
    /// The time window the trend was detected in
    pub window: TimeWindow,
    
    /// The confidence level of the trend detection (0.0 - 1.0)
    pub confidence: f64,
    
    /// The magnitude of the trend (percent change)
    pub magnitude: f64,
    
    /// The start timestamp of the trend
    pub start_timestamp: i64,
    
    /// The end timestamp of the trend
    pub end_timestamp: i64,
    
    /// The timestamp when the trend was detected
    pub detection_timestamp: i64,
    
    /// Additional details about the trend
    pub details: serde_json::Value,
}

/// Detector for trends in time series data
#[derive(Debug)]
pub struct TrendDetector {
    /// Configuration for trend detection
    config: TrendDetectionConfig,
    
    /// Time series analyzer for accessing historical data
    time_series_analyzer: Arc<RwLock<TimeSeriesAnalyzer>>,
}

impl TrendDetector {
    /// Create a new trend detector with the given configuration
    pub fn new(config: TrendDetectionConfig, 
               time_series_analyzer: Arc<RwLock<TimeSeriesAnalyzer>>) 
        -> Result<Self, AnalyticsError> 
    {
        Ok(Self {
            config,
            time_series_analyzer,
        })
    }
    
    /// Detect trends for a specific component and metric
    pub async fn detect_trends(&self, component_id: &str, metric_name: &str) 
        -> Result<Vec<Trend>, AnalyticsError> 
    {
        let mut trends = Vec::new();
        
        // Analyze each time window
        for window in &self.config.analysis_windows {
            // Get the data for this window
            let analyzer = self.time_series_analyzer.read().await;
            let data = analyzer.get_data(component_id, metric_name, *window).await?;
            
            // Check if we have enough data points
            if data.len() < self.config.min_data_points {
                // Not enough data for this window, skip it
                continue;
            }
            
            // Detect trends in this window
            let window_trends = self.detect_trends_in_window(component_id, metric_name, *window, &data)?;
            trends.extend(window_trends);
        }
        
        Ok(trends)
    }
    
    /// Detect trends in a specific time window
    fn detect_trends_in_window(&self, component_id: &str, metric_name: &str,
                               window: TimeWindow, data: &[DataPoint]) 
        -> Result<Vec<Trend>, AnalyticsError> 
    {
        let mut trends = Vec::new();
        
        // Calculate basic statistics
        let mean = data.iter().map(|dp| dp.value).sum::<f64>() / data.len() as f64;
        let variance = data.iter()
            .map(|dp| (dp.value - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        let std_dev = variance.sqrt();
        
        // Calculate the first and last values
        let first_value = data.first().unwrap().value;
        let last_value = data.last().unwrap().value;
        
        // Calculate the percent change
        let percent_change = if first_value != 0.0 {
            (last_value - first_value) / first_value.abs()
        } else {
            0.0
        };
        
        // Get timestamps
        let start_timestamp = data.first().unwrap().timestamp;
        let end_timestamp = data.last().unwrap().timestamp;
        let detection_timestamp = Utc::now().timestamp_millis();
        
        // Detect increasing/decreasing trends
        if percent_change.abs() > self.config.significance_threshold {
            let trend_type = if percent_change > 0.0 {
                TrendType::Increasing
            } else {
                TrendType::Decreasing
            };
            
            let confidence = self.calculate_confidence(percent_change.abs(), data.len());
            
            trends.push(Trend {
                component_id: component_id.to_string(),
                metric_name: metric_name.to_string(),
                trend_type,
                window,
                confidence,
                magnitude: percent_change.abs(),
                start_timestamp,
                end_timestamp,
                detection_timestamp,
                details: serde_json::json!({
                    "first_value": first_value,
                    "last_value": last_value,
                    "mean": mean,
                    "std_dev": std_dev,
                    "percent_change": percent_change,
                }),
            });
        } else {
            // Stable trend
            trends.push(Trend {
                component_id: component_id.to_string(),
                metric_name: metric_name.to_string(),
                trend_type: TrendType::Stable,
                window,
                confidence: 0.7,
                magnitude: percent_change.abs(),
                start_timestamp,
                end_timestamp,
                detection_timestamp,
                details: serde_json::json!({
                    "first_value": first_value,
                    "last_value": last_value,
                    "mean": mean,
                    "std_dev": std_dev,
                    "percent_change": percent_change,
                }),
            });
        }
        
        // Detect volatility
        let volatility = std_dev / mean.abs();
        if volatility > self.config.volatility_threshold {
            trends.push(Trend {
                component_id: component_id.to_string(),
                metric_name: metric_name.to_string(),
                trend_type: TrendType::Volatile,
                window,
                confidence: self.calculate_confidence(volatility, data.len()),
                magnitude: volatility,
                start_timestamp,
                end_timestamp,
                detection_timestamp,
                details: serde_json::json!({
                    "mean": mean,
                    "std_dev": std_dev,
                    "volatility": volatility,
                }),
            });
        }
        
        // Detect anomalies
        let anomalies = self.detect_anomalies(data, mean, std_dev)?;
        for anomaly in anomalies {
            trends.push(Trend {
                component_id: component_id.to_string(),
                metric_name: metric_name.to_string(),
                trend_type: TrendType::Anomaly,
                window,
                confidence: anomaly.confidence,
                magnitude: anomaly.magnitude,
                start_timestamp: anomaly.timestamp,
                end_timestamp: anomaly.timestamp,
                detection_timestamp,
                details: serde_json::json!({
                    "anomaly_value": anomaly.value,
                    "mean": mean,
                    "std_dev": std_dev,
                    "deviations": anomaly.deviations,
                }),
            });
        }
        
        // Detect spikes and drops
        let spikes_drops = self.detect_spikes_drops(data)?;
        for sd in spikes_drops {
            trends.push(Trend {
                component_id: component_id.to_string(),
                metric_name: metric_name.to_string(),
                trend_type: sd.trend_type,
                window,
                confidence: sd.confidence,
                magnitude: sd.magnitude,
                start_timestamp: sd.start_timestamp,
                end_timestamp: sd.end_timestamp,
                detection_timestamp,
                details: sd.details,
            });
        }
        
        // Detect cyclical patterns
        if let Some(cyclical) = self.detect_cyclical_pattern(data)? {
            trends.push(Trend {
                component_id: component_id.to_string(),
                metric_name: metric_name.to_string(),
                trend_type: TrendType::Cyclical,
                window,
                confidence: cyclical.confidence,
                magnitude: cyclical.magnitude,
                start_timestamp,
                end_timestamp,
                detection_timestamp,
                details: cyclical.details,
            });
        }
        
        Ok(trends)
    }
    
    /// Calculate confidence level based on magnitude and sample size
    fn calculate_confidence(&self, magnitude: f64, sample_size: usize) -> f64 {
        // Ensure magnitude is always positive for confidence calculation
        let abs_magnitude = magnitude.abs();
        
        // Use a function that ensures larger magnitudes produce higher confidence values
        let base_confidence = (abs_magnitude / self.config.significance_threshold).min(1.0);
        
        // Make sample size effect more significant by using a logarithmic scaling
        // This ensures that doubling the sample size always has a noticeable impact
        let sample_ratio = if sample_size <= self.config.min_data_points {
            0.0
        } else {
            (sample_size as f64 / self.config.min_data_points as f64).ln().max(0.0).min(1.0)
        };
        
        // Create a scalar factor that increases with sample size
        let sample_factor = 0.2 + sample_ratio * 0.3; // Range from 0.2 to 0.5
        
        // For the same magnitude, larger sample sizes should give higher confidence
        let combined = base_confidence * (1.0 + sample_factor);
        
        // Ensure the result is in the range [0.0, 1.0]
        combined.clamp(0.0, 1.0)
    }
    
    /// Detect anomalies in the data
    fn detect_anomalies(&self, data: &[DataPoint], mean: f64, std_dev: f64) 
        -> Result<Vec<AnomalyInfo>, AnalyticsError> 
    {
        let mut anomalies = Vec::new();
        
        for dp in data {
            let deviations = (dp.value - mean).abs() / std_dev;
            
            if deviations > self.config.anomaly_threshold {
                let confidence = (deviations / self.config.anomaly_threshold).clamp(0.0, 1.0);
                
                anomalies.push(AnomalyInfo {
                    timestamp: dp.timestamp,
                    value: dp.value,
                    deviations,
                    confidence,
                    magnitude: deviations / self.config.anomaly_threshold,
                });
            }
        }
        
        Ok(anomalies)
    }
    
    /// Detect spikes and drops in the data
    fn detect_spikes_drops(&self, data: &[DataPoint]) 
        -> Result<Vec<Trend>, AnalyticsError> 
    {
        if data.len() < 3 {
            return Ok(Vec::new());
        }
        
        let mut results = Vec::new();
        
        for i in 1..data.len() - 1 {
            let prev = data[i - 1].value;
            let current = data[i].value;
            let next = data[i + 1].value;
            
            // Calculate percent changes
            let up_change = if prev != 0.0 { (current - prev) / prev.abs() } else { 0.0 };
            let down_change = if current != 0.0 { (next - current) / current.abs() } else { 0.0 };
            
            // Detect spike: sharp increase followed by sharp decrease
            if up_change > self.config.spike_threshold && down_change < -self.config.spike_threshold {
                let magnitude = up_change.abs();
                let confidence = (magnitude / self.config.spike_threshold).clamp(0.0, 1.0);
                
                results.push(Trend {
                    component_id: String::new(), // Will be filled by caller
                    metric_name: String::new(),  // Will be filled by caller
                    trend_type: TrendType::Spike,
                    window: TimeWindow::Custom { // Specific time window for this spike
                        start: data[i - 1].timestamp,
                        end: data[i + 1].timestamp,
                    },
                    confidence,
                    magnitude,
                    start_timestamp: data[i - 1].timestamp,
                    end_timestamp: data[i + 1].timestamp,
                    detection_timestamp: 0, // Will be filled by caller
                    details: serde_json::json!({
                        "before_value": prev,
                        "spike_value": current,
                        "after_value": next,
                        "up_change": up_change,
                        "down_change": down_change,
                    }),
                });
            }
            
            // Detect drop: sharp decrease followed by sharp increase
            if up_change < -self.config.spike_threshold && down_change > self.config.spike_threshold {
                let magnitude = up_change.abs();
                let confidence = (magnitude / self.config.spike_threshold).clamp(0.0, 1.0);
                
                results.push(Trend {
                    component_id: String::new(), // Will be filled by caller
                    metric_name: String::new(),  // Will be filled by caller
                    trend_type: TrendType::Drop,
                    window: TimeWindow::Custom { // Specific time window for this drop
                        start: data[i - 1].timestamp,
                        end: data[i + 1].timestamp,
                    },
                    confidence,
                    magnitude,
                    start_timestamp: data[i - 1].timestamp,
                    end_timestamp: data[i + 1].timestamp,
                    detection_timestamp: 0, // Will be filled by caller
                    details: serde_json::json!({
                        "before_value": prev,
                        "drop_value": current,
                        "after_value": next,
                        "down_change": up_change,
                        "up_change": down_change,
                    }),
                });
            }
        }
        
        Ok(results)
    }
    
    /// Detect cyclical patterns in the data
    fn detect_cyclical_pattern(&self, data: &[DataPoint]) 
        -> Result<Option<CyclicalInfo>, AnalyticsError> 
    {
        if data.len() < self.config.min_data_points * 2 {
            return Ok(None);
        }
        
        // Simple autocorrelation to detect periodicity
        let values: Vec<f64> = data.iter().map(|dp| dp.value).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        
        // Normalize the values
        let normalized: Vec<f64> = values.iter().map(|&v| v - mean).collect();
        
        // Calculate autocorrelation for different lags
        let max_lag = values.len() / 2;
        let mut autocorr = Vec::with_capacity(max_lag);
        
        for lag in 1..=max_lag {
            let mut sum_product = 0.0;
            let mut sum_sq = 0.0;
            let mut sum_sq_lagged = 0.0;
            
            for i in 0..values.len() - lag {
                let v1 = normalized[i];
                let v2 = normalized[i + lag];
                
                sum_product += v1 * v2;
                sum_sq += v1 * v1;
                sum_sq_lagged += v2 * v2;
            }
            
            let corr = if sum_sq > 0.0 && sum_sq_lagged > 0.0 {
                sum_product / (sum_sq.sqrt() * sum_sq_lagged.sqrt())
            } else {
                0.0
            };
            
            autocorr.push((lag, corr));
        }
        
        // Find the highest autocorrelation peak
        if let Some((period, corr)) = autocorr.iter()
            .filter(|(lag, _)| *lag >= 2) // Ignore very short periods
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        {
            if *corr > 0.5 { // Threshold for cyclical pattern detection
                let confidence = (*corr - 0.5) * 2.0; // Scale to [0, 1]
                let period_ms = (data[*period].timestamp - data[0].timestamp).abs();
                
                return Ok(Some(CyclicalInfo {
                    period: *period,
                    period_ms,
                    correlation: *corr,
                    confidence,
                    magnitude: *corr,
                    details: serde_json::json!({
                        "period": *period,
                        "period_ms": period_ms,
                        "correlation": *corr,
                    }),
                }));
            }
        }
        
        Ok(None)
    }
}

/// Information about a detected anomaly
struct AnomalyInfo {
    /// Timestamp of the anomaly
    timestamp: i64,
    
    /// Value of the anomaly
    value: f64,
    
    /// Number of standard deviations from the mean
    deviations: f64,
    
    /// Confidence level of the anomaly detection
    confidence: f64,
    
    /// Magnitude of the anomaly
    magnitude: f64,
}

/// Information about a detected cyclical pattern
struct CyclicalInfo {
    /// Period of the pattern (in data points)
    period: usize,
    
    /// Period of the pattern (in milliseconds)
    period_ms: i64,
    
    /// Correlation value of the pattern
    correlation: f64,
    
    /// Confidence level of the pattern detection
    confidence: f64,
    
    /// Magnitude of the pattern
    magnitude: f64,
    
    /// Additional details about the pattern
    details: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use crate::analytics::storage::{AnalyticsStorage, StorageConfig};
    use crate::analytics::time_series::{TimeSeriesAnalyzer, TimeSeriesConfig};
    
    async fn create_test_detector() -> TrendDetector {
        let storage_config = StorageConfig::default();
        let storage = Arc::new(RwLock::new(
            AnalyticsStorage::new(storage_config).unwrap()
        ));
        
        let ts_config = TimeSeriesConfig::default();
        let ts_analyzer = Arc::new(RwLock::new(
            TimeSeriesAnalyzer::new(ts_config, Arc::clone(&storage)).unwrap()
        ));
        
        let config = TrendDetectionConfig::default();
        TrendDetector::new(config, ts_analyzer).unwrap()
    }
    
    #[tokio::test]
    async fn test_calculate_confidence() {
        let detector = create_test_detector().await;
        
        // Make sure we understand the significance threshold
        println!("Significance threshold: {}", detector.config.significance_threshold);
        
        // Use magnitudes that are below 1.0 when divided by the significance threshold
        // to ensure we don't hit the maximum confidence of 1.0
        let small_magnitude = detector.config.significance_threshold * 0.1;
        let confidence = detector.calculate_confidence(small_magnitude, 20);
        println!("Base confidence with magnitude {}: {}", small_magnitude, confidence);
        assert!(confidence > 0.0);
        assert!(confidence < 1.0); // Ensure we're not hitting the max
        
        // Use a larger magnitude but still below the threshold that would produce 1.0
        let larger_magnitude = detector.config.significance_threshold * 0.3;
        let higher_confidence = detector.calculate_confidence(larger_magnitude, 20);
        println!("Higher confidence with magnitude {}: {}", larger_magnitude, higher_confidence);
        
        // Ensure higher magnitude gives higher confidence
        assert!(higher_confidence > confidence, 
            "Higher magnitude {} should give higher confidence {} > {}", 
            larger_magnitude, higher_confidence, confidence);
        
        // Test with larger sample size
        let larger_sample_confidence = detector.calculate_confidence(small_magnitude, 40);
        println!("Confidence with larger sample: {}", larger_sample_confidence);
        assert!(larger_sample_confidence > confidence);
    }
} 