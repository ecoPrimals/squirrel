use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use uuid::Uuid;
use squirrel_plugins::{
    Plugin, PluginState, ResourceLimits, ResourceMonitor, ResourceMonitorImpl, ResourceType,
    StateManager, DefaultStateManager, MemoryStateStorage, CommandsPlugin, CommandInfo,
    PluginError,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Create resource monitor
    let resource_monitor = Arc::new(ResourceMonitorImpl::new());
    
    // Create state manager with in-memory storage
    let state_storage = Arc::new(MemoryStateStorage::new());
    let state_manager = Arc::new(DefaultStateManager::new(state_storage));

    // Create our example plugin
    let plugin_id = Uuid::new_v4();
    let plugin = Arc::new(ResourceStateDemoPlugin::new(
        plugin_id,
        resource_monitor.clone(),
        state_manager.clone(),
    ));

    // Set resource limits for our plugin
    let limits = ResourceLimits {
        max_memory: Some(50 * 1024 * 1024), // 50 MB
        max_cpu: Some(0.5),                 // 50% CPU
        max_disk: Some(10 * 1024 * 1024),   // 10 MB
        max_network: Some(1024 * 1024),     // 1 MB/s
        max_file_handles: Some(10),         // 10 files
        max_threads: Some(5),               // 5 threads
        ..Default::default()
    };
    
    resource_monitor.set_limits(plugin_id, limits).await?;
    
    // Initialize plugin
    plugin.initialize().await?;
    
    // Start background resource monitoring
    ResourceMonitorImpl::start_background_monitoring(resource_monitor.clone()).await;
    
    // Execute some commands to demonstrate resource allocation and state persistence
    println!("--- Executing increment command ---");
    let result = plugin.execute_command("increment", json!({})).await?;
    println!("Result: {}", result);
    
    println!("--- Executing increment command again ---");
    let result = plugin.execute_command("increment", json!({})).await?;
    println!("Result: {}", result);
    
    println!("--- Executing get_count command ---");
    let result = plugin.execute_command("get_count", json!({})).await?;
    println!("Result: {}", result);
    
    println!("--- Allocating memory ---");
    let result = plugin.execute_command("allocate_memory", json!({ "size_mb": 10 })).await?;
    println!("Result: {}", result);
    
    // Wait a moment for the resource monitoring to detect usage
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Get resource usage
    println!("--- Resource Usage ---");
    let usage = resource_monitor.get_usage(plugin_id).await?;
    println!("Memory: {} bytes", usage.memory);
    println!("CPU: {:.2}%", usage.cpu * 100.0);
    println!("Disk: {} bytes", usage.disk);
    println!("Network: {} bytes/s", usage.network);
    println!("File handles: {}", usage.file_handles);
    println!("Threads: {}", usage.threads);
    
    // Get state
    println!("--- Plugin State ---");
    let state = state_manager.load_state(plugin_id).await?;
    if let Some(state) = state {
        println!("State version: {}", state.version);
        println!("State data: {}", state.data);
        println!("Last updated: {}", state.updated_at);
    } else {
        println!("No state found");
    }
    
    // Clean up
    plugin.shutdown().await?;
    resource_monitor.stop_monitoring(plugin_id).await?;
    
    println!("Plugin demo completed successfully!");
    Ok(())
}

/// Example plugin that demonstrates resource management and state persistence
struct ResourceStateDemoPlugin {
    id: Uuid,
    resource_monitor: Arc<dyn ResourceMonitor>,
    state_manager: Arc<dyn StateManager>,
    // Keeping track of allocated memory to free it later
    allocated_memory: tokio::sync::Mutex<Vec<Vec<u8>>>,
}

impl ResourceStateDemoPlugin {
    fn new(
        id: Uuid,
        resource_monitor: Arc<dyn ResourceMonitor>,
        state_manager: Arc<dyn StateManager>,
    ) -> Self {
        Self {
            id,
            resource_monitor,
            state_manager,
            allocated_memory: tokio::sync::Mutex::new(Vec::new()),
        }
    }
    
    async fn increment_counter(&self) -> Result<i32, PluginError> {
        // Load current state
        let state = self.state_manager.load_state(self.id).await?;
        
        let current_count = if let Some(state) = state {
            state.data.get("count").and_then(|v| v.as_i64()).unwrap_or(0) as i32
        } else {
            0
        };
        
        let new_count = current_count + 1;
        
        // Update state
        self.state_manager
            .update_state(self.id, json!({ "count": new_count }))
            .await?;
        
        Ok(new_count)
    }
    
