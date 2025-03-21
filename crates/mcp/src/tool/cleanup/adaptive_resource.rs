use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use tracing::{info, debug, instrument};

use crate::tool::ToolError;
use super::{
    ResourceUsage, ResourceLimits, ResourceStatus,
    ResourceType, ResourceRecord
};

/// Window size for resource usage history (in seconds)
const USAGE_HISTORY_WINDOW: i64 = 3600; // 1 hour

/// Prediction window for resource usage (in seconds)
const PREDICTION_WINDOW: i64 = 300; // 5 minutes

/// Resource usage pattern for prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePattern {
    /// Average usage over time
    pub average_usage: f64,
    /// Usage trend (positive means increasing)
    pub trend: f64,
    /// Seasonality pattern if any
    pub seasonality: Option<Vec<f64>>,
    /// Last update timestamp
    pub last_update: DateTime<Utc>,
}

/// Adaptive resource limits based on usage patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveResourceLimits {
    /// Base resource limits
    pub base_limits: ResourceLimits,
    /// Maximum allowed limits
    pub max_limits: ResourceLimits,
    /// Current adaptive limits
    pub current_limits: ResourceLimits,
    /// Last adjustment timestamp
    pub last_adjustment: DateTime<Utc>,
}

/// Resource usage predictor and optimizer
#[derive(Debug)]
pub struct AdaptiveResourceManager {
    /// Resource usage patterns by tool
    patterns: Arc<RwLock<HashMap<String, HashMap<ResourceType, ResourcePattern>>>>,
    /// Adaptive limits by tool
    adaptive_limits: Arc<RwLock<HashMap<String, AdaptiveResourceLimits>>>,
    /// Usage history for pattern analysis
    usage_history: Arc<RwLock<HashMap<String, Vec<ResourceRecord>>>>,
}

