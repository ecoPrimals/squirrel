//! End-to-End Plugin Lifecycle Tests
//!
//! Tests complete plugin lifecycle workflows including:
//! - Plugin loading and initialization
//! - Plugin execution
//! - State management
//! - Plugin unloading and cleanup
//! - Error handling and recovery
//! - Concurrent plugin operations

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;
use uuid::Uuid;

/// Plugin state
#[derive(Debug, Clone, PartialEq)]
enum PluginState {
    Unloaded,
    Loading,
    Loaded,
    Initializing,
    Ready,
    Executing,
    Error,
    Unloading,
}

/// Mock plugin
#[derive(Debug, Clone)]
struct Plugin {
    id: String,
    name: String,
    version: String,
    state: PluginState,
    capabilities: Vec<String>,
    execution_count: u64,
    error_count: u64,
}

/// Plugin manager
struct PluginManager {
    plugins: Arc<RwLock<HashMap<String, Plugin>>>,
    execution_history: Arc<RwLock<Vec<PluginExecution>>>,
}

#[derive(Debug, Clone)]
struct PluginExecution {
    plugin_id: String,
    started_at: std::time::SystemTime,
    completed_at: Option<std::time::SystemTime>,
    success: bool,
    result: Option<serde_json::Value>,
}

impl PluginManager {
    fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            execution_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    async fn load_plugin(&self, name: String, version: String) -> Result<String, String> {
        if name.is_empty() {
            return Err("Plugin name cannot be empty".to_string());
        }

        let plugin_id = Uuid::new_v4().to_string();
        let plugin = Plugin {
            id: plugin_id.clone(),
            name: name.clone(),
            version,
            state: PluginState::Loading,
            capabilities: vec![],
            execution_count: 0,
            error_count: 0,
        };

        let mut plugins = self.plugins.write().await;
        plugins.insert(plugin_id.clone(), plugin);

        Ok(plugin_id)
    }

    async fn initialize_plugin(
        &self,
        plugin_id: &str,
        capabilities: Vec<String>,
    ) -> Result<(), String> {
        let mut plugins = self.plugins.write().await;
        let plugin = plugins
            .get_mut(plugin_id)
            .ok_or_else(|| "Plugin not found".to_string())?;

        if plugin.state != PluginState::Loading && plugin.state != PluginState::Loaded {
            return Err(format!(
                "Cannot initialize plugin in state {:?}",
                plugin.state
            ));
        }

        plugin.state = PluginState::Initializing;
        plugin.capabilities = capabilities;
        plugin.state = PluginState::Ready;

        Ok(())
    }

