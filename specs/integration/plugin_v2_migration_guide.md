---
version: 1.0.0
last_updated: 2024-04-04
status: active
priority: high
---

# PluginV2 Migration Guide

## Overview

This guide provides step-by-step instructions for migrating existing plugin implementations from the original `Plugin` trait to the new thread-safe `PluginV2` trait. The new trait addresses critical thread-safety issues by using callbacks instead of direct adapter references and by making thread-safety requirements explicit.

## Key Differences Between Plugin and PluginV2

The `PluginV2` trait includes these key improvements:

1. **Explicit Thread Safety**: Requires implementers to be `Send + Sync`.
2. **Callback-Based Interaction**: Uses callbacks instead of adapter references.
3. **Improved Error Handling**: Consistent approach to error handling.
4. **Web Plugin Extension**: Cleaner separation for web plugin functionality.
5. **Enhanced State Management**: Better support for plugin state.

## Migration Steps

### Step 1: Review Your Existing Plugin Implementation

Examine your current plugin implementation to identify:

- Direct adapter references that need to be replaced with callbacks
- Methods that need to be updated for thread safety
- State management that might need adjustment

Here's an example of a typical `Plugin` implementation:

```rust
#[derive(Debug)]
struct MyPlugin {
    metadata: PluginMetadata,
    adapter: Option<Arc<dyn PluginAdapter>>,
}

#[async_trait]
impl Plugin for MyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> Result<()> {
        if let Some(adapter) = &self.adapter {
            adapter.log("info", "Plugin initialized");
        }
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        if let Some(adapter) = &self.adapter {
            adapter.log("info", "Plugin shutdown");
        }
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn set_adapter(&mut self, adapter: Arc<dyn PluginAdapter>) {
        self.adapter = Some(adapter);
    }
}
```

### Step 2: Create a New PluginV2 Implementation

Create a new struct that will implement `PluginV2`:

```rust
#[derive(Debug)]
struct MyPluginV2 {
    metadata: PluginMetadata,
    // State should be thread-safe
    state: Arc<Mutex<HashMap<String, String>>>,
    // Store callbacks instead of adapter
    callbacks: Option<PluginCallbacks>,
}

impl MyPluginV2 {
    fn new(name: &str) -> Self {
        Self {
            metadata: PluginMetadata::new(
                name,
                "1.0.0",
                "My plugin with improved thread safety",
                "Plugin Author"
            ),
            state: Arc::new(Mutex::new(HashMap::new())),
            callbacks: None,
        }
    }
    
    // Helper method for logging
    fn log(&self, level: &str, message: &str) {
        if let Some(callbacks) = &self.callbacks {
            if let Some(log_fn) = &callbacks.log {
                let _ = log_fn(level, message);
            }
        }
    }
    
    // Helper method for state management
    fn set_state(&self, key: &str, value: &str) {
        let mut state = self.state.lock().unwrap();
        state.insert(key.to_string(), value.to_string());
    }
    
    fn get_state(&self, key: &str) -> Option<String> {
        let state = self.state.lock().unwrap();
        state.get(key).cloned()
    }
}
```

### Step 3: Implement the PluginV2 Trait

Implement the `PluginV2` trait for your new struct:

```rust
#[async_trait]
impl PluginV2 for MyPluginV2 {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> Result<()> {
        self.log("info", "Initializing plugin v2");
        self.set_state("startup_time", &chrono::Utc::now().to_rfc3339());
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        self.log("info", "Shutting down plugin v2");
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn register_callbacks(&mut self, callbacks: PluginCallbacks) {
        self.callbacks = Some(callbacks);
    }
}
```

### Step 4: Implement Web Functionality (if needed)

If your plugin provides web endpoints, implement the `WebPluginExtV2` trait:

