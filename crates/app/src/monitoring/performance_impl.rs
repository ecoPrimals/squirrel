use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use crate::app::monitoring::performance::{
    PerformanceConfig, PerformanceMonitorFactory, PerformanceMonitorTrait,
    PerformanceStats, PerformanceTimer, OperationTiming
};
use crate::error::{Result, SquirrelError};
use sysinfo::System;
use tokio::time::{interval, Duration};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// Production implementation of PerformanceMonitor using system metrics
#[derive(Debug)]
pub struct PerformanceMonitorImpl {
    /// Configuration for the performance monitor
    config: PerformanceConfig,
    /// System information handle
    system: Arc<RwLock<System>>,
    /// Performance statistics history
    stats_history: Arc<RwLock<Vec<PerformanceStats>>>,
    /// Operation timings
    timings: Arc<RwLock<Vec<OperationTiming>>>,
    /// Background task handle for periodic updates
    update_task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

#[async_trait]
impl PerformanceMonitorTrait for PerformanceMonitorImpl {
    async fn get_stats(&self) -> Result<PerformanceStats> {
        let system = self.system
            .read()
            .map_err(|e| SquirrelError::Monitoring(format!("Failed to read system info: {e}")))?;
        
        let cpu_usage = if self.config.track_cpu {
            // Get overall CPU usage - average of all CPU cores
            let cpus = system.cpus();
            if cpus.is_empty() {
                0.0
            } else {
                let total_usage: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
                (total_usage / cpus.len() as f32) as f64
            }
        } else {
            0.0
        };
        
        let (memory_usage, memory_available) = if self.config.track_memory {
            (system.used_memory(), system.available_memory())
        } else {
            (0, 0)
        };
        
        Ok(PerformanceStats {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            cpu_usage,
            memory_usage,
            memory_available,
            open_files: 0, // Not tracked by sysinfo directly
            active_threads: 0, // Not tracked by sysinfo directly
        })
    }
    
    async fn get_stats_history(&self, limit: Option<usize>) -> Result<Vec<PerformanceStats>> {
        let history = self.stats_history
            .read()
            .map_err(|e| SquirrelError::Monitoring(format!("Failed to read stats history: {e}")))?;
        
        let result = if let Some(limit) = limit {
            history.iter().rev().take(limit).cloned().collect::<Vec<_>>()
        } else {
            history.clone()
        };
        
        Ok(result)
    }
    
    async fn record_timing(&self, operation: &str, duration_ms: f64, success: bool) -> Result<()> {
        let timing = OperationTiming {
            name: operation.to_string(),
            duration_ms,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            success,
            context: HashMap::new(),
        };
        
        let mut timings = self.timings
            .write()
            .map_err(|e| SquirrelError::Monitoring(format!("Failed to record timing: {e}")))?;
        
        timings.push(timing);
        
        // Limit the number of timings we store
        if timings.len() > self.config.max_samples {
            let excess = timings.len() - self.config.max_samples;
            timings.drain(0..excess);
        }
        
        Ok(())
    }
    
    async fn record_timing_with_context(
        &self, 
        operation: &str, 
        duration_ms: f64, 
        success: bool, 
        context: HashMap<String, String>
    ) -> Result<()> {
        let timing = OperationTiming {
            name: operation.to_string(),
            duration_ms,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            success,
            context,
        };
        
        let mut timings = self.timings
            .write()
            .map_err(|e| SquirrelError::Monitoring(format!("Failed to record timing: {e}")))?;
        
        timings.push(timing);
        
        // Limit the number of timings we store
        if timings.len() > self.config.max_samples {
            let excess = timings.len() - self.config.max_samples;
            timings.drain(0..excess);
        }
        
        Ok(())
    }
    
    async fn get_timings(&self, operation: Option<&str>, limit: Option<usize>) -> Result<Vec<OperationTiming>> {
        let timings = self.timings
            .read()
            .map_err(|e| SquirrelError::Monitoring(format!("Failed to read timings: {e}")))?;
        
        let filtered = if let Some(op) = operation {
            timings.iter()
                .filter(|t| t.name == op)
                .cloned()
                .collect::<Vec<_>>()
        } else {
            timings.clone()
        };
        
        let result = if let Some(limit) = limit {
            filtered.into_iter().rev().take(limit).collect()
        } else {
            filtered
        };
        
        Ok(result)
    }
    
