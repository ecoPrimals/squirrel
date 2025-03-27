use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{json, Value};
use anyhow::{Result, anyhow};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use squirrel_interfaces::plugins::{CommandsPlugin, Plugin, PluginMetadata, CommandMetadata};

// Define our own TeamResourceMetrics for the example
#[derive(Debug, Clone)]
struct TeamResourceMetrics {
    /// Team identifier
    pub team_id: String,
    /// Memory usage as a percentage
    pub memory_usage: f64,
    /// Storage usage as a percentage
    pub storage_usage: f64,
    /// Network bandwidth in bytes per second
    pub network_bandwidth: f64,
    /// Number of active threads
    pub thread_count: u32,
    /// Disk I/O in bytes per second
    pub disk_io: f64,
    /// CPU usage as a percentage (0-100)
    pub cpu_usage: f64,
    /// Process information (simplified for example)
    pub processes: Vec<ProcessInfo>,
    /// Timestamp of when metrics were collected
    pub timestamp: DateTime<Utc>,
    /// Additional labels/tags for metrics
    pub labels: HashMap<String, String>,
}

// Process information structure
#[derive(Debug, Clone)]
struct ProcessInfo {
    // Just a placeholder for the example
    pub pid: u32,
    pub name: String,
}

// Simple metrics collector for the example
#[derive(Debug, Clone)]
struct ResourceMetricsCollector {
    // For a simple example, we won't implement all the functionality
}

impl ResourceMetricsCollector {
    fn new() -> Self {
        Self {}
    }
    
    fn collect_system_metrics(&self) -> Result<TeamResourceMetrics> {
        // Return dummy metrics for the example
        Ok(TeamResourceMetrics {
            team_id: "system".to_string(),
            memory_usage: 45.5, // percentage
            storage_usage: 60.2, // percentage
            network_bandwidth: 1024.0, // bytes/sec
            thread_count: 8,
            disk_io: 512.0, // bytes/sec
            cpu_usage: 25.3, // percentage
            processes: Vec::new(), // No processes for simplicity
            timestamp: Utc::now(),
            labels: HashMap::new(),
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create resource monitor
    let resource_monitor = ResourceMetricsCollector::new();
    let monitor = Arc::new(resource_monitor);
    
    // Create plugin instance
    let plugin = ResourceStateDemoPlugin::new("Resource Demo", monitor.clone());
    let plugin_ref = Arc::new(plugin);
    
    // Execute "allocate" command
    let args = json!({
        "size": 10 // Allocate 10MB
    });
    
    // Allocate some resources
    let result = plugin_ref.execute_command("allocate", args).await;
    println!("Allocate result: {:?}", result);
    
    // Get current usage
    let usage_result = plugin_ref.execute_command("usage", json!({})).await;
    println!("Usage result: {:?}", usage_result);
    
    // Shutdown
    plugin_ref.shutdown().await?;
    
    Ok(())
}

/// Command result data
#[derive(Debug, Clone)]
struct CommandResult {
    /// Whether the command was successful
    success: bool,
    /// Result message
    message: String,
    /// Additional data
    data: Option<Value>,
}

impl CommandResult {
    /// Create a success result
    fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
        }
    }
    
    /// Create an error result
    fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
        }
    }
    
    /// Add data to the result
    fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }
}

/// Sample plugin that demonstrates resource management 
#[derive(Debug)]
struct ResourceStateDemoPlugin {
    /// The name of the plugin
    name: String,
    /// Resource monitor
    monitor: Arc<ResourceMetricsCollector>,
    /// Resource data
    allocated_memory: Vec<Vec<u8>>,
    /// Plugin metadata
    metadata: PluginMetadata,
}

impl ResourceStateDemoPlugin {
    /// Create a new instance of the plugin
    fn new(name: &str, monitor: Arc<ResourceMetricsCollector>) -> Self {
        Self {
            name: name.to_string(),
            monitor,
            allocated_memory: Vec::new(),
            metadata: PluginMetadata {
                id: Uuid::new_v4().to_string(),
                name: name.to_string(),
                version: "1.0.0".to_string(),
                description: "A demo plugin showing resource management".to_string(),
                author: "DataScienceBioLab".to_string(),
                capabilities: vec!["resource_management".to_string()],
            },
        }
    }
    
    /// Allocate memory for testing resource limits
    async fn allocate_memory(&mut self, size_mb: usize) -> CommandResult {
        let size = size_mb * 1024 * 1024;
        let memory = vec![0u8; size];
        self.allocated_memory.push(memory);
        
        CommandResult::success(format!("Allocated {}MB of memory", size_mb))
    }
    