```rust
#[async_trait]
impl WebPluginExtV2 for MyPluginV2 {
    fn get_endpoints(&self) -> Vec<WebEndpoint> {
        vec![
            WebEndpoint {
                method: "GET".to_string(),
                path: "/myplugin/status".to_string(),
                permissions: vec![],
            },
            WebEndpoint {
                method: "GET".to_string(),
                path: "/myplugin/state".to_string(),
                permissions: vec![],
            },
        ]
    }
    
    async fn handle_web_endpoint(&self, endpoint: &WebEndpoint, data: Option<Value>) -> Result<Value> {
        match endpoint.path.as_str() {
            "/myplugin/status" => {
                self.log("info", "Handling status request");
                Ok(serde_json::json!({ "status": "active" }))
            },
            "/myplugin/state" => {
                self.log("info", "Handling state request");
                let startup_time = self.get_state("startup_time")
                    .unwrap_or_else(|| "unknown".to_string());
                Ok(serde_json::json!({ "startup_time": startup_time }))
            },
            _ => {
                Err(anyhow::anyhow!("Unknown endpoint: {}", endpoint.path))
            }
        }
    }
}
```

### Step 5: Update Plugin Creation and Registration

Update your plugin creation and registration code:

```rust
// Create the V2 plugin
let mut plugin_v2 = MyPluginV2::new("my-plugin");

// Set up callbacks
let callbacks = PluginCallbacks {
    log: Some(Box::new(|level, message| {
        println!("[{}] {}", level, message);
        Ok(())
    })),
    persist_state: Some(Box::new(|plugin_id, key, value| {
        println!("Persisting state for plugin {}: {} = {}", plugin_id, key, value);
        Ok(())
    })),
    load_state: Some(Box::new(|plugin_id, key| {
        println!("Loading state for plugin {}: {}", plugin_id, key);
        Ok(serde_json::Value::Null)
    })),
    ..Default::default()
};

// Register callbacks
plugin_v2.register_callbacks(callbacks);

// Use the plugin directly
plugin_v2.initialize().await?;

// Or adapt to original Plugin trait for compatibility with existing code
let adapted_plugin: Arc<dyn Plugin> = adapt_plugin_v2(plugin_v2);
```

### Step 6: Update Tests

Update your tests to use the new `PluginV2` trait:

```rust
#[tokio::test]
async fn test_plugin_v2() {
    // Create the V2 plugin
    let mut plugin_v2 = MyPluginV2::new("test-plugin");
    
    // Capture log messages for testing
    let log_messages = Arc::new(Mutex::new(Vec::new()));
    
    // Set up callbacks for testing
    let log_messages_clone = log_messages.clone();
    let callbacks = PluginCallbacks {
        log: Some(Box::new(move |level, message| {
            let mut messages = log_messages_clone.lock().unwrap();
            messages.push(format!("[{}] {}", level, message));
            Ok(())
        })),
        ..Default::default()
    };
    
    // Register callbacks
    plugin_v2.register_callbacks(callbacks);
    
    // Test initialize
    plugin_v2.initialize().await.unwrap();
    
    // Verify logging
    let messages = log_messages.lock().unwrap();
    assert!(messages.contains(&"[info] Initializing plugin v2".to_string()));
    
    // Test state management
    assert!(plugin_v2.get_state("startup_time").is_some());
    
    // Test shutdown
    plugin_v2.shutdown().await.unwrap();
}
```

## Adapter Pattern for Existing Code

If you need to use your new `PluginV2` implementation with code that expects the original `Plugin` trait, you can use the adapter pattern:

```rust
// Create and configure your V2 plugin
let mut plugin_v2 = MyPluginV2::new("my-plugin");

// Set up callbacks
let callbacks = PluginCallbacks { /* ... */ };
plugin_v2.register_callbacks(callbacks);

// Adapt to the original Plugin trait
let plugin: Arc<dyn Plugin> = adapt_plugin_v2(plugin_v2);

// Now use with existing code that expects Plugin
existing_code.register_plugin(plugin);
```

