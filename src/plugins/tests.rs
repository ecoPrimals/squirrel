#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use serde_json::json;
    use uuid::Uuid;
    
    use crate::plugins::{
        create_plugin_manager,
        examples::{create_example_command_plugin, create_example_tool_plugin},
        interfaces::{CommandsPlugin, ToolPlugin},
        PluginStatus,
    };
    
    #[tokio::test]
    async fn test_plugin_registration() {
        // Create a plugin manager
        let manager = create_plugin_manager();
        
        // Create example plugins
        let command_plugin = create_example_command_plugin();
        let tool_plugin = create_example_tool_plugin();
        
        // Register plugins
        manager.register_plugin(command_plugin.clone()).await.unwrap();
        manager.register_plugin(tool_plugin.clone()).await.unwrap();
        
        // Check plugin count
        let plugins = manager.get_all_plugins().await.unwrap();
        assert_eq!(plugins.len(), 2);
        
        // Check command plugin registration
        let plugin_id = command_plugin.metadata().id;
        let plugin = manager.get_plugin(&plugin_id).await.unwrap();
        assert_eq!(plugin.metadata().name, "example-command-plugin");
        
        // Check tool plugin registration
        let plugin_id = tool_plugin.metadata().id;
        let plugin = manager.get_plugin(&plugin_id).await.unwrap();
        assert_eq!(plugin.metadata().name, "example-tool-plugin");
    }
    
    #[tokio::test]
    async fn test_plugin_lifecycle() {
        // Create a plugin manager
        let manager = create_plugin_manager();
        
        // Create example plugins
        let command_plugin = create_example_command_plugin();
        let plugin_id = command_plugin.metadata().id;
        
        // Register plugin
        manager.register_plugin(command_plugin).await.unwrap();
        
        // Check initial status
        let status = manager.get_plugin_status(&plugin_id).await.unwrap();
        assert_eq!(status, PluginStatus::Registered);
        
        // Initialize plugin
        manager.initialize_plugin(&plugin_id).await.unwrap();
        let status = manager.get_plugin_status(&plugin_id).await.unwrap();
        assert_eq!(status, PluginStatus::Initialized);
        
        // Start plugin
        manager.start_plugin(&plugin_id).await.unwrap();
        let status = manager.get_plugin_status(&plugin_id).await.unwrap();
        assert_eq!(status, PluginStatus::Running);
        
        // Stop plugin
        manager.stop_plugin(&plugin_id).await.unwrap();
        let status = manager.get_plugin_status(&plugin_id).await.unwrap();
        assert_eq!(status, PluginStatus::Stopped);
    }
    
    #[tokio::test]
    async fn test_command_execution() {
        // Create a plugin manager
        let manager = create_plugin_manager();
        
        // Create example command plugin
        let command_plugin = create_example_command_plugin();
        let plugin_id = command_plugin.metadata().id;
        
        // Register and initialize the plugin
        manager.register_plugin(command_plugin).await.unwrap();
        manager.initialize_plugin(&plugin_id).await.unwrap();
        manager.start_plugin(&plugin_id).await.unwrap();
        
        // Execute the "hello" command
        let args = json!({
            "name": "DataScienceBioLab",
            "language": "en"
        });
        
        let result = manager.execute_command(&plugin_id, "hello", &args).await.unwrap();
        assert_eq!(result.get("greeting").unwrap(), "Hello, DataScienceBioLab!");
        
        // Execute the "hello" command with a different language
        let args = json!({
            "name": "DataScienceBioLab",
            "language": "es"
        });
        
        let result = manager.execute_command(&plugin_id, "hello", &args).await.unwrap();
        assert_eq!(result.get("greeting").unwrap(), "¡Hola, DataScienceBioLab!");
        
        // Execute the "echo" command
        let args = json!({
            "message": "Testing plugin system"
        });
        
        let result = manager.execute_command(&plugin_id, "echo", &args).await.unwrap();
        assert_eq!(result.get("message").unwrap(), "Testing plugin system");
    }
    
    #[tokio::test]
    async fn test_tool_execution() {
        // Create a plugin manager
        let manager = create_plugin_manager();
        
        // Create example tool plugin
        let tool_plugin = create_example_tool_plugin();
        let plugin_id = tool_plugin.metadata().id;
        
        // Register and initialize the plugin
        manager.register_plugin(tool_plugin).await.unwrap();
        manager.initialize_plugin(&plugin_id).await.unwrap();
        manager.start_plugin(&plugin_id).await.unwrap();
        
        // Execute the "calculator" tool with addition
        let args = json!({
            "a": 10,
            "b": 5,
            "operation": "add"
        });
        
        let result = manager.execute_tool(&plugin_id, "calculator", &args).await.unwrap();
        assert_eq!(result.get("result").unwrap().as_f64().unwrap(), 15.0);
        assert_eq!(result.get("operation").unwrap(), "add");
        
        // Execute the "calculator" tool with subtraction
        let args = json!({
            "a": 10,
            "b": 5,
            "operation": "subtract"
        });
        
        let result = manager.execute_tool(&plugin_id, "calculator", &args).await.unwrap();
        assert_eq!(result.get("result").unwrap().as_f64().unwrap(), 5.0);
        assert_eq!(result.get("operation").unwrap(), "subtract");
        
        // Execute the "calculator" tool with multiplication
        let args = json!({
            "a": 10,
            "b": 5,
            "operation": "multiply"
        });
        
        let result = manager.execute_tool(&plugin_id, "calculator", &args).await.unwrap();
        assert_eq!(result.get("result").unwrap().as_f64().unwrap(), 50.0);
        assert_eq!(result.get("operation").unwrap(), "multiply");
        
        // Execute the "calculator" tool with division
        let args = json!({
            "a": 10,
            "b": 5,
            "operation": "divide"
        });
        
        let result = manager.execute_tool(&plugin_id, "calculator", &args).await.unwrap();
        assert_eq!(result.get("result").unwrap().as_f64().unwrap(), 2.0);
        assert_eq!(result.get("operation").unwrap(), "divide");
        
        // Test division by zero error
        let args = json!({
            "a": 10,
            "b": 0,
            "operation": "divide"
        });
        
        let result = manager.execute_tool(&plugin_id, "calculator", &args).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_plugin_capabilities() {
        // Create a plugin manager
        let manager = create_plugin_manager();
        
        // Create example plugins
        let command_plugin = create_example_command_plugin();
        let tool_plugin = create_example_tool_plugin();
        
        // Register plugins
        manager.register_plugin(command_plugin).await.unwrap();
        manager.register_plugin(tool_plugin).await.unwrap();
        
        // Get plugins by capability
        let command_plugins = manager.get_plugins_by_capability("command").await.unwrap();
        assert_eq!(command_plugins.len(), 1);
        assert_eq!(command_plugins[0].metadata().name, "example-command-plugin");
        
        let tool_plugins = manager.get_plugins_by_capability("tool").await.unwrap();
        assert_eq!(tool_plugins.len(), 1);
        assert_eq!(tool_plugins[0].metadata().name, "example-tool-plugin");
    }
    
    #[tokio::test]
    async fn test_plugin_tags() {
        // Create a plugin manager
        let manager = create_plugin_manager();
        
        // Create example plugins
        let command_plugin = create_example_command_plugin();
        let tool_plugin = create_example_tool_plugin();
        
        // Register plugins
        manager.register_plugin(command_plugin).await.unwrap();
        manager.register_plugin(tool_plugin).await.unwrap();
        
        // Get plugins by tag
        let example_plugins = manager.get_plugins_by_tag("example").await.unwrap();
        assert_eq!(example_plugins.len(), 2);
        
        // The calculator tool has a math tag
        let math_plugins = manager.get_plugins_by_tag("math").await.unwrap();
        assert_eq!(math_plugins.len(), 1);
        assert_eq!(math_plugins[0].metadata().name, "example-tool-plugin");
    }
}