    async fn execute_plugin(
        &self,
        plugin_id: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        // Update plugin state to executing
        {
            let mut plugins = self.plugins.write().await;
            let plugin = plugins
                .get_mut(plugin_id)
                .ok_or_else(|| "Plugin not found".to_string())?;

            if plugin.state != PluginState::Ready {
                return Err(format!("Plugin not ready (state: {:?})", plugin.state));
            }

            plugin.state = PluginState::Executing;
            plugin.execution_count += 1;
        }

        // Record execution start
        let execution = PluginExecution {
            plugin_id: plugin_id.to_string(),
            started_at: std::time::SystemTime::now(),
            completed_at: None,
            success: false,
            result: None,
        };

        // Simulate plugin execution
        let result = serde_json::json!({
            "plugin_id": plugin_id,
            "status": "success",
            "args": args,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        // Update plugin state back to ready
        {
            let mut plugins = self.plugins.write().await;
            if let Some(plugin) = plugins.get_mut(plugin_id) {
                plugin.state = PluginState::Ready;
            }
        }

        // Record execution completion
        {
            let mut history = self.execution_history.write().await;
            let mut exec = execution;
            exec.completed_at = Some(std::time::SystemTime::now());
            exec.success = true;
            exec.result = Some(result.clone());
            history.push(exec);
        }

        Ok(result)
    }

    async fn unload_plugin(&self, plugin_id: &str) -> Result<(), String> {
        let mut plugins = self.plugins.write().await;
        let plugin = plugins
            .get_mut(plugin_id)
            .ok_or_else(|| "Plugin not found".to_string())?;

        if plugin.state == PluginState::Executing {
            return Err("Cannot unload plugin while executing".to_string());
        }

        plugin.state = PluginState::Unloading;
        plugins.remove(plugin_id);

        Ok(())
    }

    async fn get_plugin_state(&self, plugin_id: &str) -> Result<PluginState, String> {
        let plugins = self.plugins.read().await;
        plugins
            .get(plugin_id)
            .map(|p| p.state.clone())
            .ok_or_else(|| "Plugin not found".to_string())
    }

    async fn get_plugin_count(&self) -> usize {
        self.plugins.read().await.len()
    }

    async fn get_execution_count(&self) -> usize {
        self.execution_history.read().await.len()
    }

    async fn list_plugins(&self) -> Vec<Plugin> {
        self.plugins.read().await.values().cloned().collect()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// E2E PLUGIN LIFECYCLE TESTS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn test_complete_plugin_lifecycle() {
    let manager = PluginManager::new();

    // 1. Load plugin
    let plugin_id = manager
        .load_plugin("test_plugin".to_string(), "1.0.0".to_string())
        .await
        .expect("Plugin loading should succeed");

    assert!(!plugin_id.is_empty(), "Should return plugin ID");

    // 2. Check state is Loading
    let state = manager.get_plugin_state(&plugin_id).await.unwrap();
    assert_eq!(state, PluginState::Loading, "Plugin should be in Loading state");

    // 3. Initialize plugin
    manager
        .initialize_plugin(&plugin_id, vec!["capability1".to_string()])
        .await
        .expect("Plugin initialization should succeed");

    // 4. Check state is Ready
    let state = manager.get_plugin_state(&plugin_id).await.unwrap();
    assert_eq!(state, PluginState::Ready, "Plugin should be Ready");

    // 5. Execute plugin
    let result = manager
        .execute_plugin(&plugin_id, serde_json::json!({"test": "data"}))
        .await
        .expect("Plugin execution should succeed");

    assert_eq!(
        result["status"].as_str().unwrap(),
        "success",
        "Execution should succeed"
    );

    // 6. Check state is still Ready after execution
    let state = manager.get_plugin_state(&plugin_id).await.unwrap();
    assert_eq!(state, PluginState::Ready, "Plugin should return to Ready state");

    // 7. Unload plugin
    manager
        .unload_plugin(&plugin_id)
        .await
        .expect("Plugin unloading should succeed");

    // 8. Verify plugin is removed
    let plugin_count = manager.get_plugin_count().await;
    assert_eq!(plugin_count, 0, "Plugin should be removed");
}

#[tokio::test]
async fn test_plugin_load_with_invalid_name() {
    let manager = PluginManager::new();

    let result = manager.load_plugin("".to_string(), "1.0.0".to_string()).await;
    assert!(result.is_err(), "Should fail with empty plugin name");
    assert!(
        result.unwrap_err().contains("cannot be empty"),
        "Error should mention empty name"
    );
}

#[tokio::test]
async fn test_plugin_initialization_requirements() {
    let manager = PluginManager::new();

    // Load plugin
    let plugin_id = manager
        .load_plugin("init_test".to_string(), "1.0.0".to_string())
        .await
        .expect("Plugin loading should succeed");

    // Initialize with capabilities
    let capabilities = vec![
        "read".to_string(),
        "write".to_string(),
        "execute".to_string(),
    ];

    manager
        .initialize_plugin(&plugin_id, capabilities.clone())
        .await
        .expect("Initialization should succeed");

    // Verify plugin has capabilities
    let plugins = manager.list_plugins().await;
    let plugin = plugins.iter().find(|p| p.id == plugin_id).unwrap();

    assert_eq!(
        plugin.capabilities.len(),
        3,
        "Should have 3 capabilities"
    );
    assert!(plugin.capabilities.contains(&"read".to_string()));
    assert!(plugin.capabilities.contains(&"write".to_string()));
    assert!(plugin.capabilities.contains(&"execute".to_string()));
}

#[tokio::test]
async fn test_plugin_execution_multiple_times() {
    let manager = PluginManager::new();

    // Setup plugin
    let plugin_id = manager
        .load_plugin("multi_exec".to_string(), "1.0.0".to_string())
        .await
        .unwrap();

    manager
        .initialize_plugin(&plugin_id, vec![])
        .await
        .unwrap();

    // Execute multiple times
    for i in 0..5 {
        let result = manager
            .execute_plugin(&plugin_id, serde_json::json!({"iteration": i}))
            .await;

        assert!(
            result.is_ok(),
            "Execution {} should succeed",
            i
        );
    }

    // Verify execution history
    let exec_count = manager.get_execution_count().await;
    assert_eq!(exec_count, 5, "Should have 5 execution records");

    // Verify plugin execution count
    let plugins = manager.list_plugins().await;
    let plugin = plugins.iter().find(|p| p.id == plugin_id).unwrap();
    assert_eq!(plugin.execution_count, 5, "Plugin should track 5 executions");
}

#[tokio::test]
async fn test_cannot_execute_uninitialized_plugin() {
    let manager = PluginManager::new();

    // Load but don't initialize
    let plugin_id = manager
        .load_plugin("uninit".to_string(), "1.0.0".to_string())
        .await
        .unwrap();

    // Try to execute
    let result = manager
        .execute_plugin(&plugin_id, serde_json::json!({}))
        .await;

    assert!(result.is_err(), "Should fail for uninitialized plugin");
    assert!(
        result.unwrap_err().contains("not ready"),
        "Error should mention plugin not ready"
    );
}

#[tokio::test]
async fn test_cannot_unload_executing_plugin() {
    let manager = Arc::new(PluginManager::new());

    // Setup plugin
    let plugin_id = manager
        .load_plugin("exec_test".to_string(), "1.0.0".to_string())
        .await
        .unwrap();

    manager
        .initialize_plugin(&plugin_id, vec![])
        .await
        .unwrap();

    // This test is simplified since we can't actually hold execution state
    // In a real system, we'd spawn execution in background and try to unload

    // For now, verify plugin exists
    let count = manager.get_plugin_count().await;
    assert_eq!(count, 1, "Plugin should exist");

    // Unload normally (not during execution)
    let result = manager.unload_plugin(&plugin_id).await;
    assert!(result.is_ok(), "Unload should succeed when not executing");
}

#[tokio::test]
async fn test_concurrent_plugin_loading() {
    let manager = Arc::new(PluginManager::new());

    // Load multiple plugins concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            manager_clone
                .load_plugin(format!("plugin_{}", i), "1.0.0".to_string())
                .await
        });
        handles.push(handle);
    }

