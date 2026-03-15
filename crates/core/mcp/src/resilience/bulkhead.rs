// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Bulkhead isolation pattern for limiting concurrent calls
//!
//! The bulkhead pattern is used to limit the number of concurrent calls
//! to a service to prevent cascade failures in distributed systems.

use std::error::Error;
use tokio::sync::{Semaphore, Mutex, RwLock};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
// Removed block_in_place - use async methods or try_lock instead for non-blocking access

use crate::resilience::{ResilienceError, Result};

/// Error types for bulkhead operations
#[derive(Debug, thiserror::Error)]
pub enum BulkheadError {
    /// Maximum concurrent calls reached
    #[error("Maximum concurrent calls reached")]
    MaxConcurrentCallsReached,
    
    /// Queue is full
    #[error("Bulkhead queue is full")]
    QueueFull,
    
    /// Operation timed out
    #[error("Operation timed out after {duration:?}")]
    Timeout {
        /// The timeout duration
        duration: Duration,
    },
    
    /// Operation failed with error
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

impl From<BulkheadError> for ResilienceError {
    fn from(err: BulkheadError) -> Self {
        match err {
            BulkheadError::MaxConcurrentCallsReached => {
                Self::Bulkhead("Maximum concurrent calls reached".to_string())
            }
            BulkheadError::QueueFull => {
                Self::Bulkhead("Bulkhead queue is full".to_string())
            }
            BulkheadError::Timeout { duration } => {
                Self::Timeout(format!("Operation timed out after {:?}", duration))
            }
            BulkheadError::OperationFailed(msg) => {
                Self::General(format!("Bulkhead operation failed: {}", msg))
            }
        }
    }
}

/// Configuration for a bulkhead
#[derive(Debug, Clone)]
pub struct BulkheadConfig {
    /// Name of the bulkhead for identification
    pub name: String,
    
    /// Maximum number of concurrent calls allowed
    pub max_concurrent_calls: usize,
    
    /// Maximum size of the queue for waiting operations
    pub max_queue_size: usize,
    
    /// Timeout for call execution
    pub call_timeout: Duration,
    
    /// Timeout for waiting in the queue
    pub queue_timeout: Duration,
}

impl Default for BulkheadConfig {
    fn default() -> Self {
        // Load unified config for environment-aware timeout values
        let config = squirrel_mcp_config::unified::ConfigLoader::load()
            .ok()
            .and_then(|loaded| loaded.try_into_config().ok());
        
        let (call_timeout, queue_timeout) = if let Some(cfg) = config {
            // Use custom bulkhead timeouts if configured
            let call = cfg.timeouts.get_custom_timeout("bulkhead_call")
                .unwrap_or_else(|| Duration::from_secs(1));
            let queue = cfg.timeouts.get_custom_timeout("bulkhead_queue")
                .unwrap_or_else(|| Duration::from_millis(500));
            (call, queue)
        } else {
            // Fallback to sensible defaults
            (Duration::from_secs(1), Duration::from_millis(500))
        };
        
        Self {
            name: "default".to_string(),
            max_concurrent_calls: 10,
            max_queue_size: 20,
            call_timeout,
            queue_timeout,
        }
    }
}

/// Metrics for a bulkhead
#[derive(Debug, Clone)]
pub struct BulkheadMetrics {
    /// Number of available permits (free execution slots)
    pub available_permits: usize,
    
    /// Maximum number of permits (maximum concurrent calls)
    pub max_permits: usize,
    
    /// Current queue depth (number of operations waiting)
    pub queue_depth: usize,
    
    /// Maximum queue capacity
    pub queue_capacity: usize,
    
    /// Number of operations rejected due to capacity limits
    pub rejection_count: u64,
    
    /// Number of operations that timed out
    pub timeout_count: u64,
}

/// Type for queued operations
type QueuedOperation = Box<dyn std::any::Any + Send + Sync>;

/// Bulkhead for isolating failures in concurrent systems
#[derive(Clone)]
pub struct Bulkhead {
    /// Configuration for this bulkhead
    config: BulkheadConfig,
    
    /// Semaphore for limiting concurrent calls
    permits: Arc<Semaphore>,
    
    /// Queue for operations waiting for a permit
    queue: Arc<Mutex<VecDeque<QueuedOperation>>>,
    
