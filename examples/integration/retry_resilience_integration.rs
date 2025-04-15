//! Integration example demonstrating the retry mechanism with other resilience components
//! 
//! This example shows how to use the retry mechanism with circuit breakers,
//! bulkheads, and rate limiters to create a comprehensive resilience solution.

use std::error::Error as StdError;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use std::sync::RwLock;
use std::collections::HashMap;

use squirrel_mcp::resilience::{
    // Core resilience components
    RetryMechanism,
    RetryConfig,
    BackoffStrategy,
    StandardRetryPolicy,
    RetryPolicy,
    StandardCircuitBreaker,
    bulkhead,
    rate_limiter,
    
    // Integrated utility functions
    with_retry,
    with_circuit_breaker,
    with_bulkhead,
    with_rate_limiting,
    with_comprehensive_resilience,
    
    // Error and result types
    ResilienceError,
    Result as ResilienceResult,
};

use tokio::time::sleep;
use tracing::{info, warn, error, debug};

/// A service that simulates different types of failures for testing resilience mechanisms
struct FlakeyService {
    /// The number of initial failures before succeeding
    failures_before_success: AtomicUsize,
    /// The current state of failure injection
    is_failing: RwLock<bool>,
    /// Track number of calls per service endpoint
    call_counts: RwLock<HashMap<String, usize>>,
    /// Configurable response delay
    response_delay: Duration,
}

impl FlakeyService {
    /// Create a new flaky service
    pub fn new(failures_before_success: usize, response_delay: Duration) -> Self {
        Self {
            failures_before_success: AtomicUsize::new(failures_before_success),
            is_failing: RwLock::new(false),
            call_counts: RwLock::new(HashMap::new()),
            response_delay,
        }
    }
    
    /// Enable failure injection
    pub fn start_failing(&self) {
        let mut is_failing = self.is_failing.write().unwrap();
        *is_failing = true;
    }
    
    /// Disable failure injection
    pub fn stop_failing(&self) {
        let mut is_failing = self.is_failing.write().unwrap();
        *is_failing = false;
    }
    
    /// Get current call counts
    pub fn get_call_counts(&self) -> HashMap<String, usize> {
        self.call_counts.read().unwrap().clone()
    }
    
    /// Execute an operation on the service that may fail unpredictably
    pub async fn execute_unpredictable(&self, endpoint: &str) -> Result<String, Box<dyn StdError + Send + Sync>> {
        // Record call count
        {
            let mut counts = self.call_counts.write().unwrap();
            *counts.entry(endpoint.to_string()).or_insert(0) += 1;
        }
        
        // Simulate operation delay
        sleep(self.response_delay).await;
        
        // Check if we're in a failing state
        if *self.is_failing.read().unwrap() {
            warn!("Service {} in failing state, returning error", endpoint);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::ConnectionAborted,
                format!("Service {} temporarily unavailable", endpoint)
            )));
        }
        
        // Otherwise, return success
        Ok(format!("Success from {}", endpoint))
    }
    
    /// Execute an operation that will fail a set number of times, then succeed
    pub async fn execute_retryable(&self, endpoint: &str) -> Result<String, Box<dyn StdError + Send + Sync>> {
        // Record call count
        let call_count = {
            let mut counts = self.call_counts.write().unwrap();
            let count = counts.entry(endpoint.to_string()).or_insert(0);
            *count += 1;
            *count
        };
        
        debug!("Service {} called (attempt {})", endpoint, call_count);
        
        // Simulate operation delay
        sleep(self.response_delay).await;
        
        let remaining_failures = self.failures_before_success.load(Ordering::Relaxed);
        
        if remaining_failures > 0 {
            self.failures_before_success.fetch_sub(1, Ordering::Relaxed);
            
            // Random errors on each failure
            let error_kind = match call_count % 3 {
                0 => std::io::ErrorKind::ConnectionReset,
                1 => std::io::ErrorKind::ConnectionRefused,
                _ => std::io::ErrorKind::ConnectionAborted,
            };
            
            warn!("Service {} failed with {:?} (attempt {})", endpoint, error_kind, call_count);
            
            Err(Box::new(std::io::Error::new(
                error_kind,
                format!("Connection error on attempt {}", call_count)
            )))
        } else {
            info!("Service {} succeeded (attempt {})", endpoint, call_count);
            
            Ok(format!("Success from {} after {} attempts", endpoint, call_count))
        }
    }
    
    /// Reset the service state for a new test
    pub fn reset(&self, new_failures: usize) {
        self.failures_before_success.store(new_failures, Ordering::Relaxed);
        *self.is_failing.write().unwrap() = false;
        self.call_counts.write().unwrap().clear();
    }
}

