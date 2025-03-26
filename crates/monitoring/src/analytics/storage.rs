//! Storage module for the analytics system.
//!
//! This module provides functionality for storing and retrieving analytics data.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use thiserror::Error;

use crate::analytics::time_series::DataPoint;

/// Error types that can occur in the analytics storage
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Data not found: {0}")]
    NotFound(String),
    
    #[error("Other storage error: {0}")]
    Other(String),
}

/// Configuration for analytics storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Path to the storage directory
    pub storage_path: Option<PathBuf>,
    
    /// Maximum number of data points to store in memory
    pub max_in_memory_data_points: usize,
    
    /// Whether to persist data to disk
    pub persist_to_disk: bool,
    
    /// Retention policy for data
    pub retention_policy: RetentionPolicy,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            storage_path: None,
            max_in_memory_data_points: 10_000,
            persist_to_disk: true,
            retention_policy: RetentionPolicy::default(),
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

/// Storage for analytics data
pub struct AnalyticsStorage {
    /// Configuration for the storage
    config: StorageConfig,
    
    /// In-memory data store
    in_memory: InMemoryStore,
}

impl AnalyticsStorage {
    /// Create a new analytics storage with the given configuration
    pub async fn new(config: StorageConfig) -> Result<Self, StorageError> {
        // Create storage directory if it doesn't exist and persistence is enabled
        if config.persist_to_disk {
            if let Some(path) = &config.storage_path {
                if !path.exists() {
                    tokio::fs::create_dir_all(path).await?;
                }
            }
        }
        
        Ok(Self {
            config,
            in_memory: InMemoryStore::default(),
        })
    }
    
    /// Store a data point in the storage
    pub async fn store_data_point(&mut self, data_point: DataPoint) -> Result<(), StorageError> {
        let key = MetricKey::new(&data_point.component_id, &data_point.metric_name);
        
        // Get or create the metric's data points
        let data_points = self.in_memory.data.entry(key.clone()).or_insert_with(Vec::new);
        
        // Add the new data point
        data_points.push(data_point);
        
        // Apply retention policy
        self.apply_retention_policy(&key).await?;
        
        // Persist to disk if enabled
        if self.config.persist_to_disk {
            self.persist_to_disk(&key).await?;
        }
        
        Ok(())
    }
    
    /// Get data points for a specific component and metric within a time range
    pub async fn get_data_points(&self, component_id: &str, metric_name: &str, start: i64, end: i64) 
        -> Result<Vec<DataPoint>, StorageError> 
    {
        let key = MetricKey::new(component_id, metric_name);
        
        // Check if we have the data in memory
        if let Some(data_points) = self.in_memory.data.get(&key) {
            // Filter data points by time range
            let filtered: Vec<DataPoint> = data_points.iter()
                .filter(|dp| dp.timestamp >= start && dp.timestamp <= end)
                .cloned()
                .collect();
            
            if !filtered.is_empty() {
                return Ok(filtered);
            }
        }
        
        // If we don't have the data in memory and persistence is enabled, try to load it from disk
        if self.config.persist_to_disk {
            if let Some(data_points) = self.load_from_disk(&key).await? {
                // Filter data points by time range
                let filtered: Vec<DataPoint> = data_points.iter()
                    .filter(|dp| dp.timestamp >= start && dp.timestamp <= end)
                    .cloned()
                    .collect();
                
                if !filtered.is_empty() {
                    return Ok(filtered);
                }
            }
        }
        
        // No data found for the given time range
        Err(StorageError::NotFound(format!(
            "No data found for component '{}', metric '{}' in time range [{}, {}]",
            component_id, metric_name, start, end
        )))
    }
    
    /// Apply the retention policy to a specific metric
    async fn apply_retention_policy(&mut self, key: &MetricKey) -> Result<(), StorageError> {
        if let Some(data_points) = self.in_memory.data.get_mut(key) {
            // Apply max age retention policy
            if self.config.retention_policy.max_age_ms > 0 {
                let now = chrono::Utc::now().timestamp_millis();
                let cutoff = now - self.config.retention_policy.max_age_ms;
                
                data_points.retain(|dp| dp.timestamp >= cutoff);
            }
            
            // Apply max data points per metric retention policy
            if self.config.retention_policy.max_data_points_per_metric > 0 && 
               data_points.len() > self.config.retention_policy.max_data_points_per_metric 
            {
                // Sort by timestamp in descending order
                data_points.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                
                // Keep only the most recent data points
                data_points.truncate(self.config.retention_policy.max_data_points_per_metric);
                
                // Sort by timestamp in ascending order for consistent access patterns
                data_points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
            }
            
            // Apply downsampling if enabled
            if self.config.retention_policy.downsample_old_data {
                self.downsample_old_data(key).await?;
            }
        }
        
        Ok(())
    }
    
