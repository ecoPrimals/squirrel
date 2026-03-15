// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Adapter for integrating resilience health checks with the monitoring system.
//!
//! This module provides an adapter that allows resilience health checks to be
//! registered with the global monitoring system.

use std::fmt;
use tracing::debug;

use crate::error::Result;
use crate::monitoring::{
    HealthStatus as MonitoringHealth,
    SyncHealth, PersistenceHealth, ResourceHealth,
    metrics::{Metric, MetricType, MetricValue}
};
use crate::monitoring::alerts::AlertSeverity;
use crate::resilience::health::{HealthCheck, HealthStatus, HealthCheckResult};

/// Adapter for integrating resilience health checks with the monitoring system.
pub struct ResilienceHealthCheckAdapter<T> where T: HealthCheck {
    /// The inner resilience health check
    inner: T,
    /// Whether to forward metrics to the monitoring system
    forward_metrics: bool,
}

impl<T> fmt::Debug for ResilienceHealthCheckAdapter<T> where T: HealthCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResilienceHealthCheckAdapter")
            .field("inner", &self.inner)
            .field("forward_metrics", &self.forward_metrics)
            .finish()
    }
}

impl<T> ResilienceHealthCheckAdapter<T> where T: HealthCheck {
    /// Create a new adapter around a resilience health check
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            forward_metrics: true,
        }
    }
    
    /// Set whether to forward metrics to the monitoring system
    pub fn with_forward_metrics(mut self, forward_metrics: bool) -> Self {
        self.forward_metrics = forward_metrics;
        self
    }
    
    /// Get a reference to the inner health check
    pub fn inner(&self) -> &T {
        &self.inner
    }
    
    /// Get a mutable reference to the inner health check
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
    
    /// Convert a resilience health status to a monitoring system health status
    pub fn convert_status(&self, status: HealthStatus) -> MonitoringHealth {
        // Get current time
        let now = chrono::Utc::now();
        
        // Create base monitoring health status
        match status {
            HealthStatus::Healthy => MonitoringHealth {
                is_healthy: true,
                last_check: now,
                sync_status: SyncHealth {
                    is_syncing: false,
                    last_successful_sync: now,
                    consecutive_failures: 0,
                },
                persistence_status: PersistenceHealth {
                    storage_available: true,
                    last_write_success: now,
                    storage_usage_percent: 0.0,
                },
                resource_status: ResourceHealth {
                    cpu_usage_percent: 5.0,
                    memory_usage_percent: 10.0,
                    disk_usage_percent: 15.0,
                },
            },
            HealthStatus::Degraded | HealthStatus::Warning => MonitoringHealth {
                is_healthy: false,
                last_check: now,
                sync_status: SyncHealth {
                    is_syncing: false,
                    last_successful_sync: now,
                    consecutive_failures: 1,
                },
                persistence_status: PersistenceHealth {
                    storage_available: true,
                    last_write_success: now,
                    storage_usage_percent: 30.0,
                },
                resource_status: ResourceHealth {
                    cpu_usage_percent: 30.0,
                    memory_usage_percent: 40.0,
                    disk_usage_percent: 35.0,
                },
            },
            HealthStatus::Unhealthy | HealthStatus::Critical | HealthStatus::Unknown => MonitoringHealth {
                is_healthy: false,
                last_check: now,
                sync_status: SyncHealth {
                    is_syncing: false,
                    last_successful_sync: now,
                    consecutive_failures: 3,
                },
                persistence_status: PersistenceHealth {
                    storage_available: false,
                    last_write_success: now,
                    storage_usage_percent: 85.0,
                },
                resource_status: ResourceHealth {
                    cpu_usage_percent: 80.0,
                    memory_usage_percent: 85.0,
                    disk_usage_percent: 75.0,
                },
            },
        }
    }
    
    /// Convert a resilience health status to an alert severity
    pub fn convert_to_severity(&self, status: HealthStatus) -> AlertSeverity {
        match status {
            HealthStatus::Healthy => AlertSeverity::Info,
            HealthStatus::Warning => AlertSeverity::Warning,
            HealthStatus::Degraded => AlertSeverity::Warning,
            HealthStatus::Unhealthy => AlertSeverity::Error,
            HealthStatus::Critical => AlertSeverity::Critical,
            HealthStatus::Unknown => AlertSeverity::Warning,
        }
    }
    
    /// Perform the health check and convert the result to monitoring format
    pub async fn check(&self) -> Result<MonitoringHealth> {
        // Get the resilience health check result
        let result = self.inner.check().await;
        debug!("Resilience health check for {} returned status: {}", self.inner.id(), result.status);
        
        // Convert to monitoring system health status
        let health_status = self.convert_status(result.status);
        
        Ok(health_status)
    }
    
    /// Get the name of the health check
    pub fn name(&self) -> &str {
        self.inner.id()
    }
}

