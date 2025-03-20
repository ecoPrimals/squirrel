use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use std::f64;

use crate::app::monitoring::{Metric, MetricCollectorTrait};
use crate::error::{Result, SquirrelError};

/// Implementation of `MetricCollector` for collecting metrics
#[derive(Debug)]
pub struct MetricCollectorImpl {
    /// Storage for collected metrics
    metrics: Arc<RwLock<Vec<f64>>>,
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
            metrics: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl MetricCollectorTrait for MetricCollectorImpl {
    async fn collect(&self) -> Result<HashMap<String, Metric>> {
        let metrics = self.metrics.read()
            .map_err(|e| SquirrelError::Monitoring(format!("Failed to acquire read lock: {e}")))?;
        let mut result = HashMap::new();
        
        // Create some simple metrics with numeric indexes as keys
        for (i, metric) in metrics.iter().enumerate() {
            result.insert(format!("metric_{i}"), *metric);
        }
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = MetricCollectorImpl::new();
        collector.metrics.write().map_err(|_| "Failed to acquire write lock").unwrap().push(42.0);
        
        let metrics = collector.collect().await.expect("Failed to collect metrics");
        assert!(metrics.contains_key("metric_0"), "Metric should be present");
        
        let value = metrics.get("metric_0").expect("Metric should exist");
        assert!((value - 42.0).abs() < f64::EPSILON, "Metric value should be 42.0");
        
        // Update metrics
        {
            let mut data = collector.metrics.write()
                .map_err(|_| "Failed to acquire write lock")
                .expect("Failed to acquire write lock");
            data.insert(0, 100.0);
        }
        
        let collected = collector.collect().await.expect("Failed to collect updated metrics");
        let value = collected.get("metric_0").expect("Updated metric should exist");
        assert!((value - 100.0).abs() < f64::EPSILON, "Updated metric value should be 100.0");
    }
} 