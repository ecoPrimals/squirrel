---
version: 1.0.0
last_updated: 2024-07-18
status: implementation
---

# MCP Resilience Framework: Circuit Breaker Implementation

## Overview

This document provides the implementation details for the Circuit Breaker component of the MCP Resilience Framework. The circuit breaker pattern prevents cascading failures by temporarily disabling operations that are likely to fail, allowing the system to recover.

## Implementation Structure

### 1. Circuit Breaker States

The circuit breaker operates in one of three states:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed and operations are allowed to proceed normally
    Closed,
    
    /// Circuit is open and operations will fail fast without execution
    Open,
    
    /// Circuit is allowing a limited number of operations to test if the issue is resolved
    HalfOpen,
}
```

### 2. Circuit Breaker Configuration

The circuit breaker can be configured with the following parameters:

```rust
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit
    pub failure_threshold: u32,
    
    /// Time in milliseconds to wait before transitioning to half-open state
    pub recovery_timeout_ms: u64,
    
    /// Number of consecutive successful calls to close the circuit from half-open state
    pub success_threshold: u32,
    
    /// Maximum number of calls allowed in half-open state
    pub half_open_allowed_calls: u32,
    
    /// Optional name for the circuit breaker (useful for debugging and metrics)
    pub name: Option<String>,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout_ms: 30000,  // 30 seconds
            success_threshold: 2,
            half_open_allowed_calls: 1,
            name: None,
        }
    }
}
```

### 3. Circuit Breaker Implementation

The core implementation includes state tracking, failure/success counting, and automatic recovery:

```rust
/// Circuit breaker implementation for preventing cascading failures
pub struct CircuitBreaker {
    /// Current state of the circuit (closed, open, half-open)
    state: AtomicU8,
    
    /// Configuration parameters
    config: CircuitBreakerConfig,
    
    /// Counter for tracking consecutive failures
    failure_counter: AtomicU32,
    
    /// Counter for tracking consecutive successes in half-open state
    success_counter: AtomicU32,
    
    /// Timestamp of when the circuit was opened (to calculate recovery timeout)
    last_state_change_time: AtomicU64,
    
    /// Count of calls in half-open state
    half_open_call_count: AtomicU32,
    
    /// Metrics collection for circuit breaker operations
    #[cfg(feature = "metrics")]
    metrics: CircuitBreakerMetrics,
}

impl CircuitBreaker {
    /// Creates a new circuit breaker with the provided configuration
    pub fn new(config: CircuitBreakerConfig) -> Self {
        #[cfg(feature = "metrics")]
        let metrics = CircuitBreakerMetrics::new(config.name.clone());
        
        Self {
            state: AtomicU8::new(CircuitState::Closed as u8),
            config,
            failure_counter: AtomicU32::new(0),
            success_counter: AtomicU32::new(0),
            last_state_change_time: AtomicU64::new(0),
            half_open_call_count: AtomicU32::new(0),
            #[cfg(feature = "metrics")]
            metrics,
        }
    }
    