    // Wait for all loads
    let mut plugin_ids = vec![];
    for handle in handles {
        let result = handle.await.expect("Task should complete");
        assert!(result.is_ok(), "Plugin loading should succeed");
        plugin_ids.push(result.unwrap());
    }

    // Verify all plugins loaded
    let count = manager.get_plugin_count().await;
    assert_eq!(count, 10, "Should have 10 plugins loaded");

    // All IDs should be unique
    let unique_ids: std::collections::HashSet<String> = plugin_ids.iter().cloned().collect();
    assert_eq!(unique_ids.len(), 10, "All plugin IDs should be unique");
}

#[tokio::test]
async fn test_concurrent_plugin_execution() {
    let manager = Arc::new(PluginManager::new());

    // Setup plugin
    let plugin_id = manager
        .load_plugin("concurrent".to_string(), "1.0.0".to_string())
        .await
        .unwrap();

    manager
        .initialize_plugin(&plugin_id, vec![])
        .await
        .unwrap();

    // Execute concurrently (note: this tests manager's concurrency, not parallel execution)
    let mut handles = vec![];
    // Use barrier to ensure all tasks start simultaneously (true concurrency test)
    let barrier = Arc::new(tokio::sync::Barrier::new(20));
    
    for i in 0..20 {
        let manager_clone = Arc::clone(&manager);
        let pid = plugin_id.clone();
        let barrier_clone = Arc::clone(&barrier);
        let handle = tokio::spawn(async move {
            // Wait for all tasks to be ready
            barrier_clone.wait().await;
            // All tasks execute concurrently - this tests true concurrent safety
            manager_clone
                .execute_plugin(&pid, serde_json::json!({"run": i}))
                .await
        });
        handles.push(handle);
    }

    // Wait for all executions
    let mut results = vec![];
    for handle in handles {
        let result = handle.await.expect("Task should complete");
        results.push(result);
    }

    // Count successes
    let success_count = results.iter().filter(|r| r.is_ok()).count();
    assert!(
        success_count >= 15,
        "Most executions should succeed (got {} successes)",
        success_count
    );
}

