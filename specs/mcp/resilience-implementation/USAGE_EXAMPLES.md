# MCP Resilience Framework: Usage Examples

**Date**: July 21, 2024  
**Version**: 0.9.0  
**Team**: DataScienceBioLab

## 1. Overview

This document provides practical examples of how to use the MCP Resilience Framework in your applications. The examples showcase each component of the framework and demonstrate common usage patterns and best practices.

## 1. Circuit Breaker Examples

### 1.1 Basic Circuit Breaker Usage

```rust
use squirrel_mcp::resilience::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use std::time::Duration;
use std::io::Error;

// Create a circuit breaker with default configuration
let mut circuit_breaker = CircuitBreaker::default();

// Function to execute with circuit breaker protection
fn fetch_data() -> Result<String, Error> {
    // Actual data fetching logic
    Ok("Data from remote service".to_string())
}

// Execute a function with circuit breaker protection
let result = circuit_breaker.execute(|| {
    fetch_data()
});

match result {
    Ok(data) => println!("Successfully retrieved data: {}", data),
    Err(e) => println!("Failed to get data: {}", e),
}
```

### 1.2 Custom Circuit Breaker Configuration

```rust
use squirrel_mcp::resilience::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};

// Create a circuit breaker with custom configuration
let config = CircuitBreakerConfig {
    name: "database-circuit".to_string(),
    failure_threshold: 5,                // Open after 5 failures
    recovery_timeout_ms: 30000,          // Wait 30 seconds before half-open
    half_open_success_threshold: 2,      // Close after 2 successes in half-open
    half_open_allowed_calls: 3,          // Allow 3 test calls in half-open
    fallback: Some(Box::new(|| {
        // Return cached data when circuit is open
        println!("Circuit is open, using fallback data");
        Ok("Cached data".to_string())
    })),
};

let mut circuit_breaker = CircuitBreaker::new(config);

// Example DB query with circuit breaker protection
let query_result = circuit_breaker.execute(|| {
    // Database query that might fail
    db_client.query("SELECT * FROM users")
});
```

### 1.3 Monitoring Circuit Breaker State

```rust
use squirrel_mcp::resilience::circuit_breaker::{CircuitBreaker, CircuitState};

// Check the current state of the circuit breaker
let current_state = circuit_breaker.get_state();

match current_state {
    CircuitState::Closed => println!("Circuit is closed - all operations permitted"),
    CircuitState::Open => println!("Circuit is open - operations are blocked"),
    CircuitState::HalfOpen => println!("Circuit is half-open - testing if service has recovered"),
}

// Get metrics about the circuit breaker's operations
let metrics = circuit_breaker.get_metrics();
println!("Success count: {}", metrics.success_count);
println!("Failure count: {}", metrics.failure_count);
println!("Consecutive failures: {}", metrics.consecutive_failures);
println!("Last failure: {:?}", metrics.last_failure_time);
```

## 2. Retry Mechanism Examples

The following examples demonstrate how to use the Retry Mechanism once it's fully implemented:

### 2.1 Basic Retry

```rust
use squirrel_mcp::resilience::retry::{RetryMechanism, RetryConfig};

// Create a retry mechanism with default configuration
let retry = RetryMechanism::default();

// Example usage (placeholder until implementation is complete)
async fn with_retry() {
    retry.execute(async {
        // Operation that might fail transiently
        api_client.send_request().await
    }).await
}
```

### 2.2 Custom Retry Configuration

```rust
use squirrel_mcp::resilience::retry::{RetryMechanism, RetryConfig, BackoffStrategy};
use std::time::Duration;

// Create a retry configuration
let config = RetryConfig {
    max_attempts: 5,
    base_delay: Duration::from_millis(100),
    max_delay: Duration::from_secs(5),
    use_jitter: true,  // Add randomness to prevent thundering herd
    backoff_strategy: BackoffStrategy::Exponential,
};

let retry = RetryMechanism::new(config);

// Execute with custom retry configuration
let result = retry.execute(|| {
    // Operation that might fail temporarily
    external_service.process_request("data")
});
```

### 2.3 Working with Different Backoff Strategies

