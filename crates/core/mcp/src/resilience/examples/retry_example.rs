// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Example demonstrating the retry mechanism with different retry policies

use std::time::Duration;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::error::Error as StdError;
use std::sync::Arc;
use tracing::{info, warn, error};
use tokio::time::sleep;

use crate::resilience::{
    RetryMechanism, 
    RetryConfig, 
    BackoffStrategy, 
    StandardRetryPolicy,
    RetryPolicy,
    ResilienceError
};

/// A service with configurable failure behavior for testing retry mechanisms
struct FlakeyService {
    /// Counter for tracking the number of calls
    call_count: AtomicUsize,
    /// The number of initial failures before succeeding
    failures_before_success: usize,
    /// Whether the service should timeout instead of failing with an error
    use_timeouts: bool,
    /// Duration to wait before responding (for simulating timeouts)
    response_delay: Duration,
}

impl FlakeyService {
    /// Create a new flaky service
    pub fn new(failures_before_success: usize, use_timeouts: bool, response_delay: Duration) -> Self {
        Self {
            call_count: AtomicUsize::new(0),
            failures_before_success,
            use_timeouts,
            response_delay,
        }
    }
    
    /// Execute an operation on the service, which may fail
    pub async fn execute(&self) -> Result<String, Box<dyn StdError + Send + Sync>> {
        let count = self.call_count.fetch_add(1, Ordering::SeqCst);
        info!("FlakeyService called (attempt {})", count + 1);
        
        // Simulate operation delay
        sleep(self.response_delay).await;
        
        if count < self.failures_before_success {
            if self.use_timeouts {
                // In timeout mode, we already delayed, so just return an error
                warn!("FlakeyService timed out (attempt {})", count + 1);
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!("Operation timed out on attempt {}", count + 1)
                )))
            } else {
                // Random errors on each failure
                let error_kind = match count % 3 {
                    0 => std::io::ErrorKind::ConnectionReset,
                    1 => std::io::ErrorKind::ConnectionRefused,
                    _ => std::io::ErrorKind::ConnectionAborted,
                };
                
                warn!("FlakeyService failed with {:?} (attempt {})", error_kind, count + 1);
                Err(Box::new(std::io::Error::new(
                    error_kind,
                    format!("Connection error on attempt {}", count + 1)
                )))
            }
        } else {
            info!("FlakeyService succeeded (attempt {})", count + 1);
            Ok(format!("Success on attempt {}", count + 1))
        }
    }
    
    /// Reset the service for a new test
    pub fn reset(&self) {
        self.call_count.store(0, Ordering::SeqCst);
    }
}

/// Custom retry policy that only retries specific errors
struct ConnectionRetryPolicy {
    max_retries: usize,
    base_delay: Duration,
    max_delay: Duration,
}

impl ConnectionRetryPolicy {
    pub fn new(max_retries: usize, base_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_retries,
            base_delay,
            max_delay,
        }
    }
}

impl RetryPolicy for ConnectionRetryPolicy {
    fn should_retry(&self, attempt: usize, error: &ResilienceError) -> bool {
        if attempt >= self.max_retries {
            return false;
        }
        
        // Only retry connection-related errors or timeouts
        match error {
            ResilienceError::Timeout(_) => true,
            ResilienceError::General(msg) | ResilienceError::OperationFailed(msg) => {
                // Check if the message contains connection-related keywords
                msg.contains("Connection") || 
                msg.contains("connection") || 
                msg.contains("timeout") || 
                msg.contains("timed out")
            }
            _ => false,
        }
    }
    
    fn backoff_duration(&self, attempt: usize) -> Duration {
        // Use exponential backoff with jitter
        let base = self.base_delay.as_millis() as f64;
        let exp = 2.0_f64.powi(attempt as i32);
        let delay = base * exp;
        
        // Add jitter (±20%)
        let jitter_factor = 0.8 + (rand::random::<f64>() * 0.4); // 0.8 to 1.2
        let delay_with_jitter = (delay * jitter_factor) as u64;
        
        // Cap at max delay
        let final_delay = std::cmp::min(delay_with_jitter, self.max_delay.as_millis() as u64);
        Duration::from_millis(final_delay)
    }
}

