use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use crate::error::Result;
use chrono::Utc;
use std::sync::Weak;

/// Performance category for grouping metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PerfCategory {
    /// Plugin operations
    Plugin,
    /// Command operations
    Command,
    /// Context operations
    Context,
    /// File operations
    File,
    /// Network operations
    Network,
    /// UI operations
    UI,
    /// General operations
    General,
}

impl std::fmt::Display for PerfCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plugin => write!(f, "plugin"),
            Self::Command => write!(f, "command"),
            Self::Context => write!(f, "context"),
            Self::File => write!(f, "file"),
            Self::Network => write!(f, "network"),
            Self::UI => write!(f, "ui"),
            Self::General => write!(f, "general"),
        }
    }
}

/// Performance metric that tracks timing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfMetric {
    /// Name of the metric
    pub name: String,
    /// Category of the metric
    pub category: PerfCategory,
    /// Number of samples
    pub count: u64,
    /// Total duration in microseconds
    pub total_us: u64,
    /// Minimum duration in microseconds
    pub min_us: u64,
    /// Maximum duration in microseconds
    pub max_us: u64,
    /// Last sample value in microseconds
    pub last_value: u64,
    /// Timestamp of the last update
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl PerfMetric {
    /// Create a new performance metric
    #[must_use] pub fn new(name: String, category: PerfCategory) -> Self {
        Self {
            name,
            category,
            count: 0,
            total_us: 0,
            min_us: u64::MAX,
            max_us: 0,
            last_value: 0,
            timestamp: Utc::now(),
        }
    }

    /// Add a sample to the metric
    pub fn add_sample(&mut self, duration_us: u64) {
        self.count += 1;
        self.total_us += duration_us;
        self.min_us = self.min_us.min(duration_us);
        self.max_us = self.max_us.max(duration_us);
        self.last_value = duration_us;
        self.timestamp = Utc::now();
    }

    /// Calculate the average duration in microseconds
    #[must_use] pub fn avg_us(&self) -> u64 {
        if self.count == 0 {
            0
        } else {
            self.total_us / self.count
        }
    }
}

/// Memory usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct MemoryUsage {
    /// Current memory usage in bytes
    pub current_bytes: u64,
    /// Peak memory usage in bytes
    pub peak_bytes: u64,
    /// Total allocated memory in bytes
    pub allocated_bytes: u64,
}


/// Performance monitor implementation
#[derive(Debug)]
pub struct PerfMonitor {
    /// Whether performance monitoring is enabled
    enabled: RwLock<bool>,
    /// Collected metrics
    metrics: RwLock<HashMap<String, PerfMetric>>,
    /// Memory usage tracking
    memory: RwLock<MemoryUsage>,
    /// Self-reference for callbacks and timers
    arc_self: Weak<Self>,
}