```rust
use squirrel_mcp::resilience::retry::{RetryMechanism, RetryConfig, BackoffStrategy};
use std::time::Duration;

// Constant backoff (fixed delay between retries)
let constant_retry = RetryMechanism::new(RetryConfig {
    backoff_strategy: BackoffStrategy::Constant,
    base_delay: Duration::from_millis(100),
    ..RetryConfig::default()
});

// Linear backoff (delay increases linearly)
let linear_retry = RetryMechanism::new(RetryConfig {
    backoff_strategy: BackoffStrategy::Linear,
    base_delay: Duration::from_millis(100),
    ..RetryConfig::default()
});

// Exponential backoff (delay doubles each time)
let exponential_retry = RetryMechanism::new(RetryConfig {
    backoff_strategy: BackoffStrategy::Exponential,
    base_delay: Duration::from_millis(100),
    max_delay: Duration::from_secs(10),
    ..RetryConfig::default()
});

// Fibonacci backoff (delay follows Fibonacci sequence)
let fibonacci_retry = RetryMechanism::new(RetryConfig {
    backoff_strategy: BackoffStrategy::Fibonacci,
    base_delay: Duration::from_millis(50),
    max_delay: Duration::from_secs(8),
    ..RetryConfig::default()
});
```

## 3. Recovery Strategy Examples

The following examples demonstrate how to use the Recovery Strategy once it's fully implemented:

### 3.1 Basic Recovery

```rust
use squirrel_mcp::resilience::recovery::{RecoveryStrategy, FailureInfo, FailureSeverity};
use std::error::Error;

// Create a recovery strategy with default configuration
let mut recovery = RecoveryStrategy::default();

// Define failure information
let failure = FailureInfo {
    message: "Database connection lost".to_string(),
    severity: FailureSeverity::Moderate,
    context: "database.connection".to_string(),
    recovery_attempts: 0,
};

// Attempt recovery
let result = recovery.handle_failure(failure, || {
    // Recovery action
    println!("Attempting to reconnect to database...");
    reconnect_database().map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
});

match result {
    Ok(connection) => println!("Successfully reconnected to database"),
    Err(e) => println!("Failed to recover: {}", e),
}
```

### 3.2 Custom Recovery Configuration

```rust
use squirrel_mcp::resilience::recovery::{RecoveryStrategy, RecoveryConfig, FailureSeverity};

// Create a custom recovery configuration
let config = RecoveryConfig {
    max_minor_attempts: 5,      // Try minor recoveries 5 times
    max_moderate_attempts: 3,   // Try moderate recoveries 3 times
    max_severe_attempts: 1,     // Try severe recoveries once
    recover_critical: true,     // Attempt recovery for critical failures
};

let mut recovery = RecoveryStrategy::new(config);

// Now the recovery strategy will allow more attempts for minor failures
// and will attempt to recover from critical failures
```

### 3.3 Using Different Severity Levels

```rust
use squirrel_mcp::resilience::recovery::{RecoveryStrategy, FailureInfo, FailureSeverity};

let mut recovery = RecoveryStrategy::default();

// Example with different severity levels
let minor_failure = FailureInfo {
    message: "Cache miss".to_string(),
    severity: FailureSeverity::Minor,
    context: "cache.get".to_string(),
    recovery_attempts: 0,
};

let severe_failure = FailureInfo {
    message: "Disk write error".to_string(),
    severity: FailureSeverity::Severe,
    context: "storage.write".to_string(),
    recovery_attempts: 0,
};

let critical_failure = FailureInfo {
    message: "System crash".to_string(),
    severity: FailureSeverity::Critical,
    context: "system.core".to_string(),
    recovery_attempts: 0,
};

// Handle each failure differently based on severity
```

## 4. State Synchronization Examples

The following examples demonstrate how to use the State Synchronization component once it's fully implemented:

### 4.1 Basic State Synchronization

```rust
use squirrel_mcp::resilience::state_sync::{StateSynchronizer, StateSyncConfig, StateType};
use std::time::Duration;

// Create a state synchronizer with default configuration
let state_sync = StateSynchronizer::default();

// Define state to synchronize
let app_state = AppState {
    user_count: 1250,
    active_sessions: 78,
    last_update: Utc::now(),
};

// Synchronize state with backup systems
let result = state_sync.sync_state(
    StateType::Runtime,
    "app_state",
    "backup_service",
    app_state
);

match result {
    Ok(_) => println!("State successfully synchronized"),
    Err(e) => println!("Failed to synchronize state: {}", e),
}
```

### 4.2 Custom State Synchronization Configuration