// Add tests for resource management and state persistence
#[cfg(test)]
mod resource_state_tests {
    use super::*;
    use crate::plugins::resource::{ResourceLimits, ResourceMonitorImpl, ResourceType};
    use crate::plugins::state::{DefaultStateManager, MemoryStateStorage, PluginState};
    use serde_json::json;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::timeout;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_resource_monitoring() {
        let monitor = Arc::new(ResourceMonitorImpl::new());
        let plugin_id = Uuid::new_v4();

        // Set limits
        let limits = ResourceLimits {
            max_memory: Some(10 * 1024 * 1024), // 10 MB
            max_cpu: Some(0.5),                 // 50% CPU
            max_disk: Some(5 * 1024 * 1024),    // 5 MB
            max_network: Some(1024 * 1024),     // 1 MB/s
            max_file_handles: Some(10),         // 10 files
            max_threads: Some(5),               // 5 threads
            ..Default::default()
        };

        assert!(monitor.set_limits(plugin_id, limits).await.is_ok());
        assert!(monitor.start_monitoring(plugin_id).await.is_ok());

        // Report resource allocation
        assert!(monitor
            .report_allocation(plugin_id, ResourceType::Memory, 1024 * 1024)
            .await
            .is_ok());

        // Check usage
        let usage = monitor.get_usage(plugin_id).await.unwrap();
        assert!(usage.memory >= 1024 * 1024);

        // Check limits (should not violate yet)
        let violations = monitor.check_limits(plugin_id).await.unwrap();
        assert!(violations.is_empty());

        // Report large allocation that exceeds limits
        assert!(monitor
            .report_allocation(plugin_id, ResourceType::Memory, 20 * 1024 * 1024)
            .await
            .is_ok());

        // Check violations
        let violations = monitor.check_limits(plugin_id).await.unwrap();
        assert!(!violations.is_empty());
        assert_eq!(violations[0].resource_type, ResourceType::Memory);

        // Clean up
        assert!(monitor.stop_monitoring(plugin_id).await.is_ok());
    }