## Common Migration Challenges

### 1. State Management

If your plugin maintains state, ensure it uses thread-safe containers:

```rust
// Before
struct MyPlugin {
    state: HashMap<String, String>,
}

// After
struct MyPluginV2 {
    state: Arc<Mutex<HashMap<String, String>>>,
}
```

### 2. Logging and Callback Usage

Replace direct adapter method calls with callback usage:

```rust
// Before
fn log_something(&self, message: &str) {
    if let Some(adapter) = &self.adapter {
        adapter.log("info", message);
    }
}

// After
fn log_something(&self, message: &str) {
    if let Some(callbacks) = &self.callbacks {
        if let Some(log_fn) = &callbacks.log {
            let _ = log_fn("info", message);
        }
    }
}
```

### 3. Debug Implementation

Ensure your plugin implements `Debug` properly:

```rust
impl std::fmt::Debug for MyPluginV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MyPluginV2")
            .field("metadata", &self.metadata)
            .field("state", &self.state)
            .field("callbacks", &"<callbacks>") // Placeholder for callbacks
            .finish()
    }
}
```

### 4. Using Plugin Registry

Replace direct adapter access to the registry with callbacks:

```rust
// Before
fn get_other_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>> {
    if let Some(adapter) = &self.adapter {
        adapter.get_plugin(id)
    } else {
        Err(anyhow::anyhow!("No adapter set"))
    }
}

// After
fn get_other_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>> {
    if let Some(callbacks) = &self.callbacks {
        if let Some(get_plugin_fn) = &callbacks.get_plugin {
            get_plugin_fn(id)
        } else {
            Err(anyhow::anyhow!("No get_plugin callback registered"))
        }
    } else {
        Err(anyhow::anyhow!("No callbacks registered"))
    }
}
```

## Complete Migration Example

Here's a complete before-and-after example:

### Before: Original Plugin Implementation

```rust
struct MyPlugin {
    metadata: PluginMetadata,
    adapter: Option<Arc<dyn PluginAdapter>>,
    config: HashMap<String, String>,
}

impl MyPlugin {
    fn new(name: &str) -> Self {
        Self {
            metadata: PluginMetadata::new(name, "1.0.0", "Example plugin", "Example Author"),
            adapter: None,
            config: HashMap::new(),
        }
    }
    
    fn log(&self, level: &str, message: &str) {
        if let Some(adapter) = &self.adapter {
            adapter.log(level, message);
        }
    }
}

#[async_trait]
impl Plugin for MyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> Result<()> {
        self.log("info", "Plugin initialized");
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        self.log("info", "Plugin shutdown");
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn set_adapter(&mut self, adapter: Arc<dyn PluginAdapter>) {
        self.adapter = Some(adapter);
    }
}

#[async_trait]
impl WebPluginExt for MyPlugin {
    fn get_endpoints(&self) -> Vec<WebEndpoint> {
        vec![
            WebEndpoint {
                method: "GET".to_string(),
                path: "/myplugin/status".to_string(),
                permissions: vec![],
            },
        ]
    }
    
    async fn handle_web_endpoint(&self, endpoint: &WebEndpoint, _data: Option<Value>) -> Result<Value> {
        match endpoint.path.as_str() {
            "/myplugin/status" => {
                Ok(serde_json::json!({ "status": "ok" }))
            },
            _ => {
                Err(anyhow::anyhow!("Unknown endpoint"))
            }
        }
    }
}
```

### After: PluginV2 Implementation

