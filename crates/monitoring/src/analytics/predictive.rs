//! Predictive analytics module for the analytics system.
//!
//! This module provides functionality for making predictions based on time series data,
//! allowing for forecasting future values and detecting anomalies.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::Utc;

use crate::analytics::time_series::{TimeSeriesAnalyzer, DataPoint, TimeWindow};
use crate::analytics::AnalyticsError;

/// Type of predictive model
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ModelType {
    /// Simple linear regression
    LinearRegression,
    
    /// Moving average
    MovingAverage,
    
    /// Exponential smoothing
    ExponentialSmoothing,
    
    /// ARIMA (Autoregressive Integrated Moving Average)
    Arima,
    
    /// Neural network
    NeuralNetwork,
}

impl Default for ModelType {
    fn default() -> Self {
        Self::ExponentialSmoothing
    }
}

/// Configuration for predictive analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionConfig {
    /// Type of model to use for predictions
    pub model_type: ModelType,
    
    /// Time window to use for training the model
    pub training_window: TimeWindow,
    
    /// Confidence interval for predictions (0.0 - 1.0)
    pub confidence_interval: f64,
    
    /// Maximum prediction horizon in milliseconds
    pub max_prediction_horizon: i64,
}

/// Alias for backward compatibility
pub type PredictiveConfig = PredictionConfig;

impl Default for PredictionConfig {
    fn default() -> Self {
        Self {
            model_type: ModelType::default(),
            training_window: TimeWindow::Days(7),
            confidence_interval: 0.95,
            max_prediction_horizon: 24 * 60 * 60 * 1000, // 24 hours in milliseconds
        }
    }
}

/// A prediction made by a predictive model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    /// The component ID
    pub component_id: String,
    
    /// The metric name
    pub metric_name: String,
    
    /// The type of model used for the prediction
    pub model_type: ModelType,
    
    /// The time when the prediction was made
    pub prediction_time: i64,
    
    /// The start time of the prediction in milliseconds since the Unix epoch
    pub start_time: i64,
    
    /// The end time of the prediction in milliseconds since the Unix epoch
    pub end_time: i64,
    
    /// The predicted values
    pub values: Vec<PredictedValue>,
    
    /// The confidence level of the prediction (0.0 - 1.0)
    pub confidence: f64,
    
    /// Additional metadata about the prediction
    pub metadata: serde_json::Value,
}

/// A predicted value at a specific time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedValue {
    /// The timestamp in milliseconds since the Unix epoch
    pub timestamp: i64,
    
    /// The predicted value
    pub value: f64,
    
    /// The lower bound of the prediction interval
    pub lower_bound: f64,
    
    /// The upper bound of the prediction interval
    pub upper_bound: f64,
}

/// Predictive model for making predictions based on time series data
#[derive(Debug)]
pub struct PredictiveModel {
    /// Configuration for the predictive model
    config: PredictionConfig,
    
    /// Time series analyzer for accessing historical data
    time_series_analyzer: Arc<RwLock<TimeSeriesAnalyzer>>,
}

impl PredictiveModel {
    /// Create a new predictive model with the given configuration
    pub fn new(config: PredictionConfig, 
               time_series_analyzer: Arc<RwLock<TimeSeriesAnalyzer>>) 
        -> Result<Self, AnalyticsError> 
    {
        Ok(Self {
            config,
            time_series_analyzer,
        })
    }
    