    async fn get_counter(&self) -> Result<i32, PluginError> {
        // Load current state
        let state = self.state_manager.load_state(self.id).await?;
        
        let count = if let Some(state) = state {
            state.data.get("count").and_then(|v| v.as_i64()).unwrap_or(0) as i32
        } else {
            0
        };
        
        Ok(count)
    }
    
    async fn allocate_memory(&self, size_mb: usize) -> Result<String, PluginError> {
        let size_bytes = size_mb * 1024 * 1024;
        
        // Allocate memory
        let memory = vec![0u8; size_bytes];
        
        // Report allocation to resource monitor
        self.resource_monitor
            .report_allocation(self.id, ResourceType::Memory, size_bytes as u64)
            .await?;
        
        // Store the allocated memory to prevent it from being dropped
        let mut allocated = self.allocated_memory.lock().await;
        allocated.push(memory);
        
        Ok(format!("Allocated {} MB of memory", size_mb))
    }
}

impl Plugin for ResourceStateDemoPlugin {
    fn metadata(&self) -> squirrel_mcp::plugins::interfaces::PluginMetadata {
        squirrel_mcp::plugins::interfaces::PluginMetadata {
            id: self.id,
            name: "resource-state-demo".to_string(),
            version: "1.0.0".to_string(),
            description: "Demonstrates resource management and state persistence".to_string(),
            status: squirrel_mcp::plugins::interfaces::PluginStatus::Registered,
        }
    }
    
    async fn initialize(&self) -> anyhow::Result<()> {
        // Start monitoring resources
        self.resource_monitor.start_monitoring(self.id).await?;
        
        // Initialize state if not already present
        let state = self.state_manager.load_state(self.id).await?;
        if state.is_none() {
            self.state_manager
                .update_state(self.id, json!({ "count": 0 }))
                .await?;
        }
        
        println!("ResourceStateDemoPlugin initialized");
        Ok(())
    }
    
    async fn shutdown(&self) -> anyhow::Result<()> {
        // Clean up allocated resources
        let mut allocated = self.allocated_memory.lock().await;
        let total_memory = allocated.iter().map(|v| v.len()).sum::<usize>();
        
        if total_memory > 0 {
            // Report deallocation
            self.resource_monitor
                .report_deallocation(self.id, ResourceType::Memory, total_memory as u64)
                .await?;
            
            // Clear allocated memory
            allocated.clear();
        }
        
        println!("ResourceStateDemoPlugin shut down");
        Ok(())
    }
}

#[async_trait]
impl CommandsPlugin for ResourceStateDemoPlugin {
    fn get_commands(&self) -> Vec<CommandInfo> {
        vec![
            CommandInfo {
                name: "increment".to_string(),
                description: "Increment the counter".to_string(),
                category: Some("State".to_string()),
                tags: vec!["state".to_string()],
                requires_auth: false,
            },
            CommandInfo {
                name: "get_count".to_string(),
                description: "Get the current count".to_string(),
                category: Some("State".to_string()),
                tags: vec!["state".to_string()],
                requires_auth: false,
            },
            CommandInfo {
                name: "allocate_memory".to_string(),
                description: "Allocate memory (in MB)".to_string(),
                category: Some("Resource".to_string()),
                tags: vec!["resource".to_string()],
                requires_auth: false,
            },
        ]
    }
    
    async fn execute_command(&self, name: &str, args: serde_json::Value) -> Result<serde_json::Value, PluginError> {
        match name {
            "increment" => {
                let count = self.increment_counter().await?;
                Ok(json!({ "new_count": count }))
            }
            "get_count" => {
                let count = self.get_counter().await?;
                Ok(json!({ "count": count }))
            }
            "allocate_memory" => {
                let size_mb = args.get("size_mb").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
                let result = self.allocate_memory(size_mb).await?;
                Ok(json!({ "message": result }))
            }
            _ => Err(PluginError::CommandNotFound(name.to_string())),
        }
    }
    
    fn get_command_help(&self, _name: &str) -> Option<squirrel_plugins::interfaces::CommandHelp> {
        None
    }
    
    fn get_command_schema(&self, _name: &str) -> Option<serde_json::Value> {
        None
    }
} 