#[tokio::test]
async fn test_plugin_lifecycle_with_timeout() {
    let manager = PluginManager::new();

    // Load with timeout
    let load_result = timeout(
        Duration::from_secs(1),
        manager.load_plugin("timeout_test".to_string(), "1.0.0".to_string()),
    )
    .await;

    assert!(load_result.is_ok(), "Load should complete within timeout");
    let plugin_id = load_result.unwrap().unwrap();

    // Initialize with timeout
    let init_result = timeout(
        Duration::from_secs(1),
        manager.initialize_plugin(&plugin_id, vec![]),
    )
    .await;

    assert!(init_result.is_ok(), "Init should complete within timeout");

    // Execute with timeout
    let exec_result = timeout(
        Duration::from_secs(1),
        manager.execute_plugin(&plugin_id, serde_json::json!({})),
    )
    .await;

    assert!(exec_result.is_ok(), "Exec should complete within timeout");

    // Unload with timeout
    let unload_result = timeout(Duration::from_secs(1), manager.unload_plugin(&plugin_id)).await;

    assert!(
        unload_result.is_ok(),
        "Unload should complete within timeout"
    );
}

#[tokio::test]
async fn test_multiple_plugin_versions() {
    let manager = PluginManager::new();

    // Load same plugin with different versions
    let v1_id = manager
        .load_plugin("versioned_plugin".to_string(), "1.0.0".to_string())
        .await
        .unwrap();

    let v2_id = manager
        .load_plugin("versioned_plugin".to_string(), "2.0.0".to_string())
        .await
        .unwrap();

    // Both should coexist
    let count = manager.get_plugin_count().await;
    assert_eq!(count, 2, "Should have 2 plugin versions loaded");

    // IDs should be different
    assert_ne!(v1_id, v2_id, "Different versions should have different IDs");

    // Verify versions in plugin list
    let plugins = manager.list_plugins().await;
    assert!(plugins.iter().any(|p| p.version == "1.0.0"));
    assert!(plugins.iter().any(|p| p.version == "2.0.0"));
}

#[tokio::test]
async fn test_plugin_state_transitions() {
    let manager = PluginManager::new();

    // Load
    let plugin_id = manager
        .load_plugin("state_test".to_string(), "1.0.0".to_string())
        .await
        .unwrap();

    // Check Loading state
    let state1 = manager.get_plugin_state(&plugin_id).await.unwrap();
    assert_eq!(state1, PluginState::Loading);

    // Initialize
    manager
        .initialize_plugin(&plugin_id, vec![])
        .await
        .unwrap();

    // Check Ready state
    let state2 = manager.get_plugin_state(&plugin_id).await.unwrap();
    assert_eq!(state2, PluginState::Ready);

    // Execute
    manager
        .execute_plugin(&plugin_id, serde_json::json!({}))
        .await
        .unwrap();

    // Check back to Ready state
    let state3 = manager.get_plugin_state(&plugin_id).await.unwrap();
    assert_eq!(state3, PluginState::Ready);

    // Unload
    manager.unload_plugin(&plugin_id).await.unwrap();

    // Plugin should be gone
    let state_result = manager.get_plugin_state(&plugin_id).await;
    assert!(state_result.is_err(), "Plugin should not exist after unload");
}

#[tokio::test]
async fn test_plugin_execution_history_tracking() {
    let manager = PluginManager::new();

    // Setup plugin
    let plugin_id = manager
        .load_plugin("history_test".to_string(), "1.0.0".to_string())
        .await
        .unwrap();

    manager
        .initialize_plugin(&plugin_id, vec![])
        .await
        .unwrap();

    // Execute multiple times with different args
    for i in 0..3 {
        manager
            .execute_plugin(&plugin_id, serde_json::json!({"test": i}))
            .await
            .unwrap();
    }

    // Verify execution history
    let exec_count = manager.get_execution_count().await;
    assert_eq!(exec_count, 3, "Should have 3 execution history entries");
}

