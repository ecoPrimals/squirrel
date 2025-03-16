// Allow certain linting issues that are too numerous to fix individually
#![allow(clippy::cast_precision_loss)] // Allow u64 to f64 casts for metrics
#![allow(clippy::missing_errors_doc)] // Temporarily allow missing error documentation
#![allow(clippy::unused_async)] // Allow unused async functions
#![allow(clippy::redundant_closure_for_method_calls)] // Allow redundant closures

use crate::error::{Result, SquirrelError};
use crate::monitoring::metrics::{Metric, MetricCollector, MetricType};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use futures::FutureExt;
use async_trait;

/// Tool execution metrics
#[derive(Debug, Clone, Default)]
pub struct ToolMetrics {
    pub name: String,
    pub usage_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub average_duration: f64,
}

impl ToolMetrics {
    pub fn new(name: String) -> Self {
        Self {
            name,
            usage_count: 0,
            success_count: 0,
            failure_count: 0,
            average_duration: 0.0,
        }
    }

    pub fn record_usage(&mut self, duration: f64, success: bool) {
        self.usage_count += 1;
        if success {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
        }

        // Update average duration using running average formula
        self.average_duration = (self.average_duration * (self.usage_count - 1) as f64 + duration) / self.usage_count as f64;
    }
    
    pub fn success_rate(&self) -> f64 {
        if self.usage_count == 0 {
            return 0.0;
        }
        self.success_count as f64 / self.usage_count as f64
    }
}

/// Tool metrics collector
#[derive(Debug)]
pub struct ToolMetricsCollector {
    metrics: Arc<RwLock<HashMap<String, ToolMetrics>>>,
}

impl ToolMetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_tool_metrics(&self, tool_name: &str) -> Result<Option<ToolMetrics>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.get(tool_name).cloned())
    }

    pub async fn get_all_metrics(&self) -> Result<HashMap<String, ToolMetrics>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.clone())
    }

    pub async fn record_tool_usage(&self, tool_name: &str, duration: f64, success: bool) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        let tool_metrics = metrics.entry(tool_name.to_string()).or_insert_with(|| ToolMetrics::new(tool_name.to_string()));
        tool_metrics.record_usage(duration, success);
        Ok(())
    }
}

#[async_trait::async_trait]
impl MetricCollector for ToolMetricsCollector {
    async fn start(&self) -> Result<()> {
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        Ok(())
    }

    async fn collect_metrics(&self) -> Result<Vec<Metric>> {
        let metrics = self.metrics.read().await;
        let mut result = Vec::new();

        for (tool_name, tool_metrics) in metrics.iter() {
            // Tool usage count
            let mut labels = HashMap::new();
            labels.insert("tool".to_string(), tool_name.clone());
            
            result.push(Metric::new(
                "tool.usage_count".to_string(),
                tool_metrics.usage_count as f64,
                MetricType::Counter,
                Some(labels.clone()),
            ));
            
            // Tool success count
            result.push(Metric::new(
                "tool.success_count".to_string(),
                tool_metrics.success_count as f64,
                MetricType::Counter,
                Some(labels.clone()),
            ));
            
            // Tool failure count
            result.push(Metric::new(
                "tool.failure_count".to_string(),
                tool_metrics.failure_count as f64,
                MetricType::Counter,
                Some(labels.clone()),
            ));
            
            // Tool average duration
            result.push(Metric::new(
                "tool.average_duration".to_string(),
                tool_metrics.average_duration,
                MetricType::Gauge,
                Some(labels.clone()),
            ));
            
            // Tool success rate
            if tool_metrics.usage_count > 0 {
                result.push(Metric::new(
                    "tool.success_rate".to_string(),
                    tool_metrics.success_rate(),
                    MetricType::Gauge,
                    Some(labels),
                ));
            }
        }
        
        Ok(result)
    }

    async fn record_metric(&self, _metric: Metric) -> Result<()> {
        // Not implemented for tool metrics collector
        Ok(())
    }
}

impl Default for ToolMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

// Static collector instance
static TOOL_COLLECTOR: tokio::sync::OnceCell<Arc<ToolMetricsCollector>> = tokio::sync::OnceCell::const_new();

pub async fn initialize() -> Result<()> {
    let collector = Arc::new(ToolMetricsCollector::new());
    TOOL_COLLECTOR.set(collector).map_err(|_| SquirrelError::metric("Tool metrics collector already initialized"))?;
    Ok(())
}

pub async fn get_tool_metrics(tool_name: &str) -> Option<ToolMetrics> {
    TOOL_COLLECTOR
        .get()
        .and_then(|c| c.get_tool_metrics(tool_name).now_or_never().and_then(|r| r.ok()).flatten())
}

pub async fn get_all_metrics() -> Option<HashMap<String, ToolMetrics>> {
    TOOL_COLLECTOR
        .get()
        .and_then(|c| c.get_all_metrics().now_or_never().and_then(|r| r.ok()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_metrics_collector() {
        let collector = ToolMetricsCollector::new();
        
        // Record tool usage
        collector.record_tool_usage("test_tool", 0.5, true).await.unwrap();
        collector.record_tool_usage("test_tool", 1.5, true).await.unwrap();
        collector.record_tool_usage("test_tool", 0.8, false).await.unwrap();
        
        // Get tool metrics
        let metrics = collector.get_tool_metrics("test_tool").await.unwrap().unwrap();
        
        assert_eq!(metrics.name, "test_tool");
        assert_eq!(metrics.usage_count, 3);
        assert_eq!(metrics.success_count, 2);
        assert_eq!(metrics.failure_count, 1);
        assert!(metrics.average_duration > 0.0);
        
        // Collect metrics
        let collected = collector.collect_metrics().await.unwrap();
        assert!(!collected.is_empty());
    }
} 