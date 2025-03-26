//! Pattern recognition module for the analytics system.
//!
//! This module provides functionality for recognizing patterns
//! in time series data, such as seasonality, cycles, and trends.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::Utc;

use crate::analytics::time_series::{TimeSeriesAnalyzer, DataPoint, TimeWindow};
use crate::analytics::AnalyticsError;

/// A pattern in time series data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// The name of the pattern
    pub name: String,
    
    /// The description of the pattern
    pub description: String,
    
    /// The confidence level of the pattern match (0.0 - 1.0)
    pub confidence: f64,
    
    /// Additional metadata about the pattern
    pub metadata: serde_json::Value,
}

/// Configuration for pattern recognition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternConfig {
    /// Minimum confidence level for pattern recognition
    pub min_confidence: f64,
    
    /// Maximum number of patterns to recognize
    pub max_patterns: usize,
    
    /// Time windows to analyze for pattern recognition
    pub analysis_windows: Vec<TimeWindow>,
    
    /// Minimum number of data points required for pattern recognition
    pub min_data_points: usize,
}

/// Alias for backward compatibility
pub type PatternRecognitionConfig = PatternConfig;

impl Default for PatternConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.7,
            max_patterns: 10,
            analysis_windows: vec![
                TimeWindow::Hours(24),
                TimeWindow::Days(7),
                TimeWindow::Days(30),
            ],
            min_data_points: 30,
        }
    }
}

/// A recognized pattern in time series data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognizedPattern {
    /// The component ID
    pub component_id: String,
    
    /// The metric name
    pub metric_name: String,
    
    /// The pattern that was recognized
    pub pattern: Pattern,
    
    /// The time window the pattern was recognized in
    pub window: TimeWindow,
    
    /// The timestamp when the pattern was recognized
    pub recognition_timestamp: i64,
    
    /// Additional details about the recognized pattern
    pub details: serde_json::Value,
}

/// Pattern recognizer for identifying patterns in time series data
#[derive(Debug)]
pub struct PatternRecognizer {
    /// Configuration for pattern recognition
    config: PatternConfig,
    
    /// Time series analyzer for accessing data
    time_series_analyzer: Arc<RwLock<TimeSeriesAnalyzer>>,
    
    /// Predefined patterns to recognize
    patterns: Vec<Pattern>,
}

impl PatternRecognizer {
    /// Create a new pattern recognizer with the given configuration
    pub fn new(config: PatternConfig, 
               time_series_analyzer: Arc<RwLock<TimeSeriesAnalyzer>>) 
        -> Result<Self, AnalyticsError> 
    {
        // Initialize with some built-in patterns
        let patterns = vec![
            Pattern {
                name: "Daily Cycle".to_string(),
                description: "A pattern that repeats every 24 hours".to_string(),
                confidence: 1.0,
                metadata: serde_json::json!({
                    "period": 24 * 60 * 60 * 1000, // 24 hours in milliseconds
                    "type": "cycle"
                }),
            },
            Pattern {
                name: "Weekly Cycle".to_string(),
                description: "A pattern that repeats every 7 days".to_string(),
                confidence: 1.0,
                metadata: serde_json::json!({
                    "period": 7 * 24 * 60 * 60 * 1000, // 7 days in milliseconds
                    "type": "cycle"
                }),
            },
            Pattern {
                name: "Monthly Cycle".to_string(),
                description: "A pattern that repeats approximately every 30 days".to_string(),
                confidence: 1.0,
                metadata: serde_json::json!({
                    "period": 30_i64 * 24 * 60 * 60 * 1000, // 30 days in milliseconds
                    "type": "cycle"
                }),
            },
            Pattern {
                name: "Linear Growth".to_string(),
                description: "A linear growth trend".to_string(),
                confidence: 1.0,
                metadata: serde_json::json!({
                    "type": "trend",
                    "shape": "linear",
                    "direction": "increasing"
                }),
            },
            Pattern {
                name: "Linear Decay".to_string(),
                description: "A linear decay trend".to_string(),
                confidence: 1.0,
                metadata: serde_json::json!({
                    "type": "trend",
                    "shape": "linear",
                    "direction": "decreasing"
                }),
            },
            Pattern {
                name: "Exponential Growth".to_string(),
                description: "An exponential growth trend".to_string(),
                confidence: 1.0,
                metadata: serde_json::json!({
                    "type": "trend",
                    "shape": "exponential",
                    "direction": "increasing"
                }),
            },
            Pattern {
                name: "Exponential Decay".to_string(),
                description: "An exponential decay trend".to_string(),
                confidence: 1.0,
                metadata: serde_json::json!({
                    "type": "trend",
                    "shape": "exponential",
                    "direction": "decreasing"
                }),
            },
        ];
        
        Ok(Self {
            config,
            time_series_analyzer,
            patterns,
        })
    }
    