/// Example demonstrating basic retry mechanism with fixed backoff
async fn demonstrate_basic_retry() -> Result<(), Box<dyn StdError + Send + Sync>> {
    info!("=== Basic Retry Example ===");
    
    // Create a service that fails 3 times before succeeding
    let service = Arc::new(FlakeyService::new(
        3, // Failures before success
        false, // Don't use timeouts
        Duration::from_millis(10), // Quick responses
    ));
    
    // Create a retry mechanism with constant backoff
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 5,
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_millis(500),
        use_jitter: false,
        backoff_strategy: BackoffStrategy::Constant,
    });
    
    let service_clone = service.clone();
    
    // Execute with retry
    let result = retry.execute(move || {
        let service = service_clone.clone();
        Box::pin(async move { service.execute().await })
    }).await;
    
    match result {
        Ok(message) => {
            info!("Basic retry succeeded: {}", message);
        }
        Err(err) => {
            error!("Basic retry failed: {}", err);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Basic retry example failed: {}", err)
            )));
        }
    }
    
    Ok(())
}

/// Example demonstrating retry with exponential backoff and jitter
async fn demonstrate_exponential_backoff() -> Result<(), Box<dyn StdError + Send + Sync>> {
    info!("=== Exponential Backoff Retry Example ===");
    
    // Create a service that fails 4 times before succeeding
    let service = Arc::new(FlakeyService::new(
        4, // More failures this time
        false, // Don't use timeouts
        Duration::from_millis(10), // Quick responses
    ));
    
    // Create a retry mechanism with exponential backoff and jitter
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 6,
        base_delay: Duration::from_millis(50),
        max_delay: Duration::from_millis(1000),
        use_jitter: true, // Add jitter to avoid retry storms
        backoff_strategy: BackoffStrategy::Exponential,
    });
    
    let service_clone = service.clone();
    
    // Execute with retry
    let result = retry.execute(move || {
        let service = service_clone.clone();
        Box::pin(async move { service.execute().await })
    }).await;
    
    match result {
        Ok(message) => {
            info!("Exponential backoff retry succeeded: {}", message);
        }
        Err(err) => {
            error!("Exponential backoff retry failed: {}", err);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Exponential backoff retry example failed: {}", err)
            )));
        }
    }
    
    Ok(())
}

/// Example demonstrating retry with a predicate to filter which errors should be retried
async fn demonstrate_predicate_retry() -> Result<(), Box<dyn StdError + Send + Sync>> {
    info!("=== Predicate Retry Example ===");
    
    // Create a service with mixed errors
    let service = Arc::new(FlakeyService::new(
        3, // Failures before success
        false, // Don't use timeouts
        Duration::from_millis(10), // Quick responses
    ));
    
    // Create a retry mechanism with a predicate
    let retry = RetryMechanism::default();
    
    let service_clone = service.clone();
    
    // Execute with predicate retry - only retry connection reset errors
    let result = retry.execute_with_predicate(
        move || {
            let service = service_clone.clone();
            Box::pin(async move { service.execute().await })
        },
        |err| {
            // Only retry connection reset errors
            if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
                io_err.kind() == std::io::ErrorKind::ConnectionReset
            } else {
                false
            }
        }
    ).await;
    
    // This will likely fail because we only retry specific errors
    match result {
        Ok(message) => {
            info!("Predicate retry succeeded: {}", message);
        }
        Err(err) => {
            // We expect this to fail since we're only retrying ConnectionReset errors
            warn!("Predicate retry failed as expected: {}", err);
        }
    }
    
    Ok(())
}

/// Example demonstrating retry with timeouts
async fn demonstrate_timeout_retry() -> Result<(), Box<dyn StdError + Send + Sync>> {
    info!("=== Timeout Retry Example ===");
    
    // Create a service that times out 2 times before succeeding
    let service = Arc::new(FlakeyService::new(
        2, // Failures before success
        true, // Use timeouts
        Duration::from_millis(50), // Shorter response time for testing
    ));
    
    // Create a retry mechanism
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 5, // Increase max attempts
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        use_jitter: false,
        backoff_strategy: BackoffStrategy::Constant,
    });
    
    let service_clone = service.clone();
    
    // Execute with timeout
    let result = retry.execute_with_timeout(
        move || {
            let service = service_clone.clone();
            Box::pin(async move { service.execute().await })
        },
        Duration::from_millis(100) // Timeout after 100ms
    ).await;
    
    match result {
        Ok(message) => {
            info!("Timeout retry succeeded: {}", message);
        }
        Err(err) => {
            error!("Timeout retry failed: {}", err);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Timeout retry example failed: {}", err)
            )));
        }
    }
    
    Ok(())
}