impl PerfMonitor {
    /// Create a new performance monitor
    #[must_use] pub fn new() -> Arc<Self> {
        // We need to use this approach to create a self-referential Arc
        
        
        Arc::new_cyclic(|weak| {
            PerfMonitor {
                enabled: RwLock::new(true),
                metrics: RwLock::new(HashMap::new()),
                memory: RwLock::new(MemoryUsage::default()),
                arc_self: weak.clone(),
            }
        })
    }

    /// Get a strong reference to self
    fn get_self_ref(&self) -> Option<Arc<Self>> {
        self.arc_self.upgrade()
    }

    /// Enable or disable metrics collection
    /// 
    /// # Errors
    /// 
    /// Returns an error if the lock for enabling status cannot be acquired
    pub async fn set_enabled(&self, enabled: bool) -> Result<()> {
        let mut lock = self.enabled.write().await;
        *lock = enabled;
        Ok(())
    }

    /// Check if metrics collection is enabled
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }

    /// Start timing an operation
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the operation to time
    /// * `category` - The category of the operation
    /// 
    /// # Returns
    /// 
    /// A `TimingGuard` that will record the duration when dropped
    /// 
    /// # Panics
    /// 
    /// This function may panic if internal weak references cannot be upgraded to strong references,
    /// which should only happen if the `PerfMonitor` has been dropped while timing is in progress.
    pub async fn time(&self, name: &str, category: PerfCategory) -> TimingGuard {
        if !*self.enabled.read().await {
            return TimingGuard::disabled();
        }
        
        // Get a strong reference to self or return a disabled guard if not possible
        match self.get_self_ref() {
            Some(strong_ref) => TimingGuard::new(Arc::clone(&strong_ref), name.to_string(), category),
            None => {
                // This should only happen if the PerfMonitor is being dropped while timing is in progress
                // Instead of panicking, return a disabled guard
                TimingGuard::disabled()
            }
        }
    }

    /// Record a timing for an operation
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the operation
    /// * `category` - The category of the operation
    /// * `duration_us` - The duration in microseconds
    /// 
    /// # Returns
    /// 
    /// Returns Ok(()) if the timing was recorded successfully
    /// 
    /// # Errors
    /// 
    /// Returns an error if recording the timing fails, such as when the lock cannot be acquired
    pub async fn record_timing(&self, name: &str, category: PerfCategory, duration_us: u64) -> Result<()> {
        if !*self.enabled.read().await {
            return Ok(());
        }
        
        let mut metrics = self.metrics.write().await;
        let metric = metrics
            .entry(format!("{category}.{name}"))
            .or_insert_with(|| PerfMetric::new(name.to_string(), category));
        
        metric.add_sample(duration_us);
        
        Ok(())
    }

    /// Updates memory usage metrics
    /// 
    /// # Errors
    /// Returns an error if updating metrics fails
    pub async fn update_memory(&self, current_bytes: u64, allocated_bytes: u64) -> Result<()> {
        let mut memory = self.memory.write().await;
        memory.current_bytes = current_bytes;
        memory.allocated_bytes = allocated_bytes;
        
        if current_bytes > memory.peak_bytes {
            memory.peak_bytes = current_bytes;
        }
        
        Ok(())
    }

    /// Gets memory usage metrics
    /// 
    /// # Errors
    /// Returns an error if getting metrics fails
    pub async fn get_memory(&self) -> Result<MemoryUsage> {
        Ok(self.memory.read().await.clone())
    }

    /// Gets all metrics
    /// 
    /// # Errors
    /// Returns an error if getting metrics fails
    pub async fn get_metrics(&self) -> Result<HashMap<String, PerfMetric>> {
        Ok(self.metrics.read().await.clone())
    }

    /// Gets a specific metric by name and category
    /// 
    /// # Errors
    /// Returns an error if getting the metric fails
    pub async fn get_metric(&self, name: &str, category: PerfCategory) -> Result<Option<PerfMetric>> {
        let full_name = format!("{category}.{name}");
        let metrics = self.metrics.read().await;
        Ok(metrics.get(&full_name).cloned())
    }

    /// Resets all metrics
    /// 
    /// # Errors
    /// Returns an error if resetting metrics fails
    pub async fn reset(&self) -> Result<()> {
        // Reset metrics
        let mut metrics = self.metrics.write().await;
        metrics.clear();
        
        // Reset memory usage
        let mut memory = self.memory.write().await;
        *memory = MemoryUsage::default();
        
        Ok(())
    }

    /// Generates a performance report
    /// 
    /// # Errors
    /// Returns an error if generating the report fails
    pub async fn generate_report(&self) -> Result<PerfReport> {
        let metrics = self.get_metrics().await?;
        let memory = self.get_memory().await?;
        
        Ok(PerfReport {
            metrics: metrics.into_values().collect(),
            memory,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Creates a disabled instance of `PerfMonitor`
    /// 
    /// Returns a `PerfMonitor` that doesn't track metrics
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            enabled: RwLock::new(false),
            metrics: RwLock::new(HashMap::new()),
            memory: RwLock::new(MemoryUsage::default()),
            arc_self: Weak::new(),
        }
    }
}

/// Timing guard to automatically record durations when dropped
pub struct TimingGuard {
    /// Performance monitor to record timing
    monitor: Option<Arc<PerfMonitor>>,
    /// Name of the metric
    name: String,
    /// Start time
    start: Instant,
    /// Whether timing is enabled
    enabled: bool,
    /// Performance category
    category: PerfCategory,
}

impl TimingGuard {
    /// Create a new timing guard
    pub fn new(monitor: Arc<PerfMonitor>, name: String, category: PerfCategory) -> Self {
        Self {
            monitor: Some(monitor),
            name,
            start: Instant::now(),
            enabled: true,
            category,
        }
    }

    /// Create a disabled timing guard that doesn't record anything
    #[must_use] pub fn disabled() -> Self {
        Self {
            monitor: None,
            name: String::new(),
            start: Instant::now(),
            enabled: false,
            category: PerfCategory::General,
        }
    }

    /// Cancel this timing
    pub fn cancel(&mut self) {
        self.enabled = false;
    }
}

impl Drop for TimingGuard {
    fn drop(&mut self) {
        if !self.enabled {
            return;
        }

        if let Some(monitor) = self.monitor.take() {
            let duration = self.start.elapsed();
            
            // Convert from u128 to u64 safely, with a fallback for extremely large values
            // This is unlikely to ever be reached, as it would require a timing over 584 years
            let duration_us = if let Ok(us) = u64::try_from(duration.as_micros()) {
                us
            } else {
                tracing::warn!("Timing duration exceeds u64 capacity, capping at u64::MAX");
                u64::MAX
            };
            
            let name = self.name.clone();
            let category = self.category;

            tokio::spawn(async move {
                if let Err(e) = monitor.record_timing(&name, category, duration_us).await {
                    tracing::error!("Failed to record timing: {}", e);
                }
            });
        }
    }
}

