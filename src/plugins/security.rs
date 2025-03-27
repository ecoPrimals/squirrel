// Plugin Security Module
//
// This module provides security-related functionality for plugins, including validation
// and resource limits.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, atomic::{AtomicU64, AtomicU8, AtomicU32, Ordering}};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use crate::plugins::errors::PluginError;
use crate::plugins::Result;
use crate::plugins::{PluginSecurityLevel, ResourceLimits};
use squirrel_mcp::plugins::interfaces::{Plugin, PluginMetadata};

/// Plugin security manager
///
/// This trait defines the interface for plugin security.
#[async_trait]
pub trait PluginSecurityManager: Send + Sync + Debug {
    /// Validate a plugin
    async fn validate_plugin(&self, plugin: &Arc<dyn Plugin>) -> Result<()>;
    
    /// Check if a plugin has permission to perform an action
    async fn check_permission(&self, plugin_id: Uuid, permission: &str) -> Result<bool>;
    
    /// Start monitoring a plugin
    async fn start_monitoring(&self, plugin_id: Uuid) -> Result<()>;
    
    /// Stop monitoring a plugin
    async fn stop_monitoring(&self, plugin_id: Uuid) -> Result<()>;
    
    /// Get resource usage for a plugin
    async fn get_resource_usage(&self, plugin_id: Uuid) -> Result<ResourceUsage>;
}

/// Resource usage for a plugin
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    /// Plugin ID
    pub plugin_id: Uuid,
    
    /// Memory usage in bytes
    pub memory_bytes: u64,
    
    /// CPU usage percentage
    pub cpu_percent: u8,
    
    /// Number of threads
    pub threads: u32,
    
    /// Number of open files
    pub open_files: u32,
    
    /// Network bandwidth in bytes per second
    pub network_bps: u64,
}

/// Implementation of the PluginSecurityManager trait
#[derive(Debug)]
pub struct PluginSecurityValidator {
    /// Security level
    security_level: PluginSecurityLevel,
    
    /// Resource limits
    resource_limits: Option<ResourceLimits>,
    
    /// Resource usage by plugin
    resource_usage: RwLock<HashMap<Uuid, Arc<ResourceMonitor>>>,
    
    /// Permissions by plugin
    permissions: RwLock<HashMap<Uuid, Vec<String>>>,
}

impl PluginSecurityValidator {
    /// Create a new plugin security validator
    pub fn new(security_level: PluginSecurityLevel, resource_limits: Option<ResourceLimits>) -> Self {
        Self {
            security_level,
            resource_limits,
            resource_usage: RwLock::new(HashMap::new()),
            permissions: RwLock::new(HashMap::new()),
        }
    }
    
    /// Get resource limits
    pub fn resource_limits(&self) -> Option<&ResourceLimits> {
        self.resource_limits.as_ref()
    }
    