```rust
use squirrel_mcp::resilience::state_sync::{StateSynchronizer, StateSyncConfig, StateType};
use std::time::Duration;

// Create a custom state sync configuration
let config = StateSyncConfig {
    sync_timeout: Duration::from_secs(5),
    max_state_size: 5 * 1024 * 1024, // 5MB
    validate_state: true,
};

let state_sync = StateSynchronizer::new(config);

// Use the state synchronizer with the custom configuration
```

## 5. Health Monitoring Examples

The following examples demonstrate how to use the Health Monitoring component once it's fully implemented:

### 5.1 Basic Health Monitoring

```rust
use squirrel_mcp::resilience::health::{HealthMonitor, HealthConfig, HealthStatus};
use std::time::Duration;

// Create a health monitor with default configuration
let health_monitor = HealthMonitor::default();

// Register a component for health checks
health_monitor.register_component("database", async move {
    match check_database_connection().await {
        Ok(_) => HealthStatus::Healthy,
        Err(e) => {
            if e.is_timeout() {
                HealthStatus::Degraded { reason: "Slow response".to_string() }
            } else {
                HealthStatus::Unhealthy { reason: e.to_string() }
            }
        }
    }
});

// Check health of a specific component
let db_health = health_monitor.check_health("database").await;

match db_health {
    HealthStatus::Healthy => println!("Database is healthy"),
    HealthStatus::Degraded { reason } => println!("Database is degraded: {}", reason),
    HealthStatus::Unhealthy { reason } => println!("Database is unhealthy: {}", reason),
}

// Check overall system health
let system_health = health_monitor.check_overall_health().await;
println!("System health: {}", system_health);
```

### 5.2 Custom Health Monitoring Configuration

```rust
use squirrel_mcp::resilience::health::{HealthMonitor, HealthConfig};
use std::time::Duration;

// Create a custom health monitoring configuration
let config = HealthConfig {
    check_interval: Duration::from_secs(30),
    check_timeout: Duration::from_secs(5),
    failure_threshold: 3,
};

let health_monitor = HealthMonitor::new(config);

// Use the health monitor with custom configuration
```

## 6. Integration Examples

### 6.1 Combining Circuit Breaker and Retry

```rust
use squirrel_mcp::resilience::{with_resilience, ResilienceError};
use squirrel_mcp::resilience::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use squirrel_mcp::resilience::retry::{RetryMechanism, RetryConfig, BackoffStrategy};
use std::time::Duration;

// Create components
let mut circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
    name: "api-service".to_string(),
    failure_threshold: 5,
    recovery_timeout_ms: 30000,
    half_open_success_threshold: 2,
    half_open_allowed_calls: 2,
    fallback: None,
});

let retry = RetryMechanism::new(RetryConfig {
    max_attempts: 3,
    base_delay: Duration::from_millis(100),
    max_delay: Duration::from_secs(1),
    use_jitter: true,
    backoff_strategy: BackoffStrategy::Exponential,
});

// Use both together with the helper function
let result = with_resilience(
    &mut circuit_breaker,
    &retry,
    || {
        // Operation that might fail
        api_client.send_request("GET", "/users")
    }
);

match result {
    Ok(data) => println!("Request succeeded: {}", data),
    Err(ResilienceError::CircuitOpen(_)) => println!("Circuit is open, request not sent"),
    Err(ResilienceError::RetryExceeded(_)) => println!("Request failed after multiple retries"),
    Err(e) => println!("Request failed: {}", e),
}
```

### 6.2 Combining Retry and Recovery

```rust
use squirrel_mcp::resilience::retry::{RetryMechanism, RetryConfig};
use squirrel_mcp::resilience::recovery::{RecoveryStrategy, FailureInfo, FailureSeverity};
use std::error::Error;

let retry = RetryMechanism::default();
let mut recovery = RecoveryStrategy::default();

// First try with retry
let retry_result = retry.execute(|| {
    // Operation that might fail temporarily
    db_client.query("SELECT * FROM users")
});

// If retry fails, try recovery
if let Err(retry_error) = retry_result {
    let failure = FailureInfo {
        message: format!("Database query failed after retries: {}", retry_error),
        severity: FailureSeverity::Moderate,
        context: "database.query".to_string(),
        recovery_attempts: 0,
    };
    
    let recovery_result = recovery.handle_failure(failure, || {
        // Try different approach or rebuild connection
        println!("Rebuilding database connection...");
        db_client.rebuild_connection();
        
        // Try query again with new connection
        db_client.query("SELECT * FROM users")
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
    });
    
    match recovery_result {
        Ok(data) => println!("Recovery succeeded: {:?}", data),
        Err(e) => println!("Recovery failed: {}", e),
    }
}
```