/// Performance report containing metrics and memory usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfReport {
    /// List of performance metrics
    pub metrics: Vec<PerfMetric>,
    /// Memory usage information
    pub memory: MemoryUsage,
    /// Timestamp of the report
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl PerfReport {
    /// Saves the performance report to a file
    /// 
    /// # Errors
    /// Returns an error if saving to file fails
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<()> {
        let file = std::fs::File::create(path)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }

    /// Loads a performance report from a file
    /// 
    /// # Errors
    /// Returns an error if loading from file fails
    pub fn load_from_file(path: &std::path::Path) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        let report = serde_json::from_reader(file)?;
        Ok(report)
    }
}

/// Middleware for timing HTTP requests
pub struct TimingMiddleware {
    /// Performance monitor
    monitor: Arc<PerfMonitor>,
}

impl TimingMiddleware {
    /// Create a new timing middleware
    pub fn new(monitor: Arc<PerfMonitor>) -> Self {
        Self { monitor }
    }
    
    /// Times a request and returns its result
    /// 
    /// # Errors
    /// Returns an error if the request handler fails or if timing fails
    pub async fn time_request<F, T>(&self, path: &str, handler: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        let guard = self.monitor.time(&format!("http.{path}"), PerfCategory::Network).await;
        let result = handler()?;
        drop(guard);
        Ok(result)
    }
}

/// Track memory usage over time
pub struct MemoryTracker {
    /// Performance monitor
    monitor: Arc<PerfMonitor>,
    /// Update interval
    interval: Duration,
    /// Whether the tracker is running
    running: Arc<RwLock<bool>>,
}

impl MemoryTracker {
    /// Create a new memory tracker
    pub fn new(monitor: Arc<PerfMonitor>, interval: Duration) -> Self {
        Self {
            monitor,
            interval,
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Starts performance monitoring
    /// 
    /// # Errors
    /// Returns an error if starting monitoring fails
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        
        *running = true;
        
        let monitor = Arc::clone(&self.monitor);
        let interval = self.interval;
        let running_clone = Arc::clone(&self.running);
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                // Check if we should stop
                if !*running_clone.read().await {
                    break;
                }
                
                // Get current memory usage
                // This is a placeholder - in a real implementation, we would use a memory profiling library
                // or system APIs to get actual memory usage. For now we'll use dummy values.
                let current_bytes = 1024 * 1024; // 1 MB
                let allocated_bytes = 2 * 1024 * 1024; // 2 MB
                
                if let Err(e) = monitor.update_memory(current_bytes, allocated_bytes).await {
                    tracing::error!("Failed to update memory usage: {}", e);
                }
            }
            
            tracing::debug!("Memory tracker stopped");
        });
        
        Ok(())
    }
    
    /// Stops performance monitoring
    /// 
    /// # Errors
    /// Returns an error if stopping monitoring fails
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_perf_metric() {
        let mut metric = PerfMetric::new("test".to_string(), PerfCategory::General);
        
        // Add samples
        metric.add_sample(100);
        metric.add_sample(200);
        metric.add_sample(300);
        
        // Check stats
        assert_eq!(metric.count, 3);
        assert_eq!(metric.total_us, 600);
        assert_eq!(metric.min_us, 100);
        assert_eq!(metric.max_us, 300);
        assert_eq!(metric.avg_us(), 200);
    }

    #[tokio::test]
    async fn test_perf_monitor() {
        let monitor = PerfMonitor::new();
        
        // Test timing
        {
            let guard = monitor.time("test", PerfCategory::General).await;
            tokio::time::sleep(Duration::from_millis(10)).await;
            drop(guard);
        }
        
        // Give the async task time to complete after the guard is dropped
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Test direct recording
        monitor.record_timing("direct", PerfCategory::Command, 50_000).await.unwrap();
        
        let metrics = monitor.get_metrics().await.unwrap();
        assert!(metrics.contains_key("general.test"));
        assert!(metrics.contains_key("command.direct"));
        
        let report = monitor.generate_report().await.unwrap();
        assert_eq!(report.metrics.len(), 2);
        
        // Test disabling
        let _ = monitor.set_enabled(false).await;
        monitor.record_timing("disabled", PerfCategory::Context, 100_000).await.unwrap();
        
        let metrics = monitor.get_metrics().await.unwrap();
        assert!(!metrics.contains_key("context.disabled"));
        
        // Test re-enabling
        let _ = monitor.set_enabled(true).await;
        monitor.record_timing("reenabled", PerfCategory::Context, 100_000).await.unwrap();
        
        let metrics = monitor.get_metrics().await.unwrap();
        assert!(metrics.contains_key("context.reenabled"));
    }
} 