    #[tokio::test]
    async fn test_state_management() {
        let storage = Arc::new(MemoryStateStorage::new());
        let manager = Arc::new(DefaultStateManager::new(storage));
        let plugin_id = Uuid::new_v4();

        // Initial state should be None
        let state = manager.load_state(plugin_id).await.unwrap();
        assert!(state.is_none());

        // Create initial state
        let initial_state = PluginState::new(plugin_id, "1.0.0", json!({ "count": 0 }));
        assert!(manager.save_state(initial_state).await.is_ok());

        // Load state
        let state = manager.load_state(plugin_id).await.unwrap().unwrap();
        assert_eq!(state.version, "1.0.0");
        assert_eq!(state.data["count"], 0);

        // Update state
        let updated_state = manager
            .update_state(plugin_id, json!({ "count": 1 }))
            .await
            .unwrap();
        assert_eq!(updated_state.data["count"], 1);

        // Verify updated state
        let state = manager.load_state(plugin_id).await.unwrap().unwrap();
        assert_eq!(state.data["count"], 1);

        // Begin transaction
        let mut transaction = manager.begin_transaction(plugin_id).await.unwrap();
        assert!(transaction.current_state().is_some());
        
        // Update in transaction
        transaction.update_state(json!({ "count": 2 })).unwrap();
        
        // State should not be updated yet
        let state = manager.load_state(plugin_id).await.unwrap().unwrap();
        assert_eq!(state.data["count"], 1);
        
        // Commit transaction
        let committed_state = manager.commit_transaction(transaction).await.unwrap().unwrap();
        assert_eq!(committed_state.data["count"], 2);
        
        // Verify after commit
        let state = manager.load_state(plugin_id).await.unwrap().unwrap();
        assert_eq!(state.data["count"], 2);

        // Delete state
        assert!(manager.delete_state(plugin_id).await.is_ok());

        // State should be gone
        let state = manager.load_state(plugin_id).await.unwrap();
        assert!(state.is_none());
    }

    #[tokio::test]
    async fn test_resource_state_integration() {
        // Create resource monitor
        let resource_monitor = Arc::new(ResourceMonitorImpl::new());
        
        // Create state manager
        let state_storage = Arc::new(MemoryStateStorage::new());
        let state_manager = Arc::new(DefaultStateManager::new(state_storage));
        
        // Plugin ID
        let plugin_id = Uuid::new_v4();
        
        // Set resource limits
        let limits = ResourceLimits::default();
        assert!(resource_monitor.set_limits(plugin_id, limits).await.is_ok());
        assert!(resource_monitor.start_monitoring(plugin_id).await.is_ok());
        
        // Create initial state
        let initial_state = PluginState::new(
            plugin_id, 
            "1.0.0", 
            json!({
                "count": 0,
                "resources": {
                    "memory_allocated": 0,
                    "disk_allocated": 0
                }
            })
        );
        assert!(state_manager.save_state(initial_state).await.is_ok());
        
        // Simulate resource allocation and state update
        let memory_size = 5 * 1024 * 1024; // 5 MB
        
        // Start a transaction
        let mut transaction = state_manager.begin_transaction(plugin_id).await.unwrap();
        
        // Update state with allocation info
        let state = transaction.current_state().unwrap();
        let mut resources = state.data["resources"].clone();
        resources["memory_allocated"] = json!(memory_size);
        
        let mut new_data = state.data.clone();
        new_data["resources"] = resources;
        
        transaction.update_state(new_data).unwrap();
        
        // Simulate resource allocation
        assert!(resource_monitor
            .report_allocation(plugin_id, ResourceType::Memory, memory_size)
            .await
            .is_ok());
        
        // Commit transaction
        assert!(state_manager.commit_transaction(transaction).await.is_ok());
        
        // Verify state
        let state = state_manager.load_state(plugin_id).await.unwrap().unwrap();
        assert_eq!(state.data["resources"]["memory_allocated"], memory_size);
        
        // Verify resource usage
        let usage = resource_monitor.get_usage(plugin_id).await.unwrap();
        assert!(usage.memory >= memory_size as u64);
        
        // Clean up
        assert!(resource_monitor.stop_monitoring(plugin_id).await.is_ok());
        assert!(state_manager.delete_state(plugin_id).await.is_ok());
    }
} 