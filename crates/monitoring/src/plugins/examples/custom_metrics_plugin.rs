// Example custom metrics plugin
//
// This example demonstrates how to create a custom monitoring plugin
// for the monitoring crate.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use anyhow::Result;

use crate::plugins::common::{MonitoringPlugin, PluginMetadata};

/// Example custom metrics plugin that generates simulated metrics
#[derive(Debug)]
pub struct CustomMetricsPlugin {
    /// Plugin metadata
    metadata: PluginMetadata,
    
    /// Counter for metrics
    counter: AtomicU64,
    
    /// Last collection time
    last_collection: std::sync::Mutex<Option<Instant>>,
    
    /// Simulated metrics
    simulated_metrics: std::sync::RwLock<Vec<SimulatedMetric>>,
}

/// Simulated metric
#[derive(Debug, Clone)]
struct SimulatedMetric {
    /// Metric name
    name: String,
    
    /// Metric value
    value: f64,
    
    /// Metric rate of change per second
    rate: f64,
    
    /// Metric minimum value
    min: f64,
    
    /// Metric maximum value
    max: f64,
}

impl CustomMetricsPlugin {
    /// Create a new custom metrics plugin
    pub fn new() -> Self {
        let metadata = PluginMetadata::new(
            "Custom Metrics Plugin",
            "1.0.0",
            "Example plugin that generates simulated metrics",
            "DataScienceBioLab",
        )
        .with_capability("metrics")
        .with_capability("simulation");
        
        Self {
            metadata,
            counter: AtomicU64::new(0),
            last_collection: std::sync::Mutex::new(None),
            simulated_metrics: std::sync::RwLock::new(Vec::new()),
        }
    }
    
    /// Add a simulated metric
    pub fn add_metric(&self, name: &str, value: f64, rate: f64, min: f64, max: f64) {
        let metric = SimulatedMetric {
            name: name.to_string(),
            value,
            rate,
            min,
            max,
        };
        
        let mut metrics = self.simulated_metrics.write().unwrap();
        metrics.push(metric);
    }
    
    /// Update simulated metrics
    fn update_metrics(&self) {
        let now = Instant::now();
        let elapsed = {
            let mut last_collection = self.last_collection.lock().unwrap();
            let elapsed = last_collection
                .map(|last| now.duration_since(last).as_secs_f64())
                .unwrap_or(0.0);
            
            *last_collection = Some(now);
            elapsed
        };
        
        if elapsed > 0.0 {
            let mut metrics = self.simulated_metrics.write().unwrap();
            
            for metric in metrics.iter_mut() {
                // Update metric value based on rate of change
                let change = metric.rate * elapsed;
                metric.value += change;
                
                // Bound metric value
                if metric.value > metric.max {
                    metric.value = metric.max;
                    // Reverse direction
                    metric.rate = -metric.rate.abs();
                } else if metric.value < metric.min {
                    metric.value = metric.min;
                    // Reverse direction
                    metric.rate = metric.rate.abs();
                }
            }
        }
    }
}

#[async_trait]
impl MonitoringPlugin for CustomMetricsPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> Result<()> {
        // Initialize default simulated metrics if none exist
        {
            let metrics = self.simulated_metrics.read().unwrap();
            if metrics.is_empty() {
                drop(metrics);
                
                // Add some default simulated metrics
                self.add_metric("cpu_usage", 10.0, 2.0, 0.0, 100.0);
                self.add_metric("memory_usage", 200.0, 10.0, 100.0, 1000.0);
                self.add_metric("disk_usage", 50.0, 1.0, 0.0, 100.0);
                self.add_metric("network_traffic", 1000.0, 100.0, 0.0, 5000.0);
            }
        }
        
        // Set initial collection time
        {
            let mut last_collection = self.last_collection.lock().unwrap();
            *last_collection = Some(Instant::now());
        }
        
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        // Nothing to clean up
        Ok(())
    }
    
    async fn collect_metrics(&self) -> Result<Value> {
        // Increment counter
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        
        // Update simulated metrics
        self.update_metrics();
        
        // Collect metrics
        let metrics = self.simulated_metrics.read().unwrap();
        let mut metric_values = Vec::new();
        
        for metric in metrics.iter() {
            metric_values.push(json!({
                "name": metric.name,
                "value": metric.value,
                "rate": metric.rate,
                "min": metric.min,
                "max": metric.max,
            }));
        }
        
        // Return metrics as JSON
        Ok(json!({
            "collection_count": count,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "metrics": metric_values,
        }))
    }
    
    fn get_monitoring_targets(&self) -> Vec<String> {
        let metrics = self.simulated_metrics.read().unwrap();
        metrics.iter().map(|m| m.name.clone()).collect()
    }
    
    async fn handle_alert(&self, _alert: Value) -> Result<()> {
        // This plugin doesn't handle alerts
        Ok(())
    }
}

// Example of how to use this plugin
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::time::sleep;
    
    #[tokio::test]
    async fn test_custom_metrics_plugin() {
        // Create plugin
        let plugin = Arc::new(CustomMetricsPlugin::new());
        
        // Initialize plugin
        plugin.initialize().await.unwrap();
        
        // Collect initial metrics
        let metrics1 = plugin.collect_metrics().await.unwrap();
        println!("Initial metrics: {}", metrics1);
        
        // Wait a bit
        sleep(Duration::from_millis(100)).await;
        
        // Collect metrics again
        let metrics2 = plugin.collect_metrics().await.unwrap();
        println!("Updated metrics: {}", metrics2);
        
        // Verify collection count increased
        let count1 = metrics1["collection_count"].as_u64().unwrap();
        let count2 = metrics2["collection_count"].as_u64().unwrap();
        assert_eq!(count2, count1 + 1);
        
        // Verify metrics changed
        let cpu1 = metrics1["metrics"][0]["value"].as_f64().unwrap();
        let cpu2 = metrics2["metrics"][0]["value"].as_f64().unwrap();
        assert_ne!(cpu1, cpu2);
    }
} 