    async fn start(&self) -> Result<()> {
        let mut update_task = self.update_task.lock().await;
        if update_task.is_some() {
            return Ok(());
        }
        
        // Clone what we need for the task
        let config = self.config.clone();
        let system = self.system.clone();
        let stats_history = self.stats_history.clone();
        
        // Create a background task to periodically update performance stats
        let handle = tokio::spawn(async move {
            let mut update_interval = interval(Duration::from_secs(config.interval));
            
            loop {
                update_interval.tick().await;
                
                // Update system info
                if let Ok(mut sys) = system.write() {
                    sys.refresh_all();
                    
                    let cpu_usage = if config.track_cpu {
                        // Get overall CPU usage - average of all CPU cores
                        let cpus = sys.cpus();
                        if cpus.is_empty() {
                            0.0
                        } else {
                            let total_usage: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
                            (total_usage / cpus.len() as f32) as f64
                        }
                    } else {
                        0.0
                    };
                    
                    let stats = PerformanceStats {
                        timestamp: SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                        cpu_usage,
                        memory_usage: if config.track_memory { sys.used_memory() } else { 0 },
                        memory_available: if config.track_memory { sys.available_memory() } else { 0 },
                        open_files: 0, // Not tracked by sysinfo directly
                        active_threads: 0, // Not tracked by sysinfo directly
                    };
                    
                    if let Ok(mut history) = stats_history.write() {
                        history.push(stats);
                        
                        // Limit the history size
                        if history.len() > config.max_samples {
                            let excess = history.len() - config.max_samples;
                            history.drain(0..excess);
                        }
                    }
                }
            }
        });
        
        *update_task = Some(handle);
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        let mut update_task = self.update_task.lock().await;
        if let Some(handle) = update_task.take() {
            handle.abort();
        }
        Ok(())
    }
    
    fn create_timer(&self, operation: &str) -> crate::app::monitoring::performance::PerformanceTimer {
        crate::app::monitoring::performance::PerformanceTimer::new(
            operation, 
            Some(Arc::new(self.clone()))
        )
    }
}

impl PerformanceMonitorImpl {
    /// Create a new performance monitor with the specified configuration
    pub fn new(config: PerformanceConfig) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self {
            config,
            system: Arc::new(RwLock::new(system)),
            stats_history: Arc::new(RwLock::new(Vec::new())),
            timings: Arc::new(RwLock::new(Vec::new())),
            update_task: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Create a new performance monitor with default configuration
    pub fn default_config() -> Self {
        Self::new(PerformanceConfig::default())
    }
}

// Implement Clone manually since the trait objects don't implement it
impl Clone for PerformanceMonitorImpl {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            system: self.system.clone(),
            stats_history: self.stats_history.clone(),
            timings: self.timings.clone(),
            update_task: self.update_task.clone(),
        }
    }
}

/// Factory for creating PerformanceMonitorImpl instances
pub struct PerformanceMonitorFactoryImpl {
    config: PerformanceConfig,
}

impl PerformanceMonitorFactory for PerformanceMonitorFactoryImpl {
    fn create_monitor(&self) -> Arc<dyn PerformanceMonitorTrait> {
        Arc::new(PerformanceMonitorImpl::new(self.config.clone()))
    }
}

impl PerformanceMonitorFactoryImpl {
    /// Create a new factory with the specified configuration
    pub fn new(config: PerformanceConfig) -> Self {
        Self { config }
    }
    
    /// Create a new factory with default configuration
    pub fn default_config() -> Self {
        Self::new(PerformanceConfig::default())
    }
}

impl Default for PerformanceMonitorFactoryImpl {
    fn default() -> Self {
        Self::default_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration as TokioDuration;
    
    #[tokio::test]
    async fn test_performance_monitor_impl() {
        let factory = PerformanceMonitorFactoryImpl::default_config();
        let monitor = factory.create_monitor();
        
        // Start monitoring
        monitor.start().await.unwrap();
        
        // Record an operation timing
        assert!(monitor.record_timing("test_operation", 100.0, true).await.is_ok());
        
        // Test the timer
        let timer = monitor.create_timer("timed_operation");
        tokio::time::sleep(TokioDuration::from_millis(10)).await;
        let duration = timer.stop(true).await.unwrap();
        assert!(duration >= 10.0, "Duration should be at least 10ms");
        
        // Sleep to allow time for at least one system stat update
        tokio::time::sleep(TokioDuration::from_millis(100)).await;
        
        // Get current stats
        let stats = monitor.get_stats().await.unwrap();
        println!("CPU Usage: {}%", stats.cpu_usage);
        println!("Memory Usage: {} bytes", stats.memory_usage);
        
        // Get operation timings
        let timings = monitor.get_timings(None, None).await.unwrap();
        assert!(!timings.is_empty(), "Should have at least one timing record");
        
        // Stop monitoring
        monitor.stop().await.unwrap();
    }
} 
