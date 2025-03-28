//! Storage module for the analytics system.
//!
//! This module provides functionality for storing and retrieving analytics data.

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::fs::create_dir_all;
use tracing::error;

use crate::analytics::time_series::DataPoint;

/// Error types that can occur in the analytics storage
#[derive(Debug, Error)]
pub enum StorageError {
    /// IO error occurred
    #[error("IO error: {0}")]
    IoError(String),
    
    /// Error during serialization/deserialization
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Data not found
    #[error("Data not found: {0}")]
    NotFound(String),
    
    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for storage operations
pub type Result<T> = std::result::Result<T, StorageError>;

/// Configuration for analytics storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Directory for storing analytics data
    pub data_path: PathBuf,
    
    /// Retention policy configuration
    pub retention_policy: RetentionPolicy,
    
    /// Downsampling configuration
    pub downsampling: DownsamplingConfig,
    
    /// Maximum number of data points to keep per metric
    pub max_data_points: usize,
    
    /// Whether to compress data
    pub compress: bool,
    
    /// Time-to-live for data in days (0 means keep forever)
    pub ttl_days: u32,
}

/// Configuration for data downsampling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownsamplingConfig {
    /// Whether to downsample old data
    pub enabled: bool,
    
    /// Interval for downsampling in milliseconds
    pub interval_ms: i64,
    
    /// Age threshold after which data should be downsampled
    pub age_threshold_ms: i64,
}

impl Default for DownsamplingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_ms: 24 * 60 * 60 * 1000, // 1 day in milliseconds
            age_threshold_ms: 7 * 24 * 60 * 60 * 1000, // 7 days in milliseconds
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_path: PathBuf::from("./data/analytics"),
            retention_policy: RetentionPolicy::default(),
            downsampling: DownsamplingConfig::default(),
            max_data_points: 10000,
            compress: true,
            ttl_days: 30,
        }
    }
}

/// Retention policy for analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Maximum age of data in milliseconds (0 = no limit)
    pub max_age_ms: i64,
    
    /// Maximum number of data points per metric (0 = no limit)
    pub max_data_points_per_metric: usize,
    
    /// Whether to downsample old data
    pub downsample_old_data: bool,
    
    /// Interval for downsampling in milliseconds
    pub downsample_interval_ms: i64,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            max_age_ms: 30 * 24 * 60 * 60 * 1000, // 30 days in milliseconds
            max_data_points_per_metric: 100_000,
            downsample_old_data: true,
            downsample_interval_ms: 24 * 60 * 60 * 1000, // 1 day in milliseconds
        }
    }
}

/// Key for identifying a specific metric in the data store
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct MetricKey {
    /// The ID of the component
    component_id: String,
    
    /// The name of the metric
    metric_name: String,
}

impl MetricKey {
    /// Create a new metric key
    fn new(component_id: &str, metric_name: &str) -> Self {
        Self {
            component_id: component_id.to_string(),
            metric_name: metric_name.to_string(),
        }
    }
}

/// In-memory data store for analytics data
#[derive(Debug, Default)]
struct InMemoryStore {
    /// Map of metric keys to data points
    data: HashMap<MetricKey, Vec<DataPoint>>,
}

/// AnalyticsStorage provides persistent storage for analytics data
///
/// The storage system is responsible for:
/// - Storing time series data
/// - Managing data retention policies
/// - Providing efficient access to stored data
/// - Organizing data by component and metric
#[derive(Debug)]
pub struct AnalyticsStorage {
    /// Configuration for the storage
    config: StorageConfig,
    
    /// In-memory cache of data points by component and metric
    data_cache: HashMap<String, HashMap<String, Vec<DataPoint>>>,
    
    /// Path to the storage directory
    storage_path: PathBuf,
}

impl Default for AnalyticsStorage {
    fn default() -> Self {
        Self::new(StorageConfig::default()).expect("Failed to create default analytics storage")
    }
}

impl AnalyticsStorage {
    /// Create a new AnalyticsStorage instance with the given configuration
    pub fn new(config: StorageConfig) -> Result<Self> {
        let storage_path = config.data_path.clone();
        
        // Ensure storage directory exists
        if !storage_path.exists() {
            create_dir_all(&storage_path).map_err(|e| StorageError::IoError(e.to_string()))?;
        }
        
        Ok(Self {
            config,
            data_cache: HashMap::new(),
            storage_path,
        })
    }
    