### 6.3 Full Resilience Framework

```rust
use squirrel_mcp::resilience::{with_full_resilience, ResilienceError};
use squirrel_mcp::resilience::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use squirrel_mcp::resilience::retry::{RetryMechanism, RetryConfig};
use squirrel_mcp::resilience::recovery::{RecoveryStrategy, RecoveryConfig, FailureInfo, FailureSeverity};

// Create all components
let mut circuit_breaker = CircuitBreaker::default();
let retry = RetryMechanism::default();
let mut recovery = RecoveryStrategy::default();

// Define failure information
let failure = FailureInfo {
    message: "API connection failed".to_string(),
    severity: FailureSeverity::Moderate,
    context: "api.connection".to_string(),
    recovery_attempts: 0,
};

// Use all components together
let result = with_full_resilience(
    &mut circuit_breaker,
    &retry,
    &mut recovery,
    failure,
    // Operation to try
    || api_client.get_data(),
    // Recovery action if operation fails
    || {
        println!("Attempting recovery...");
        api_client.reset_connection();
        api_client.get_cached_data()
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
    }
);

match result {
    Ok(data) => println!("Operation succeeded: {:?}", data),
    Err(e) => println!("All resilience mechanisms failed: {}", e),
}
```

## 7. Advanced Patterns

### 7.1 Circuit Breaker with Custom Fallback

```rust
use squirrel_mcp::resilience::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};

// Create a circuit breaker with a custom fallback function
let mut circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
    name: "payment-service".to_string(),
    failure_threshold: 3,
    recovery_timeout_ms: 60000, // 1 minute timeout
    half_open_success_threshold: 2,
    half_open_allowed_calls: 2,
    fallback: Some(Box::new(|| {
        // Fallback to offline payment processing mode
        println!("Payment service is down, switching to offline mode");
        save_transaction_for_later_processing();
        Ok("Transaction saved for later processing".to_string())
    })),
});

// When the circuit is open, the fallback will be used automatically
let result = circuit_breaker.execute(|| {
    payment_service.process_payment(user_id, amount)
});
```

### 7.2 Retry with Conditional Logic

```rust
use squirrel_mcp::resilience::retry::{RetryMechanism, RetryConfig};
use std::time::Duration;

let retry = RetryMechanism::default();

// Custom retry logic with condition
let result = retry.execute(|| {
    let response = api_client.send_request();
    
    match response {
        Ok(resp) if resp.status_code == 429 => {
            // Rate limited - this is retriable
            Err(Box::new(TestError("Rate limited".to_string())))
        },
        Ok(resp) if resp.status_code >= 500 => {
            // Server error - this is retriable
            Err(Box::new(TestError("Server error".to_string())))
        },
        Ok(resp) if resp.status_code == 404 => {
            // Not found - no point retrying
            // Immediately return the error without retry
            return Ok(resp);
        },
        Ok(resp) => {
            // Success or other status codes
            Ok(resp)
        },
        Err(e) => {
            // Connection errors are retriable
            Err(Box::new(e))
        }
    }
});
```

### 7.3 Runtime Configuration Updates

```rust
use squirrel_mcp::resilience::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use squirrel_mcp::resilience::retry::{RetryMechanism, RetryConfig};

// Create components with initial configuration
let mut circuit_breaker = CircuitBreaker::default();
let mut retry = RetryMechanism::default();

// Later, update configuration based on runtime conditions
if system_under_high_load() {
    // Update circuit breaker to be more sensitive during high load
    let new_cb_config = CircuitBreakerConfig {
        failure_threshold: 2, // Lower threshold
        recovery_timeout_ms: 60000, // Longer timeout
        ..circuit_breaker.get_config().clone()
    };
    circuit_breaker.update_config(new_cb_config);
    
    // Update retry to be more conservative during high load
    let new_retry_config = RetryConfig {
        max_attempts: 2, // Fewer retry attempts
        base_delay: Duration::from_millis(200), // Longer delays
        ..retry.get_config().clone()
    };
    retry.update_config(new_retry_config);
}
```