/// Example demonstrating custom retry policy
async fn demonstrate_custom_retry_policy() -> Result<(), Box<dyn StdError + Send + Sync>> {
    info!("=== Custom Retry Policy Example ===");
    
    // Create a service with mixed errors
    let service = Arc::new(FlakeyService::new(
        3, // Failures before success
        false, // Don't use timeouts
        Duration::from_millis(10), // Quick responses
    ));
    
    // Create a custom retry policy
    let policy = Box::new(ConnectionRetryPolicy::new(
        5,
        Duration::from_millis(50),
        Duration::from_millis(500)
    ));
    
    let service_clone = service.clone();
    
    // Use policy with a custom retry loop
    let mut attempt = 0;
    let max_attempts = 5;
    
    loop {
        match service_clone.execute().await {
            Ok(result) => {
                info!("Custom policy retry succeeded: {}", result);
                break;
            }
            Err(err) => {
                attempt += 1;
                
                // Convert to ResilienceError for policy evaluation
                let resilience_err = ResilienceError::General(err.to_string());
                
                if attempt >= max_attempts || !policy.should_retry(attempt, &resilience_err) {
                    error!("Custom policy retry failed after {} attempts: {}", attempt, resilience_err);
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Custom policy retry example failed: {}", resilience_err)
                    )));
                }
                
                let delay = policy.backoff_duration(attempt);
                warn!("Attempt {} failed, retrying after {:?}", attempt, delay);
                sleep(delay).await;
            }
        }
    }
    
    Ok(())
}

/// Example demonstrating retry using the StandardRetryPolicy
async fn demonstrate_standard_retry_policy() -> Result<(), Box<dyn StdError + Send + Sync>> {
    info!("=== Standard Retry Policy Example ===");
    
    // Create a service with mixed errors
    let service = Arc::new(FlakeyService::new(
        3, // Failures before success
        false, // Don't use timeouts
        Duration::from_millis(10), // Quick responses
    ));
    
    // Create a standard retry policy
    let policy = StandardRetryPolicy::with_exponential_backoff(
        5,
        Duration::from_millis(50),
        Duration::from_millis(500)
    );
    
    let service_clone = service.clone();
    
    // Use policy with a custom retry loop
    let mut attempt = 0;
    let max_attempts = 5;
    
    loop {
        match service_clone.execute().await {
            Ok(result) => {
                info!("Standard policy retry succeeded: {}", result);
                break;
            }
            Err(err) => {
                attempt += 1;
                
                // Convert to ResilienceError for policy evaluation
                let resilience_err = ResilienceError::General(err.to_string());
                
                if attempt >= max_attempts || !policy.should_retry(attempt, &resilience_err) {
                    error!("Standard policy retry failed after {} attempts: {}", attempt, resilience_err);
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Standard policy retry example failed: {}", resilience_err)
                    )));
                }
                
                let delay = policy.backoff_duration(attempt);
                warn!("Attempt {} failed, retrying after {:?}", attempt, delay);
                sleep(delay).await;
            }
        }
    }
    
    Ok(())
}

/// Run all retry examples
pub async fn run_retry_example() -> Result<(), Box<dyn StdError + Send + Sync>> {
    info!("Running retry examples");
    
    // Run basic retry example
    demonstrate_basic_retry().await?;
    
    // Run exponential backoff example
    demonstrate_exponential_backoff().await?;
    
    // Run predicate retry example
    demonstrate_predicate_retry().await?;
    
    // Run timeout retry example
    demonstrate_timeout_retry().await?;
    
    // Run custom retry policy example
    demonstrate_custom_retry_policy().await?;
    
    // Run standard retry policy example
    demonstrate_standard_retry_policy().await?;
    
    info!("All retry examples completed successfully");
    Ok(())
} 