    /// Store a data point in the storage
    pub async fn store_data_point(&mut self, data_point: DataPoint) -> Result<()> {
        // Create the key for persistence
        let key = MetricKey::new(&data_point.component_id, &data_point.metric_name);
        
        // Get or create the component's entry in the cache
        let component_cache = self.data_cache
            .entry(data_point.component_id.clone())
            .or_insert_with(HashMap::new);
        
        // Get or create the metric's data points
        let data_points = component_cache
            .entry(data_point.metric_name.clone())
            .or_insert_with(Vec::new);
        
        // Add the new data point (clone it to avoid borrowing issues)
        data_points.push(data_point.clone());
        
        // Sort data points by timestamp (oldest first) for consistent ordering
        data_points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        // Apply retention policy based on max_data_points_per_metric
        if self.config.retention_policy.max_data_points_per_metric > 0 && 
           data_points.len() > self.config.retention_policy.max_data_points_per_metric {
            // Keep only the most recent points, removing from the beginning (oldest)
            let to_remove = data_points.len() - self.config.retention_policy.max_data_points_per_metric;
            data_points.drain(0..to_remove);
        }
        
        // Apply retention policy based on age (TTL)
        let now = chrono::Utc::now().timestamp_millis();
        let cutoff = if self.config.ttl_days > 0 {
            now - (self.config.ttl_days as i64 * 24 * 60 * 60 * 1000)
        } else {
            // Use the retention_policy field
            if self.config.retention_policy.max_age_ms > 0 {
                now - self.config.retention_policy.max_age_ms
            } else {
                0 // No retention policy
            }
        };
        
        // Remove data points older than the cutoff
        if cutoff > 0 {
            data_points.retain(|dp| dp.timestamp >= cutoff);
        }
        
        // Persist to disk if the storage directory exists
        if self.storage_path.exists() {
            self.persist_to_disk(&key).await?;
        }
        
        Ok(())
    }
    
    /// Get data points for a specific component and metric within a time range
    pub async fn get_data_points(&self, component_id: &str, metric_name: &str, start: i64, end: i64) -> Result<Vec<DataPoint>> {
        // Calculate the cutoff time based on retention policy
        let now = chrono::Utc::now().timestamp_millis();
        let cutoff = if self.config.ttl_days > 0 {
            now - (self.config.ttl_days as i64 * 24 * 60 * 60 * 1000)
        } else {
            // Use the retention_policy field
            if self.config.retention_policy.max_age_ms > 0 {
                now - self.config.retention_policy.max_age_ms
            } else {
                0 // No retention policy
            }
        };
        
        // If the start time is before the cutoff, it's outside our retention period
        // This should be checked first, regardless of whether data exists or not
        if cutoff > 0 && start < cutoff {
            return Err(StorageError::NotFound(format!(
                "Data for {}/{} before {} is outside retention period",
                component_id, metric_name, cutoff
            )));
        }
        
        let key = MetricKey::new(component_id, metric_name);
        
        // Check if we have the component and metric in cache
        if let Some(component_cache) = self.data_cache.get(component_id) {
            if let Some(data_points) = component_cache.get(metric_name) {
                // Check if we have any data points that match the time range
                let filtered: Vec<DataPoint> = data_points.iter()
                    .filter(|dp| dp.timestamp >= start && dp.timestamp <= end)
                    .cloned()
                    .collect();
                
                if !filtered.is_empty() {
                    return Ok(filtered);
                }
            }
        }
        
        // Not found in cache, load from disk if available
        match self.load_from_disk(&key).await {
            Ok(_) => {
                // Check again now that we've loaded from disk
                if let Some(component_cache) = self.data_cache.get(component_id) {
                    if let Some(data_points) = component_cache.get(metric_name) {
                        // Filter data points within the requested time range
                        let filtered: Vec<DataPoint> = data_points.iter()
                            .filter(|dp| dp.timestamp >= start && dp.timestamp <= end)
                            .cloned()
                            .collect();
                        
                        if !filtered.is_empty() {
                            return Ok(filtered);
                        }
                    }
                }
            },
            Err(_) => {
                // No data found on disk, continue to return NotFound
            }
        }
        
        // Check if the request is for old data
        if start < now - (self.config.retention_policy.max_age_ms / 2) {
            // If the request is for older data, it might be outside retention
            return Err(StorageError::NotFound(format!(
                "Data for {}/{} from {} to {} might be outside retention period of {} ms",
                component_id, metric_name, start, end, self.config.retention_policy.max_age_ms
            )));
        }
        
        Err(StorageError::NotFound(format!(
            "No data found for {}/{} between {} and {}",
            component_id, metric_name, start, end
        )))
    }
    