    /// Check resource limits
    async fn check_resource_limits(&self, plugin_id: Uuid) -> Result<()> {
        // Get resource usage
        let usage = self.get_resource_usage(plugin_id).await?;
        
        // Check against limits
        if let Some(limits) = &self.resource_limits {
            // Check memory limit
            if let Some(max_memory) = limits.max_memory_bytes {
                if usage.memory_bytes > max_memory {
                    return Err(PluginError::ResourceLimitExceeded(format!(
                        "Memory usage exceeded: {} > {}",
                        usage.memory_bytes, max_memory
                    )));
                }
            }
            
            // Check CPU limit
            if let Some(max_cpu) = limits.max_cpu_percent {
                if usage.cpu_percent > max_cpu {
                    return Err(PluginError::ResourceLimitExceeded(format!(
                        "CPU usage exceeded: {}% > {}%",
                        usage.cpu_percent, max_cpu
                    )));
                }
            }
            
            // Check thread limit
            if let Some(max_threads) = limits.max_threads {
                if usage.threads > max_threads {
                    return Err(PluginError::ResourceLimitExceeded(format!(
                        "Thread count exceeded: {} > {}",
                        usage.threads, max_threads
                    )));
                }
            }
            
            // Check file handle limit
            if let Some(max_files) = limits.max_file_handles {
                if usage.open_files > max_files {
                    return Err(PluginError::ResourceLimitExceeded(format!(
                        "Open file count exceeded: {} > {}",
                        usage.open_files, max_files
                    )));
                }
            }
            
            // Check network bandwidth limit
            if let Some(max_network) = limits.max_network_bps {
                if usage.network_bps > max_network {
                    return Err(PluginError::ResourceLimitExceeded(format!(
                        "Network bandwidth exceeded: {} > {}",
                        usage.network_bps, max_network
                    )));
                }
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl PluginSecurityManager for PluginSecurityValidator {
    async fn validate_plugin(&self, plugin: &Arc<dyn Plugin>) -> Result<()> {
        let metadata = plugin.metadata();
        let id = metadata.id;
        
        // Check security level
        match self.security_level {
            PluginSecurityLevel::None => {
                // No security checks
                debug!("Skipping security checks for plugin {}", id);
            }
            PluginSecurityLevel::Basic => {
                // Basic checks
                info!("Performing basic security checks for plugin {}", id);
                
                // Validate plugin metadata
                if metadata.name.is_empty() {
                    return Err(PluginError::ValidationError(
                        "Plugin name cannot be empty".to_string(),
                    ));
                }
                
                if metadata.version.is_empty() {
                    return Err(PluginError::ValidationError(
                        "Plugin version cannot be empty".to_string(),
                    ));
                }
                
                // Initialize resource monitor
                let mut resource_usage = self.resource_usage.write().await;
                let monitor = Arc::new(ResourceMonitor::new(id));
                resource_usage.insert(id, monitor);
            }
            PluginSecurityLevel::Standard => {
                // Standard checks
                info!("Performing standard security checks for plugin {}", id);
                
                // Validate plugin metadata
                if metadata.name.is_empty() {
                    return Err(PluginError::ValidationError(
                        "Plugin name cannot be empty".to_string(),
                    ));
                }
                
                if metadata.version.is_empty() {
                    return Err(PluginError::ValidationError(
                        "Plugin version cannot be empty".to_string(),
                    ));
                }
                
                if metadata.author.is_empty() {
                    return Err(PluginError::ValidationError(
                        "Plugin author cannot be empty".to_string(),
                    ));
                }
                
                // Initialize resource monitor
                let mut resource_usage = self.resource_usage.write().await;
                let monitor = Arc::new(ResourceMonitor::new(id));
                resource_usage.insert(id, monitor);
                
                // Set up permissions
                let mut permissions = self.permissions.write().await;
                permissions.insert(id, metadata.permissions.clone());
            }
            PluginSecurityLevel::Enhanced => {
                // Enhanced checks
                info!("Performing enhanced security checks for plugin {}", id);
                
                // Validate plugin metadata
                if metadata.name.is_empty() {
                    return Err(PluginError::ValidationError(
                        "Plugin name cannot be empty".to_string(),
                    ));
                }
                
                if metadata.version.is_empty() {
                    return Err(PluginError::ValidationError(
                        "Plugin version cannot be empty".to_string(),
                    ));
                }
                
                if metadata.author.is_empty() {
                    return Err(PluginError::ValidationError(
                        "Plugin author cannot be empty".to_string(),
                    ));
                }
                
                if metadata.description.is_empty() {
                    return Err(PluginError::ValidationError(
                        "Plugin description cannot be empty".to_string(),
                    ));
                }
                
                // Check for valid semver
                if !is_valid_semver(&metadata.version) {
                    return Err(PluginError::ValidationError(
                        format!("Invalid semantic version: {}", metadata.version),
                    ));
                }
                
                // Initialize resource monitor
                let mut resource_usage = self.resource_usage.write().await;
                let monitor = Arc::new(ResourceMonitor::new(id));
                resource_usage.insert(id, monitor);
                
                // Set up permissions
                let mut permissions = self.permissions.write().await;
                permissions.insert(id, metadata.permissions.clone());
                
                // Validate signature (if present)
                if let Some(signature) = &metadata.signature {
                    // In a real implementation, we would verify the signature here
                    info!("Plugin {} has signature", id);
                }
            }
        }
        
        Ok(())
    }
    
    async fn check_permission(&self, plugin_id: Uuid, permission: &str) -> Result<bool> {
        let permissions = self.permissions.read().await;
        
        match permissions.get(&plugin_id) {
            Some(plugin_permissions) => {
                // Check if plugin has permission
                Ok(plugin_permissions.contains(&permission.to_string()))
            }
            None => {
                // Plugin not found
                Err(PluginError::NotFound(plugin_id))
            }
        }
    }
    
    async fn start_monitoring(&self, plugin_id: Uuid) -> Result<()> {
        let resource_usage = self.resource_usage.read().await;
        
        match resource_usage.get(&plugin_id) {
            Some(monitor) => {
                // Start monitoring
                monitor.start();
                Ok(())
            }
            None => {
                // Plugin not found
                Err(PluginError::NotFound(plugin_id))
            }
        }
    }
    
    async fn stop_monitoring(&self, plugin_id: Uuid) -> Result<()> {
        let resource_usage = self.resource_usage.read().await;
        
        match resource_usage.get(&plugin_id) {
            Some(monitor) => {
                // Stop monitoring
                monitor.stop();
                Ok(())
            }
            None => {
                // Plugin not found
                Err(PluginError::NotFound(plugin_id))
            }
        }
    }
    
    async fn get_resource_usage(&self, plugin_id: Uuid) -> Result<ResourceUsage> {
        let resource_usage = self.resource_usage.read().await;
        
        match resource_usage.get(&plugin_id) {
            Some(monitor) => {
                // Get resource usage
                let usage = ResourceUsage {
                    plugin_id,
                    memory_bytes: monitor.memory_bytes(),
                    cpu_percent: monitor.cpu_percent(),
                    threads: monitor.threads(),
                    open_files: monitor.open_files(),
                    network_bps: monitor.network_bps(),
                };
                
                Ok(usage)
            }
            None => {
                // Plugin not found
                Err(PluginError::NotFound(plugin_id))
            }
        }
    }
}

/// Resource monitor
#[derive(Debug)]
struct ResourceMonitor {
    /// Plugin ID
    plugin_id: Uuid,
    
    /// Memory usage in bytes
    memory_bytes: AtomicU64,
    
    /// CPU usage percentage
    cpu_percent: AtomicU8,
    
    /// Number of threads
    threads: AtomicU32,
    
    /// Number of open files
    open_files: AtomicU32,
    
    /// Network bandwidth in bytes per second
    network_bps: AtomicU64,
    
    /// Whether monitoring is active
    active: std::sync::atomic::AtomicBool,
}

impl ResourceMonitor {
    /// Create a new resource monitor
    fn new(plugin_id: Uuid) -> Self {
        Self {
            plugin_id,
            memory_bytes: AtomicU64::new(0),
            cpu_percent: AtomicU8::new(0),
            threads: AtomicU32::new(0),
            open_files: AtomicU32::new(0),
            network_bps: AtomicU64::new(0),
            active: std::sync::atomic::AtomicBool::new(false),
        }
    }
    
    /// Start monitoring
    fn start(&self) {
        self.active.store(true, Ordering::SeqCst);
    }
    
    /// Stop monitoring
    fn stop(&self) {
        self.active.store(false, Ordering::SeqCst);
    }
    
    /// Get memory usage
    fn memory_bytes(&self) -> u64 {
        self.memory_bytes.load(Ordering::SeqCst)
    }
    
    /// Get CPU usage
    fn cpu_percent(&self) -> u8 {
        self.cpu_percent.load(Ordering::SeqCst)
    }
    
    /// Get thread count
    fn threads(&self) -> u32 {
        self.threads.load(Ordering::SeqCst)
    }
    
    /// Get open file count
    fn open_files(&self) -> u32 {
        self.open_files.load(Ordering::SeqCst)
    }
    
    /// Get network bandwidth
    fn network_bps(&self) -> u64 {
        self.network_bps.load(Ordering::SeqCst)
    }
    
    /// Update resource usage
    fn update(&self, memory: u64, cpu: u8, threads: u32, files: u32, network: u64) {
        if !self.active.load(Ordering::SeqCst) {
            return;
        }
        
        self.memory_bytes.store(memory, Ordering::SeqCst);
        self.cpu_percent.store(cpu, Ordering::SeqCst);
        self.threads.store(threads, Ordering::SeqCst);
        self.open_files.store(files, Ordering::SeqCst);
        self.network_bps.store(network, Ordering::SeqCst);
    }
}

/// Check if a string is a valid semantic version
fn is_valid_semver(version: &str) -> bool {
    // Simple semver validation (major.minor.patch)
    let parts: Vec<&str> = version.split('.').collect();
    
    if parts.len() != 3 {
        return false;
    }
    
    for part in parts {
        if part.parse::<u32>().is_err() {
            return false;
        }
    }
    
    true
}