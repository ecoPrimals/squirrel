use async_trait::async_trait;
use anyhow::Result;
use serde_json::Value;
use std::any::Any;
use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;

use squirrel_plugins::{
    PluginV2, PluginCallbacks, PluginMetadata, Plugin, WebEndpoint,
    PluginManager, DefaultPluginManager, adapt_plugin_v2, PluginRegistry,
};

/// A simple example plugin implementing PluginV2
#[derive(Debug)]
struct ExamplePluginV2 {
    metadata: PluginMetadata,
    state: Mutex<ExamplePluginState>,
    callbacks: Mutex<PluginCallbacks>,
}

/// Internal state for the example plugin
#[derive(Debug, Default)]
struct ExamplePluginState {
    initialized: bool,
    counter: usize,
    config: Value,
}

impl ExamplePluginV2 {
    /// Create a new example plugin
    fn new(name: &str, version: &str) -> Self {
        let metadata = PluginMetadata::new(name, version, "Example V2 Plugin", "Example Author")
            .with_capability("example")
            .with_capability("v2-pattern");
            
        Self {
            metadata,
            state: Mutex::new(ExamplePluginState::default()),
            callbacks: Mutex::new(PluginCallbacks::default()),
        }
    }
    
    /// Log a message using the callback if available
    fn log(&self, level: &str, message: &str) -> Result<()> {
        let callbacks = self.callbacks.lock().unwrap();
        if let Some(log) = &callbacks.log {
            log(level, message)
        } else {
            println!("[{}] {}", level, message);
            Ok(())
        }
    }
    
    /// Increment counter and return the new value
    fn increment_counter(&self) -> usize {
        let mut state = self.state.lock().unwrap();
        state.counter += 1;
        state.counter
    }
    
    /// Get current counter value
    fn get_counter(&self) -> usize {
        let state = self.state.lock().unwrap();
        state.counter
    }
    
    /// Save plugin state
    async fn save_state(&self) -> Result<()> {
        let callbacks = self.callbacks.lock().unwrap();
        let state = self.state.lock().unwrap();
        
        if let Some(persist) = &callbacks.persist_state {
            let state_value = serde_json::json!({
                "counter": state.counter,
                "config": state.config,
            });
            
            persist(self.metadata.id, "state", state_value)?;
            self.log("info", &format!("State saved, counter: {}", state.counter))?;
        } else {
            self.log("warn", "No persist_state callback available")?;
        }
        
        Ok(())
    }
    
    /// Load plugin state
    async fn load_state(&self) -> Result<()> {
        let callbacks = self.callbacks.lock().unwrap();
        
        if let Some(load) = &callbacks.load_state {
            let state_value = load(self.metadata.id, "state")?;
            
            let mut plugin_state = self.state.lock().unwrap();
            if let Some(counter) = state_value.get("counter").and_then(|v| v.as_u64()) {
                plugin_state.counter = counter as usize;
            }
            
            if let Some(config) = state_value.get("config") {
                plugin_state.config = config.clone();
            }
            
            self.log("info", &format!("State loaded, counter: {}", plugin_state.counter))?;
        } else {
            self.log("warn", "No load_state callback available")?;
        }
        
        Ok(())
    }
}

#[async_trait]
impl PluginV2 for ExamplePluginV2 {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> Result<()> {
        self.log("info", &format!("Initializing plugin: {}", self.metadata.name))?;
        
        // Load state if available
        let _ = self.load_state().await;
        
        // Set initialized flag
        let mut state = self.state.lock().unwrap();
        state.initialized = true;
        
        self.log("info", &format!("Plugin initialized: {}", self.metadata.name))?;
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        self.log("info", &format!("Shutting down plugin: {}", self.metadata.name))?;
        
        // Save state
        let _ = self.save_state().await;
        
        // Clear initialized flag
        let mut state = self.state.lock().unwrap();
        state.initialized = false;
        