    /// Creates a new circuit breaker with default configuration
    pub fn default() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }
    
    /// Gets the current state of the circuit
    pub fn get_state(&self) -> CircuitState {
        let state_value = self.state.load(Ordering::SeqCst);
        match state_value {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => panic!("Invalid circuit state value"),
        }
    }
    
    /// Executes an operation through the circuit breaker
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, ResilienceError<E>>
    where
        F: Future<Output = Result<T, E>>,
        E: Error + 'static,
    {
        self.execute_with_fallback(operation, None).await
    }
    
    /// Executes an operation with a fallback function for when the circuit is open
    pub async fn execute_with_fallback<F, FB, T, E>(
        &self, 
        operation: F,
        fallback: Option<FB>
    ) -> Result<T, ResilienceError<E>>
    where
        F: Future<Output = Result<T, E>>,
        FB: FnOnce() -> BoxFuture<'static, Result<T, E>> + Send + 'static,
        E: Error + 'static,
    {
        // Check if the circuit is open
        let current_state = self.get_state();
        
        #[cfg(feature = "metrics")]
        self.metrics.record_attempt(current_state);
        
        match current_state {
            CircuitState::Open => {
                // Check if it's time to transition to half-open
                let last_change_time = self.last_state_change_time.load(Ordering::SeqCst);
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                
                if current_time - last_change_time >= self.config.recovery_timeout_ms {
                    // Transition to half-open state
                    self.transition_to_half_open();
                    
                    // Try to execute the operation
                    return self.try_half_open_execution(operation, fallback).await;
                }
                
                // Circuit is open, return error or execute fallback
                if let Some(fallback_fn) = fallback {
                    match fallback_fn().await {
                        Ok(value) => {
                            #[cfg(feature = "metrics")]
                            self.metrics.record_fallback_success();
                            
                            Ok(value)
                        }
                        Err(err) => {
                            #[cfg(feature = "metrics")]
                            self.metrics.record_fallback_failure();
                            
                            Err(ResilienceError::Fallback(Box::new(err)))
                        }
                    }
                } else {
                    Err(ResilienceError::CircuitOpen)
                }
            }
            CircuitState::HalfOpen => {
                // Check if we're allowing more test calls
                let current_calls = self.half_open_call_count.fetch_add(1, Ordering::SeqCst);
                if current_calls >= self.config.half_open_allowed_calls {
                    // Too many calls in half-open state
                    if let Some(fallback_fn) = fallback {
                        match fallback_fn().await {
                            Ok(value) => {
                                #[cfg(feature = "metrics")]
                                self.metrics.record_fallback_success();
                                
                                Ok(value)
                            }
                            Err(err) => {
                                #[cfg(feature = "metrics")]
                                self.metrics.record_fallback_failure();
                                
                                Err(ResilienceError::Fallback(Box::new(err)))
                            }
                        }
                    } else {
                        Err(ResilienceError::CircuitHalfOpenLimitExceeded)
                    }
                } else {
                    // Try to execute the operation
                    self.try_half_open_execution(operation, fallback).await
                }
            }
            CircuitState::Closed => {
                // Normal execution
                match operation.await {
                    Ok(value) => {
                        // Reset failure counter on success
                        self.failure_counter.store(0, Ordering::SeqCst);
                        
                        #[cfg(feature = "metrics")]
                        self.metrics.record_success();
                        
                        Ok(value)
                    }
                    Err(err) => {
                        // Increment failure counter
                        let failures = self.failure_counter.fetch_add(1, Ordering::SeqCst) + 1;
                        
                        #[cfg(feature = "metrics")]
                        self.metrics.record_failure();
                        
                        // Open circuit if threshold reached
                        if failures >= self.config.failure_threshold {
                            self.transition_to_open();
                        }
                        
                        Err(ResilienceError::Operation(Box::new(err)))
                    }
                }
            }
        }
    }
    
    /// Attempts to execute an operation in half-open state
    async fn try_half_open_execution<F, FB, T, E>(
        &self,
        operation: F,
        fallback: Option<FB>,
    ) -> Result<T, ResilienceError<E>>
    where
        F: Future<Output = Result<T, E>>,
        FB: FnOnce() -> BoxFuture<'static, Result<T, E>> + Send + 'static,
        E: Error + 'static,
    {
        match operation.await {
            Ok(value) => {
                // Increment success counter
                let successes = self.success_counter.fetch_add(1, Ordering::SeqCst) + 1;
                
                #[cfg(feature = "metrics")]
                self.metrics.record_success();
                
                // Check if we've reached the success threshold to close the circuit
                if successes >= self.config.success_threshold {
                    self.transition_to_closed();
                }
                
                Ok(value)
            }
            Err(err) => {
                // Reset success counter and reopen the circuit
                self.success_counter.store(0, Ordering::SeqCst);
                self.transition_to_open();
                
                #[cfg(feature = "metrics")]
                self.metrics.record_failure();
                
                // Try fallback if provided
                if let Some(fallback_fn) = fallback {
                    match fallback_fn().await {
                        Ok(value) => {
                            #[cfg(feature = "metrics")]
                            self.metrics.record_fallback_success();
                            
                            Ok(value)
                        }
                        Err(fallback_err) => {
                            #[cfg(feature = "metrics")]
                            self.metrics.record_fallback_failure();
                            
                            Err(ResilienceError::Fallback(Box::new(fallback_err)))
                        }
                    }
                } else {
                    Err(ResilienceError::Operation(Box::new(err)))
                }
            }
        }
    }
    
    /// Transitions the circuit to closed state
    fn transition_to_closed(&self) {
        self.state.store(CircuitState::Closed as u8, Ordering::SeqCst);
        self.failure_counter.store(0, Ordering::SeqCst);
        self.success_counter.store(0, Ordering::SeqCst);
        self.half_open_call_count.store(0, Ordering::SeqCst);
        self.update_state_change_time();
        
        #[cfg(feature = "metrics")]
        self.metrics.record_state_change(CircuitState::Closed);
    }
    
    /// Transitions the circuit to open state
    fn transition_to_open(&self) {
        self.state.store(CircuitState::Open as u8, Ordering::SeqCst);
        self.success_counter.store(0, Ordering::SeqCst);
        self.half_open_call_count.store(0, Ordering::SeqCst);
        self.update_state_change_time();
        
        #[cfg(feature = "metrics")]
        self.metrics.record_state_change(CircuitState::Open);
    }
    
    /// Transitions the circuit to half-open state
    fn transition_to_half_open(&self) {
        self.state.store(CircuitState::HalfOpen as u8, Ordering::SeqCst);
        self.success_counter.store(0, Ordering::SeqCst);
        self.half_open_call_count.store(0, Ordering::SeqCst);
        self.update_state_change_time();
        
        #[cfg(feature = "metrics")]
        self.metrics.record_state_change(CircuitState::HalfOpen);
    }
    
    /// Updates the last state change timestamp
    fn update_state_change_time(&self) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        self.last_state_change_time.store(current_time, Ordering::SeqCst);
    }
    
    /// Manually resets the circuit breaker to closed state
    pub fn reset(&self) {
        self.transition_to_closed();
    }
    
    /// Manually forces the circuit breaker to open state
    pub fn force_open(&self) {
        self.transition_to_open();
    }
}
```

### 4. Error Types for Circuit Breaker

The resilience framework provides specific error types for circuit breaker operations:

```rust
#[derive(Debug, Error)]
pub enum ResilienceError<E: Error + 'static> {
    #[error("Circuit is open")]
    CircuitOpen,
    
    #[error("Circuit is half-open and call limit exceeded")]
    CircuitHalfOpenLimitExceeded,
    
    #[error("Operation failed: {0}")]
    Operation(Box<dyn Error + Send + Sync>),
    
    #[error("Fallback failed: {0}")]
    Fallback(Box<dyn Error + Send + Sync>),
    
    #[error("Error of type {0}")]
    Other(String),
}
```

### 5. Optional Metrics Collection

When the "metrics" feature is enabled, the circuit breaker collects operational metrics:

```rust
#[cfg(feature = "metrics")]
#[derive(Debug)]
pub struct CircuitBreakerMetrics {
    /// Name of this circuit breaker instance
    name: String,
    