    /// Make a prediction for a specific component and metric
    pub async fn predict(&self, component_id: &str, metric_name: &str, prediction_horizon: i64) 
        -> Result<Prediction, AnalyticsError> 
    {
        // Check if the prediction horizon is too large
        if prediction_horizon > self.config.max_prediction_horizon {
            return Err(AnalyticsError::ConfigError(format!(
                "Prediction horizon too large: {} ms (max: {} ms)",
                prediction_horizon, self.config.max_prediction_horizon
            )));
        }
        
        // Get historical data for training
        let analyzer = self.time_series_analyzer.read().await;
        let training_data = analyzer.get_data(component_id, metric_name, self.config.training_window).await?;
        
        // Check if we have enough data for training
        if training_data.len() < 10 {
            return Err(AnalyticsError::AnalysisError(
                "Not enough historical data for prediction".to_string()
            ));
        }
        
        // Current time
        let now = Utc::now().timestamp_millis();
        
        // Make the prediction using the appropriate model
        match self.config.model_type {
            ModelType::LinearRegression => self.predict_linear_regression(component_id, metric_name, &training_data, now, prediction_horizon),
            ModelType::MovingAverage => self.predict_moving_average(component_id, metric_name, &training_data, now, prediction_horizon),
            ModelType::ExponentialSmoothing => self.predict_exponential_smoothing(component_id, metric_name, &training_data, now, prediction_horizon),
            ModelType::Arima => self.predict_arima(component_id, metric_name, &training_data, now, prediction_horizon),
            ModelType::NeuralNetwork => self.predict_neural_network(component_id, metric_name, &training_data, now, prediction_horizon),
        }
    }
    
    /// Make a prediction using linear regression
    fn predict_linear_regression(&self, component_id: &str, metric_name: &str, 
                                 training_data: &[DataPoint], now: i64, prediction_horizon: i64) 
        -> Result<Prediction, AnalyticsError> 
    {
        // Simple linear regression implementation
        // In a real implementation, this would fit a line to the training data
        
        // Calculate average time interval between data points
        let avg_interval = if training_data.len() > 1 {
            (training_data.last().unwrap().timestamp - training_data.first().unwrap().timestamp) / (training_data.len() - 1) as i64
        } else {
            60 * 1000 // Default to 1 minute if we only have one data point
        };
        
        // Number of points to predict
        let num_points = (prediction_horizon / avg_interval).max(1) as usize;
        
        // Calculate the slope and intercept
        let n = training_data.len() as f64;
        let sum_x: f64 = training_data.iter().enumerate().map(|(i, _)| i as f64).sum();
        let sum_y: f64 = training_data.iter().map(|dp| dp.value).sum();
        let sum_xy: f64 = training_data.iter().enumerate().map(|(i, dp)| i as f64 * dp.value).sum();
        let sum_xx: f64 = training_data.iter().enumerate().map(|(i, _)| (i as f64).powi(2)).sum();
        
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x.powi(2));
        let intercept = (sum_y - slope * sum_x) / n;
        
        // Generate the predictions
        let mut values = Vec::with_capacity(num_points);
        
        for i in 0..num_points {
            let timestamp = now + i as i64 * avg_interval;
            let value = intercept + slope * (training_data.len() + i) as f64;
            
            // Simple prediction interval
            let std_dev = training_data.iter().map(|dp| (dp.value - (intercept + slope * dp.timestamp as f64)).powi(2)).sum::<f64>() / (training_data.len() - 2) as f64;
            let std_dev = std_dev.sqrt();
            
            // Assume normal distribution for prediction interval
            let z = 1.96; // 95% confidence interval
            let prediction_error = z * std_dev;
            
            values.push(PredictedValue {
                timestamp,
                value,
                lower_bound: value - prediction_error,
                upper_bound: value + prediction_error,
            });
        }
        