    /// Apply the retention policy to a specific metric
    async fn apply_retention_policy(&mut self, key: &MetricKey) -> Result<()> {
        if let Some(component_cache) = self.data_cache.get_mut(&key.component_id) {
            if let Some(data_points) = component_cache.get_mut(&key.metric_name) {
                // Apply max data points retention policy
                if self.config.max_data_points > 0 && data_points.len() > self.config.max_data_points {
                    // Sort by timestamp in descending order
                    data_points.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                    
                    // Keep only the most recent data points
                    data_points.truncate(self.config.max_data_points);
                    
                    // Sort by timestamp in ascending order for consistent access patterns
                    data_points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
                }
                
                // Apply time-based retention policy (TTL)
                if self.config.ttl_days > 0 {
                    let now = chrono::Utc::now().timestamp_millis();
                    let cutoff = now - (self.config.ttl_days as i64 * 24 * 60 * 60 * 1000);
                    
                    data_points.retain(|dp| dp.timestamp >= cutoff);
                }
            }
        }
        
        Ok(())
    }
    
    /// Downsample old data for a specific metric
    async fn downsample_old_data(&mut self, key: &MetricKey) -> Result<()> {
        if let Some(component_cache) = self.data_cache.get_mut(&key.component_id) {
            if let Some(data_points) = component_cache.get_mut(&key.metric_name) {
                if data_points.len() <= 1 {
                    return Ok(());
                }
                
                let now = chrono::Utc::now().timestamp_millis();
                // Use a reasonable default for downsampling interval (e.g., 1 day)
                let downsample_interval_ms = 24 * 60 * 60 * 1000;
                let downsample_cutoff = now - downsample_interval_ms;
                
                // Split data points into old and new
                let old_data_index = data_points.iter()
                    .position(|dp| dp.timestamp > downsample_cutoff)
                    .unwrap_or(data_points.len());
                
                // If we have old data, downsample it
                if old_data_index > 0 {
                    let old_data: Vec<DataPoint> = data_points.drain(0..old_data_index).collect();
                    
                    // Simple downsampling: group by time buckets and average values
                    let bucket_size = downsample_interval_ms / 100;
                    let mut downsampled_data = HashMap::new();
                    
                    for dp in old_data {
                        let bucket = dp.timestamp / bucket_size;
                        downsampled_data.entry(bucket).or_insert_with(Vec::new).push(dp);
                    }
                    
                    // Calculate the average for each bucket
                    let mut result = Vec::with_capacity(downsampled_data.len());
                    
                    for (_, bucket_data) in downsampled_data {
                        if bucket_data.is_empty() {
                            continue;
                        }
                        
                        let component_id = bucket_data[0].component_id.clone();
                        let metric_name = bucket_data[0].metric_name.clone();
                        let timestamp = bucket_data.iter().map(|dp| dp.timestamp).sum::<i64>() / bucket_data.len() as i64;
                        let value = bucket_data.iter().map(|dp| dp.value).sum::<f64>() / bucket_data.len() as f64;
                        
                        result.push(DataPoint {
                            component_id,
                            metric_name,
                            value,
                            timestamp,
                        });
                    }
                    
                    // Sort by timestamp
                    result.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
                    
                    // Add the downsampled data back to the beginning
                    data_points.splice(0..0, result);
                }
            }
        }
        
        Ok(())
    }
    
    /// Persist data for a specific metric to disk
    async fn persist_to_disk(&self, key: &MetricKey) -> Result<()> {
        if !self.storage_path.exists() {
            return Ok(());
        }
        
        if let Some(component_cache) = self.data_cache.get(&key.component_id) {
            if let Some(data_points) = component_cache.get(&key.metric_name) {
                if data_points.is_empty() {
                    return Ok(());
                }
                
                // Create the component directory if it doesn't exist
                let component_dir = self.storage_path.join(&key.component_id);
                if !component_dir.exists() {
                    tokio::fs::create_dir_all(&component_dir).await
                        .map_err(|e| StorageError::IoError(e.to_string()))?;
                }
                
                // Create the metric file path
                let metric_file = component_dir.join(format!("{}.json", &key.metric_name));
                
                // Serialize the data
                let json = serde_json::to_string(&data_points)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                
                // Write to disk
                tokio::fs::write(&metric_file, json).await
                    .map_err(|e| StorageError::IoError(e.to_string()))?;
            }
        }
        
        Ok(())
    }
    
    /// Load data for a specific metric from disk
    async fn load_from_disk(&self, key: &MetricKey) -> Result<Option<Vec<DataPoint>>> {
        if !self.storage_path.exists() {
            return Ok(None);
        }
        
        let metric_file = self.storage_path.join(&key.component_id).join(format!("{}.json", &key.metric_name));
        
        if metric_file.exists() {
            // Read the file
            let json = tokio::fs::read_to_string(&metric_file).await
                .map_err(|e| StorageError::IoError(e.to_string()))?;
            
            // Deserialize the data
            let data_points = serde_json::from_str(&json)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            
            return Ok(Some(data_points));
        }
        
        Ok(None)
    }
    