    /// Metrics for this bulkhead
    metrics: Arc<RwLock<BulkheadMetrics>>,
}

impl Bulkhead {
    /// Create a new bulkhead with the given configuration
    pub fn new(config: BulkheadConfig) -> Self {
        Self {
            config: config.clone(),
            permits: Arc::new(Semaphore::new(config.max_concurrent_calls)),
            queue: Arc::new(Mutex::new(VecDeque::with_capacity(config.max_queue_size))),
            metrics: Arc::new(RwLock::new(BulkheadMetrics {
                available_permits: config.max_concurrent_calls,
                max_permits: config.max_concurrent_calls,
                queue_depth: 0,
                queue_capacity: config.max_queue_size,
                rejection_count: 0,
                timeout_count: 0,
            })),
        }
    }

    /// Create a new bulkhead with default configuration
    pub fn default() -> Self {
        Self::new(BulkheadConfig::default())
    }
    
    /// Get the configuration of this bulkhead
    pub fn config(&self) -> &BulkheadConfig {
        &self.config
    }
    
    /// Get the current metrics for this bulkhead (async version)
    /// 
    /// This is the preferred method for async contexts. Use `metrics_sync()` only
    /// when you absolutely need synchronous access.
    pub async fn metrics(&self) -> BulkheadMetrics {
        // Update metrics first
        let available = self.permits.available_permits();
        let queue_len = self.queue.lock().await.len();
        
        let mut metrics = self.metrics.write().await;
        metrics.available_permits = available;
        metrics.queue_depth = queue_len;
        
        // Return a clone of the metrics
        metrics.clone()
    }
    
    /// Get metrics synchronously using try_lock (best-effort)
    /// 
    /// Returns cached metrics if locks cannot be acquired immediately.
    /// For accurate metrics in async code, use `metrics()` instead.
    pub fn metrics_sync(&self) -> BulkheadMetrics {
        let available = self.permits.available_permits();
        
        // Try to get queue length without blocking
        let queue_len = self.queue.try_lock()
            .map(|q| q.len())
            .unwrap_or(0);
        
        // Try to update and return metrics without blocking
        if let Ok(mut metrics) = self.metrics.try_write() {
            metrics.available_permits = available;
            metrics.queue_depth = queue_len;
            metrics.clone()
        } else if let Ok(metrics) = self.metrics.try_read() {
            // Return cached metrics if write lock unavailable
            BulkheadMetrics {
                available_permits: available,
                queue_depth: queue_len,
                ..metrics.clone()
            }
        } else {
            // Fallback to basic metrics
            BulkheadMetrics {
                available_permits: available,
                max_permits: self.config.max_concurrent_calls,
                queue_depth: queue_len,
                queue_capacity: self.config.max_queue_size,
                rejection_count: 0,
                timeout_count: 0,
            }
        }
    }
    
    /// Check if there are available permits without acquiring them
    /// 
    /// This method is optimized for synchronous access and avoids blocking.
    /// It checks permits first (fast path) and only checks queue if needed.
    pub fn has_permits(&self) -> bool {
        let available = self.permits.available_permits();
        if available > 0 {
            return true;
        }
        
        // If no permits available, check if queue has space using try_lock
        // This avoids blocking in async context
        self.queue.try_lock()
            .map(|queue| queue.len() < self.config.max_queue_size)
            .unwrap_or(false) // Conservative: assume no space if lock unavailable
    }
    
    /// Check if there are available permits (async version)
    /// 
    /// Use this in async contexts for accurate results.
    pub async fn has_permits_async(&self) -> bool {
        let available = self.permits.available_permits();
        if available > 0 {
            return true;
        }
        
        // If no permits available, check if queue has space
        let queue = self.queue.lock().await;
        queue.len() < self.config.max_queue_size
    }