    /// Recognize patterns for a specific component and metric
    pub async fn recognize_patterns(&self, component_id: &str, metric_name: &str) 
        -> Result<Vec<RecognizedPattern>, AnalyticsError> 
    {
        let mut recognized_patterns = Vec::new();
        
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
            
            // Recognize patterns in this window
            let patterns = self.recognize_patterns_in_window(component_id, metric_name, *window, &data)?;
            recognized_patterns.extend(patterns);
            
            // If we've reached the maximum number of patterns, stop
            if recognized_patterns.len() >= self.config.max_patterns {
                break;
            }
        }
        
        // Filter out patterns below the minimum confidence level
        recognized_patterns.retain(|p| p.pattern.confidence >= self.config.min_confidence);
        
        // Sort by confidence level (descending)
        recognized_patterns.sort_by(|a, b| b.pattern.confidence.partial_cmp(&a.pattern.confidence).unwrap_or(std::cmp::Ordering::Equal));
        
        // Limit to max patterns
        if recognized_patterns.len() > self.config.max_patterns {
            recognized_patterns.truncate(self.config.max_patterns);
        }
        
        Ok(recognized_patterns)
    }
    
    /// Recognize patterns in a specific time window
    fn recognize_patterns_in_window(&self, component_id: &str, metric_name: &str,
                                  window: TimeWindow, data: &[DataPoint]) 
        -> Result<Vec<RecognizedPattern>, AnalyticsError> 
    {
        let mut results = Vec::new();
        
        // Current timestamp for recognition timestamp
        let now = Utc::now().timestamp_millis();
        
        // Check for daily cycles
        if let Some(confidence) = self.detect_daily_cycle(data) {
            if confidence >= self.config.min_confidence {
                let pattern = self.patterns.iter().find(|p| p.name == "Daily Cycle").unwrap().clone();
                
                results.push(RecognizedPattern {
                    component_id: component_id.to_string(),
                    metric_name: metric_name.to_string(),
                    pattern: Pattern {
                        confidence,
                        ..pattern
                    },
                    window,
                    recognition_timestamp: now,
                    details: serde_json::json!({
                        "data_points": data.len(),
                        "window": format!("{:?}", window),
                    }),
                });
            }
        }
        
        // Check for weekly cycles
        if let Some(confidence) = self.detect_weekly_cycle(data) {
            if confidence >= self.config.min_confidence {
                let pattern = self.patterns.iter().find(|p| p.name == "Weekly Cycle").unwrap().clone();
                
                results.push(RecognizedPattern {
                    component_id: component_id.to_string(),
                    metric_name: metric_name.to_string(),
                    pattern: Pattern {
                        confidence,
                        ..pattern
                    },
                    window,
                    recognition_timestamp: now,
                    details: serde_json::json!({
                        "data_points": data.len(),
                        "window": format!("{:?}", window),
                    }),
                });
            }
        }
        
        // Check for linear trends
        if let Some((confidence, is_increasing)) = self.detect_linear_trend(data) {
            if confidence >= self.config.min_confidence {
                let pattern_name = if is_increasing {
                    "Linear Growth"
                } else {
                    "Linear Decay"
                };
                
                let pattern = self.patterns.iter().find(|p| p.name == pattern_name).unwrap().clone();
                
                results.push(RecognizedPattern {
                    component_id: component_id.to_string(),
                    metric_name: metric_name.to_string(),
                    pattern: Pattern {
                        confidence,
                        ..pattern
                    },
                    window,
                    recognition_timestamp: now,
                    details: serde_json::json!({
                        "data_points": data.len(),
                        "window": format!("{:?}", window),
                        "direction": if is_increasing { "increasing" } else { "decreasing" },
                    }),
                });
            }
        }
        
        // Check for exponential trends
        if let Some((confidence, is_increasing)) = self.detect_exponential_trend(data) {
            if confidence >= self.config.min_confidence {
                let pattern_name = if is_increasing {
                    "Exponential Growth"
                } else {
                    "Exponential Decay"
                };
                
                let pattern = self.patterns.iter().find(|p| p.name == pattern_name).unwrap().clone();
                
                results.push(RecognizedPattern {
                    component_id: component_id.to_string(),
                    metric_name: metric_name.to_string(),
                    pattern: Pattern {
                        confidence,
                        ..pattern
                    },
                    window,
                    recognition_timestamp: now,
                    details: serde_json::json!({
                        "data_points": data.len(),
                        "window": format!("{:?}", window),
                        "direction": if is_increasing { "increasing" } else { "decreasing" },
                    }),
                });
            }
        }
        
        Ok(results)
    }
    
    /// Detect a daily cycle in the data
    fn detect_daily_cycle(&self, data: &[DataPoint]) -> Option<f64> {
        // Placeholder implementation for the daily cycle detection
        // In a real implementation, this would analyze the data for daily patterns
        // Returns a confidence level if a daily cycle is detected
        
        // This is a simple placeholder implementation
        if data.len() < 24 {
            return None;
        }
        
        // Return a mock confidence level for demonstration
        Some(0.8)
    }
    
    /// Detect a weekly cycle in the data
    fn detect_weekly_cycle(&self, data: &[DataPoint]) -> Option<f64> {
        // Placeholder implementation for the weekly cycle detection
        // In a real implementation, this would analyze the data for weekly patterns
        // Returns a confidence level if a weekly cycle is detected
        
        // This is a simple placeholder implementation
        if data.len() < 7 * 24 {
            return None;
        }
        
        // Return a mock confidence level for demonstration
        Some(0.75)
    }
    
    /// Detect a linear trend in the data
    fn detect_linear_trend(&self, data: &[DataPoint]) -> Option<(f64, bool)> {
        // Placeholder implementation for linear trend detection
        // In a real implementation, this would perform linear regression
        // Returns a confidence level and whether the trend is increasing if a linear trend is detected
        
        if data.len() < self.config.min_data_points {
            return None;
        }
        
        // Simple calculation of average first half vs. average second half
        let mid_point = data.len() / 2;
        let first_half_avg: f64 = data[0..mid_point].iter().map(|dp| dp.value).sum::<f64>() / mid_point as f64;
        let second_half_avg: f64 = data[mid_point..].iter().map(|dp| dp.value).sum::<f64>() / (data.len() - mid_point) as f64;
        
        let difference = second_half_avg - first_half_avg;
        let is_increasing = difference > 0.0;
        
        // Calculate a simple confidence level based on the magnitude of the difference
        let confidence = (difference.abs() / first_half_avg.abs()).min(1.0);
        
        if confidence > 0.1 {
            Some((confidence, is_increasing))
        } else {
            None
        }
    }
    
    /// Detect an exponential trend in the data
    fn detect_exponential_trend(&self, data: &[DataPoint]) -> Option<(f64, bool)> {
        // Placeholder implementation for exponential trend detection
        // In a real implementation, this would perform exponential regression
        // Returns a confidence level and whether the trend is increasing if an exponential trend is detected
        
        if data.len() < self.config.min_data_points {
            return None;
        }
        
        // Placeholder - in a real implementation, this would perform proper exponential trend analysis
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use crate::analytics::storage::{AnalyticsStorage, StorageConfig};
    use crate::analytics::time_series::{TimeSeriesAnalyzer, TimeSeriesConfig};
    
    async fn create_test_recognizer() -> PatternRecognizer {
        let storage_config = StorageConfig::default();
        let storage = Arc::new(RwLock::new(
            AnalyticsStorage::new(storage_config).unwrap()
        ));
        
        let ts_config = TimeSeriesConfig::default();
        let ts_analyzer = Arc::new(RwLock::new(
            TimeSeriesAnalyzer::new(ts_config, Arc::clone(&storage)).unwrap()
        ));
        
        let config = PatternConfig::default();
        PatternRecognizer::new(config, ts_analyzer).unwrap()
    }
    
    #[tokio::test]
    async fn test_pattern_recognizer_creation() {
        let recognizer = create_test_recognizer().await;
        assert!(!recognizer.patterns.is_empty());
    }
} 