impl AdaptiveResourceManager {
    /// Creates a new adaptive resource manager
    pub fn new() -> Self {
        Self {
            patterns: Arc::new(RwLock::new(HashMap::new())),
            adaptive_limits: Arc::new(RwLock::new(HashMap::new())),
            usage_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initializes adaptive resource management for a tool
    #[instrument(skip(self))]
    pub async fn initialize_tool(
        &self,
        tool_id: &str,
        base_limits: ResourceLimits,
        max_limits: ResourceLimits,
    ) -> Result<(), ToolError> {
        // Initialize patterns
        {
            let mut patterns = self.patterns.write().await;
            patterns.insert(tool_id.to_string(), HashMap::new());
        }
        
        // Initialize adaptive limits
        {
            let mut adaptive_limits = self.adaptive_limits.write().await;
            adaptive_limits.insert(tool_id.to_string(), AdaptiveResourceLimits {
                base_limits: base_limits.clone(),
                max_limits,
                current_limits: base_limits,
                last_adjustment: Utc::now(),
            });
        }

        // Initialize usage history
        {
            let mut usage_history = self.usage_history.write().await;
            usage_history.insert(tool_id.to_string(), Vec::new());
        }

        info!("Initialized adaptive resource management for tool {}", tool_id);
        Ok(())
    }

    /// Records resource usage for pattern analysis
    #[instrument(skip(self))]
    pub async fn record_usage(
        &self,
        tool_id: &str,
        resource_type: ResourceType,
        usage: &ResourceUsage,
    ) -> Result<(), ToolError> {
        // Create the record first
        let record = ResourceRecord {
            tool_id: tool_id.to_string(),
            timestamp: Utc::now(),
            resource_type,
            usage: match resource_type {
                ResourceType::Memory => format!("{}", usage.memory_bytes),
                ResourceType::CpuTime => format!("{}", usage.cpu_time_ms),
                ResourceType::FileHandle => format!("{}", usage.file_handles.len()),
                ResourceType::NetworkConnection => format!("{}", usage.network_connections.len()),
            },
            status: ResourceStatus::Normal,
        };

        // Update history in a separate block to minimize lock time
        {
            let mut history = self.usage_history.write().await;
            let tool_history = history.get_mut(tool_id).ok_or_else(|| {
                ToolError::ToolNotFound(format!("Tool {} not found for usage recording", tool_id))
            })?;

            tool_history.push(record);

            // Cleanup old records
            let cutoff = Utc::now() - Duration::seconds(USAGE_HISTORY_WINDOW);
            tool_history.retain(|r| r.timestamp > cutoff);
        } // Lock is released here

        // Update patterns after releasing the history lock
        self.update_patterns(tool_id).await?;

        Ok(())
    }

    /// Updates resource usage patterns based on history
    #[instrument(skip(self))]
    async fn update_patterns(&self, tool_id: &str) -> Result<(), ToolError> {
        // First, get the history data we need
        let tool_history = {
            let history = self.usage_history.read().await;
            history.get(tool_id)
                .ok_or_else(|| ToolError::ToolNotFound(format!("Tool {} not found for pattern update", tool_id)))?
                .clone()
        };

        // Calculate patterns for each resource type
        let mut new_patterns = HashMap::new();
        for resource_type in &[
            ResourceType::Memory,
            ResourceType::CpuTime,
            ResourceType::FileHandle,
            ResourceType::NetworkConnection,
        ] {
            let type_history: Vec<_> = tool_history
                .iter()
                .filter(|r| r.resource_type == *resource_type)
                .collect();

            if type_history.is_empty() {
                continue;
            }

            // Calculate average usage
            let average_usage = type_history
                .iter()
                .map(|r| r.usage.parse::<f64>().unwrap_or(0.0))
                .sum::<f64>()
                / type_history.len() as f64;

            // Calculate trend
            let trend = if type_history.len() > 1 {
                let first = type_history.first().unwrap().usage.parse::<f64>().unwrap_or(0.0);
                let last = type_history.last().unwrap().usage.parse::<f64>().unwrap_or(0.0);
                (last - first) / USAGE_HISTORY_WINDOW as f64
            } else {
                0.0
            };

            // Create new pattern
            new_patterns.insert(*resource_type, ResourcePattern {
                average_usage,
                trend,
                seasonality: None, // TODO: Implement seasonality detection
                last_update: Utc::now(),
            });
        }

        // Update patterns with the new data
        let mut patterns = self.patterns.write().await;
        let tool_patterns = patterns.get_mut(tool_id)
            .ok_or_else(|| ToolError::ToolNotFound(format!("Tool {} not found for pattern update", tool_id)))?;
        
        // Update all patterns at once
        *tool_patterns = new_patterns;

        Ok(())
    }

    /// Predicts resource usage for the next window
    #[instrument(skip(self))]
    pub async fn predict_usage(
        &self,
        tool_id: &str,
        resource_type: ResourceType,
    ) -> Result<f64, ToolError> {
        let patterns = self.patterns.read().await;
        let tool_patterns = patterns.get(tool_id).ok_or_else(|| {
            ToolError::ToolNotFound(format!("Tool {} not found for usage prediction", tool_id))
        })?;

        let pattern = tool_patterns.get(&resource_type).ok_or_else(|| {
            ToolError::ToolNotFound(format!(
                "No pattern found for resource type {:?} in tool {}",
                resource_type, tool_id
            ))
        })?;

        // Simple linear prediction
        let predicted = pattern.average_usage + (pattern.trend * PREDICTION_WINDOW as f64);
        
        Ok(predicted.max(0.0)) // Ensure non-negative prediction
    }

    /// Adjusts resource limits based on usage patterns
    #[instrument(skip(self))]
    pub async fn adjust_limits(&self, tool_id: &str) -> Result<ResourceLimits, ToolError> {
        // First check if adjustment is needed
        let should_adjust = {
            let adaptive_limits = self.adaptive_limits.read().await;
            let tool_limits = adaptive_limits.get(tool_id).ok_or_else(|| {
                ToolError::ToolNotFound(format!("Tool {} not found for limit adjustment", tool_id))
            })?;
            
            let now = Utc::now();
            (now - tool_limits.last_adjustment).num_seconds() >= 300
        };

        if !should_adjust {
            let adaptive_limits = self.adaptive_limits.read().await;
            let tool_limits = adaptive_limits.get(tool_id).ok_or_else(|| {
                ToolError::ToolNotFound(format!("Tool {} not found for limit adjustment", tool_id))
            })?;
            return Ok(tool_limits.current_limits.clone());
        }

        // Make predictions without holding any locks
        let memory_prediction = self.predict_usage(tool_id, ResourceType::Memory).await?;
        let cpu_prediction = self.predict_usage(tool_id, ResourceType::CpuTime).await?;
        let file_prediction = self.predict_usage(tool_id, ResourceType::FileHandle).await?;
        let network_prediction = self.predict_usage(tool_id, ResourceType::NetworkConnection).await?;

        // Now update limits with all predictions ready
        let mut adaptive_limits = self.adaptive_limits.write().await;
        let tool_limits = adaptive_limits.get_mut(tool_id).ok_or_else(|| {
            ToolError::ToolNotFound(format!("Tool {} not found for limit adjustment", tool_id))
        })?;

        // Calculate new limits with 20% headroom
        let new_limits = ResourceLimits {
            max_memory_bytes: ((memory_prediction * 1.2) as usize)
                .min(tool_limits.max_limits.max_memory_bytes)
                .max(tool_limits.base_limits.max_memory_bytes),
            max_cpu_time_ms: ((cpu_prediction * 1.2) as u64)
                .min(tool_limits.max_limits.max_cpu_time_ms)
                .max(tool_limits.base_limits.max_cpu_time_ms),
            max_file_handles: ((file_prediction * 1.2) as usize)
                .min(tool_limits.max_limits.max_file_handles)
                .max(tool_limits.base_limits.max_file_handles),
            max_network_connections: ((network_prediction * 1.2) as usize)
                .min(tool_limits.max_limits.max_network_connections)
                .max(tool_limits.base_limits.max_network_connections),
        };

        tool_limits.current_limits = new_limits.clone();
        tool_limits.last_adjustment = Utc::now();

        debug!(
            "Adjusted resource limits for tool {}: {:?}",
            tool_id, new_limits
        );

        Ok(new_limits)
    }

    /// Gets the current adaptive limits for a tool
    #[instrument(skip(self))]
    pub async fn get_current_limits(&self, tool_id: &str) -> Result<ResourceLimits, ToolError> {
        let adaptive_limits = self.adaptive_limits.read().await;
        let tool_limits = adaptive_limits.get(tool_id).ok_or_else(|| {
            ToolError::ToolNotFound(format!("Tool {} not found for limit lookup", tool_id))
        })?;

        Ok(tool_limits.current_limits.clone())
    }

    /// Cleans up adaptive resource management for a tool
    #[instrument(skip(self))]
    pub async fn cleanup_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        // Clean up patterns
        {
            let mut patterns = self.patterns.write().await;
            patterns.remove(tool_id);
        }

        // Clean up adaptive limits
        {
            let mut adaptive_limits = self.adaptive_limits.write().await;
            adaptive_limits.remove(tool_id);
        }

        // Clean up usage history
        {
            let mut usage_history = self.usage_history.write().await;
            usage_history.remove(tool_id);
        }

        info!("Cleaned up adaptive resource management for tool {}", tool_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration as TokioDuration};

    #[tokio::test]
    async fn test_adaptive_resource_management() {
        let manager = AdaptiveResourceManager::new();
        let tool_id = "test_tool";

        // Initialize with base and max limits
        let base_limits = ResourceLimits {
            max_memory_bytes: 100_000_000,
            max_cpu_time_ms: 30_000,
            max_file_handles: 50,
            max_network_connections: 10,
        };

        let max_limits = ResourceLimits {
            max_memory_bytes: 500_000_000,
            max_cpu_time_ms: 120_000,
            max_file_handles: 200,
            max_network_connections: 50,
        };

        manager
            .initialize_tool(tool_id, base_limits.clone(), max_limits.clone())
            .await
            .unwrap();

        // Simulate resource usage over time
        for i in 0..5 {
            let usage = ResourceUsage {
                memory_bytes: 100_000_000 + (i * 10_000_000),
                cpu_time_ms: 10_000u64 + (i as u64 * 1_000),
                file_handles: vec![1, 2, 3],
                network_connections: vec![1],
            };

            manager
                .record_usage(tool_id, ResourceType::Memory, &usage)
                .await
                .unwrap();

            manager
                .record_usage(tool_id, ResourceType::CpuTime, &usage)
                .await
                .unwrap();

            sleep(TokioDuration::from_millis(100)).await;
        }

        // Check predictions
        let memory_prediction = manager
            .predict_usage(tool_id, ResourceType::Memory)
            .await
            .unwrap();
        assert!(memory_prediction > 100_000_000.0);

        // Check limit adjustments
        let new_limits = manager.adjust_limits(tool_id).await.unwrap();
        assert!(new_limits.max_memory_bytes >= base_limits.max_memory_bytes);
        assert!(new_limits.max_memory_bytes <= max_limits.max_memory_bytes);

        // Cleanup
        manager.cleanup_tool(tool_id).await.unwrap();
    }
} 