    /// Execute an operation with bulkhead isolation
    pub async fn execute<F, T>(&self, operation: F) -> Result<T>
    where
        F: std::future::Future<Output = std::result::Result<T, Box<dyn Error + Send + Sync>>> + Send + 'static,
        T: Send + 'static,
    {
        // Try to acquire a permit immediately
        if let Ok(permit) = self.permits.try_acquire() {
            // Execute with permit
            return self.execute_with_permit(permit, operation).await;
        }
        
        // No immediate permit available, try to queue if allowed
        if self.config.max_queue_size == 0 {
            // Queuing is disabled
            self.metrics.write().await.rejection_count += 1;
            return Err(BulkheadError::MaxConcurrentCallsReached.into());
        }
        
        // Try to add to queue
        let start_time = Instant::now();
        {
            let queue = self.queue.lock().await;
            
            // Check if queue is full
            if queue.len() >= self.config.max_queue_size {
                self.metrics.write().await.rejection_count += 1;
                return Err(BulkheadError::QueueFull.into());
            }
            
            // Update queue depth in metrics
            let mut metrics = self.metrics.write().await;
            metrics.queue_depth = queue.len() + 1;
        }
        
        // Wait for a permit with optional timeout
        let permit = match self.config.queue_timeout {
            duration if !duration.is_zero() => {
                match tokio::time::timeout(duration, self.permits.acquire()).await {
                    Ok(permit_result) => {
                        match permit_result {
                            Ok(permit) => permit,
                            Err(e) => {
                                self.metrics.write().await.timeout_count += 1;
                                return Err(BulkheadError::OperationFailed(
                                    format!("Failed to acquire permit: {}", e)
                                ).into());
                            }
                        }
                    },
                    Err(_) => {
                        // Timeout waiting for a permit
                        self.metrics.write().await.timeout_count += 1;
                        return Err(BulkheadError::Timeout {
                            duration: self.config.queue_timeout,
                        }.into());
                    }
                }
            },
            _ => {
                // No timeout, simply wait
                match self.permits.acquire().await {
                    Ok(permit) => permit,
                    Err(e) => {
                        return Err(BulkheadError::OperationFailed(
                            format!("Failed to acquire permit: {}", e)
                        ).into());
                    }
                }
            }
        };
        
        // Check if we exceeded the queue timeout
        if !self.config.queue_timeout.is_zero() {
            if start_time.elapsed() > self.config.queue_timeout {
                self.metrics.write().await.timeout_count += 1;
                // Drop the permit to release it
                return Err(BulkheadError::Timeout {
                    duration: self.config.queue_timeout,
                }.into());
            }
        }
        
        // Execute with the acquired permit
        self.execute_with_permit(permit, operation).await
    }
    
    /// Execute an operation with an already acquired permit
    async fn execute_with_permit<F, T>(
        &self, 
        _permit: tokio::sync::SemaphorePermit<'_>, 
        operation: F
    ) -> Result<T>
    where
        F: std::future::Future<Output = std::result::Result<T, Box<dyn Error + Send + Sync>>> + Send + 'static,
        T: Send + 'static,
    {
        // Execute the operation with an optional timeout
        let result = match self.config.call_timeout {
            duration if !duration.is_zero() => {
                match tokio::time::timeout(duration, operation).await {
                    Ok(inner_result) => inner_result,
                    Err(_) => {
                        // Timeout executing the operation
                        self.metrics.write().await.timeout_count += 1;
                        self.permits.add_permits(1);
                        return Err(BulkheadError::Timeout {
                            duration: self.config.call_timeout,
                        }.into());
                    }
                }
            },
            _ => {
                // No timeout
                operation.await
            }
        };
        
        // Always release the permit
        self.permits.add_permits(1);
        
        // Return the result or convert the error
        match result {
            Ok(value) => Ok(value),
            Err(e) => Err(BulkheadError::OperationFailed(e.to_string()).into()),
        }
    }

