//! Resource monitoring for plugins
//!
//! This module provides functionality for monitoring and limiting plugin resource usage.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use anyhow::Result;
use tokio::time::{Duration, interval};
use serde::{Serialize, Deserialize};
use crate::errors::PluginError;
use crate::security::{ResourceUsage, SandboxConfig};

/// Resource limits exceeded action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceLimitAction {
    /// Log the excess
    Log,
    
    /// Pause the plugin
    Pause,
    
    /// Stop the plugin
    Stop,
    
    /// Restart the plugin
    Restart,
}

impl Default for ResourceLimitAction {
    fn default() -> Self {
        Self::Log
    }
}

/// Resource monitor configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ResourceMonitorConfig {
    /// Check interval in milliseconds
    pub check_interval_ms: u64,
    
    /// Action to take when resource limits are exceeded
    pub limit_action: ResourceLimitAction,
    
    /// Grace period in milliseconds before taking action
    pub grace_period_ms: u64,
    
    /// Whether to automatically update resource usage
    pub auto_update: bool,
    
    /// Maximum memory samples to keep
    pub max_samples: usize,
}

impl Default for ResourceMonitorConfig {
    fn default() -> Self {
        Self {
            check_interval_ms: 1000, // 1 second
            limit_action: ResourceLimitAction::Log,
            grace_period_ms: 5000, // 5 seconds
            auto_update: true,
            max_samples: 10,
        }
    }
}

/// Plugin resource stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStats {
    /// Plugin ID
    pub plugin_id: Uuid,
    
    /// Current resource usage
    pub current_usage: ResourceUsage,
    
    /// Peak memory usage
    pub peak_memory: u64,
    
    /// Peak CPU usage
    pub peak_cpu: f64,
    
    /// Peak disk usage
    pub peak_disk: u64,
    
    /// Avg memory usage
    pub avg_memory: u64,
    
    /// Avg CPU usage
    pub avg_cpu: f64,
    
    /// Avg disk usage
    pub avg_disk: u64,
    
    /// Resource limit exceeded count
    pub limit_exceeded_count: u32,
    
    /// Last limit check timestamp
    pub last_check: u64,
    
    /// Resource usage samples (for averaging)
    pub samples: Vec<ResourceUsage>,
}

impl ResourceStats {
    /// Create new resource stats
    pub fn new(plugin_id: Uuid) -> Self {
        Self {
            plugin_id,
            current_usage: ResourceUsage::default(),
            peak_memory: 0,
            peak_cpu: 0.0,
            peak_disk: 0,
            avg_memory: 0,
            avg_cpu: 0.0,
            avg_disk: 0,
            limit_exceeded_count: 0,
            last_check: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            samples: Vec::new(),
        }
    }
    
    /// Add a resource usage sample
    pub fn add_sample(&mut self, usage: ResourceUsage, max_samples: usize) {
        // Update current usage
        self.current_usage = usage.clone();
        
        // Update peak values
        self.peak_memory = self.peak_memory.max(usage.memory_usage);
        self.peak_cpu = self.peak_cpu.max(usage.cpu_usage);
        self.peak_disk = self.peak_disk.max(usage.disk_usage);
        
        // Add sample
        self.samples.push(usage);
        
        // Keep only the latest max_samples
        if self.samples.len() > max_samples {
            self.samples.remove(0);
        }
        
        // Update averages
        if !self.samples.is_empty() {
            let sum_memory: u64 = self.samples.iter().map(|s| s.memory_usage).sum();
            let sum_cpu: f64 = self.samples.iter().map(|s| s.cpu_usage).sum();
            let sum_disk: u64 = self.samples.iter().map(|s| s.disk_usage).sum();
            
            let count = self.samples.len() as u64;
            self.avg_memory = sum_memory / count;
            self.avg_cpu = sum_cpu / (count as f64);
            self.avg_disk = sum_disk / count;
        }
        
        // Update last check time
        self.last_check = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
    
    /// Check if resource limits are exceeded
    pub fn check_limits(&mut self, config: &SandboxConfig) -> bool {
        let mut exceeded = false;
        
        if let Some(max_memory) = config.max_memory {
            if self.current_usage.memory_usage > max_memory {
                exceeded = true;
            }
        }
        
        if let Some(max_cpu) = config.max_cpu {
            if self.current_usage.cpu_usage > max_cpu {
                exceeded = true;
            }
        }
        
        if let Some(max_disk) = config.max_disk {
            if self.current_usage.disk_usage > max_disk {
                exceeded = true;
            }
        }
        
        if exceeded {
            self.limit_exceeded_count += 1;
        }
        
        exceeded
    }
}

/// Resource monitor
#[derive(Debug)]
pub struct ResourceMonitor {
    /// Plugin stats
    stats: Arc<RwLock<HashMap<Uuid, ResourceStats>>>,
    