    /// Total number of attempts (all states)
    attempts: AtomicU64,
    
    /// Number of successful executions
    successes: AtomicU64,
    
    /// Number of failed executions
    failures: AtomicU64,
    
    /// Number of short-circuits (rejected without execution)
    short_circuits: AtomicU64,
    
    /// Number of successful fallbacks
    fallback_successes: AtomicU64,
    
    /// Number of failed fallbacks
    fallback_failures: AtomicU64,
    
    /// State transition counts
    state_transitions: RwLock<HashMap<CircuitState, u64>>,
    
    /// Current state
    current_state: AtomicU8,
}

#[cfg(feature = "metrics")]
impl CircuitBreakerMetrics {
    pub fn new(name: Option<String>) -> Self {
        let name = name.unwrap_or_else(|| format!("circuit_breaker-{}", Uuid::new_v4()));
        
        let mut state_transitions = HashMap::new();
        state_transitions.insert(CircuitState::Closed, 0);
        state_transitions.insert(CircuitState::Open, 0);
        state_transitions.insert(CircuitState::HalfOpen, 0);
        
        Self {
            name,
            attempts: AtomicU64::new(0),
            successes: AtomicU64::new(0),
            failures: AtomicU64::new(0),
            short_circuits: AtomicU64::new(0),
            fallback_successes: AtomicU64::new(0),
            fallback_failures: AtomicU64::new(0),
            state_transitions: RwLock::new(state_transitions),
            current_state: AtomicU8::new(CircuitState::Closed as u8),
        }
    }
    
    pub fn record_attempt(&self, state: CircuitState) {
        self.attempts.fetch_add(1, Ordering::SeqCst);
        
        if state == CircuitState::Open {
            self.short_circuits.fetch_add(1, Ordering::SeqCst);
        }
    }
    