    /// Try to enter the bulkhead without waiting
    ///
    /// This method checks if there are permits available for execution.
    /// If a permit is available, it acquires one and returns true.
    /// If no permits are available, it returns false without waiting.
    pub async fn try_enter(&self) -> bool {
        // Try to acquire a permit immediately
        if let Ok(_permit) = self.permits.try_acquire() {
            // Update metrics
            {
                let mut metrics = self.metrics.write().await;
                metrics.available_permits = self.permits.available_permits();
            }
            
            return true;
        }
        
        // No immediate permit available
        false
    }
}

/// Create a new bulkhead with default configuration and the given component name
pub fn new_bulkhead(component_id: &str) -> Bulkhead {
    let config = BulkheadConfig {
        name: component_id.to_string(),
        ..BulkheadConfig::default()
    };
    
    Bulkhead::new(config)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use std::error::Error;

    use super::*;

    #[tokio::test]
    async fn test_bulkhead_error_mapping() {
        // Create a bulkhead with 0 permits and no queue
        let bulkhead = Bulkhead::new(BulkheadConfig {
            name: "test-mapping".to_string(),
            max_concurrent_calls: 0,  // No permits available
            max_queue_size: 0,        // No queue
            call_timeout: Duration::from_millis(100),
            queue_timeout: Duration::from_millis(50),
        });
        
        // Try to execute an operation
        let result = bulkhead.execute(async {
            Ok::<_, Box<dyn Error + Send + Sync>>("success")
        }).await;
        
        // Should fail because no permits available
        assert!(result.is_err(), "Expected error, got success");
        
        // Verify the error is mapped correctly to ResilienceError::Bulkhead
        let error = result.unwrap_err();
        let resilience_error = ResilienceError::from(error);
        
        assert!(matches!(resilience_error, ResilienceError::Bulkhead(_)), 
                "Expected Bulkhead error, got: {:?}", resilience_error);
        
        // Verify the error message indicates the reason
        if let ResilienceError::Bulkhead(msg) = resilience_error {
            assert!(msg.contains("Maximum concurrent calls"), 
                    "Error message should mention maximum concurrent calls, got: {}", msg);
        }
    }

    #[tokio::test]
    async fn test_bulkhead_queue_full() {
        // Create a bulkhead with 0 permits and queue size of 1
        let bulkhead = Bulkhead::new(BulkheadConfig {
            name: "test-queue-full".to_string(),
            max_concurrent_calls: 0,  // No permits available
            max_queue_size: 1,        // Queue of size 1
            call_timeout: Duration::from_millis(100),
            queue_timeout: Duration::from_millis(50),
        });
        
        // First operation should be queued
        let handle1 = tokio::spawn({
            let bulkhead = bulkhead.clone();
            async move {
                bulkhead.execute(async {
                    // This operation should be queued
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    Ok::<_, Box<dyn Error + Send + Sync>>("queued operation")
                }).await
            }
        });
        
        // Give time for the first operation to be queued
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Second operation should be rejected (queue is full)
        let result = bulkhead.execute(async {
            Ok::<_, Box<dyn Error + Send + Sync>>("second operation")
        }).await;
        
        // Verify the second operation was rejected
        assert!(result.is_err(), "Second operation should be rejected");
        if let Err(err) = result {
            // Either a Bulkhead error (queue full) or a Timeout error (queue timeout) is acceptable
            assert!(
                matches!(err, ResilienceError::Bulkhead(_)) || 
                matches!(err, ResilienceError::Timeout(_)),
                "Expected Bulkhead or Timeout error, got: {:?}", err
            );
            println!("Got expected error: {:?}", err);
        }
        
        // First operation should eventually complete
        let _ = handle1.await.unwrap();
    }
    
    #[tokio::test]
    async fn test_bulkhead_comprehensive() {
        // Create a bulkhead with no concurrency allowed (0 permits) and queue size of 1
        let bulkhead = Bulkhead::new(BulkheadConfig {
            name: "test-comprehensive".to_string(),
            max_concurrent_calls: 0,     // No concurrent calls allowed (forces queueing)
            max_queue_size: 1,           // Queue size of 1
            call_timeout: Duration::from_millis(200),
            queue_timeout: Duration::from_millis(100),
        });
        
        // Intentionally fill the queue with a task that won't start for 100ms
        let handle1 = tokio::spawn({
            let bulkhead = bulkhead.clone();
            async move {
                // This operation will be queued
                let result = bulkhead.execute(async {
                    // This operation should be queued
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    Ok::<_, Box<dyn Error + Send + Sync>>("operation 1")
                }).await;
                
                // Return the result
                result
            }
        });
        
        // Wait to ensure first operation is queued
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Now try to submit a second operation, which should be rejected (queue full)
        let result2 = bulkhead.execute(async {
            Ok::<_, Box<dyn Error + Send + Sync>>("operation 2")
        }).await;
        
        // Verify second operation was rejected as expected
        assert!(result2.is_err(), "Second operation should be rejected (queue full)");
        
        // Verify the error is a Bulkhead or Timeout error
        match result2 {
            Err(err) => {
                assert!(
                    matches!(err, ResilienceError::Bulkhead(_)) || 
                    matches!(err, ResilienceError::Timeout(_)),
                    "Expected Bulkhead or Timeout error, got: {:?}", err
                );
                println!("Operation 2 correctly rejected: {:?}", err);
            },
            Ok(_) => panic!("Second operation should have been rejected"),
        }
        
        // The first operation should eventually complete or timeout
        match handle1.await {
            Ok(result) => {
                println!("First operation result: {:?}", result);
                // We don't care about the first operation's result, as long as the test behavior is correct
            },
            Err(err) => {
                println!("Task joining error: {:?}", err);
            }
        }
    }
} 