```rust
struct MyPluginV2 {
    metadata: PluginMetadata,
    state: Arc<Mutex<HashMap<String, String>>>,
    callbacks: Option<PluginCallbacks>,
}

impl std::fmt::Debug for MyPluginV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MyPluginV2")
            .field("metadata", &self.metadata)
            .field("state", &self.state)
            .field("callbacks", &"<callbacks>")
            .finish()
    }
}

impl MyPluginV2 {
    fn new(name: &str) -> Self {
        Self {
            metadata: PluginMetadata::new(name, "1.0.0", "Example plugin v2", "Example Author"),
            state: Arc::new(Mutex::new(HashMap::new())),
            callbacks: None,
        }
    }
    
    fn log(&self, level: &str, message: &str) {
        if let Some(callbacks) = &self.callbacks {
            if let Some(log_fn) = &callbacks.log {
                let _ = log_fn(level, message);
            }
        }
    }
    
    fn set_state(&self, key: &str, value: &str) {
        let mut state = self.state.lock().unwrap();
        state.insert(key.to_string(), value.to_string());
    }
    
    fn get_state(&self, key: &str) -> Option<String> {
        let state = self.state.lock().unwrap();
        state.get(key).cloned()
    }
}

#[async_trait]
impl PluginV2 for MyPluginV2 {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> Result<()> {
        self.log("info", "Initializing plugin v2");
        self.set_state("startup_time", &chrono::Utc::now().to_rfc3339());
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        self.log("info", "Shutting down plugin v2");
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn register_callbacks(&mut self, callbacks: PluginCallbacks) {
        self.callbacks = Some(callbacks);
    }
}

#[async_trait]
impl WebPluginExtV2 for MyPluginV2 {
    fn get_endpoints(&self) -> Vec<WebEndpoint> {
        vec![
            WebEndpoint {
                method: "GET".to_string(),
                path: "/myplugin/status".to_string(),
                permissions: vec![],
            },
            WebEndpoint {
                method: "GET".to_string(),
                path: "/myplugin/state".to_string(),
                permissions: vec![],
            },
        ]
    }
    
    async fn handle_web_endpoint(&self, endpoint: &WebEndpoint, _data: Option<Value>) -> Result<Value> {
        match endpoint.path.as_str() {
            "/myplugin/status" => {
                self.log("info", "Handling status request");
                Ok(serde_json::json!({ "status": "active" }))
            },
            "/myplugin/state" => {
                self.log("info", "Handling state request");
                let startup_time = self.get_state("startup_time")
                    .unwrap_or_else(|| "unknown".to_string());
                Ok(serde_json::json!({ "startup_time": startup_time }))
            },
            _ => {
                Err(anyhow::anyhow!("Unknown endpoint: {}", endpoint.path))
            }
        }
    }
}

// Usage
fn create_and_use_plugin() -> Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    
    runtime.block_on(async {
        // Create V2 plugin
        let mut plugin_v2 = MyPluginV2::new("my-plugin");
        
        // Set up callbacks
        let callbacks = PluginCallbacks {
            log: Some(Box::new(|level, message| {
                println!("[{}] {}", level, message);
                Ok(())
            })),
            ..Default::default()
        };
        
        // Register callbacks
        plugin_v2.register_callbacks(callbacks);
        
        // Initialize the plugin
        plugin_v2.initialize().await?;
        
        // Use the plugin
        let response = plugin_v2.handle_web_endpoint(
            &WebEndpoint {
                method: "GET".to_string(),
                path: "/myplugin/status".to_string(),
                permissions: vec![],
            },
            None
        ).await?;
        
        println!("Response: {}", response);
        
        // Adapt to original Plugin trait if needed
        let plugin: Arc<dyn Plugin> = adapt_plugin_v2(plugin_v2);
        
        // Use with existing code that expects Plugin
        // existing_code.register_plugin(plugin);
        
        Ok(())
    })
}
```

## Conclusion

Migrating to the `PluginV2` trait offers significant benefits in terms of thread safety, testability, and maintainability. By following this guide, you can convert your existing plugins to the new pattern while maintaining backward compatibility.

The callback-based approach ensures proper thread safety while providing a clean separation of concerns. This approach aligns with Rust's philosophy of making important requirements like thread safety explicit in the type system.

<version>1.0.0</version> 