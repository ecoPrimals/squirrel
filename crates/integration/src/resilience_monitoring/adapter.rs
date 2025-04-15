use std::collections::HashMap;
use std::time::Instant;
use std::sync::Arc;
use std::fmt::Debug;
use async_trait::async_trait;
use squirrel_mcp::resilience::circuit_breaker::{
    CircuitBreaker, StandardCircuitBreaker, BreakerState
};
use squirrel_monitoring::health::component::{ComponentHealth, HealthCheck};
use squirrel_monitoring::health::status::Status;
use squirrel_monitoring::health::adapter::HealthCheckerAdapter;
use squirrel_monitoring::alerts::{Alert, AlertLevel};
use squirrel_monitoring::alerts::manager::AlertManager;
use squirrel_monitoring::plugins::AlertHandler;
use crate::error::IntegrationError;

/// Configuration for the resilience monitoring adapter
#[derive(Debug, Clone)]
pub struct ResilienceMonitoringConfig {
    /// Interval between metrics reports in seconds
    pub metrics_interval_secs: u64,
    /// Whether to include detailed metrics
    pub include_detailed_metrics: bool,
    /// Alert level for circuit breaker open events
    pub circuit_open_alert_level: AlertLevel,
}

impl Default for ResilienceMonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_interval_secs: 60, // Report metrics every 60 seconds by default
            include_detailed_metrics: true,
            circuit_open_alert_level: AlertLevel::Warning,
        }
    }
}

/// Monitored circuit breaker with additional metadata
struct MonitoredCircuitBreaker {
    /// The underlying circuit breaker
    circuit_breaker: Arc<StandardCircuitBreaker>,
    /// Human-readable name for the circuit breaker
    name: String,
    /// Component ID for health checks and alerts
    component_id: String,
    /// Last time metrics were reported
    last_metrics_report: Instant,
    /// Last known state of the circuit breaker
    last_known_state: BreakerState,
}

/// Adapter for monitoring circuit breakers
pub struct ResilienceMonitoringAdapter {
    /// Configuration for the adapter
    config: ResilienceMonitoringConfig,
    /// List of monitored circuit breakers
    monitored_circuit_breakers: Vec<MonitoredCircuitBreaker>,
    /// Health checker adapter
    health_manager: Option<Arc<HealthCheckerAdapter>>,
    /// Alert manager for state change alerts
    alert_manager: Option<Arc<AlertManager>>,
}

impl ResilienceMonitoringAdapter {
    /// Create a new resilience monitoring adapter with default configuration
    pub fn new() -> Self {
        Self {
            config: ResilienceMonitoringConfig::default(),
            monitored_circuit_breakers: Vec::new(),
            health_manager: None,
            alert_manager: None,
        }
    }
    
    /// Create a new resilience monitoring adapter with custom configuration
    pub fn with_config(config: ResilienceMonitoringConfig) -> Self {
        Self {
            config,
            monitored_circuit_breakers: Vec::new(),
            health_manager: None,
            alert_manager: None,
        }
    }
    
    /// Set the health manager for the adapter
    pub fn with_health_manager(mut self, health_manager: Arc<HealthCheckerAdapter>) -> Self {
        self.health_manager = Some(health_manager);
        self
    }
    
    /// Set the alert manager for the adapter
    pub fn with_alert_manager(mut self, alert_manager: Arc<AlertManager>) -> Self {
        self.alert_manager = Some(alert_manager);
        self
    }
    
    /// Register a circuit breaker for monitoring
    pub fn register_circuit_breaker(
        &mut self,
        circuit_breaker: Arc<StandardCircuitBreaker>,
        name: impl Into<String>,
        component_id: impl Into<String>,
    ) -> Result<(), IntegrationError> {
        let name = name.into();
        let component_id = component_id.into();
        
        // Check for duplicates
        if self.monitored_circuit_breakers.iter().any(|cb| cb.component_id == component_id) {
            return Err(IntegrationError::DuplicateComponent(format!(
                "Circuit breaker for component '{}' already registered",
                component_id
            )));
        }
        
        // Add the circuit breaker to the monitored list
        let monitored = MonitoredCircuitBreaker {
            circuit_breaker,
            name,
            component_id,
            last_metrics_report: Instant::now(),
            last_known_state: BreakerState::Closed,
        };
        
        self.monitored_circuit_breakers.push(monitored);
        
        Ok(())
    }
    
    /// Start monitoring all registered circuit breakers
    pub async fn start_monitoring(&self) -> Result<(), IntegrationError> {
        // In a real implementation, this would start a background task to poll the circuit breakers
        // For simplicity, we'll just return Ok() here
        Ok(())
    }
}

