//! Monitoring integration for circuit breakers

use std::error::Error;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use tracing::{debug, error, info, warn};

use super::{BreakerError, BreakerMetrics, BreakerState, CircuitBreaker};

// Determine if we need to mock the monitoring client
#[cfg(test)]
use crate::test_utils::monitoring::MockMonitoringClient as MonitoringClient;

#[cfg(not(test))]
use crate::monitoring::MonitoringClient;

/// A circuit breaker that reports metrics to the monitoring system
pub struct MonitoringCircuitBreaker {
    /// The underlying circuit breaker
    inner: Arc<dyn CircuitBreaker>,
    
    /// The monitoring client
    monitoring: Option<Arc<MonitoringClient>>,
    
    /// Name for this monitored circuit breaker
    name: String,
    
    /// Reporting interval in milliseconds
    report_interval_ms: u64,
    
    /// Last time metrics were reported
    last_report: std::time::Instant,
}

impl MonitoringCircuitBreaker {
    /// Create a new monitoring circuit breaker wrapping the provided circuit breaker
    pub fn new(circuit_breaker: impl CircuitBreaker + 'static, name: String) -> Self {
        Self {
            inner: Arc::new(circuit_breaker),
            monitoring: MonitoringClient::global().map(Arc::clone),
            name,
            report_interval_ms: 1000, // Default to 1 second
            last_report: std::time::Instant::now(),
        }
    }
    
    /// Set a custom reporting interval
    pub fn with_reporting_interval(mut self, interval_ms: u64) -> Self {
        self.report_interval_ms = interval_ms;
        self
    }
    
    /// Internal method to report metrics to the monitoring system if interval has elapsed
    async fn report_metrics_if_needed(&self) {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_report).as_millis() as u64;
        
        if elapsed >= self.report_interval_ms {
            if let Some(ref monitoring) = self.monitoring {
                let metrics = self.inner.metrics().await;
                
                // Convert to monitoring-compatible metrics
                let monitoring_metrics = metrics.to_monitoring_metrics();
                
                // Report each metric to monitoring
                for (name, value, labels) in monitoring_metrics {
                    let _ = monitoring.record_gauge(&name, value, labels).await;
                }
                
                // Additional event for state transitions
                if metrics.state_transition_count > 0 {
                    let state_name = match metrics.current_state {
                        BreakerState::Closed => "closed",
                        BreakerState::HalfOpen => "half-open",
                        BreakerState::Open => "open",
                    };
                    
                    let _ = monitoring.record_event(
                        "circuit_breaker_state_change",
                        &[
                            ("name", &self.name),
                            ("state", state_name),
                            ("time_in_previous_state", &metrics.time_in_state.as_secs().to_string()),
                        ],
                    ).await;
                }
            }
        }
    }
}

#[async_trait]
impl CircuitBreaker for MonitoringCircuitBreaker {
    async fn execute<F, T, E>(&self, operation: F) -> Result<T, BreakerError<E>>
    where
        F: Future<Output = Result<T, E>> + Send + 'static,
        T: Send + 'static,
        E: Error + Send + Sync + 'static,
    {
        // Report metrics before operation if needed
        self.report_metrics_if_needed().await;
        
        // Execute the operation through the inner circuit breaker
        let result = self.inner.execute(operation).await;
        
        // Report metrics immediately after failure or state change
        if result.is_err() {
            self.report_metrics_if_needed().await;
        }
        
        result
    }
    
    async fn state(&self) -> BreakerState {
        self.inner.state().await
    }
    
    async fn reset(&self) -> Result<(), BreakerError<anyhow::Error>> {
        let result = self.inner.reset().await;
        
        // Report metrics after reset
        if result.is_ok() {
            if let Some(ref monitoring) = self.monitoring {
                let _ = monitoring.record_event(
                    "circuit_breaker_manual_reset",
                    &[("name", &self.name)],
                ).await;
            }
        }
        
        result
    }
    
    async fn trip(&self) -> Result<(), BreakerError<anyhow::Error>> {
        let result = self.inner.trip().await;
        
        // Report metrics after manual trip
        if result.is_ok() {
            if let Some(ref monitoring) = self.monitoring {
                let _ = monitoring.record_event(
                    "circuit_breaker_manual_trip",
                    &[("name", &self.name)],
                ).await;
            }
        }
        
        result
    }
    
    async fn metrics(&self) -> BreakerMetrics {
        self.inner.metrics().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use crate::resilience::circuit_breaker::{BreakerConfig, StandardCircuitBreaker};
    use crate::test_utils::monitoring::MockMonitoringClient;
    
    #[tokio::test]
    async fn test_monitoring_integration() {
        // Setup mock monitoring client
        let mock_monitoring = MockMonitoringClient::new();
        MockMonitoringClient::set_global(Arc::new(mock_monitoring.clone()));
        
        // Create monitored circuit breaker
        let inner = StandardCircuitBreaker::new(BreakerConfig::default());
        let breaker = MonitoringCircuitBreaker::new(inner, "test-breaker".to_string())
            .with_reporting_interval(10); // Small interval for testing
            
        // Execute a successful operation
        let result = breaker.execute(|| async {
            Ok::<_, anyhow::Error>(42)
        }).await;
        
        assert!(result.is_ok());
        
        // Verify metrics were reported
        assert!(mock_monitoring.recorded_gauges().len() > 0);
        
        // Execute a failed operation
        let result = breaker.execute(|| async {
            Err::<i32, _>(anyhow!("test failure"))
        }).await;
        
        assert!(result.is_err());
        
        // Verify error event was recorded
        let events = mock_monitoring.recorded_events();
        assert!(!events.is_empty());
        
        // Trip the circuit breaker
        let _ = breaker.trip().await;
        
        // Verify trip event was recorded
        let events = mock_monitoring.recorded_events();
        assert!(events.iter().any(|e| e.0 == "circuit_breaker_manual_trip"));
        
        // Reset the circuit breaker
        let _ = breaker.reset().await;
        
        // Verify reset event was recorded
        let events = mock_monitoring.recorded_events();
        assert!(events.iter().any(|e| e.0 == "circuit_breaker_manual_reset"));
    }
} 