    /// Sandbox configurations
    sandbox_configs: Arc<RwLock<HashMap<Uuid, SandboxConfig>>>,
    
    /// Monitor configuration
    config: Arc<RwLock<ResourceMonitorConfig>>,
    
    /// Monitor running state
    running: Arc<RwLock<bool>>,
    
    /// Plugins to stop due to resource limits
    plugins_to_stop: Arc<RwLock<Vec<Uuid>>>,
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(HashMap::new())),
            sandbox_configs: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(ResourceMonitorConfig::default())),
            running: Arc::new(RwLock::new(false)),
            plugins_to_stop: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Create a new resource monitor with custom configuration
    pub fn with_config(config: ResourceMonitorConfig) -> Self {
        Self {
            stats: Arc::new(RwLock::new(HashMap::new())),
            sandbox_configs: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(config)),
            running: Arc::new(RwLock::new(false)),
            plugins_to_stop: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Start the resource monitor
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(()); // Already running
        }
        
        *running = true;
        drop(running);
        
        let config = self.config.read().await.clone();
        
        // Clone necessary data for the monitoring task
        let stats = self.stats.clone();
        let sandbox_configs = self.sandbox_configs.clone();
        let plugins_to_stop = self.plugins_to_stop.clone();
        let running_clone = self.running.clone();
        
        // Spawn a monitoring task
        tokio::spawn(async move {
            let mut interval_timer = interval(Duration::from_millis(config.check_interval_ms));
            
            loop {
                interval_timer.tick().await;
                
                // Check if we should still be running
                if !*running_clone.read().await {
                    break;
                }
                
                // Check resource limits for all plugins
                let mut stats_lock = stats.write().await;
                let sandbox_configs_lock = sandbox_configs.read().await;
                let mut plugins_to_stop_lock = plugins_to_stop.write().await;
                
                for (plugin_id, plugin_stats) in stats_lock.iter_mut() {
                    if let Some(sandbox_config) = sandbox_configs_lock.get(plugin_id) {
                        if plugin_stats.check_limits(sandbox_config) {
                            // If limits exceeded for longer than grace period, take action
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            
                            let exceeded_duration = now - plugin_stats.last_check;
                            
                            if exceeded_duration > config.grace_period_ms / 1000 {
                                match config.limit_action {
                                    ResourceLimitAction::Log => {
                                        tracing::warn!(
                                            "Plugin {} exceeded resource limits: memory={}MB, CPU={}%, disk={}MB",
                                            plugin_id,
                                            plugin_stats.current_usage.memory_usage / (1024 * 1024),
                                            plugin_stats.current_usage.cpu_usage * 100.0,
                                            plugin_stats.current_usage.disk_usage / (1024 * 1024),
                                        );
                                    }
                                    ResourceLimitAction::Stop | ResourceLimitAction::Restart => {
                                        // Add to list of plugins to stop
                                        if !plugins_to_stop_lock.contains(plugin_id) {
                                            plugins_to_stop_lock.push(*plugin_id);
                                            
                                            tracing::warn!(
                                                "Scheduling plugin {} to stop due to resource limits",
                                                plugin_id
                                            );
                                        }
                                    }
                                    ResourceLimitAction::Pause => {
                                        // Pause functionality would be implemented here
                                        tracing::warn!("Pausing plugin {} due to resource limits", plugin_id);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Stop the resource monitor
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        Ok(())
    }
    
    /// Add or update a plugin's sandbox configuration
    pub async fn set_sandbox_config(&self, plugin_id: Uuid, config: SandboxConfig) -> Result<()> {
        let mut sandbox_configs = self.sandbox_configs.write().await;
        sandbox_configs.insert(plugin_id, config);
        
        // Initialize stats if needed
        let mut stats = self.stats.write().await;
        if !stats.contains_key(&plugin_id) {
            stats.insert(plugin_id, ResourceStats::new(plugin_id));
        }
        
        Ok(())
    }
    
    /// Remove a plugin's sandbox configuration
    pub async fn remove_sandbox_config(&self, plugin_id: &Uuid) -> Result<()> {
        let mut sandbox_configs = self.sandbox_configs.write().await;
        sandbox_configs.remove(plugin_id);
        
        // Also remove stats
        let mut stats = self.stats.write().await;
        stats.remove(plugin_id);
        
        Ok(())
    }
    
    /// Update a plugin's resource usage
    pub async fn update_resource_usage(&self, plugin_id: Uuid, usage: ResourceUsage) -> Result<()> {
        let mut stats = self.stats.write().await;
        
        // Get or create stats for this plugin
        let plugin_stats = stats
            .entry(plugin_id)
            .or_insert_with(|| ResourceStats::new(plugin_id));
        
        // Get the max samples config
        let config = self.config.read().await;
        let max_samples = config.max_samples;
        
        // Update stats
        plugin_stats.add_sample(usage, max_samples);
        
        Ok(())
    }
    
    /// Get a plugin's resource stats
    pub async fn get_resource_stats(&self, plugin_id: Uuid) -> Result<ResourceStats> {
        let stats = self.stats.read().await;
        
        stats
            .get(&plugin_id)
            .cloned()
            .ok_or_else(|| PluginError::NotFound(plugin_id).into())
    }
    
    /// Get plugins that have exceeded resource limits and should be stopped
    pub async fn get_plugins_to_stop(&self) -> Result<Vec<Uuid>> {
        let plugins_to_stop = self.plugins_to_stop.read().await;
        Ok(plugins_to_stop.clone())
    }
    
    /// Clear the list of plugins to stop
    pub async fn clear_plugins_to_stop(&self) -> Result<()> {
        let mut plugins_to_stop = self.plugins_to_stop.write().await;
        plugins_to_stop.clear();
        Ok(())
    }
    
    /// Set the resource monitor configuration
    pub async fn set_config(&self, config: ResourceMonitorConfig) -> Result<()> {
        let mut current_config = self.config.write().await;
        *current_config = config;
        Ok(())
    }
    
    /// Get the resource monitor configuration
    pub async fn get_config(&self) -> Result<ResourceMonitorConfig> {
        let config = self.config.read().await;
        Ok(config.clone())
    }
    
    /// Get a plugin's sandbox configuration
    pub async fn get_sandbox_config(&self, plugin_id: &Uuid) -> Result<Option<SandboxConfig>> {
        let sandbox_configs = self.sandbox_configs.read().await;
        Ok(sandbox_configs.get(plugin_id).cloned())
    }
    
    /// Get all plugin resource stats
    pub async fn get_all_resource_stats(&self) -> Result<HashMap<Uuid, ResourceStats>> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }
    
    /// Sample plugin resource usage
    ///
    /// In a real implementation, this would use OS APIs to sample actual resource usage.
    /// This implementation just generates sample data for demonstration purposes.
    pub async fn sample_plugin_resource_usage(&self, plugin_id: Uuid) -> Result<ResourceUsage> {
        // In a real implementation, this would use OS-specific APIs to get actual usage
        // For now, we'll just generate some sample data
        
        // Get the current resource usage as a starting point
        let stats = self.stats.read().await;
        let current_usage = stats
            .get(&plugin_id)
            .map(|s| s.current_usage.clone())
            .unwrap_or_default();
        
        // Create a new sample with small random variations
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Ensure we have non-zero variations to avoid empty range errors
        let memory_variation = (current_usage.memory_usage as f64 * 0.05) as u64 + 1;
        let cpu_variation = current_usage.cpu_usage * 0.05 + 0.001;
        let disk_variation = (current_usage.disk_usage as f64 * 0.01) as u64 + 1;
        
        let memory_usage = if rng.gen::<bool>() {
            current_usage.memory_usage.saturating_add(rng.gen_range(0..memory_variation))
        } else {
            current_usage.memory_usage.saturating_sub(rng.gen_range(0..memory_variation))
        };
        
        let cpu_usage = if rng.gen::<bool>() {
            (current_usage.cpu_usage + rng.gen_range(0.0..cpu_variation)).min(1.0)
        } else {
            (current_usage.cpu_usage - rng.gen_range(0.0..cpu_variation)).max(0.0)
        };
        
        let disk_usage = if rng.gen::<bool>() {
            current_usage.disk_usage.saturating_add(rng.gen_range(0..disk_variation))
        } else {
            current_usage.disk_usage
        };
        
        let network_usage = current_usage.network_usage + rng.gen_range(1..1000);
        
        let usage = ResourceUsage {
            memory_usage,
            cpu_usage,
            disk_usage,
            network_usage,
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        
        // In a real implementation, we would call update_resource_usage here
        // to update the stats. For the sample implementation, we'll let the
        // caller decide whether to update.
        
        Ok(usage)
    }
}

impl Default for ResourceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_resource_stats() {
        // Create resource stats
        let plugin_id = Uuid::new_v4();
        let mut stats = ResourceStats::new(plugin_id);
        
        // Add some samples
        for i in 1..=5 {
            let usage = ResourceUsage {
                memory_usage: i * 100,
                cpu_usage: i as f64 * 0.1,
                disk_usage: i * 1000,
                network_usage: i * 10,
                last_updated: 0,
            };
            
            stats.add_sample(usage, 10);
        }
        
        // Check averages
        assert_eq!(stats.avg_memory, 300);
        assert!((stats.avg_cpu - 0.3).abs() < 0.001);
        assert_eq!(stats.avg_disk, 3000);
        
        // Check peaks
        assert_eq!(stats.peak_memory, 500);
        assert!((stats.peak_cpu - 0.5).abs() < 0.001);
        assert_eq!(stats.peak_disk, 5000);
        
        // Check current usage
        assert_eq!(stats.current_usage.memory_usage, 500);
        assert!((stats.current_usage.cpu_usage - 0.5).abs() < 0.001);
        assert_eq!(stats.current_usage.disk_usage, 5000);
        
        // Test limits
        let config = SandboxConfig {
            max_memory: Some(400),
            max_cpu: Some(0.4),
            max_disk: Some(4000),
            ..Default::default()
        };
        
        // Should exceed limits
        assert!(stats.check_limits(&config));
        assert_eq!(stats.limit_exceeded_count, 1);
        
        // Add sample below limits
        let usage = ResourceUsage {
            memory_usage: 300,
            cpu_usage: 0.3,
            disk_usage: 3000,
            network_usage: 10,
            last_updated: 0,
        };
        
        stats.add_sample(usage, 10);
        
        // Should not exceed limits
        assert!(!stats.check_limits(&config));
        assert_eq!(stats.limit_exceeded_count, 1);
    }
    
    #[tokio::test]
    async fn test_resource_monitor() {
        // Create monitor
        let monitor = ResourceMonitor::new();
        
        // Add plugin configuration
        let plugin_id = Uuid::new_v4();
        let config = SandboxConfig {
            max_memory: Some(1024 * 1024), // 1MB
            max_cpu: Some(0.5),            // 50%
            max_disk: Some(10 * 1024 * 1024), // 10MB
            ..Default::default()
        };
        
        monitor.set_sandbox_config(plugin_id, config).await.unwrap();
        
        // Sample and update resource usage
        let usage = monitor.sample_plugin_resource_usage(plugin_id).await.unwrap();
        monitor.update_resource_usage(plugin_id, usage).await.unwrap();
        
        // Get resource stats
        let stats = monitor.get_resource_stats(plugin_id).await.unwrap();
        assert_eq!(stats.plugin_id, plugin_id);
        assert_eq!(stats.samples.len(), 1);
        
        // Check sandbox config
        let config = monitor.get_sandbox_config(&plugin_id).await.unwrap();
        assert!(config.is_some());
        let config = config.unwrap();
        assert_eq!(config.max_memory, Some(1024 * 1024));
        
        // Update configuration
        let new_config = ResourceMonitorConfig {
            check_interval_ms: 500,
            ..Default::default()
        };
        
        monitor.set_config(new_config.clone()).await.unwrap();
        
        // Get configuration
        let config = monitor.get_config().await.unwrap();
        assert_eq!(config.check_interval_ms, 500);
        
        // Remove plugin
        monitor.remove_sandbox_config(&plugin_id).await.unwrap();
        
        // Should not find stats after removal
        assert!(monitor.get_resource_stats(plugin_id).await.is_err());
    }
} 