---
version: 1.0.0
last_updated: 2024-07-18
status: implementation
---

# MCP Resilience Framework: Retry Mechanism Implementation

## Overview

This document provides the implementation details for the Retry Mechanism component of the MCP Resilience Framework. The retry mechanism automatically retries failed operations with configurable backoff strategies, enabling the system to recover from transient failures.

## Implementation Structure

### 1. Backoff Strategies

The retry mechanism supports multiple backoff strategies to control the delay between retry attempts:

```rust
/// Defines different strategies for calculating delay between retry attempts
#[derive(Debug, Clone)]
pub enum BackoffStrategy {
    /// Fixed delay between retries
    Constant {
        /// Delay in milliseconds
        delay_ms: u64,
    },
    
    /// Delay that increases by a fixed amount each attempt
    Linear {
        /// Initial delay in milliseconds
        initial_delay_ms: u64,
        
        /// Amount to increase delay by each attempt (in milliseconds)
        increment_ms: u64,
    },
    
    /// Delay that doubles (or multiplies by factor) each attempt
    Exponential {
        /// Initial delay in milliseconds
        initial_delay_ms: u64,
        
        /// Factor to multiply delay by each attempt
        multiplier: f64,
        
        /// Maximum delay in milliseconds
        max_delay_ms: u64,
    },
    
    /// Delay that follows Fibonacci sequence
    Fibonacci {
        /// Initial delay in milliseconds
        initial_delay_ms: u64,
        
        /// Maximum delay in milliseconds
        max_delay_ms: u64,
    },
    
    /// Adds random jitter to another backoff strategy
    Jittered {
        /// The base strategy to apply jitter to
        base_strategy: Box<BackoffStrategy>,
        
        /// Jitter factor (0.0-1.0) controls how much randomness to add
        jitter_factor: f64,
    },
}

impl BackoffStrategy {
    /// Calculate the delay for a specific attempt
    pub fn calculate_delay_ms(&self, attempt: u32) -> u64 {
        match self {
            Self::Constant { delay_ms } => *delay_ms,
            
            Self::Linear { initial_delay_ms, increment_ms } => {
                *initial_delay_ms + (*increment_ms * (attempt as u64 - 1))
            }
            
            Self::Exponential { initial_delay_ms, multiplier, max_delay_ms } => {
                let delay = (*initial_delay_ms as f64 * multiplier.powf((attempt - 1) as f64)) as u64;
                std::cmp::min(delay, *max_delay_ms)
            }
            
            Self::Fibonacci { initial_delay_ms, max_delay_ms } => {
                if attempt <= 1 {
                    return *initial_delay_ms;
                }
                
                let mut prev = *initial_delay_ms;
                let mut curr = *initial_delay_ms;
                
                for _ in 2..=attempt {
                    let next = prev + curr;
                    prev = curr;
                    curr = next;
                }
                
                std::cmp::min(curr, *max_delay_ms)
            }
            
            Self::Jittered { base_strategy, jitter_factor } => {
                let base_delay = base_strategy.calculate_delay_ms(attempt);
                
                // Add jitter by applying a random factor between (1-jitter_factor) and (1+jitter_factor)
                let jitter_range = *jitter_factor;
                let jitter_multiplier = 1.0 - jitter_range + (2.0 * jitter_range * rand::random::<f64>());
                
                (base_delay as f64 * jitter_multiplier).round() as u64
            }
        }
    }
}

impl Default for BackoffStrategy {
    fn default() -> Self {
        Self::Exponential {
            initial_delay_ms: 100,
            multiplier: 2.0,
            max_delay_ms: 10000, // 10 seconds
        }
    }
}
```

### 2. Retry Configuration

The retry mechanism can be configured with the following parameters:

```rust
/// Configuration for the retry mechanism
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of attempts (including the initial attempt)
    pub max_attempts: u32,
    
    /// Strategy to use for calculating delay between attempts
    pub backoff_strategy: BackoffStrategy,
    
    /// Optional function to determine if an error should trigger a retry
    pub should_retry: Option<Arc<dyn Fn(&dyn Error) -> bool + Send + Sync>>,
    
    /// Optional name for the retry mechanism (useful for metrics and logging)
    pub name: Option<String>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff_strategy: BackoffStrategy::default(),
            should_retry: None,
            name: None,
        }
    }
}
```

### 3. Retry Mechanism Implementation

The core implementation includes retry logic with backoff, error handling, and metrics collection:

```rust
/// Retry mechanism for automatically retrying failed operations
pub struct RetryMechanism {
    /// Configuration parameters
    config: RetryConfig,
    
    /// Metrics collection for retry operations
    #[cfg(feature = "metrics")]
    metrics: RetryMetrics,
}

impl RetryMechanism {
    /// Creates a new retry mechanism with the provided configuration
    pub fn new(config: RetryConfig) -> Self {
        #[cfg(feature = "metrics")]
        let metrics = RetryMetrics::new(config.name.clone());
        
        Self {
            config,
            #[cfg(feature = "metrics")]
            metrics,
        }
    }
    
    /// Creates a new retry mechanism with default configuration
    pub fn default() -> Self {
        Self::new(RetryConfig::default())
    }
    
    /// Creates a new retry mechanism with a specific number of attempts
    pub fn with_max_attempts(max_attempts: u32) -> Self {
        Self::new(RetryConfig {
            max_attempts,
            ..Default::default()
        })
    }
    
    /// Creates a new retry mechanism with a specific backoff strategy
    pub fn with_backoff(backoff_strategy: BackoffStrategy) -> Self {
        Self::new(RetryConfig {
            backoff_strategy,
            ..Default::default()
        })
    }
    
    /// Executes an operation with retry capability
    pub async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, ResilienceError<E>>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: Future<Output = Result<T, E>> + Send,
        E: Error + 'static,
    {
        let mut attempt = 1;
        
        loop {
            #[cfg(feature = "metrics")]
            self.metrics.record_attempt(attempt);
            
            match operation().await {
                Ok(value) => {
                    #[cfg(feature = "metrics")]
                    self.metrics.record_success(attempt);
                    
                    return Ok(value);
                }
                Err(err) => {
                    #[cfg(feature = "metrics")]
                    self.metrics.record_failure();
                    
                    // Check if we should retry based on the error and retry predicate
                    let should_retry = match &self.config.should_retry {
                        Some(predicate) => predicate(&err),
                        None => true, // By default, retry all errors
                    };
                    
                    if !should_retry {
                        return Err(ResilienceError::NonRetryableError(Box::new(err)));
                    }
                    
                    // Check if we've exceeded max attempts
                    if attempt >= self.config.max_attempts {
                        return Err(ResilienceError::MaxAttemptsExceeded(Box::new(err)));
                    }
                    
                    // Calculate and apply backoff delay
                    let delay_ms = self.config.backoff_strategy.calculate_delay_ms(attempt);
                    
                    #[cfg(feature = "metrics")]
                    self.metrics.record_retry(delay_ms);
                    
                    // Wait before retrying
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    
                    // Increment attempt counter
                    attempt += 1;
                }
            }
        }
    }
    
    /// Gets the maximum number of attempts
    pub fn max_attempts(&self) -> u32 {
        self.config.max_attempts
    }
    
    /// Gets a reference to the backoff strategy
    pub fn backoff_strategy(&self) -> &BackoffStrategy {
        &self.config.backoff_strategy
    }
    
    /// Gets metrics for this retry mechanism
    #[cfg(feature = "metrics")]
    pub fn metrics(&self) -> RetryMetricsSnapshot {
        self.metrics.get_metrics()
    }
}
```

### 4. Error Types for Retry Mechanism

The resilience framework provides specific error types for retry operations:

```rust
#[derive(Debug, Error)]
pub enum ResilienceError<E: Error + 'static> {
    #[error("Maximum number of attempts ({0}) exceeded")]
    MaxAttemptsExceeded(Box<dyn Error + Send + Sync>),
    
    #[error("Operation error is not retryable: {0}")]
    NonRetryableError(Box<dyn Error + Send + Sync>),
    
    #[error("Operation failed: {0}")]
    Operation(Box<dyn Error + Send + Sync>),
    
    #[error("Error of type {0}")]
    Other(String),
}
```

### 5. Optional Metrics Collection

When the "metrics" feature is enabled, the retry mechanism collects operational metrics:

```rust
#[cfg(feature = "metrics")]
#[derive(Debug)]
pub struct RetryMetrics {
    /// Name of this retry mechanism instance
    name: String,
    
    /// Total number of attempts
    attempts: AtomicU64,
    
    /// Number of operations that eventually succeeded
    successes: AtomicU64,
    
    /// Number of operations that failed after all retries
    failures: AtomicU64,
    
    /// Total retry count
    retries: AtomicU64,
    
    /// Histogram of attempts until success
    attempts_until_success: Arc<RwLock<HashMap<u32, u64>>>,
    
    /// Total delay across all retries (ms)
    total_delay_ms: AtomicU64,
}

#[cfg(feature = "metrics")]
impl RetryMetrics {
    pub fn new(name: Option<String>) -> Self {
        let name = name.unwrap_or_else(|| format!("retry_mechanism-{}", Uuid::new_v4()));
        
        Self {
            name,
            attempts: AtomicU64::new(0),
            successes: AtomicU64::new(0),
            failures: AtomicU64::new(0),
            retries: AtomicU64::new(0),
            attempts_until_success: Arc::new(RwLock::new(HashMap::new())),
            total_delay_ms: AtomicU64::new(0),
        }
    }
    
    pub fn record_attempt(&self, attempt: u32) {
        self.attempts.fetch_add(1, Ordering::SeqCst);
    }
    
    pub fn record_success(&self, attempt: u32) {
        self.successes.fetch_add(1, Ordering::SeqCst);
        
        let mut attempts = self.attempts_until_success.write().unwrap();
        let count = attempts.entry(attempt).or_insert(0);
        *count += 1;
    }
    
    pub fn record_failure(&self) {
        self.failures.fetch_add(1, Ordering::SeqCst);
    }
    
    pub fn record_retry(&self, delay_ms: u64) {
        self.retries.fetch_add(1, Ordering::SeqCst);
        self.total_delay_ms.fetch_add(delay_ms, Ordering::SeqCst);
    }
    
    pub fn get_metrics(&self) -> RetryMetricsSnapshot {
        RetryMetricsSnapshot {
            name: self.name.clone(),
            attempts: self.attempts.load(Ordering::SeqCst),
            successes: self.successes.load(Ordering::SeqCst),
            failures: self.failures.load(Ordering::SeqCst),
            retries: self.retries.load(Ordering::SeqCst),
            attempts_until_success: self.attempts_until_success.read().unwrap().clone(),
            total_delay_ms: self.total_delay_ms.load(Ordering::SeqCst),
            average_delay_ms: if self.retries.load(Ordering::SeqCst) > 0 {
                self.total_delay_ms.load(Ordering::SeqCst) / self.retries.load(Ordering::SeqCst)
            } else {
                0
            },
        }
    }
}

#[cfg(feature = "metrics")]
#[derive(Debug, Clone, Serialize)]
pub struct RetryMetricsSnapshot {
    pub name: String,
    pub attempts: u64,
    pub successes: u64,
    pub failures: u64,
    pub retries: u64,
    pub attempts_until_success: HashMap<u32, u64>,
    pub total_delay_ms: u64,
    pub average_delay_ms: u64,
}
```

## Usage Examples

### Basic Retry Usage

```rust
// Create a retry mechanism with default settings
let retry = RetryMechanism::default();

// Create a retry mechanism with custom settings
let retry = RetryMechanism::new(RetryConfig {
    max_attempts: 5,
    backoff_strategy: BackoffStrategy::Exponential {
        initial_delay_ms: 100,
        multiplier: 2.0,
        max_delay_ms: 30000,
    },
    should_retry: None,
    name: Some("api_client".to_string()),
});

// Execute operation with retry
let result = retry.execute(|| async {
    api_client.get_data().await
}).await;

// Handle result
match result {
    Ok(data) => println!("Got data: {:?}", data),
    Err(ResilienceError::MaxAttemptsExceeded(err)) => {
        println!("Failed after {} attempts: {}", retry.max_attempts(), err)
    }
    Err(err) => println!("Other error: {}", err),
}
```

### Selective Retry with Predicate

```rust
// Create a retry mechanism that only retries on specific errors
let retry = RetryMechanism::new(RetryConfig {
    max_attempts: 3,
    backoff_strategy: BackoffStrategy::Linear {
        initial_delay_ms: 200,
        increment_ms: 300,
    },
    should_retry: Some(Arc::new(|err| {
        // Only retry on connection or timeout errors
        if let Some(api_err) = err.downcast_ref::<ApiError>() {
            matches!(api_err, 
                ApiError::ConnectionError(_) | 
                ApiError::Timeout(_) |
                ApiError::RateLimited
            )
        } else {
            false
        }
    })),
    name: Some("selective_retry".to_string()),
});

// Execute with selective retry
let result = retry.execute(|| async {
    api_client.send_request(request).await
}).await;
```

### Using Fibonacci Backoff Strategy

