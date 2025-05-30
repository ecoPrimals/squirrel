# Combined Resilience Examples

**Date**: July 21, 2024  
**Version**: 0.9.0  
**Team**: DataScienceBioLab

This document provides examples of how to combine multiple resilience components of the MCP Resilience Framework to create comprehensive resilience strategies.

## 1. Circuit Breaker + Retry Example

This example demonstrates how to use both the Circuit Breaker and Retry Mechanism together:

```rust
use squirrel_mcp::resilience::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use squirrel_mcp::resilience::retry::{RetryMechanism, RetryConfig, BackoffStrategy};
use std::time::Duration;

// Create a database client (example)
let db_client = DatabaseClient::new("localhost:5432");

// Configure the retry mechanism
let retry_config = RetryConfig {
    max_attempts: 3,
    base_delay: Duration::from_millis(50),
    max_delay: Duration::from_secs(1),
    use_jitter: true,
    backoff_strategy: BackoffStrategy::Exponential,
};
let retry = RetryMechanism::new(retry_config);

// Configure the circuit breaker
let circuit_config = CircuitBreakerConfig {
    name: "database-connection".to_string(),
    failure_threshold: 5,                // Open after 5 failures
    recovery_timeout_ms: 30000,          // Wait 30 seconds before half-open
    half_open_success_threshold: 3,      // Close after 3 successes in half-open
    half_open_allowed_calls: 3,          // Allow 3 test calls in half-open
    fallback: Some(Box::new(|| {
        // Return cached data when circuit is open
        println!("Circuit is open, using fallback data");
        Ok(get_cached_data())
    })),
};
let mut circuit_breaker = CircuitBreaker::new(circuit_config);

// Function to get data with resilience
fn get_data_with_resilience() -> Result<Data, Error> {
    // First layer: Circuit Breaker
    circuit_breaker.execute(|| {
        // Second layer: Retry Mechanism
        retry.execute(|| {
            // Actual operation that might fail
            db_client.query("SELECT * FROM users")
        }).map_err(|e| e.into())
    })
}

// Usage
match get_data_with_resilience() {
    Ok(data) => {
        // Process data
        update_cache(data.clone());
        display_data(data);
    },
    Err(err) => {
        log_error!("Failed to get data: {}", err);
        display_error_message();
    }
}
```

## 2. Full Resilience Pipeline Example

This example demonstrates a comprehensive resilience strategy using all components:

```rust
use squirrel_mcp::resilience::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use squirrel_mcp::resilience::retry::{RetryMechanism, RetryConfig, BackoffStrategy};
use squirrel_mcp::resilience::recovery::{RecoveryStrategy, RecoveryConfig, FailureSeverity, FailureInfo};
use squirrel_mcp::resilience::state_sync::{StateSynchronizer, StateSyncConfig, StateType};
use squirrel_mcp::resilience::health::{HealthMonitor, HealthConfig, HealthStatus};
use std::time::Duration;
use std::sync::Arc;

// Create service components
let api_client = Arc::new(ApiClient::new("https://api.example.com"));
let db_client = Arc::new(DatabaseClient::new("localhost:5432"));
let cache = Arc::new(CacheService::new());

// 1. Health Monitoring
let health_config = HealthConfig {
    check_interval: Duration::from_secs(30),
    check_timeout: Duration::from_secs(5),
    failure_threshold: 3,
};
let health_monitor = Arc::new(HealthMonitor::new(health_config));

// Register components for health checks
health_monitor.register_component("api_client", move |monitor| {
    let client = api_client.clone();
    Box::pin(async move {
        match client.health_check().await {
            Ok(_) => HealthStatus::Healthy,
            Err(e) => {
                if e.is_timeout() {
                    HealthStatus::Degraded { reason: "Timeout".to_string() }
                } else {
                    HealthStatus::Unhealthy { reason: e.to_string() }
                }
            }
        }
    })
});

// 2. State Synchronization
let sync_config = StateSyncConfig {
    sync_timeout: Duration::from_secs(10),
    max_state_size: 1 * 1024 * 1024, // 1MB
    validate_state: true,
};
let state_sync = Arc::new(StateSynchronizer::new(sync_config));

// 3. Recovery Strategy
let recovery_config = RecoveryConfig {
    max_minor_attempts: 5,
    max_moderate_attempts: 3,
    max_severe_attempts: 1,
    recover_critical: false,
};
let recovery = Arc::new(RecoveryStrategy::new(recovery_config));

// 4. Retry Mechanism
let retry_config = RetryConfig {
    max_attempts: 3,
    base_delay: Duration::from_millis(50),
    max_delay: Duration::from_secs(2),
    use_jitter: true,
    backoff_strategy: BackoffStrategy::Exponential,
};
let retry = Arc::new(RetryMechanism::new(retry_config));

// 5. Circuit Breaker
let circuit_config = CircuitBreakerConfig {
    name: "api-service".to_string(),
    failure_threshold: 5,
    recovery_timeout_ms: 30000,
    half_open_success_threshold: 2,
    half_open_allowed_calls: 3,
    fallback: Some(Box::new(|| {
        Ok(cache.get_cached_data())
    })),
};
let circuit_breaker = Arc::new(CircuitBreaker::new(circuit_config));

// Combined resilience function
async fn get_user_data_resilient(user_id: String) -> Result<UserData, Error> {
    // Check health status first
    let api_health = health_monitor.check_health("api_client").await;
    if api_health == HealthStatus::Unhealthy {
        // If unhealthy, use cached data and return
        return Ok(cache.get_user_data(user_id));
    }
    
    // Use circuit breaker as outer protection
    circuit_breaker.execute(|| {
        // Use retry for transient failures
        retry.execute(|| {
            // Actual API call
            let result = api_client.get_user_data(user_id.clone());
            
            // If successful, update cache
            if let Ok(data) = result.as_ref() {
                cache.store_user_data(user_id.clone(), data.clone());
                
                // Synchronize state to backup service
                state_sync.sync_state(
                    StateType::Runtime,
                    "user_data",
                    "backup_service",
                    data.clone(),
                );
            }
            
            result
        }).map_err(|retry_error| {
            // Handle retry failure with recovery strategy
            let failure = FailureInfo {
                message: retry_error.to_string(),
                severity: FailureSeverity::Moderate,
                context: "api_client.user_data".to_string(),
                recovery_attempts: 0,
            };
            
            // Attempt recovery
            let _ = recovery.handle_failure(failure, || {
                // Recovery action - rebuild connection
                api_client.rebuild_connection();
                Ok(())
            });
            
            // Update health status
            health_monitor.update_status("api_client", HealthStatus::Degraded);
            
            retry_error.into()
        })
    })
}
```

## 3. Adapter Pattern for Resilient Services

This example shows how to use an adapter pattern to create resilient services:

```rust
// Define a trait for services
trait Service {
    fn execute(&self, request: Request) -> Result<Response, Error>;
}

// Define a resilient wrapper for any service
struct ResilientService<S: Service> {
    service: S,
    circuit_breaker: CircuitBreaker,
    retry: RetryMechanism,
}

impl<S: Service> ResilientService<S> {
    pub fn new(service: S) -> Self {
        Self {
            service,
            circuit_breaker: CircuitBreaker::default(),
            retry: RetryMechanism::default(),
        }
    }
    
    pub fn with_circuit_breaker(mut self, config: CircuitBreakerConfig) -> Self {
        self.circuit_breaker = CircuitBreaker::new(config);
        self
    }
    
    pub fn with_retry(mut self, config: RetryConfig) -> Self {
        self.retry = RetryMechanism::new(config);
        self
    }
    
    pub fn execute(&self, request: Request) -> Result<Response, Error> {
        self.circuit_breaker.execute(|| {
            self.retry.execute(|| {
                self.service.execute(request.clone())
            }).map_err(|e| e.into())
        })
    }
}

// Example usage
let database_service = DatabaseService::new("localhost:5432");

let resilient_db = ResilientService::new(database_service)
    .with_circuit_breaker(CircuitBreakerConfig {
        name: "database".to_string(),
        failure_threshold: 3,
        recovery_timeout_ms: 10000,
        half_open_success_threshold: 2,
        half_open_allowed_calls: 2,
        fallback: None,
    })
    .with_retry(RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(50),
        max_delay: Duration::from_secs(1),
        use_jitter: true,
        backoff_strategy: BackoffStrategy::Exponential,
    });

// Use the resilient service
let result = resilient_db.execute(Request::new("SELECT * FROM users"));
```

