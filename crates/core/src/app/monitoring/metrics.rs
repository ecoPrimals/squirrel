use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use time::OffsetDateTime;
use crate::monitoring::metrics::Metric;
use crate::error::Error;
use crate::monitoring::MonitoringServiceFactory;

/// Application metric types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppMetricType {
    /// Request/Response metrics
    Request {
        /// Request path or identifier
        path: String,
        /// HTTP method or operation type
        method: String,
        /// Response status code
        status: u16,
        /// Request duration in milliseconds
        duration_ms: f64,
    },
    /// Resource usage metrics
    Resource {
        /// Resource type (memory, cpu, disk, etc.)
        resource_type: String,
        /// Resource usage value
        usage: f64,
        /// Resource limit
        limit: Option<f64>,
    },
    /// Business metrics
    Business {
        /// Metric category
        category: String,
        /// Metric name
        name: String,
        /// Metric value
        value: f64,
    },
    /// Performance metrics
    Performance {
        /// Operation name
        operation: String,
        /// Duration in milliseconds
        duration_ms: f64,
        /// Success flag
        success: bool,
    },
}

impl AppMetricType {
    /// Convert to a monitoring metric
    pub fn to_metric(&self, prefix: &str) -> Metric {
        let (name, value, mut labels) = match self {
            AppMetricType::Request { path, method, status, duration_ms } => {
                let mut labels = HashMap::new();
                labels.insert("path".to_string(), path.clone());
                labels.insert("method".to_string(), method.clone());
                labels.insert("status".to_string(), status.to_string());
                (
                    format!("{}.request.duration", prefix),
                    *duration_ms,
                    labels,
                )
            },
            AppMetricType::Resource { resource_type, usage, limit } => {
                let mut labels = HashMap::new();
                labels.insert("type".to_string(), resource_type.clone());
                if let Some(limit) = limit {
                    labels.insert("limit".to_string(), limit.to_string());
                }
                (
                    format!("{}.resource.usage", prefix),
                    *usage,
                    labels,
                )
            },
            AppMetricType::Business { category, name, value } => {
                let mut labels = HashMap::new();
                labels.insert("category".to_string(), category.clone());
                (
                    format!("{}.business.{}", prefix, name),
                    *value,
                    labels,
                )
            },
            AppMetricType::Performance { operation, duration_ms, success } => {
                let mut labels = HashMap::new();
                labels.insert("operation".to_string(), operation.clone());
                labels.insert("success".to_string(), success.to_string());
                (
                    format!("{}.performance.duration", prefix),
                    *duration_ms,
                    labels,
                )
            },
        };

        labels.insert("component".to_string(), "app".to_string());

        Metric {
            name,
            value,
            labels,
            timestamp: OffsetDateTime::now_utc(),
        }
    }
}

/// Application metric collector
#[derive(Debug)]
pub struct AppMetricCollector {
    /// Metric prefix
    prefix: String,
}

impl AppMetricCollector {
    /// Create a new application metric collector
    pub fn new(prefix: String) -> Self {
        Self { prefix }
    }

    /// Record a request metric
    pub async fn record_request(
        &self,
        path: &str,
        method: &str,
        status: u16,
        duration_ms: f64,
    ) -> Result<(), Error> {
        let metric = AppMetricType::Request {
            path: path.to_string(),
            method: method.to_string(),
            status,
            duration_ms,
        };
        self.record_metric(&metric).await
    }

    /// Record a resource metric
    pub async fn record_resource(
        &self,
        resource_type: &str,
        usage: f64,
        limit: Option<f64>,
    ) -> Result<(), Error> {
        let metric = AppMetricType::Resource {
            resource_type: resource_type.to_string(),
            usage,
            limit,
        };
        self.record_metric(&metric).await
    }

    /// Record a business metric
    pub async fn record_business(
        &self,
        category: &str,
        name: &str,
        value: f64,
    ) -> Result<(), Error> {
        let metric = AppMetricType::Business {
            category: category.to_string(),
            name: name.to_string(),
            value,
        };
        self.record_metric(&metric).await
    }

    /// Record a performance metric
    pub async fn record_performance(
        &self,
        operation: &str,
        duration_ms: f64,
        success: bool,
    ) -> Result<(), Error> {
        let metric = AppMetricType::Performance {
            operation: operation.to_string(),
            duration_ms,
            success,
        };
        self.record_metric(&metric).await
    }

    /// Record a metric
    async fn record_metric(&self, metric: &AppMetricType) -> Result<(), Error> {
        let metric = metric.to_metric(&self.prefix);
        crate::monitoring::metrics::record_metric(metric)
            .await
            .map_err(|e| Error::Monitoring(format!("Failed to record metric: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_app_metrics() {
        // Reset any previous monitoring state
        let _ = crate::monitoring::shutdown().await;
        
        // Initialize monitoring with factory pattern
        let config = crate::monitoring::MonitoringConfig::default();
        let factory = MonitoringServiceFactory::new(config);
        let service = factory.create_service();
        service.start().await.unwrap();

        let collector = AppMetricCollector::new("test".to_string());

        // Test request metrics
        assert!(collector.record_request(
            "/api/test",
            "GET",
            200,
            42.0,
        ).await.is_ok());

        // Test resource metrics
        assert!(collector.record_resource(
            "memory",
            1024.0,
            Some(2048.0),
        ).await.is_ok());

        // Test business metrics
        assert!(collector.record_business(
            "users",
            "active",
            100.0,
        ).await.is_ok());

        // Test performance metrics
        assert!(collector.record_performance(
            "database_query",
            15.0,
            true,
        ).await.is_ok());

        // Allow time for metrics to be collected
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Verify metrics were recorded
        let metrics = service.metric_collector().get_metrics().await;

        assert!(!metrics.is_empty());
        assert!(metrics.iter().any(|m| m.name == "test.request.duration"));
        assert!(metrics.iter().any(|m| m.name == "test.resource.usage"));
        assert!(metrics.iter().any(|m| m.name == "test.business.active"));
        assert!(metrics.iter().any(|m| m.name == "test.performance.duration"));
        
        // Clean up
        service.stop().await.unwrap();
    }
} 