```rust
// Create a retry mechanism with Fibonacci backoff
let retry = RetryMechanism::new(RetryConfig {
    max_attempts: 8,
    backoff_strategy: BackoffStrategy::Fibonacci {
        initial_delay_ms: 100,
        max_delay_ms: 60000, // 1 minute max
    },
    should_retry: None,
    name: Some("fibonacci_retry".to_string()),
});

// Execute with Fibonacci backoff
let result = retry.execute(|| async {
    database.query(sql).await
}).await;
```

### Using Jittered Backoff for Distributed Systems

```rust
// Create a retry mechanism with jittered exponential backoff
let retry = RetryMechanism::new(RetryConfig {
    max_attempts: 5,
    backoff_strategy: BackoffStrategy::Jittered {
        base_strategy: Box::new(BackoffStrategy::Exponential {
            initial_delay_ms: 200,
            multiplier: 2.0,
            max_delay_ms: 30000,
        }),
        jitter_factor: 0.3, // +/- 30% randomness
    },
    should_retry: None,
    name: Some("distributed_retry".to_string()),
});

// Execute with jittered backoff
let result = retry.execute(|| async {
    distributed_system.perform_operation().await
}).await;
```

### Integration with MCP Protocol

```rust
// Create retry mechanism for MCP protocol operations
let mcp_retry = RetryMechanism::new(RetryConfig {
    max_attempts: 3,
    backoff_strategy: BackoffStrategy::Exponential {
        initial_delay_ms: 250,
        multiplier: 2.0,
        max_delay_ms: 5000,
    },
    should_retry: Some(Arc::new(|err| {
        // Only retry on connection or transient errors
        if let Some(mcp_err) = err.downcast_ref::<McpError>() {
            matches!(mcp_err,
                McpError::ConnectionError(_) |
                McpError::Timeout(_) |
                McpError::ServerBusy |
                McpError::TemporaryUnavailable(_)
            )
        } else {
            false
        }
    })),
    name: Some("mcp_protocol".to_string()),
});

// Create a wrapper for MCP protocol that uses retry
struct RetryingMcpProtocol {
    inner: Arc<dyn McpProtocol>,
    retry: RetryMechanism,
}

impl RetryingMcpProtocol {
    pub fn new(protocol: Arc<dyn McpProtocol>) -> Self {
        Self {
            inner: protocol,
            retry: RetryMechanism::new(RetryConfig {
                max_attempts: 3,
                backoff_strategy: BackoffStrategy::Exponential {
                    initial_delay_ms: 250,
                    multiplier: 2.0,
                    max_delay_ms: 5000,
                },
                should_retry: Some(Arc::new(|err| {
                    // Only retry on connection or transient errors
                    if let Some(mcp_err) = err.downcast_ref::<McpError>() {
                        matches!(mcp_err,
                            McpError::ConnectionError(_) |
                            McpError::Timeout(_) |
                            McpError::ServerBusy |
                            McpError::TemporaryUnavailable(_)
                        )
                    } else {
                        false
                    }
                })),
                name: Some("mcp_protocol".to_string()),
            }),
        }
    }
    
    pub async fn send_message(&self, message: McpMessage) -> Result<McpResponse, McpError> {
        // Execute with retry
        let result = self.retry.execute(|| {
            let msg = message.clone();
            let protocol = self.inner.clone();
            
            async move {
                protocol.send_message(msg).await
            }
        }).await;
        
        match result {
            Ok(response) => Ok(response),
            Err(ResilienceError::MaxAttemptsExceeded(err)) => {
                if let Some(mcp_err) = err.downcast_ref::<McpError>() {
                    Err(mcp_err.clone())
                } else {
                    Err(McpError::RetryFailed(format!("Max attempts exceeded: {}", err)))
                }
            },
            Err(ResilienceError::NonRetryableError(err)) => {
                if let Some(mcp_err) = err.downcast_ref::<McpError>() {
                    Err(mcp_err.clone())
                } else {
                    Err(McpError::OperationFailed(format!("Non-retryable error: {}", err)))
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
async fn test_retry_success_first_attempt() {
    let retry = RetryMechanism::default();
    
    let mut attempt = 0;
    
    // Operation succeeds on first attempt
    let result = retry.execute(|| {
        attempt += 1;
        async {
            Ok::<_, anyhow::Error>("success")
        }
    }).await;
    
    assert!(result.is_ok());
    assert_eq!(attempt, 1);
}

#[tokio::test]
async fn test_retry_success_after_retries() {
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 3,
        backoff_strategy: BackoffStrategy::Constant { delay_ms: 10 },
        ..Default::default()
    });
    
    let mut attempt = 0;
    
    // Operation fails on first attempt but succeeds on second
    let result = retry.execute(|| {
        attempt += 1;
        async move {
            if attempt == 1 {
                Err(anyhow::anyhow!("first attempt error"))
            } else {
                Ok::<_, anyhow::Error>("success")
            }
        }
    }).await;
    
    assert!(result.is_ok());
    assert_eq!(attempt, 2);
}

#[tokio::test]
async fn test_retry_max_attempts_exceeded() {
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 3,
        backoff_strategy: BackoffStrategy::Constant { delay_ms: 10 },
        ..Default::default()
    });
    
    let mut attempt = 0;
    
    // Operation fails on all attempts
    let result = retry.execute(|| {
        attempt += 1;
        async {
            Err(anyhow::anyhow!("persistent error"))
        }
    }).await;
    
    assert!(matches!(result, Err(ResilienceError::MaxAttemptsExceeded(_))));
    assert_eq!(attempt, 3);
}

#[tokio::test]
async fn test_retry_with_predicate() {
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 3,
        backoff_strategy: BackoffStrategy::Constant { delay_ms: 10 },
        should_retry: Some(Arc::new(|err| {
            err.to_string().contains("retryable")
        })),
        ..Default::default()
    });
    
    // Test with non-retryable error
    let result1 = retry.execute(|| async {
        Err(anyhow::anyhow!("non-retryable error"))
    }).await;
    
    assert!(matches!(result1, Err(ResilienceError::NonRetryableError(_))));
    
    // Test with retryable error
    let mut attempt = 0;
    let result2 = retry.execute(|| {
        attempt += 1;
        async move {
            Err(anyhow::anyhow!("retryable error"))
        }
    }).await;
    
    assert!(matches!(result2, Err(ResilienceError::MaxAttemptsExceeded(_))));
    assert_eq!(attempt, 3);
}

#[tokio::test]
async fn test_backoff_strategies() {
    // Test constant backoff
    let constant = BackoffStrategy::Constant { delay_ms: 100 };
    assert_eq!(constant.calculate_delay_ms(1), 100);
    assert_eq!(constant.calculate_delay_ms(5), 100);
    
    // Test linear backoff
    let linear = BackoffStrategy::Linear { 
        initial_delay_ms: 100, 
        increment_ms: 50 
    };
    assert_eq!(linear.calculate_delay_ms(1), 100);
    assert_eq!(linear.calculate_delay_ms(2), 150);
    assert_eq!(linear.calculate_delay_ms(5), 300);
    
    // Test exponential backoff
    let exponential = BackoffStrategy::Exponential { 
        initial_delay_ms: 100, 
        multiplier: 2.0,
        max_delay_ms: 1000
    };
    assert_eq!(exponential.calculate_delay_ms(1), 100);
    assert_eq!(exponential.calculate_delay_ms(2), 200);
    assert_eq!(exponential.calculate_delay_ms(3), 400);
    assert_eq!(exponential.calculate_delay_ms(5), 1000); // Capped at max
    
    // Test Fibonacci backoff
    let fibonacci = BackoffStrategy::Fibonacci { 
        initial_delay_ms: 100, 
        max_delay_ms: 1000 
    };
    assert_eq!(fibonacci.calculate_delay_ms(1), 100);
    assert_eq!(fibonacci.calculate_delay_ms(2), 100);
    assert_eq!(fibonacci.calculate_delay_ms(3), 200);
    assert_eq!(fibonacci.calculate_delay_ms(4), 300);
    assert_eq!(fibonacci.calculate_delay_ms(5), 500);
    assert_eq!(fibonacci.calculate_delay_ms(6), 800);
    assert_eq!(fibonacci.calculate_delay_ms(7), 1000); // Capped at max
}
```

## Conclusion

The Retry Mechanism implementation provides a sophisticated system for handling transient failures in the MCP protocol. It supports:

1. Multiple backoff strategies (constant, linear, exponential, Fibonacci, jittered)
2. Selective retry based on error types
3. Configurable maximum attempts
4. Detailed metrics collection
5. Thread-safe operation
6. Integration with MCP protocol

This implementation satisfies the requirements outlined in the resilience framework specification and provides a powerful tool for building resilient MCP systems that can recover from transient failures. 