        Ok(Prediction {
            component_id: component_id.to_string(),
            metric_name: metric_name.to_string(),
            model_type: ModelType::LinearRegression,
            prediction_time: now,
            start_time: now,
            end_time: now + prediction_horizon,
            values,
            confidence: self.config.confidence_interval,
            metadata: serde_json::json!({
                "slope": slope,
                "intercept": intercept,
                "num_training_points": training_data.len(),
            }),
        })
    }
    
    /// Make a prediction using moving average
    fn predict_moving_average(&self, component_id: &str, metric_name: &str, 
                              training_data: &[DataPoint], now: i64, prediction_horizon: i64) 
        -> Result<Prediction, AnalyticsError> 
    {
        // Simple moving average implementation
        // In a real implementation, this would use a proper moving average formula
        
        // Calculate average time interval between data points
        let avg_interval = if training_data.len() > 1 {
            (training_data.last().unwrap().timestamp - training_data.first().unwrap().timestamp) / (training_data.len() - 1) as i64
        } else {
            60 * 1000 // Default to 1 minute if we only have one data point
        };
        
        // Number of points to predict
        let num_points = (prediction_horizon / avg_interval).max(1) as usize;
        
        // Calculate the moving average (simple average for now)
        let avg_value = training_data.iter().map(|dp| dp.value).sum::<f64>() / training_data.len() as f64;
        
        // Calculate standard deviation for prediction interval
        let std_dev = (training_data.iter().map(|dp| (dp.value - avg_value).powi(2)).sum::<f64>() / training_data.len() as f64).sqrt();
        
        // Generate the predictions
        let mut values = Vec::with_capacity(num_points);
        
        for i in 0..num_points {
            let timestamp = now + i as i64 * avg_interval;
            
            // Assume normal distribution for prediction interval
            let z = 1.96; // 95% confidence interval
            let prediction_error = z * std_dev;
            
            values.push(PredictedValue {
                timestamp,
                value: avg_value,
                lower_bound: avg_value - prediction_error,
                upper_bound: avg_value + prediction_error,
            });
        }
        
        Ok(Prediction {
            component_id: component_id.to_string(),
            metric_name: metric_name.to_string(),
            model_type: ModelType::MovingAverage,
            prediction_time: now,
            start_time: now,
            end_time: now + prediction_horizon,
            values,
            confidence: self.config.confidence_interval,
            metadata: serde_json::json!({
                "avg_value": avg_value,
                "std_dev": std_dev,
                "num_training_points": training_data.len(),
            }),
        })
    }
    
    /// Make a prediction using exponential smoothing
    fn predict_exponential_smoothing(&self, component_id: &str, metric_name: &str, 
                                    training_data: &[DataPoint], now: i64, prediction_horizon: i64) 
        -> Result<Prediction, AnalyticsError> 
    {
        // Simple exponential smoothing implementation
        // In a real implementation, this would use the proper exponential smoothing formula
        
        // Calculate average time interval between data points
        let avg_interval = if training_data.len() > 1 {
            (training_data.last().unwrap().timestamp - training_data.first().unwrap().timestamp) / (training_data.len() - 1) as i64
        } else {
            60 * 1000 // Default to 1 minute if we only have one data point
        };
        
        // Number of points to predict
        let num_points = (prediction_horizon / avg_interval).max(1) as usize;
        
        // Alpha parameter for exponential smoothing (0 < alpha < 1)
        let alpha = 0.3;
        
        // Initialize the smoothed value with the first data point
        let mut smoothed_value = training_data.first().unwrap().value;
        
        // Apply exponential smoothing to the training data
        for dp in &training_data[1..] {
            smoothed_value = alpha * dp.value + (1.0 - alpha) * smoothed_value;
        }
        
        // Calculate standard deviation for prediction interval
        let mut sum_squared_errors = 0.0;
        let mut current_smoothed = training_data.first().unwrap().value;
        
        for dp in &training_data[1..] {
            let next_smoothed = alpha * dp.value + (1.0 - alpha) * current_smoothed;
            let error = dp.value - next_smoothed;
            sum_squared_errors += error.powi(2);
            current_smoothed = next_smoothed;
        }
        
        let std_dev = (sum_squared_errors / (training_data.len() - 1) as f64).sqrt();
        
        // Generate the predictions
        let mut values = Vec::with_capacity(num_points);
        
        for i in 0..num_points {
            let timestamp = now + i as i64 * avg_interval;
            
            // In simple exponential smoothing, the forecast is the last smoothed value
            // Assume normal distribution for prediction interval
            let z = 1.96; // 95% confidence interval
            let prediction_error = z * std_dev * (1.0 + i as f64).sqrt();
            
            values.push(PredictedValue {
                timestamp,
                value: smoothed_value,
                lower_bound: smoothed_value - prediction_error,
                upper_bound: smoothed_value + prediction_error,
            });
        }
        
        Ok(Prediction {
            component_id: component_id.to_string(),
            metric_name: metric_name.to_string(),
            model_type: ModelType::ExponentialSmoothing,
            prediction_time: now,
            start_time: now,
            end_time: now + prediction_horizon,
            values,
            confidence: self.config.confidence_interval,
            metadata: serde_json::json!({
                "alpha": alpha,
                "smoothed_value": smoothed_value,
                "std_dev": std_dev,
                "num_training_points": training_data.len(),
            }),
        })
    }
    
    /// Predict future values using ARIMA model
    fn predict_arima(&self, _component_id: &str, _metric_name: &str,
                    _training_data: &[DataPoint], _now: i64, _prediction_horizon: i64)
        -> Result<Prediction, AnalyticsError> {
        // Not implemented in basic version
        Err(AnalyticsError::AnalysisError(
            "ARIMA prediction is not yet implemented".to_string()
        ))
    }
    
    /// Predict future values using neural network
    fn predict_neural_network(&self, _component_id: &str, _metric_name: &str,
                              _training_data: &[DataPoint], _now: i64, _prediction_horizon: i64)
        -> Result<Prediction, AnalyticsError> {
        // Not implemented in basic version
        Err(AnalyticsError::AnalysisError(
            "Neural network prediction is not yet implemented".to_string()
        ))
    }
}