        self.log("info", &format!("Plugin shutdown: {}", self.metadata.name))?;
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn register_callbacks(&mut self, callbacks: PluginCallbacks) {
        let mut cb = self.callbacks.lock().unwrap();
        *cb = callbacks;
    }
}

/// A plugin manager that supports registering PluginV2 implementations
struct PluginManagerV2 {
    inner: DefaultPluginManager,
}

impl PluginManagerV2 {
    fn new() -> Self {
        Self {
            inner: DefaultPluginManager::default(),
        }
    }
    
    /// Register a V2 plugin with the manager
    async fn register_plugin_v2<T: PluginV2 + 'static>(&self, mut plugin: T) -> Result<Arc<dyn Plugin>> {
        // Create callbacks
        let inner_clone = self.inner.clone();
        let callbacks = PluginCallbacks {
            log: Some(Box::new(move |level, message| {
                println!("[{}] {}", level, message);
                Ok(())
            })),
            get_plugin: Some(Box::new(move |id| {
                tokio::runtime::Handle::current().block_on(inner_clone.get_plugin(id))
            })),
            get_plugin_by_name: Some(Box::new(move |name| {
                let inner = inner_clone.clone();
                tokio::runtime::Handle::current().block_on(inner.get_plugin_by_name(name))
            })),
            list_plugins: Some(Box::new(move || {
                let inner = inner_clone.clone();
                tokio::runtime::Handle::current().block_on(inner.list_plugins())
            })),
            persist_state: Some(Box::new(move |id, key, value| {
                println!("Persisting state for plugin {}: {} = {}", id, key, value);
                Ok(())
            })),
            load_state: Some(Box::new(move |id, key| {
                println!("Loading state for plugin {}: {}", id, key);
                // Return empty state for demo purposes
                Ok(serde_json::json!({
                    "counter": 42,
                    "config": { "demo": true }
                }))
            })),
            ..Default::default()
        };
        
        // Register callbacks with the plugin
        plugin.register_callbacks(callbacks);
        
        // Adapt and register
        let adapted_plugin = adapt_plugin_v2(plugin);
        self.inner.register_plugin(adapted_plugin.clone()).await?;
        
        Ok(adapted_plugin)
    }
    
    /// Initialize a plugin
    async fn initialize_plugin(&self, id: Uuid) -> Result<()> {
        self.inner.initialize_plugin(id).await
    }
    
    /// Shutdown a plugin
    async fn shutdown_plugin(&self, id: Uuid) -> Result<()> {
        self.inner.shutdown_plugin(id).await
    }
    
    /// Get a plugin by ID
    async fn get_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>> {
        self.inner.get_plugin(id).await
    }
    
    /// List all plugins
    async fn list_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>> {
        self.inner.list_plugins().await
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create plugin manager
    let manager = PluginManagerV2::new();
    
    // Create and register a V2 plugin
    let plugin = ExamplePluginV2::new("example-v2-plugin", "1.0.0");
    let registered_plugin = manager.register_plugin_v2(plugin).await?;
    
    println!("Registered plugin: {}", registered_plugin.metadata().name);
    
    // Initialize plugin
    let id = registered_plugin.metadata().id;
    manager.initialize_plugin(id).await?;
    
    // Get plugin and test type conversion
    let plugin = manager.get_plugin(id).await?;
    
    // Type demonstration: How to downcast to the original type if needed
    // Note: This is for demonstration purposes - in real code, you should
    // avoid downcasting when possible and use callbacks for communication
    if let Some(original) = plugin.as_any().downcast_ref::<ExamplePluginV2>() {
        // Can now use original methods
        let counter = original.increment_counter();
        println!("Incremented counter: {}", counter);
        
        let counter = original.increment_counter();
        println!("Incremented counter again: {}", counter);
        
        let final_counter = original.get_counter();
        println!("Final counter value: {}", final_counter);
    } else {
        println!("Failed to downcast plugin");
    }
    
    // Shutdown plugin
    manager.shutdown_plugin(id).await?;
    
    println!("Plugin example completed successfully");
    Ok(())
} 