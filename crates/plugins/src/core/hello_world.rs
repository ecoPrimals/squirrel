use crate::plugin::{Plugin, PluginMetadata, WebPluginExt};
use crate::web::{WebPluginEndpoint, HttpMethod};
use async_trait::async_trait;
use anyhow::Result;
use serde_json::{Value, json};
use std::sync::Arc;

/// A simple Hello World plugin that demonstrates basic functionality
pub struct HelloWorldPlugin {
    metadata: PluginMetadata,
    active: bool,
}

impl HelloWorldPlugin {
    /// Create a new instance of the HelloWorldPlugin
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "hello-world",
                "1.0.0",
                "A simple Hello World plugin for demonstration",
                "Squirrel Team",
            )
            .with_capability("web")
            .with_capability("api"),
            active: false,
        }
    }
    
    /// Get the web endpoints for this plugin
    pub fn get_endpoints(&self) -> Vec<WebPluginEndpoint> {
        vec![
            WebPluginEndpoint {
                path: "/hello".to_string(),
                method: HttpMethod::Get,
                permissions: vec![],
            },
            WebPluginEndpoint {
                path: "/echo".to_string(),
                method: HttpMethod::Post,
                permissions: vec![],
            },
        ]
    }
}

#[async_trait]
impl Plugin for HelloWorldPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
    
    fn is_active(&self) -> bool {
        self.active
    }
}

#[async_trait]
impl WebPluginExt for HelloWorldPlugin {
    async fn handle_web_endpoint(&self, endpoint: &WebPluginEndpoint, data: Value) -> Result<Value> {
        match (endpoint.path.as_str(), &endpoint.method) {
            ("/hello", HttpMethod::Get) => {
                Ok(json!({
                    "message": "Hello, World!",
                    "plugin": self.metadata.name,
                    "version": self.metadata.version
                }))
            },
            ("/echo", HttpMethod::Post) => {
                Ok(json!({
                    "received": data,
                    "plugin": self.metadata.name
                }))
            },
            _ => Err(anyhow::anyhow!("Endpoint not found: {} {}", endpoint.method, endpoint.path)),
        }
    }
} 