    pub fn record_success(&self) {
        self.successes.fetch_add(1, Ordering::SeqCst);
    }
    
    pub fn record_failure(&self) {
        self.failures.fetch_add(1, Ordering::SeqCst);
    }
    
    pub fn record_fallback_success(&self) {
        self.fallback_successes.fetch_add(1, Ordering::SeqCst);
    }
    
    pub fn record_fallback_failure(&self) {
        self.fallback_failures.fetch_add(1, Ordering::SeqCst);
    }
    
    pub fn record_state_change(&self, state: CircuitState) {
        self.current_state.store(state as u8, Ordering::SeqCst);
        
        let mut transitions = self.state_transitions.write().unwrap();
        let count = transitions.entry(state).or_insert(0);
        *count += 1;
    }
    
    pub fn get_metrics(&self) -> CircuitBreakerMetricsSnapshot {
        let state_value = self.current_state.load(Ordering::SeqCst);
        let current_state = match state_value {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Closed,
        };
        
        let transitions = self.state_transitions.read().unwrap().clone();
        
        CircuitBreakerMetricsSnapshot {
            name: self.name.clone(),
            attempts: self.attempts.load(Ordering::SeqCst),
            successes: self.successes.load(Ordering::SeqCst),
            failures: self.failures.load(Ordering::SeqCst),
            short_circuits: self.short_circuits.load(Ordering::SeqCst),
            fallback_successes: self.fallback_successes.load(Ordering::SeqCst),
            fallback_failures: self.fallback_failures.load(Ordering::SeqCst),
            state_transitions: transitions,
            current_state,
        }
    }
}

#[cfg(feature = "metrics")]
#[derive(Debug, Clone, Serialize)]
pub struct CircuitBreakerMetricsSnapshot {
    pub name: String,
    pub attempts: u64,
    pub successes: u64,
    pub failures: u64,
    pub short_circuits: u64,
    pub fallback_successes: u64,
    pub fallback_failures: u64,
    pub state_transitions: HashMap<CircuitState, u64>,
    pub current_state: CircuitState,
}
```

## Usage Examples

### Basic Circuit Breaker Usage

```rust
// Create a circuit breaker with default settings
let circuit_breaker = CircuitBreaker::default();

// Create a circuit breaker with custom settings
let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
    failure_threshold: 3,              // Open after 3 failures
    recovery_timeout_ms: 10000,        // Wait 10 seconds before trying again
    success_threshold: 2,              // Close after 2 consecutive successes
    half_open_allowed_calls: 1,        // Allow only 1 test call in half-open state
    name: Some("api_service".to_string()),
});

// Execute operation with circuit breaker
let result = circuit_breaker.execute(async {
    api_client.get_data().await
}).await;

// Handle result
match result {
    Ok(data) => println!("Got data: {:?}", data),
    Err(ResilienceError::CircuitOpen) => println!("Circuit is open, not executing operation"),
    Err(ResilienceError::Operation(err)) => println!("Operation failed: {}", err),
    Err(err) => println!("Other error: {}", err),
}
```

### Circuit Breaker with Fallback

```rust
// Create a circuit breaker
let circuit_breaker = CircuitBreaker::default();

// Execute with fallback
let result = circuit_breaker.execute_with_fallback(
    async {
        // Primary operation
        api_client.get_user_data(user_id).await
    },
    Some(|| Box::pin(async {
        // Fallback operation
        get_cached_user_data(user_id).await
    }))
).await;

// Handle result
match result {
    Ok(data) => println!("Got user data: {:?}", data),
    Err(ResilienceError::Fallback(err)) => println!("Both primary and fallback failed: {}", err),
    Err(err) => println!("Error: {}", err),
}
```

### Integration with MCP Protocol

```rust
// Create a circuit breaker for MCP operations
let mcp_circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
    failure_threshold: 5,
    recovery_timeout_ms: 30000,
    success_threshold: 3,
    half_open_allowed_calls: 2,
    name: Some("mcp_protocol".to_string()),
});

// Create a wrapper for MCP protocol that uses circuit breaker
struct ResilientMcpProtocol {
    inner: Arc<dyn McpProtocol>,
    circuit_breaker: CircuitBreaker,
}