    /// Get current resource usage
    async fn get_resource_usage(&self) -> CommandResult {
        // Get system metrics from the collector
        match self.monitor.collect_system_metrics() {
            Ok(usage) => {
                let data = json!({
                    "memory_usage": usage.memory_usage,
                    "cpu_usage": usage.cpu_usage,
                    "storage_usage": usage.storage_usage,
                    "network_bandwidth": usage.network_bandwidth,
                    "thread_count": usage.thread_count,
                });
                CommandResult::success("Resource usage retrieved").with_data(data)
            },
            Err(e) => CommandResult::error(format!("Failed to get resource usage: {}", e)),
        }
    }
}

#[async_trait]
impl Plugin for ResourceStateDemoPlugin {
    async fn shutdown(&self) -> Result<()> {
        println!("Shutting down ResourceStateDemoPlugin");
        
        // Clean up allocated resources
        let memory_used = self.allocated_memory.iter().map(|v| v.len()).sum::<usize>();
        println!("Deallocating {}MB of memory", memory_used / (1024 * 1024));
        
        Ok(())
    }

    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
}

#[async_trait]
impl CommandsPlugin for ResourceStateDemoPlugin {
    fn get_available_commands(&self) -> Vec<CommandMetadata> {
        vec![
            CommandMetadata {
                id: "allocate".to_string(),
                name: "Allocate Memory".to_string(),
                description: "Allocate memory for testing resource limits".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "size": {
                            "type": "integer",
                            "description": "Size in MB",
                            "default": 10
                        }
                    }
                }),
                output_schema: json!({
                    "type": "object",
                    "properties": {
                        "success": {"type": "boolean"},
                        "message": {"type": "string"},
                        "data": {"type": "object"}
                    }
                }),
                permissions: vec![],
            },
            CommandMetadata {
                id: "usage".to_string(),
                name: "Get Resource Usage".to_string(),
                description: "Get current resource usage information".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
                output_schema: json!({
                    "type": "object",
                    "properties": {
                        "success": {"type": "boolean"},
                        "message": {"type": "string"},
                        "data": {"type": "object"}
                    }
                }),
                permissions: vec![],
            },
            CommandMetadata {
                id: "clear".to_string(),
                name: "Clear Memory".to_string(),
                description: "Clear allocated memory".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
                output_schema: json!({
                    "type": "object",
                    "properties": {
                        "success": {"type": "boolean"},
                        "message": {"type": "string"}
                    }
                }),
                permissions: vec![],
            },
        ]
    }

    async fn execute_command(&self, command_id: &str, input: Value) -> Result<Value> {
        // This implementation is simplified for the example
        match command_id {
            "allocate" => {
                let size = input.get("size")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as usize;
                
                let mut plugin = self.clone();
                let result = plugin.allocate_memory(size).await;
                Ok(json!({"success": result.success, "message": result.message, "data": result.data}))
            },
            "usage" => {
                let result = self.get_resource_usage().await;
                Ok(json!({"success": result.success, "message": result.message, "data": result.data}))
            },
            "clear" => {
                Ok(json!({"success": true, "message": "Memory cleared (demo only)"}))
            },
            _ => Err(anyhow!("Unknown command: {}", command_id))
        }
    }

    fn get_command_metadata(&self, command_id: &str) -> Option<CommandMetadata> {
        self.get_available_commands()
            .into_iter()
            .find(|cmd| cmd.id == command_id)
    }

    fn get_command_help(&self, command_id: &str) -> Option<String> {
        match command_id {
            "allocate" => Some("allocate [size=10]: Allocate memory for testing resource limits".to_string()),
            "usage" => Some("usage: Get current resource usage information".to_string()),
            "clear" => Some("clear: Clear allocated memory".to_string()),
            _ => None
        }
    }
}

// Allow cloning for demo purposes
impl Clone for ResourceStateDemoPlugin {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            monitor: self.monitor.clone(),
            allocated_memory: Vec::new(), // Note: We don't clone the memory, just for demo
            metadata: PluginMetadata {
                id: self.metadata.id.clone(),
                name: self.metadata.name.clone(),
                version: self.metadata.version.clone(),
                description: self.metadata.description.clone(),
                author: self.metadata.author.clone(),
                capabilities: self.metadata.capabilities.clone(),
            },
        }
    }
} 