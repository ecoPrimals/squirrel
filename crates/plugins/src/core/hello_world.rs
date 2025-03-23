use crate::plugin::{Plugin, PluginMetadata, WebPluginExt, WebEndpoint};
use async_trait::async_trait;
use anyhow::Result;
use serde_json::{Value, json};
use std::fmt;

/// A simple Hello World plugin that demonstrates basic functionality
#[derive(Clone)]
pub struct HelloWorldPlugin {
    metadata: PluginMetadata,
    active: bool,
}

impl fmt::Debug for HelloWorldPlugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HelloWorldPlugin")
            .field("metadata", &self.metadata)
            .field("active", &self.active)
            .finish()
    }
}

impl Default for HelloWorldPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl HelloWorldPlugin {
    /// Create a new instance of the `HelloWorldPlugin`
    #[must_use] pub fn new() -> Self {
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
}

#[async_trait]
impl Plugin for HelloWorldPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> Result<()> {
        println!("Initializing HelloWorldPlugin");
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        println!("Shutting down HelloWorldPlugin");
        Ok(())
    }
}

#[async_trait]
impl WebPluginExt for HelloWorldPlugin {
    fn get_endpoints(&self) -> Vec<WebEndpoint> {
        vec![
            WebEndpoint {
                path: "/hello".to_string(),
                method: "GET".to_string(),
                permissions: vec![],
            },
            WebEndpoint {
                path: "/echo".to_string(),
                method: "POST".to_string(),
                permissions: vec![],
            },
        ]
    }
    
    async fn handle_web_endpoint(&self, endpoint: &WebEndpoint, body: Option<Value>) -> Result<Value> {
        match (endpoint.path.as_str(), endpoint.method.as_str()) {
            ("/hello", "GET") => {
                Ok(json!({ "message": "Hello, World!" }))
            },
            ("/echo", "POST") => {
                if let Some(body) = body {
                    Ok(body)
                } else {
                    Ok(json!({ "error": "No body provided" }))
                }
            },
            _ => Ok(json!({ "error": "Endpoint not found" })),
        }
    }
} 