/// Create metrics from a resilience health check result
pub fn create_metrics_from_health_result(
    component_id: &str, 
    result: &HealthCheckResult
) -> Vec<Metric> {
    let mut metrics = Vec::new();
    
    // Add status metric
    let status_value = match result.status {
        HealthStatus::Healthy => 0,
        HealthStatus::Degraded => 1,
        HealthStatus::Warning => 2,
        HealthStatus::Unhealthy => 3,
        HealthStatus::Critical => 4,
        HealthStatus::Unknown => 5,
    };
    
    // Create status metric
    let status_metric = Metric::new(
        format!("mcp.resilience.{}.status", component_id),
        format!("Health status of component {}", component_id),
        MetricType::Gauge,
        MetricValue::Integer(status_value as i64)
    );
    metrics.push(status_metric);
    
    // Add component-specific metrics
    for (name, value) in &result.metrics {
        let metric_name = format!("mcp.resilience.{}.{}", component_id, name);
        let description = format!("{} metric for component {}", name, component_id);
        let metric = Metric::new(
            metric_name,
            description,
            MetricType::Gauge,
            MetricValue::Float(*value)
        )
        .with_label("component", component_id);
        
        metrics.push(metric);
    }
    
    metrics
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use async_trait::async_trait;
    
    /// Simple mock health check for testing
    #[derive(Debug, Clone)]
    struct MockHealthCheck {
        id: String,
        status: HealthStatus,
    }
    
    impl MockHealthCheck {
        fn new(id: &str, status: HealthStatus) -> Self {
            Self {
                id: id.to_string(),
                status,
            }
        }
        
        fn set_status(&mut self, status: HealthStatus) {
            self.status = status;
        }
    }
    
    #[async_trait]
    impl HealthCheck for MockHealthCheck {
        fn id(&self) -> &str {
            &self.id
        }
        
        async fn check(&self) -> HealthCheckResult {
            let mut result = HealthCheckResult::new(
                self.id.clone(),
                self.status,
                format!("Mock health check for {} is {}", self.id, self.status),
            );
            
            // Add some metrics
            result = result.with_metric("response_time_ms", 123.45);
            result = result.with_metric("error_rate", 0.05);
            
            result
        }
        
        fn config(&self) -> &crate::resilience::health::HealthCheckConfig {
            // Return a static default config for testing
            use std::sync::LazyLock;
            static CONFIG: LazyLock<crate::resilience::health::HealthCheckConfig> = 
                LazyLock::new(|| crate::resilience::health::HealthCheckConfig::default());
            &CONFIG
        }
        
        fn config_mut(&mut self) -> &mut crate::resilience::health::HealthCheckConfig {
            // This is only used for tests, so we panic if called
            panic!("config_mut not implemented for MockHealthCheck")
        }
    }
    
    /// Test basic adapter initialization
    #[test]
    fn test_adapter_init() {
        let health_check = MockHealthCheck::new("test_component", HealthStatus::Healthy);
        let adapter = ResilienceHealthCheckAdapter::new(health_check);
        
        assert_eq!(adapter.name(), "test_component");
        assert!(adapter.forward_metrics);
    }
    
    /// Test health status conversion
    #[test]
    fn test_status_conversion() {
        let health_check = MockHealthCheck::new("test_component", HealthStatus::Healthy);
        let adapter = ResilienceHealthCheckAdapter::new(health_check);
        
        let healthy_status = adapter.convert_status(HealthStatus::Healthy);
        assert!(healthy_status.is_healthy);
        assert_eq!(healthy_status.sync_status.consecutive_failures, 0);
        
        let warning_status = adapter.convert_status(HealthStatus::Warning);
        assert!(!warning_status.is_healthy);
        assert_eq!(warning_status.sync_status.consecutive_failures, 1);
        
        let critical_status = adapter.convert_status(HealthStatus::Critical);
        assert!(!critical_status.is_healthy);
        assert_eq!(critical_status.sync_status.consecutive_failures, 3);
        assert!(!critical_status.persistence_status.storage_available);
    }
    
    /// Test severity conversion
    #[test]
    fn test_severity_conversion() {
        let health_check = MockHealthCheck::new("test_component", HealthStatus::Healthy);
        let adapter = ResilienceHealthCheckAdapter::new(health_check);
        
        assert_eq!(adapter.convert_to_severity(HealthStatus::Healthy), AlertSeverity::Info);
        assert_eq!(adapter.convert_to_severity(HealthStatus::Warning), AlertSeverity::Warning);
        assert_eq!(adapter.convert_to_severity(HealthStatus::Degraded), AlertSeverity::Warning);
        assert_eq!(adapter.convert_to_severity(HealthStatus::Unhealthy), AlertSeverity::Error);
        assert_eq!(adapter.convert_to_severity(HealthStatus::Critical), AlertSeverity::Critical);
        assert_eq!(adapter.convert_to_severity(HealthStatus::Unknown), AlertSeverity::Warning);
    }
    
    /// Test metrics creation
    #[test]
    fn test_create_metrics() {
        let component_id = "test_component";
        let mut result = HealthCheckResult::new(
            component_id.to_string(),
            HealthStatus::Warning,
            "Component is degraded".to_string(),
        );
        
        result = result.with_metric("response_time", 123.45);
        result = result.with_metric("error_rate", 0.05);
        
        let metrics = create_metrics_from_health_result(component_id, &result);
        
        // Should have 3 metrics: status, response_time, error_rate
        assert_eq!(metrics.len(), 3);
        
        // Check status metric
        let status_metric = metrics.iter().find(|m| m.name.ends_with(".status")).unwrap();
        assert_eq!(status_metric.metric_type, MetricType::Gauge);
        match status_metric.value {
            MetricValue::Integer(val) => assert_eq!(val, 2), // Warning status value
            _ => panic!("Expected integer value for status metric"),
        }
        
        // Check other metrics
        let response_time_metric = metrics.iter().find(|m| m.name.ends_with(".response_time")).unwrap();
        match response_time_metric.value {
            MetricValue::Float(val) => assert_eq!(val, 123.45),
            _ => panic!("Expected float value for response_time metric"),
        }
    }
    
    /// Test health check execution
    #[tokio::test]
    async fn test_health_check() {
        let mut health_check = MockHealthCheck::new("test_component", HealthStatus::Healthy);
        let adapter = ResilienceHealthCheckAdapter::new(health_check.clone());
        
        // Check with healthy status
        let result = adapter.check().await.unwrap();
        assert!(result.is_healthy);
        
        // Set component to unhealthy and recreate adapter
        health_check.set_status(HealthStatus::Unhealthy);
        let adapter = ResilienceHealthCheckAdapter::new(health_check);
        
        // Check with unhealthy status
        let result = adapter.check().await.unwrap();
        assert!(!result.is_healthy);
        assert_eq!(result.sync_status.consecutive_failures, 3);
    }
} 