/// Example demonstrating retry mechanism with circuit breaker
async fn demonstrate_retry_with_circuit_breaker() -> ResilienceResult<()> {
    info!("=== Retry with Circuit Breaker Example ===");
    
    // Create a service that fails 3 times before succeeding
    let service = Arc::new(FlakeyService::new(
        3, // Failures before success
        Duration::from_millis(10), // Quick responses
    ));
    
    // Create a retry mechanism
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 2, // Only retry once
        base_delay: Duration::from_millis(50),
        max_delay: Duration::from_millis(500),
        use_jitter: true,
        backoff_strategy: BackoffStrategy::Exponential,
    });
    
    // Create a circuit breaker that will open after 5 failures
    let mut circuit_breaker = StandardCircuitBreaker::new(
        "demo-service".to_string(),
        5, // Failure threshold
        Duration::from_secs(5), // Reset timeout
    );
    
    // Create endpoints to test
    let endpoints = vec!["api/users", "api/products", "api/orders"];
    
    // Try multiple operations
    for i in 0..10 {
        let endpoint = endpoints[i % endpoints.len()];
        let service_clone = service.clone();
        
        // Execute with circuit breaker and retry
        let result = with_circuit_breaker(
            &mut circuit_breaker,
            endpoint,
            || async move {
                with_retry(&retry, async {
                    service_clone.execute_retryable(endpoint).await
                        .map_err(|e| ResilienceError::General(e.to_string()))
                }).await
            }
        ).await;
        
        match result {
            Ok(response) => {
                info!("Operation succeeded: {}", response);
            }
            Err(e) => {
                match e {
                    ResilienceError::CircuitOpen(_) => {
                        warn!("Circuit is open, fast failing: {}", e);
                    }
                    _ => {
                        error!("Operation failed: {}", e);
                    }
                }
            }
        }
        
        // Add a short delay between operations
        sleep(Duration::from_millis(100)).await;
    }
    
    // Show call counts
    let call_counts = service.get_call_counts();
    info!("Total call counts: {:?}", call_counts);
    
    // Show circuit breaker state
    info!("Circuit breaker final state: {:?}", circuit_breaker.state().await);
    
    Ok(())
}

/// Example demonstrating retry mechanism with bulkhead
async fn demonstrate_retry_with_bulkhead() -> ResilienceResult<()> {
    info!("=== Retry with Bulkhead Example ===");
    
    // Create a service with delays
    let service = Arc::new(FlakeyService::new(
        2, // Failures before success
        Duration::from_millis(100), // Slow responses
    ));
    
    // Create a retry mechanism
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(20),
        max_delay: Duration::from_millis(200),
        use_jitter: true,
        backoff_strategy: BackoffStrategy::Exponential,
    });
    
    // Create a bulkhead that limits concurrent operations
    let bulkhead = bulkhead::Bulkhead::new(bulkhead::BulkheadConfig {
        max_concurrent_calls: 2, // Only allow 2 concurrent calls
        max_queue_size: Some(2), // And 2 more waiting
        call_timeout: Some(Duration::from_millis(500)),
        queue_timeout: Some(Duration::from_millis(200)),
    });
    
    // Create multiple concurrent operations
    let mut handles = Vec::new();
    for i in 0..8 {
        let endpoint = format!("api/endpoint{}", i);
        let service_clone = service.clone();
        let bulkhead_clone = bulkhead.clone();
        let retry_clone = retry.clone();
        
        // Spawn concurrent tasks
        let handle = tokio::spawn(async move {
            let result = with_bulkhead(
                &bulkhead_clone,
                async {
                    with_retry(&retry_clone, async {
                        service_clone.execute_retryable(&endpoint).await
                            .map_err(|e| ResilienceError::General(e.to_string()))
                    }).await
                }
            ).await;
            
            (endpoint, result)
        });
        
        handles.push(handle);
        
        // Stagger starting the tasks slightly
        sleep(Duration::from_millis(10)).await;
    }
    
    // Wait for all operations to complete
    for handle in handles {
        let (endpoint, result) = handle.await.unwrap();
        match result {
            Ok(response) => {
                info!("Operation {} succeeded: {}", endpoint, response);
            }
            Err(e) => {
                match e {
                    ResilienceError::Bulkhead(_) => {
                        warn!("Bulkhead rejected operation {}: {}", endpoint, e);
                    }
                    _ => {
                        error!("Operation {} failed: {}", endpoint, e);
                    }
                }
            }
        }
    }
    
    // Show call counts
    let call_counts = service.get_call_counts();
    info!("Total call counts: {:?}", call_counts);
    
    // Show bulkhead metrics
    let metrics = bulkhead.metrics().await;
    info!("Bulkhead metrics: available_permits={}, queue_depth={}, rejection_count={}, timeout_count={}",
        metrics.available_permits,
        metrics.queue_depth,
        metrics.rejection_count,
        metrics.timeout_count
    );
    
    Ok(())
}