/// Predictive analyzer for making predictions based on time series data
#[derive(Debug)]
pub struct PredictiveAnalyzer {
    /// Predictive model for making predictions
    model: PredictiveModel,
}

impl PredictiveAnalyzer {
    /// Create a new predictive analyzer with the given configuration
    pub fn new(config: PredictionConfig, 
               time_series_analyzer: Arc<RwLock<TimeSeriesAnalyzer>>) 
        -> Result<Self, AnalyticsError> 
    {
        let model = PredictiveModel::new(config, time_series_analyzer)?;
        Ok(Self { model })
    }
    
    /// Create a new predictive analyzer with the given configuration
    pub fn with_config(config: PredictionConfig) -> Self {
        // Create a default time series analyzer
        let storage_config = crate::analytics::storage::StorageConfig::default();
        let storage = Arc::new(RwLock::new(
            crate::analytics::storage::AnalyticsStorage::new(storage_config).expect("Failed to create analytics storage")
        ));
        
        let ts_config = crate::analytics::time_series::TimeSeriesConfig::default();
        let ts_analyzer = Arc::new(RwLock::new(
            crate::analytics::time_series::TimeSeriesAnalyzer::new(ts_config, Arc::clone(&storage)).expect("Failed to create time series analyzer")
        ));
        
        // Create the model with the provided config
        Self::new(config, ts_analyzer).expect("Failed to create predictive analyzer")
    }
    
    /// Make a prediction for a specific component and metric
    pub async fn predict(&self, data: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        // Extract component_id, metric_name, and prediction_horizon from data
        let component_id = data["component_id"].as_str().ok_or_else(|| 
            anyhow::anyhow!("Missing component_id in prediction request"))?;
            
        let metric_name = data["metric_name"].as_str().ok_or_else(|| 
            anyhow::anyhow!("Missing metric_name in prediction request"))?;
            
        let prediction_horizon = data["prediction_horizon"].as_i64().ok_or_else(|| 
            anyhow::anyhow!("Missing or invalid prediction_horizon in prediction request"))?;
        
        // Make the prediction
        let prediction = self.model.predict(component_id, metric_name, prediction_horizon).await?;
        
        // Convert the prediction to a JSON value
        Ok(serde_json::to_value(prediction)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use crate::analytics::storage::{AnalyticsStorage, StorageConfig};
    use crate::analytics::time_series::{TimeSeriesAnalyzer, TimeSeriesConfig};
    
    async fn create_test_model() -> PredictiveModel {
        let storage_config = StorageConfig::default();
        let storage = Arc::new(RwLock::new(
            AnalyticsStorage::new(storage_config).unwrap()
        ));
        
        let ts_config = TimeSeriesConfig::default();
        let ts_analyzer = Arc::new(RwLock::new(
            TimeSeriesAnalyzer::new(ts_config, Arc::clone(&storage)).unwrap()
        ));
        
        let config = PredictionConfig {
            model_type: ModelType::LinearRegression,
            ..PredictionConfig::default()
        };
        
        PredictiveModel::new(config, ts_analyzer).unwrap()
    }
} 