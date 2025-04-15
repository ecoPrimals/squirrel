use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::Value;
use anyhow::Result;

use squirrel_plugins::{
    PluginCallbacks, PluginMetadata, PluginV2, WebEndpoint, WebPluginExtV2,
    adapt_plugin_v2, Plugin,
};

/// A simple example plugin implementing PluginV2
struct ExamplePluginV2 {
    metadata: PluginMetadata,
    state: Arc<Mutex<HashMap<String, String>>>,
    callbacks: Option<PluginCallbacks>,
}

impl std::fmt::Debug for ExamplePluginV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExamplePluginV2")
            .field("metadata", &self.metadata)
            .field("state", &self.state)
            .field("callbacks", &"<callbacks>")
            .finish()
    }
}

impl ExamplePluginV2 {
    /// Create a new example plugin
    fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "Example Plugin V2",
                "0.1.0",
                "Example plugin demonstrating the PluginV2 trait",
                "Squirrel Team",
            ).with_capability("example"),
            state: Arc::new(Mutex::new(HashMap::new())),
            callbacks: None,
        }
    }
    
    /// Log a message using the callback if available
    fn log(&self, level: &str, message: &str) {
        if let Some(callbacks) = &self.callbacks {
            if let Some(log_fn) = &callbacks.log {
                log_fn(level, &format!("[ExamplePluginV2] {}", message)).ok();
            } else {
                println!("[ExamplePluginV2] [{level}] {message}");
            }
        } else {
            println!("[ExamplePluginV2] [{level}] {message}");
        }
    }
    
    /// Get current state
    fn get_state(&self, key: &str) -> Option<String> {
        self.state.lock().unwrap().get(key).cloned()
    }
    
    /// Set state
    fn set_state(&self, key: &str, value: &str) {
        self.state.lock().unwrap().insert(key.to_string(), value.to_string());
    }
}

#[async_trait]
impl PluginV2 for ExamplePluginV2 {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> Result<()> {
        self.log("info", "Initializing example plugin v2");
        self.set_state("startup_time", &chrono::Utc::now().to_rfc3339());
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        self.log("info", "Shutting down example plugin v2");
        Ok(())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn register_callbacks(&mut self, callbacks: PluginCallbacks) {
        self.callbacks = Some(callbacks);
    }
}

#[async_trait]
impl WebPluginExtV2 for ExamplePluginV2 {
    fn get_endpoints(&self) -> Vec<WebEndpoint> {
        vec![
            WebEndpoint {
                method: "GET".to_string(),
                path: "/example/v2/status".to_string(),
                permissions: vec![],
            },
            WebEndpoint {
                method: "GET".to_string(),
                path: "/example/v2/state".to_string(),
                permissions: vec![],
            },
        ]
    }
    
    async fn handle_web_endpoint(&self, endpoint: &WebEndpoint, _data: Option<Value>) -> Result<Value> {
        match endpoint.path.as_str() {
            "/example/v2/status" => {
                self.log("info", "Handling status request");
                Ok(serde_json::json!({ "status": "initialized" }))
            },
            "/example/v2/state" => {
                self.log("info", "Handling state request");
                Ok(serde_json::to_value(&*self.state.lock().unwrap())?)
            },
            _ => {
                self.log("error", &format!("Unknown path: {}", endpoint.path));
                Err(anyhow::anyhow!("Unknown path: {}", endpoint.path))
            }
        }
    }
}

fn main() -> Result<()> {
    // Create a tokio runtime
    let rt = tokio::runtime::Runtime::new()?;
    
    rt.block_on(async {
        println!("Plugin V2 Example");
        
        // Create a plugin
        let mut example_plugin = ExamplePluginV2::new();
        
        // Set up callbacks
        let callbacks = PluginCallbacks {
            log: Some(Box::new(|level, message| {
                println!("[{}] {}", level, message);
                Ok(())
            })),
            ..Default::default()
        };
        
        example_plugin.register_callbacks(callbacks);
        
        // Initialize the plugin
        example_plugin.initialize().await?;
        
        // Print metadata
        println!(
            "Plugin: {} ({})",
            example_plugin.metadata().name,
            example_plugin.metadata().id
        );
        
        // Test web endpoints
        println!("Web endpoints:");
        for endpoint in example_plugin.get_endpoints() {
            println!("  {} {}", endpoint.method, endpoint.path);
            
            // Test the endpoint
            let response = example_plugin.handle_web_endpoint(&endpoint, None).await?;
            println!("  Response: {}", response);
        }
        
        // Demonstrate adapting a V2 plugin to a V1 plugin
        let mut example_plugin_v2 = ExamplePluginV2::new();
        
        // Set up callbacks before adaptation
        let callbacks = PluginCallbacks {
            log: Some(Box::new(|level, message| {
                println!("[{}] {}", level, message);
                Ok(())
            })),
            ..Default::default()
        };
        
        example_plugin_v2.register_callbacks(callbacks);
        
        let adapted_plugin = adapt_plugin_v2(example_plugin_v2);
        
        println!(
            "Adapted plugin: {} ({})",
            adapted_plugin.metadata().name,
            adapted_plugin.metadata().id
        );
        
        // Initialize the adapted plugin
        adapted_plugin.initialize().await?;
        
        // Shutdown the adapted plugin
        adapted_plugin.shutdown().await?;
        
        // Shutdown our original plugin
        example_plugin.shutdown().await?;
        
        Ok(())
    })
} 