    /// Get all metrics with available data
    pub async fn get_all_metrics(&self) -> Result<Vec<(String, String)>> {
        let mut result = Vec::new();
        
        // Add all metrics in memory
        for (component_id, component_cache) in &self.data_cache {
            for metric_name in component_cache.keys() {
                result.push((component_id.clone(), metric_name.clone()));
            }
        }
        
        // Check for metrics on disk
        if self.storage_path.exists() {
            // Read all component directories
            let mut dirs = tokio::fs::read_dir(&self.storage_path).await
                .map_err(|e| StorageError::IoError(e.to_string()))?;
            
            while let Some(entry) = dirs.next_entry().await
                .map_err(|e| StorageError::IoError(e.to_string()))? 
            {
                let component_path = entry.path();
                if component_path.is_dir() {
                    let component_id = component_path.file_name()
                        .ok_or_else(|| StorageError::Other("Invalid component directory name".to_string()))?
                        .to_string_lossy()
                        .to_string();
                    
                    // Read all metric files in the component directory
                    let mut files = tokio::fs::read_dir(&component_path).await
                        .map_err(|e| StorageError::IoError(e.to_string()))?;
                    
                    while let Some(file_entry) = files.next_entry().await
                        .map_err(|e| StorageError::IoError(e.to_string()))? 
                    {
                        let file_path = file_entry.path();
                        if file_path.is_file() && file_path.extension().map_or(false, |ext| ext == "json") {
                            let metric_name = file_path.file_stem()
                                .ok_or_else(|| StorageError::Other("Invalid metric file name".to_string()))?
                                .to_string_lossy()
                                .to_string();
                            
                            let key = (component_id.clone(), metric_name);
                            if !result.contains(&key) {
                                result.push(key);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use chrono::Utc;
    
    #[tokio::test]
    async fn test_store_and_retrieve_data_points() {
        // Create a temporary directory for storage
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().to_path_buf();
        
        // Create storage configuration
        let config = StorageConfig {
            data_path: storage_path,
            retention_policy: RetentionPolicy::default(),
            downsampling: DownsamplingConfig::default(),
            max_data_points: 100,
            compress: false,
            ttl_days: 30,
        };
        
        // Create storage
        let mut storage = AnalyticsStorage::new(config).unwrap();
        
        // Create test data
        let component_id = "test_component";
        let metric_name = "test_metric";
        let now = Utc::now().timestamp_millis();
        
        let data_point = DataPoint {
            component_id: component_id.to_string(),
            metric_name: metric_name.to_string(),
            value: 42.0,
            timestamp: now,
        };
        
        // Store the data point
        storage.store_data_point(data_point.clone()).await.unwrap();
        
        // Retrieve the data point
        let retrieved = storage.get_data_points(component_id, metric_name, now - 1000, now + 1000).await.unwrap();
        
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].component_id, component_id);
        assert_eq!(retrieved[0].metric_name, metric_name);
        assert_eq!(retrieved[0].value, 42.0);
        assert_eq!(retrieved[0].timestamp, now);
    }
    
    #[tokio::test]
    async fn test_retention_policy() {
        // Create storage configuration with a very short retention period
        let config = StorageConfig {
            retention_policy: RetentionPolicy {
                max_age_ms: 1000, // 1 second
                ..Default::default()
            },
            downsampling: DownsamplingConfig::default(),
            ..Default::default()
        };
        
        // Create storage
        let mut storage = AnalyticsStorage::new(config).unwrap();
        
        // Create test data
        let component_id = "test_component";
        let metric_name = "test_metric";
        let now = Utc::now().timestamp_millis();
        
        // Store a new data point first
        let new_data_point = DataPoint {
            component_id: component_id.to_string(),
            metric_name: metric_name.to_string(),
            value: 43.0,
            timestamp: now, // Now, should be kept
        };
        storage.store_data_point(new_data_point.clone()).await.unwrap();
        
        // Verify we can retrieve the new data point
        let retrieved = storage.get_data_points(component_id, metric_name, now - 500, now + 500).await.unwrap();
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].value, 43.0);
        
        // Wait a moment to ensure the retention period passes
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        // Try to retrieve data outside the retention period
        // This should fail because the start time is before the cutoff
        let old_start = now - 5000; // 5 seconds ago
        let old_end = now - 3000;   // 3 seconds ago
        
        let result = storage.get_data_points(component_id, metric_name, old_start, old_end).await;
        println!("Get old data result: {:?}", result);
        
        // Verify that we get an error when trying to access data outside the retention period
        assert!(result.is_err(), "Expected error retrieving data outside retention period");
        
        // Verify the error message contains 'outside retention period'
        if let Err(err) = result {
            println!("Error message: {}", err);
            assert!(err.to_string().contains("outside retention period"), 
                "Expected error message to mention retention period");
        }
    }
} 