## 4. Creating a Resilience Strategy Builder

This example demonstrates a builder pattern for creating a comprehensive resilience strategy:

```rust
struct ResilienceStrategy {
    circuit_breaker: Option<CircuitBreaker>,
    retry_mechanism: Option<RetryMechanism>,
    recovery_strategy: Option<RecoveryStrategy>,
    health_monitor: Option<Arc<HealthMonitor>>,
    state_sync: Option<StateSynchronizer>,
}

impl ResilienceStrategy {
    pub fn new() -> Self {
        Self {
            circuit_breaker: None,
            retry_mechanism: None,
            recovery_strategy: None,
            health_monitor: None,
            state_sync: None,
        }
    }
    
    pub fn with_circuit_breaker(mut self, config: CircuitBreakerConfig) -> Self {
        self.circuit_breaker = Some(CircuitBreaker::new(config));
        self
    }
    
    pub fn with_retry(mut self, config: RetryConfig) -> Self {
        self.retry_mechanism = Some(RetryMechanism::new(config));
        self
    }
    
    pub fn with_recovery(mut self, config: RecoveryConfig) -> Self {
        self.recovery_strategy = Some(RecoveryStrategy::new(config));
        self
    }
    
    pub fn with_health_monitor(mut self, config: HealthConfig) -> Self {
        self.health_monitor = Some(Arc::new(HealthMonitor::new(config)));
        self
    }
    
    pub fn with_state_sync(mut self, config: StateSyncConfig) -> Self {
        self.state_sync = Some(StateSynchronizer::new(config));
        self
    }
    
    pub fn execute<F, T, E>(&self, operation: F) -> Result<T, Error>
    where
        F: Fn() -> Result<T, E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        // Start with the original operation
        let mut current_op = Box::new(operation);
        
        // Wrap with retry if configured
        if let Some(retry) = &self.retry_mechanism {
            let retry_clone = retry.clone();
            let previous_op = current_op;
            
            current_op = Box::new(move || {
                retry_clone.execute(|| previous_op())
                    .map_err(|e| e.into())
            });
        }
        
        // Wrap with circuit breaker if configured
        if let Some(cb) = &self.circuit_breaker {
            let cb_clone = cb.clone();
            let previous_op = current_op;
            
            current_op = Box::new(move || {
                cb_clone.execute(|| previous_op())
            });
        }
        
        // Execute the fully wrapped operation
        current_op()
    }
}

// Example usage
let resilience = ResilienceStrategy::new()
    .with_circuit_breaker(CircuitBreakerConfig {
        name: "database".to_string(),
        failure_threshold: 3,
        recovery_timeout_ms: 10000,
        half_open_success_threshold: 2,
        half_open_allowed_calls: 2,
        fallback: None,
    })
    .with_retry(RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(50),
        max_delay: Duration::from_secs(1),
        use_jitter: true,
        backoff_strategy: BackoffStrategy::Exponential,
    });

// Use the resilience strategy
let result = resilience.execute(|| {
    database.query("SELECT * FROM users")
});
```

## 5. Best Practices for Combining Resilience Components

When combining multiple resilience components, follow these best practices:

1. **Layer components properly**:
   - Circuit Breaker should typically be the outermost layer since it needs to track all failures and successes
   - Retry Mechanism should be inside the Circuit Breaker to ensure retries are counted correctly
   - Recovery Strategies should be applied after retry failures but before circuit breaker failures

2. **Configure components to work together**:
   - Set retry attempts lower than circuit breaker failure threshold
   - Use different timeouts for different layers (retry timeout < circuit breaker timeout)
   - Ensure recovery actions don't trigger circuit breaker false positives

3. **Monitor and adjust**:
   - Collect metrics from all resilience components
   - Adjust configuration based on observed behavior
   - Use health monitoring to proactively prevent failures

4. **Handle resource limitations**:
   - Be aware that combining resilience components can increase resource usage
   - Limit retry attempts and circuit breaker recovery times
   - Use timeouts at each layer to prevent resource exhaustion

5. **Test failure scenarios**:
   - Test each resilience component independently
   - Test combined behavior under various failure conditions
   - Verify metrics and monitoring during failure scenarios

---

**Document prepared by:** DataScienceBioLab  
**Contact:** N/A 