/// Example demonstrating retry mechanism with rate limiter
async fn demonstrate_retry_with_rate_limiter() -> ResilienceResult<()> {
    info!("=== Retry with Rate Limiter Example ===");
    
    // Create a service
    let service = Arc::new(FlakeyService::new(
        1, // Fail once for each call
        Duration::from_millis(10), // Quick responses
    ));
    
    // Create a retry mechanism
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_millis(500),
        use_jitter: true,
        backoff_strategy: BackoffStrategy::Exponential,
    });
    
    // Create a rate limiter that limits request rate
    let rate_limiter = rate_limiter::RateLimiter::new(rate_limiter::RateLimiterConfig {
        limit_for_period: 5, // Only allow 5 requests
        limit_refresh_period: Duration::from_secs(1), // Per second
        timeout_duration: Some(Duration::from_millis(300)), // Wait up to 300ms for permit
    });
    
    // Execute multiple operations in rapid succession
    for i in 0..15 {
        let endpoint = format!("api/endpoint{}", i);
        let service_clone = service.clone();
        let rate_limiter_clone = rate_limiter.clone();
        let retry_clone = retry.clone();
        
        // Execute with rate limiter and retry
        let result = with_rate_limiting(
            &rate_limiter_clone,
            async {
                with_retry(&retry_clone, async {
                    service_clone.execute_retryable(&endpoint).await
                        .map_err(|e| ResilienceError::General(e.to_string()))
                }).await
            }
        ).await;
        
        match result {
            Ok(response) => {
                info!("Operation {} succeeded: {}", endpoint, response);
            }
            Err(e) => {
                match e {
                    ResilienceError::RateLimit(_) => {
                        warn!("Rate limit exceeded for operation {}: {}", endpoint, e);
                    }
                    _ => {
                        error!("Operation {} failed: {}", endpoint, e);
                    }
                }
            }
        }
        
        // No delay between operations to test rate limiting
    }
    
    // Show call counts
    let call_counts = service.get_call_counts();
    info!("Total call counts: {:?}", call_counts);
    
    // Show rate limiter metrics
    let metrics = rate_limiter.metrics().await;
    info!("Rate limiter metrics: available_permits={}, waiting_threads={}, rejection_count={}",
        metrics.available_permits,
        metrics.waiting_threads,
        metrics.rejection_count
    );
    
    Ok(())
}

