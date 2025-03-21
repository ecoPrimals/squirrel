use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use std::f64;

use crate::monitoring::{Metric, MetricCollectorTrait};
use crate::error::{Result, CoreError};
use squirrel_core::error::SquirrelError;

/// Metrics data structure
#[derive(Debug, Default)]
pub struct Metrics {
    /// Key-value pairs for metrics
    pub values: HashMap<String, f64>,
}

/// Implementation of `MetricCollector` for collecting metrics
#[derive(Debug)]
pub struct MetricCollectorImpl {
    /// Storage for collected metrics
    metrics: Arc<RwLock<HashMap<String, f64>>>,
}

impl Default for MetricCollectorImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricCollectorImpl {
    /// Create a new `MetricCollectorImpl`
    #[must_use]
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl MetricCollectorTrait for MetricCollectorImpl {
    async fn collect(&self) -> Result<HashMap<String, Metric>> {
        let metrics = self.metrics.read()
            .map_err(|e| SquirrelError::generic(format!("Failed to acquire read lock: {e}")))
            .map_err(|e| CoreError::Monitoring(e.to_string()))?;
        
        let mut result = HashMap::new();
        
        // Create metrics from the stored values
        for (key, value) in metrics.iter() {
            result.insert(key.clone(), *value);
        }
        
        Ok(result)
    }

    /// Get a specific metric by name
    /// 
    /// # Errors
    /// Returns an error if the metric cannot be found or if there's a problem accessing the metric
    async fn get_metric(&self, name: &str) -> Result<Option<f64>> {
        let metrics = self.metrics.read()
            .map_err(|e| SquirrelError::generic(format!("Failed to read metric values: {e}")))
            .map_err(|e| CoreError::Monitoring(e.to_string()))?;
        
        Ok(metrics.get(name).copied())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = MetricCollectorImpl::new();
        
        {
            let mut metrics = collector.metrics.write()
                .expect("Failed to acquire write lock");
            metrics.insert("metric_0".to_string(), 42.0);
        }
        
        let metrics = collector.collect().await.expect("Failed to collect metrics");
        assert!(metrics.contains_key("metric_0"), "Metric should be present");
        
        let value = metrics.get("metric_0").expect("Metric should exist");
        assert!((value - 42.0).abs() < f64::EPSILON, "Metric value should be 42.0");
        
        // Test the get_metric method
        let metric_value = collector.get_metric("metric_0").await.expect("Failed to get metric");
        assert_eq!(metric_value, Some(42.0), "get_metric should return the correct value");
        
        let non_existent = collector.get_metric("non_existent").await.expect("Failed to get metric");
        assert_eq!(non_existent, None, "get_metric should return None for non-existent metrics");
    }
} 