    /// Downsample old data for a specific metric
    async fn downsample_old_data(&mut self, key: &MetricKey) -> Result<(), StorageError> {
        if let Some(data_points) = self.in_memory.data.get_mut(key) {
            if data_points.len() <= 1 {
                return Ok(());
            }
            
            let now = chrono::Utc::now().timestamp_millis();
            let downsample_cutoff = now - self.config.retention_policy.downsample_interval_ms;
            
            // Split data points into old and new
            let old_data_index = data_points.iter()
                .position(|dp| dp.timestamp > downsample_cutoff)
                .unwrap_or(data_points.len());
            
            // If we have old data, downsample it
            if old_data_index > 0 {
                let mut old_data: Vec<DataPoint> = data_points.drain(0..old_data_index).collect();
                
                // Simple downsampling: group by time buckets and average values
                let bucket_size = self.config.retention_policy.downsample_interval_ms / 100;
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
        
        Ok(())
    }
    
    /// Persist data for a specific metric to disk
    async fn persist_to_disk(&self, key: &MetricKey) -> Result<(), StorageError> {
        if let Some(path) = &self.config.storage_path {
            if let Some(data_points) = self.in_memory.data.get(key) {
                // Create the component directory if it doesn't exist
                let component_dir = path.join(&key.component_id);
                if !component_dir.exists() {
                    tokio::fs::create_dir_all(&component_dir).await?;
                }
                
                // Create the metric file path
                let metric_file = component_dir.join(format!("{}.json", &key.metric_name));
                
                // Serialize the data
                let json = serde_json::to_string(&data_points)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                
                // Write to disk
                tokio::fs::write(&metric_file, json).await?;
            }
        }
        
        Ok(())
    }
    
    /// Load data for a specific metric from disk
    async fn load_from_disk(&self, key: &MetricKey) -> Result<Option<Vec<DataPoint>>, StorageError> {
        if let Some(path) = &self.config.storage_path {
            let metric_file = path.join(&key.component_id).join(format!("{}.json", &key.metric_name));
            
            if metric_file.exists() {
                // Read the file
                let json = tokio::fs::read_to_string(&metric_file).await?;
                
                // Deserialize the data
                let data_points: Vec<DataPoint> = serde_json::from_str(&json)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                
                return Ok(Some(data_points));
            }
        }
        
        Ok(None)
    }
    
    /// Get all metrics with available data
    pub async fn get_all_metrics(&self) -> Result<Vec<(String, String)>, StorageError> {
        let mut result = Vec::new();
        
        // Add all metrics in memory
        for key in self.in_memory.data.keys() {
            result.push((key.component_id.clone(), key.metric_name.clone()));
        }
        
        // If persistence is enabled, check for metrics on disk
        if self.config.persist_to_disk {
            if let Some(path) = &self.config.storage_path {
                if path.exists() {
                    // Read all component directories
                    let mut dirs = tokio::fs::read_dir(path).await?;
                    
                    while let Some(entry) = dirs.next_entry().await? {
                        let component_path = entry.path();
                        if component_path.is_dir() {
                            let component_id = component_path.file_name()
                                .ok_or_else(|| StorageError::Other("Invalid component directory name".to_string()))?
                                .to_string_lossy()
                                .to_string();
                            
                            // Read all metric files in the component directory
                            let mut files = tokio::fs::read_dir(&component_path).await?;
                            
                            while let Some(file_entry) = files.next_entry().await? {
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
            storage_path: Some(storage_path),
            persist_to_disk: true,
            ..Default::default()
        };
        
        // Create storage
        let mut storage = AnalyticsStorage::new(config).await.unwrap();
        
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
        // Create storage configuration with a short retention period
        let config = StorageConfig {
            retention_policy: RetentionPolicy {
                max_age_ms: 1000, // 1 second
                ..Default::default()
            },
            ..Default::default()
        };
        
        // Create storage
        let mut storage = AnalyticsStorage::new(config).await.unwrap();
        
        // Create test data
        let component_id = "test_component";
        let metric_name = "test_metric";
        let now = Utc::now().timestamp_millis();
        
        // Store an old data point
        let old_data_point = DataPoint {
            component_id: component_id.to_string(),
            metric_name: metric_name.to_string(),
            value: 42.0,
            timestamp: now - 2000, // 2 seconds ago, should be removed by retention policy
        };
        
        // Store a new data point
        let new_data_point = DataPoint {
            component_id: component_id.to_string(),
            metric_name: metric_name.to_string(),
            value: 43.0,
            timestamp: now, // Now, should be kept
        };
        
        // Store the data points
        storage.store_data_point(old_data_point).await.unwrap();
        storage.store_data_point(new_data_point).await.unwrap();
        
        // Try to retrieve the old data point (should fail)
        let result = storage.get_data_points(component_id, metric_name, now - 3000, now - 1500).await;
        assert!(result.is_err());
        
        // Retrieve the new data point (should succeed)
        let retrieved = storage.get_data_points(component_id, metric_name, now - 500, now + 500).await.unwrap();
        
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].value, 43.0);
    }
} 