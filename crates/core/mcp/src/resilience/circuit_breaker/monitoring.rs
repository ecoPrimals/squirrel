// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Monitoring integration for circuit breakers

use std::sync::Arc;
use std::time::Duration;

use crate::error::Result;
use crate::monitoring::{create_production_monitoring_client, MonitoringClient, MonitoringEvent, MetricValue, AlertLevel};
use std::collections::HashMap;
use chrono::Utc;

/// Circuit breaker monitoring integration
pub struct CircuitBreakerMonitoring {
    monitoring_client: Arc<dyn MonitoringClient>,
    service_name: String,
}

impl CircuitBreakerMonitoring {
    /// Create new circuit breaker monitoring with production client
    pub fn new(service_name: String) -> Self {
        Self {
            monitoring_client: create_production_monitoring_client(),
            service_name,
        }
    }

    /// Create with custom monitoring client (for testing)
    pub fn with_client(service_name: String, client: Arc<dyn MonitoringClient>) -> Self {
        Self {
            monitoring_client: client,
            service_name,
        }
    }

    /// Record circuit breaker state change
    pub async fn record_state_change(
        &self,
        circuit_name: &str,
        old_state: &str,
        new_state: &str,
    ) -> Result<()> {
        let event = MonitoringEvent {
            timestamp: Utc::now(),
            event_type: "circuit_breaker_state_change".to_string(),
            message: format!(
                "Circuit breaker '{}' changed from {} to {}",
                circuit_name, old_state, new_state
            ),
            level: match new_state {
                "Open" => AlertLevel::High,
                "HalfOpen" => AlertLevel::Medium,
                "Closed" => AlertLevel::Info,
                _ => AlertLevel::Low,
            },
            source: self.service_name.clone(),
            tags: {
                let mut tags = HashMap::new();
                tags.insert("circuit_name".to_string(), circuit_name.to_string());
                tags.insert("old_state".to_string(), old_state.to_string());
                tags.insert("new_state".to_string(), new_state.to_string());
                tags
            },
            metadata: HashMap::new(),
        };

        self.monitoring_client.record_event(event).await
    }

    /// Record circuit breaker failure
    pub async fn record_failure(
        &self,
        circuit_name: &str,
        error_message: &str,
    ) -> Result<()> {
        let event = MonitoringEvent {
            timestamp: Utc::now(),
            event_type: "circuit_breaker_failure".to_string(),
            message: format!(
                "Circuit breaker '{}' recorded failure: {}",
                circuit_name, error_message
            ),
            level: AlertLevel::Medium,
            source: self.service_name.clone(),
            tags: {
                let mut tags = HashMap::new();
                tags.insert("circuit_name".to_string(), circuit_name.to_string());
                tags.insert("error".to_string(), error_message.to_string());
                tags
            },
            metadata: HashMap::new(),
        };

        self.monitoring_client.record_event(event).await
    }

    /// Record circuit breaker success
    pub async fn record_success(&self, circuit_name: &str) -> Result<()> {
        // Record success metric
        let mut tags = HashMap::new();
        tags.insert("circuit_name".to_string(), circuit_name.to_string());
        
        self.monitoring_client
            .record_metric(
                "circuit_breaker_success",
                MetricValue::Integer(1),
                Some(tags),
            )
            .await
    }

    /// Record circuit breaker timeout
    pub async fn record_timeout(
        &self,
        circuit_name: &str,
        duration: Duration,
    ) -> Result<()> {
        let event = MonitoringEvent {
            timestamp: Utc::now(),
            event_type: "circuit_breaker_timeout".to_string(),
            message: format!(
                "Circuit breaker '{}' timed out after {:?}",
                circuit_name, duration
            ),
            level: AlertLevel::Medium,
            source: self.service_name.clone(),
            tags: {
                let mut tags = HashMap::new();
                tags.insert("circuit_name".to_string(), circuit_name.to_string());
                tags.insert("duration_ms".to_string(), duration.as_millis().to_string());
                tags
            },
            metadata: HashMap::new(),
        };

        self.monitoring_client.record_event(event).await
    }

    /// Record circuit breaker metrics
    pub async fn record_metrics(
        &self,
        circuit_name: &str,
        failure_count: u64,
        success_count: u64,
        success_rate: f64,
    ) -> Result<()> {
        let mut tags = HashMap::new();
        tags.insert("circuit_name".to_string(), circuit_name.to_string());

        // Record failure count
        self.monitoring_client
            .record_metric(
                "circuit_breaker_failures",
                MetricValue::Integer(failure_count as i64),
                Some(tags.clone()),
            )
            .await?;

        // Record success count
        self.monitoring_client
            .record_metric(
                "circuit_breaker_successes",
                MetricValue::Integer(success_count as i64),
                Some(tags.clone()),
            )
            .await?;

        // Record success rate
        self.monitoring_client
            .record_metric(
                "circuit_breaker_success_rate",
                MetricValue::Float(success_rate),
                Some(tags),
            )
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::create_production_monitoring_client;

    #[tokio::test]
    async fn test_circuit_breaker_monitoring_creation() {
        let monitoring = CircuitBreakerMonitoring::new("test-service".to_string());
        assert_eq!(monitoring.service_name, "test-service");
    }

    #[tokio::test]
    async fn test_record_state_change() {
        let monitoring = CircuitBreakerMonitoring::new("test-service".to_string());
        let result = monitoring
            .record_state_change("test-circuit", "Closed", "Open")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_failure() {
        let monitoring = CircuitBreakerMonitoring::new("test-service".to_string());
        let result = monitoring
            .record_failure("test-circuit", "Connection timeout")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_metrics() {
        let monitoring = CircuitBreakerMonitoring::new("test-service".to_string());
        let result = monitoring
            .record_metrics("test-circuit", 5, 95, 0.95)
            .await;
        assert!(result.is_ok());
    }
} 