/// Health check adapter for circuit breakers
pub struct CircuitBreakerHealthCheck {
    /// Name of the circuit breaker
    name: String,
    /// Component ID for the circuit breaker
    component_id: String,
    /// The monitored circuit breaker
    circuit_breaker: Arc<StandardCircuitBreaker>,
}

impl Debug for CircuitBreakerHealthCheck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CircuitBreakerHealthCheck")
            .field("name", &self.name)
            .field("component_id", &self.component_id)
            .field("circuit_breaker", &"<Arc<StandardCircuitBreaker>>")
            .finish()
    }
}

impl CircuitBreakerHealthCheck {
    /// Create a new circuit breaker health check
    pub fn new(
        name: impl Into<String>,
        component_id: impl Into<String>,
        circuit_breaker: Arc<StandardCircuitBreaker>,
    ) -> Self {
        Self {
            name: name.into(),
            component_id: component_id.into(),
            circuit_breaker,
        }
    }
}

#[async_trait]
impl HealthCheck for CircuitBreakerHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn check(&self) -> squirrel_core::error::Result<ComponentHealth> {
        let state = self.circuit_breaker.state().await;
        let status = match state {
            BreakerState::Closed => Status::Healthy,
            BreakerState::HalfOpen => Status::Degraded,
            BreakerState::Open => Status::Unhealthy,
        };
        
        // Create a detailed health status with circuit breaker state information
        let mut details = HashMap::new();
        details.insert("state".to_string(), format!("{:?}", state));
        
        Ok(ComponentHealth::new(
            self.name.clone(),
            status,
            Some(format!("Circuit breaker state: {:?}", state))
        ).with_details(details))
    }
}

/// Alert handler implementation for circuit breaker recovery
pub struct CircuitBreakerAlertHandler {
    /// Map of circuit breakers by component ID
    circuit_breakers: HashMap<String, Arc<StandardCircuitBreaker>>,
}

impl Debug for CircuitBreakerAlertHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CircuitBreakerAlertHandler")
            .field("circuit_breakers", &format!("<{} circuit breakers>", self.circuit_breakers.len()))
            .finish()
    }
}

impl CircuitBreakerAlertHandler {
    /// Create a new circuit breaker alert handler
    pub fn new() -> Self {
        Self {
            circuit_breakers: HashMap::new(),
        }
    }
    
    /// Register a circuit breaker with the handler
    pub fn register_circuit_breaker(
        &mut self,
        component_id: impl Into<String>,
        circuit_breaker: Arc<StandardCircuitBreaker>,
    ) -> Result<(), IntegrationError> {
        let component_id = component_id.into();
        
        // Check for duplicates
        if self.circuit_breakers.contains_key(&component_id) {
            return Err(IntegrationError::DuplicateComponent(format!(
                "Circuit breaker for component '{}' already registered",
                component_id
            )));
        }
        
        // Add to registry
        self.circuit_breakers.insert(component_id, circuit_breaker);
        
        Ok(())
    }
}

#[async_trait]
impl AlertHandler for CircuitBreakerAlertHandler {
    fn name(&self) -> &str {
        "circuit_breaker_alert_handler"
    }
    
    fn supported_types(&self) -> &[&str] {
        &["circuit_breaker.state_change"]
    }
    
    async fn handle_alert(&self, alert: &Alert) -> anyhow::Result<()> {
        println!("Handling circuit breaker alert: {:?}", alert);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use squirrel_mcp::resilience::circuit_breaker::BreakerConfig;
    
    #[tokio::test]
    async fn test_register_circuit_breaker() {
        let mut adapter = ResilienceMonitoringAdapter::new();
        let config = BreakerConfig::new("test-breaker");
        let circuit_breaker = Arc::new(StandardCircuitBreaker::new(config));
        
        let result = adapter.register_circuit_breaker(
            circuit_breaker,
            "test-breaker",
            "test-component"
        );
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_duplicate_registration() {
        let mut adapter = ResilienceMonitoringAdapter::new();
        let config1 = BreakerConfig::new("test-breaker-1");
        let circuit_breaker1 = Arc::new(StandardCircuitBreaker::new(config1));
        
        let config2 = BreakerConfig::new("test-breaker-2");
        let circuit_breaker2 = Arc::new(StandardCircuitBreaker::new(config2));
        
        let result1 = adapter.register_circuit_breaker(
            circuit_breaker1,
            "test-breaker",
            "test-component-2"
        );
        
        assert!(result1.is_ok());
        
        let result2 = adapter.register_circuit_breaker(
            circuit_breaker2,
            "test-breaker-2",
            "test-component-2",
        );
        
        assert!(result2.is_err());
    }
} 