impl ResilientMcpProtocol {
    pub fn new(protocol: Arc<dyn McpProtocol>) -> Self {
        Self {
            inner: protocol,
            circuit_breaker: CircuitBreaker::new(CircuitBreakerConfig {
                name: Some("mcp_protocol".to_string()),
                ..Default::default()
            }),
        }
    }
    
    pub async fn send_message(&self, message: McpMessage) -> Result<McpResponse, McpError> {
        // Execute with circuit breaker
        let result = self.circuit_breaker.execute(async {
            self.inner.send_message(message.clone()).await
        }).await;
        
        match result {
            Ok(response) => Ok(response),
            Err(ResilienceError::CircuitOpen) => {
                Err(McpError::ServiceUnavailable("Circuit is open, service unavailable".into()))
            },
            Err(ResilienceError::Operation(err)) => {
                if let Some(mcp_err) = err.downcast_ref::<McpError>() {
                    Err(mcp_err.clone())
                } else {
                    Err(McpError::Internal(format!("Unknown error: {}", err)))
                }
            },
            Err(err) => Err(McpError::Internal(format!("Resilience error: {}", err))),
        }
    }
}
```

## Testing

### Unit Testing 

```rust
#[tokio::test]
async fn test_circuit_breaker_success() {
    let circuit_breaker = CircuitBreaker::default();
    
    // Successful operation
    let result = circuit_breaker.execute(async {
        Ok::<_, anyhow::Error>("success")
    }).await;
    
    assert!(result.is_ok());
    assert_eq!(circuit_breaker.get_state(), CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_opens_after_failures() {
    let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 2,
        ..Default::default()
    });
    
    // First failure
    let result1 = circuit_breaker.execute(async {
        Err::<String, _>(anyhow::anyhow!("test error"))
    }).await;
    
    assert!(result1.is_err());
    assert_eq!(circuit_breaker.get_state(), CircuitState::Closed);
    
    // Second failure - should open the circuit
    let result2 = circuit_breaker.execute(async {
        Err::<String, _>(anyhow::anyhow!("test error"))
    }).await;
    
    assert!(result2.is_err());
    assert_eq!(circuit_breaker.get_state(), CircuitState::Open);
    
    // Next call should be rejected with CircuitOpen
    let result3 = circuit_breaker.execute(async {
        Ok::<_, anyhow::Error>("success")
    }).await;
    
    assert!(matches!(result3, Err(ResilienceError::CircuitOpen)));
}

#[tokio::test]
async fn test_circuit_breaker_half_open_transition() {
    let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 1,
        recovery_timeout_ms: 100, // Very short for testing
        ..Default::default()
    });
    
    // Fail to open the circuit
    let _ = circuit_breaker.execute(async {
        Err::<String, _>(anyhow::anyhow!("test error"))
    }).await;
    
    assert_eq!(circuit_breaker.get_state(), CircuitState::Open);
    
    // Wait for recovery timeout
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    // Force a check of the state by attempting an operation
    let result = circuit_breaker.execute(async {
        Ok::<_, anyhow::Error>("success")
    }).await;
    
    // Should succeed and circuit should be half-open
    assert!(result.is_ok());
    assert_eq!(circuit_breaker.get_state(), CircuitState::HalfOpen);
}

#[tokio::test]
async fn test_circuit_breaker_closes_after_success() {
    let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 1,
        recovery_timeout_ms: 100,
        success_threshold: 1,
        ..Default::default()
    });
    
    // Fail to open the circuit
    let _ = circuit_breaker.execute(async {
        Err::<String, _>(anyhow::anyhow!("test error"))
    }).await;
    
    assert_eq!(circuit_breaker.get_state(), CircuitState::Open);
    
    // Wait for recovery timeout
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    // Successful operation in half-open state
    let result = circuit_breaker.execute(async {
        Ok::<_, anyhow::Error>("success")
    }).await;
    
    // Should succeed and close the circuit
    assert!(result.is_ok());
    assert_eq!(circuit_breaker.get_state(), CircuitState::Closed);
}
```

## Conclusion

The Circuit Breaker implementation provides a robust mechanism for preventing cascading failures in the MCP system. It supports:

1. Configurable failure thresholds
2. Automatic recovery via half-open state
3. Fallback mechanisms
4. Detailed metrics collection
5. Thread-safe operation
6. Integration with MCP protocol

This implementation satisfies the requirements outlined in the resilience framework specification and provides the foundation for building resilient MCP systems. 