/// Example demonstrating comprehensive resilience with all components
async fn demonstrate_comprehensive_resilience() -> ResilienceResult<()> {
    info!("=== Comprehensive Resilience Example ===");
    
    // Create a service
    let service = Arc::new(FlakeyService::new(
        2, // Fail twice for each call
        Duration::from_millis(20), // Quick responses
    ));
    
    // Create a circuit breaker
    let mut circuit_breaker = StandardCircuitBreaker::new(
        "comprehensive-demo".to_string(),
        10, // Failure threshold
        Duration::from_secs(5), // Reset timeout
    );
    
    // Create a bulkhead
    let bulkhead = bulkhead::Bulkhead::new(bulkhead::BulkheadConfig {
        max_concurrent_calls: 3, 
        max_queue_size: Some(3),
        call_timeout: Some(Duration::from_millis(500)),
        queue_timeout: Some(Duration::from_millis(200)),
    });
    
    // Create a rate limiter
    let rate_limiter = rate_limiter::RateLimiter::new(rate_limiter::RateLimiterConfig {
        limit_for_period: 10, 
        limit_refresh_period: Duration::from_secs(1),
        timeout_duration: Some(Duration::from_millis(300)),
    });
    
    // Create a retry policy
    let retry_policy = Box::new(StandardRetryPolicy::with_exponential_backoff(
        3, // Max retries
        Duration::from_millis(50), // Base delay
        Duration::from_millis(500), // Max delay
    ));
    
    // Execute multiple concurrent operations
    let mut handles = Vec::new();
    for i in 0..20 {
        let endpoint = format!("api/endpoint{}", i);
        let service_clone = service.clone();
        let circuit_breaker_clone = circuit_breaker.clone();
        let bulkhead_clone = bulkhead.clone();
        let rate_limiter_clone = rate_limiter.clone();
        let retry_policy_clone = StandardRetryPolicy::with_exponential_backoff(
            3, // Max retries
            Duration::from_millis(50), // Base delay
            Duration::from_millis(500), // Max delay
        );
        
        // Spawn concurrent tasks with all resilience mechanisms
        let handle = tokio::spawn(async move {
            let mut cb = circuit_breaker_clone;
            
            let result = with_comprehensive_resilience(
                &mut cb,
                &bulkhead_clone,
                &rate_limiter_clone,
                retry_policy_clone,
                Duration::from_millis(300), // Operation timeout
                &endpoint,
                move || {
                    let service = service_clone.clone();
                    let ep = endpoint.clone();
                    
                    Box::pin(async move {
                        service.execute_retryable(&ep).await
                            .map_err(|e| ResilienceError::General(e.to_string()))
                    })
                }
            ).await;
            
            (endpoint, result)
        });
        
        handles.push(handle);
        
        // Stagger starting the tasks slightly
        sleep(Duration::from_millis(20)).await;
    }
    
    // After 10 operations, start injecting failures
    sleep(Duration::from_millis(200)).await;
    service.start_failing();
    
    // Wait for all operations to complete
    for handle in handles {
        let (endpoint, result) = handle.await.unwrap();
        match result {
            Ok(response) => {
                info!("Operation {} succeeded: {}", endpoint, response);
            }
            Err(e) => {
                // Categorize the error based on type
                let error_type = match e {
                    ResilienceError::CircuitOpen(_) => "Circuit Open",
                    ResilienceError::Bulkhead(_) => "Bulkhead Rejection",
                    ResilienceError::RateLimit(_) => "Rate Limit",
                    ResilienceError::Timeout(_) => "Timeout",
                    ResilienceError::RetryExceeded(_) => "Retry Exceeded",
                    _ => "Other Error",
                };
                
                warn!("Operation {} failed with {}: {}", endpoint, error_type, e);
            }
        }
    }
    
    // Show call counts
    let call_counts = service.get_call_counts();
    info!("Total call counts: {:?}", call_counts);
    
    // Show circuit breaker state
    info!("Circuit breaker final state: {:?}", circuit_breaker.state().await);
    
    // Show bulkhead metrics
    let bulkhead_metrics = bulkhead.metrics().await;
    info!("Bulkhead metrics: available_permits={}, queue_depth={}, rejection_count={}, timeout_count={}",
        bulkhead_metrics.available_permits,
        bulkhead_metrics.queue_depth,
        bulkhead_metrics.rejection_count,
        bulkhead_metrics.timeout_count
    );
    
    // Show rate limiter metrics
    let rate_metrics = rate_limiter.metrics().await;
    info!("Rate limiter metrics: available_permits={}, waiting_threads={}, rejection_count={}",
        rate_metrics.available_permits,
        rate_metrics.waiting_threads,
        rate_metrics.rejection_count
    );
    
    Ok(())
}

/// Main entrypoint for the example
#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting resilience integration example");
    
    // Demonstrate retry with circuit breaker
    demonstrate_retry_with_circuit_breaker().await?;
    
    // Demonstrate retry with bulkhead
    demonstrate_retry_with_bulkhead().await?;
    
    // Demonstrate retry with rate limiter
    demonstrate_retry_with_rate_limiter().await?;
    
    // Demonstrate comprehensive resilience
    demonstrate_comprehensive_resilience().await?;
    
    info!("Resilience integration example completed");
    Ok(())
} 