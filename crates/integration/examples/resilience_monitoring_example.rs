use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Result};
use tracing::{info, warn};
use squirrel_integration::{
    resilience_monitoring::{
        ResilienceMonitoringAdapter,
        ResilienceMonitoringConfig,
    },
};
use squirrel_mcp::resilience::circuit_breaker::{
    CircuitBreaker, StandardCircuitBreaker, BreakerConfig, BreakerError,
};
use squirrel_monitoring::{
    alerts::AlertLevel,
    health::adapter::HealthCheckerAdapter,
};

// Example API client that can fail
#[derive(Clone)]
struct ApiClient {
    // Failure simulation
    should_fail: bool,
}

impl ApiClient {
    // Create a new API client
    fn new() -> Self {
        Self {
            should_fail: false,
        }
    }
    
    // Set failure simulation
    fn set_should_fail(&mut self, should_fail: bool) {
        self.should_fail = should_fail;
    }
    
    // Simulate an API call
    async fn make_request(&self) -> Result<String> {
        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Simulate potential failure
        if self.should_fail {
            Err(anyhow!("API request failed"))
        } else {
            Ok("API response data".to_string())
        }
    }
}

// Example service that uses circuit breaker for resilience
struct ResilientService {
    // API client
    api_client: ApiClient,
    
    // Circuit breaker for API calls
    circuit_breaker: Arc<StandardCircuitBreaker>,
}

impl ResilientService {
    // Create a new resilient service
    fn new() -> Self {
        // Configure circuit breaker
        let config = BreakerConfig::new("api-service-breaker")
            .with_failure_threshold(0.5)  // Fail after 50% failures
            .with_reset_timeout(Duration::from_secs(5));  // 5 second reset timeout
        
        Self {
            api_client: ApiClient::new(),
            circuit_breaker: Arc::new(StandardCircuitBreaker::new(config)),
        }
    }
    
    // Set API client failure simulation
    fn set_api_failure(&mut self, should_fail: bool) {
        self.api_client.set_should_fail(should_fail);
    }
    
    // Make a request with circuit breaker protection
    async fn make_request(&self) -> Result<String> {
        // Clone what we need for the closure
        let api_client = self.api_client.clone();
        
        // Execute the request through the circuit breaker
        match self.circuit_breaker.execute(move || {
            Box::pin(async move {
                match api_client.make_request().await {
                    Ok(response) => Ok(response),
                    Err(err) => Err(BreakerError::OperationFailed { 
                        name: "api-call".to_string(), 
                        reason: err.to_string() 
                    }),
                }
            })
        }).await {
            Ok(response) => Ok(response),
            Err(err) => Err(anyhow!("Service error: {}", err)),
        }
    }
    
    // Get the circuit breaker for monitoring
    fn circuit_breaker(&self) -> Arc<StandardCircuitBreaker> {
        self.circuit_breaker.clone()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Create monitoring components
    let health_manager = Arc::new(HealthCheckerAdapter::new());
    
    // Create resilient service
    let mut service = ResilientService::new();
    
    // Create monitoring adapter without alert manager
    let config = ResilienceMonitoringConfig {
        metrics_interval_secs: 1, // Shorter interval for example
        include_detailed_metrics: true,
        circuit_open_alert_level: AlertLevel::Warning,
    };
    
    let mut monitoring_adapter = ResilienceMonitoringAdapter::with_config(config)
        .with_health_manager(health_manager.clone());
    
    // Register circuit breaker with the adapter
    monitoring_adapter.register_circuit_breaker(
        service.circuit_breaker(),
        "api-service-breaker",
        "api-service",
    )?;
    
    // Start monitoring
    monitoring_adapter.start_monitoring().await?;
    
    // Demonstrate circuit breaker in action
    info!("Making successful requests...");
    for _ in 0..5 {
        match service.make_request().await {
            Ok(response) => info!("Request succeeded: {}", response),
            Err(err) => warn!("Request failed: {}", err),
        }
        
        // Show current metrics
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Trigger failures to open the circuit breaker
    info!("Introducing failures...");
    service.set_api_failure(true);
    
    for i in 0..10 {
        match service.make_request().await {
            Ok(response) => info!("Request succeeded: {}", response),
            Err(err) => warn!("Request failed: {}", err),
        }
        
        info!("After {} failures", i+1);
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Wait for the circuit breaker to reset
    info!("Waiting for circuit breaker to reset...");
    tokio::time::sleep(Duration::from_secs(6)).await;
    
    // Fix the service
    service.set_api_failure(false);
    
    // Try again
    info!("Making requests after reset...");
    for _ in 0..5 {
        match service.make_request().await {
            Ok(response) => info!("Request succeeded: {}", response),
            Err(err) => warn!("Request failed: {}", err),
        }
        
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Success
    info!("Example completed successfully");
    Ok(())
} 