## 8. Best Practices

### 8.1. Configuration Guidelines

- **Circuit Breaker**:
  - Set `failure_threshold` based on the criticality of the operation (lower for more critical services)
  - Set `recovery_timeout_ms` based on the expected recovery time of the dependent service
  - Use meaningful names for circuit breakers to identify them in logs and metrics

- **Retry Mechanism**:
  - Keep `max_attempts` relatively low (3-5) to avoid overwhelming services
  - Use jitter to prevent retry storms in distributed systems
  - Consider exponentially increasing delays for network-related operations

- **Recovery Strategy**:
  - Set recovery attempts based on the severity level
  - Provide detailed failure information to help with recovery

- **State Synchronization**:
  - Enable validation for critical state
  - Set reasonable timeout values based on state size

- **Health Monitoring**:
  - Adjust check intervals based on service importance
  - Set failure thresholds based on service stability

### 8.2. Testing Resilience Components

```rust
// Example test for circuit breaker
#[test]
fn test_circuit_breaker() {
    let mut circuit_breaker = CircuitBreaker::default();
    
    // Test successful operation
    let result = circuit_breaker.execute(|| Ok::<_, Box<dyn std::error::Error + Send + Sync>>(42));
    assert!(result.is_ok());
    
    // Test failure behavior
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        ..CircuitBreakerConfig::default()
    };
    let mut circuit_breaker = CircuitBreaker::new(config);
    
    // First failure
    let result = circuit_breaker.execute(|| Err::<i32, _>(Box::new(TestError::generic("test failure"))));
    assert!(result.is_err());
    assert_eq!(circuit_breaker.state(), CircuitState::Closed);
    
    // Second failure should open the circuit
    let result = circuit_breaker.execute(|| Err::<i32, _>(Box::new(TestError::generic("test failure"))));
    assert!(result.is_err());
    assert_eq!(circuit_breaker.state(), CircuitState::Open);
}
```

## 9. Common Patterns

### 9.1. The Resilience Provider Pattern

This pattern creates a reusable resilience wrapper for services:

```rust
struct ResilientService<T> {
    service: T,
    circuit_breaker: CircuitBreaker,
    retry: RetryMechanism,
}

impl<T: Service> ResilientService<T> {
    pub fn new(service: T) -> Self {
        Self {
            service,
            circuit_breaker: CircuitBreaker::default(),
            retry: RetryMechanism::default(),
        }
    }
    
    pub async fn call(&mut self, request: Request) -> Result<Response, Error> {
        self.circuit_breaker.execute(|| {
            self.retry.execute(async {
                self.service.call(request).await
            }).await
        })
    }
}
```

### 9.2. The Resilience Builder Pattern

This pattern allows for fluent configuration of resilience components:

```rust
struct ResilienceBuilder {
    circuit_breaker_config: Option<CircuitBreakerConfig>,
    retry_config: Option<RetryConfig>,
    recovery_config: Option<RecoveryConfig>,
}

impl ResilienceBuilder {
    pub fn new() -> Self {
        Self {
            circuit_breaker_config: None,
            retry_config: None,
            recovery_config: None,
        }
    }
    
    pub fn with_circuit_breaker(mut self, config: CircuitBreakerConfig) -> Self {
        self.circuit_breaker_config = Some(config);
        self
    }
    
    pub fn with_retry(mut self, config: RetryConfig) -> Self {
        self.retry_config = Some(config);
        self
    }
    
    pub fn with_recovery(mut self, config: RecoveryConfig) -> Self {
        self.recovery_config = Some(config);
        self
    }
    
    pub fn build<T: Service>(self, service: T) -> ResilientService<T> {
        let circuit_breaker = match self.circuit_breaker_config {
            Some(config) => CircuitBreaker::new(config),
            None => CircuitBreaker::default(),
        };
        
        let retry = match self.retry_config {
            Some(config) => RetryMechanism::new(config),
            None => RetryMechanism::default(),
        };
        
        ResilientService {
            service,
            circuit_breaker,
            retry,
        }
    }
}

// Usage
let resilient_service = ResilienceBuilder::new()
    .with_circuit_breaker(circuit_breaker_config)
    .with_retry(retry_config)
    .build(my_service);
```

---

**Document prepared by:** DataScienceBioLab  
**Contact:** N/A 