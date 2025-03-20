#[allow(unused_imports)]
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::error::Result;
use serde::{Serialize, Deserialize};
use std::fmt::{self, Debug};
use crate::monitoring::MonitoringConfig;

/// Configuration for performance monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Interval in seconds between performance stats updates
    pub interval: u64,
    /// Maximum samples to keep for each metric
    pub max_samples: usize,
    /// Whether to track CPU usage
    pub track_cpu: bool,
    /// Whether to track memory usage
    pub track_memory: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            interval: 60,
            max_samples: 100,
            track_cpu: true,
            track_memory: true,
        }
    }
}

/// Performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    /// Timestamp of the measurement
    pub timestamp: u64,
    /// CPU usage percentage (0-100)
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Available memory in bytes
    pub memory_available: u64,
    /// Number of open file handles
    pub open_files: u64,
    /// Number of active threads
    pub active_threads: u64,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            cpu_usage: 0.0,
            memory_usage: 0,
            memory_available: 0,
            open_files: 0,
            active_threads: 0,
        }
    }
}

/// Operation timing data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationTiming {
    /// Operation name
    pub name: String,
    /// Operation duration in milliseconds
    pub duration_ms: f64,
    /// Timestamp when the operation was recorded
    pub timestamp: u64,
    /// Success flag
    pub success: bool,
    /// Additional context
    pub context: HashMap<String, String>,
}

/// Performance monitoring interface
#[async_trait]
pub trait PerformanceMonitorTrait: Send + Sync + std::fmt::Debug {
    /// Get the current performance statistics
    async fn get_stats(&self) -> Result<PerformanceStats>;
    
    /// Get performance statistics history
    async fn get_stats_history(&self, limit: Option<usize>) -> Result<Vec<PerformanceStats>>;
    
    /// Record an operation timing
    async fn record_timing(&self, operation: &str, duration_ms: f64, success: bool) -> Result<()>;
    
    /// Record an operation timing with context
    async fn record_timing_with_context(
        &self, 
        operation: &str, 
        duration_ms: f64, 
        success: bool, 
        context: HashMap<String, String>
    ) -> Result<()>;
    
    /// Get operation timings
    async fn get_timings(&self, operation: Option<&str>, limit: Option<usize>) -> Result<Vec<OperationTiming>>;
    
    /// Start the performance monitor
    async fn start(&self) -> Result<()>;
    
    /// Stop the performance monitor
    async fn stop(&self) -> Result<()>;
    
    /// Create a timer for measuring operation duration
    fn create_timer(&self, operation: &str) -> PerformanceTimer;
}

/// Factory for creating performance monitor instances
pub trait PerformanceMonitorFactory: Send + Sync {
    /// Create a new performance monitor
    fn create_monitor(&self) -> Arc<dyn PerformanceMonitorTrait>;
}

/// Timer for measuring operation duration
pub struct PerformanceTimer {
    /// Operation name
    operation: String,
    /// Start time
    start: Instant,
    /// Monitor reference
    monitor: Option<Arc<dyn PerformanceMonitorTrait>>,
}

impl PerformanceTimer {
    /// Create a new timer for the given operation
    #[must_use]
    pub fn new(operation: &str, monitor: Option<Arc<dyn PerformanceMonitorTrait>>) -> Self {
        Self {
            operation: operation.to_string(),
            start: Instant::now(),
            monitor,
        }
    }
    
    /// Stop the timer and record the duration
    /// 
    /// # Errors
    /// 
    /// Returns an error if the performance monitor fails to record the timing.
    pub async fn stop(self, success: bool) -> Result<f64> {
        let duration = self.start.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;
        
        if let Some(monitor) = self.monitor {
            monitor.record_timing(&self.operation, duration_ms, success).await?;
        }
        
        Ok(duration_ms)
    }
    
    /// Stop the timer and record the duration with context
    /// 
    /// # Errors
    /// 
    /// Returns an error if the performance monitor fails to record the timing with context.
    pub async fn stop_with_context(self, success: bool, context: HashMap<String, String>) -> Result<f64> {
        let duration = self.start.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;
        
        if let Some(monitor) = self.monitor {
            monitor.record_timing_with_context(&self.operation, duration_ms, success, context).await?;
        }
        
        Ok(duration_ms)
    }
    
    /// Get the elapsed time without stopping the timer
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
    
    /// Get the elapsed time in milliseconds without stopping the timer
    #[must_use]
    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
}

/// Performance monitor implementation
#[derive(Debug)]
#[allow(dead_code)]
pub struct PerformanceMonitorImpl {
    /// Configuration
    config: MonitoringConfig,
}

impl PerformanceMonitorImpl {
    /// Create a new performance monitor
    #[must_use]
    pub fn new(config: MonitoringConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl PerformanceMonitorTrait for PerformanceMonitorImpl {
    async fn start(&self) -> Result<()> {
        // Implementation
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        // Implementation
        Ok(())
    }
    
    async fn get_stats(&self) -> Result<PerformanceStats> {
        // Implementation needed
        Ok(PerformanceStats::default())
    }
    
    async fn get_stats_history(&self, _limit: Option<usize>) -> Result<Vec<PerformanceStats>> {
        // Implementation needed
        Ok(Vec::new())
    }
    
    async fn record_timing(&self, _operation: &str, _duration_ms: f64, _success: bool) -> Result<()> {
        // Implementation needed
        Ok(())
    }
    
    async fn record_timing_with_context(
        &self,
        _operation: &str,
        _duration_ms: f64,
        _success: bool,
        _context: HashMap<String, String>,
    ) -> Result<()> {
        // Implementation needed
        Ok(())
    }
    
    async fn get_timings(&self, _operation: Option<&str>, _limit: Option<usize>) -> Result<Vec<OperationTiming>> {
        // Implementation needed
        Ok(Vec::new())
    }
    
    fn create_timer(&self, operation: &str) -> PerformanceTimer {
        PerformanceTimer::new(operation, None)
    }
}

#[cfg(test)]
#[allow(unused_imports, missing_docs)]
pub mod tests {
    use super::*;
    use tokio::time::Duration as TokioDuration;
    
    /// Test performance monitor
    #[derive(Debug)]
    pub struct TestPerformanceMonitor {
        /// Started flag
        pub started: bool,
        /// Stopped flag
        pub stopped: bool,
    }
    
    impl TestPerformanceMonitor {
        /// Creates a new test performance monitor.
        /// 
        /// # Returns
        /// 
        /// A new instance of `TestPerformanceMonitor`.
        #[must_use]
        pub fn new() -> Self {
            Self {
                started: false,
                stopped: false,
            }
        }

        /// Starts the performance monitor.
        /// 
        /// # Errors
        /// 
        /// This implementation never returns an error.
        pub fn start(&mut self) -> Result<()> {
            self.started = true;
            Ok(())
        }

        /// Stops the performance monitor.
        /// 
        /// # Errors
        /// 
        /// This implementation never returns an error.
        pub fn stop(&mut self) -> Result<()> {
            self.stopped = true;
            Ok(())
        }
    }

    impl Default for TestPerformanceMonitor {
        fn default() -> Self {
            Self::new()
        }
    }
} 