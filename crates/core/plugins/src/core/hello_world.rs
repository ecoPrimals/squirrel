use crate::plugin::{Plugin, PluginMetadata, PluginStatus, WebEndpoint, WebPluginExt};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::any::Any;
use std::fmt;

/// A simple Hello World plugin that demonstrates basic functionality
#[derive(Clone)]
pub struct HelloWorldPlugin {
    metadata: PluginMetadata,
    status: PluginStatus,
}

impl fmt::Debug for HelloWorldPlugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HelloWorldPlugin")
            .field("metadata", &self.metadata)
            .field("status", &self.status)
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
    #[must_use]
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: uuid::Uuid::new_v4(),
                name: "hello_world".to_string(),
                version: "1.0.0".to_string(),
                description: "A simple Hello World plugin".to_string(),
                author: "SquirrelLabs".to_string(),
                capabilities: vec!["core".to_string()],
                dependencies: Vec::new(),
            },
            status: PluginStatus::Registered,
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

    fn as_any(&self) -> &dyn Any {
        self
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

    async fn handle_web_endpoint(
        &self,
        endpoint: &WebEndpoint,
        body: Option<Value>,
    ) -> Result<Value> {
        match (endpoint.path.as_str(), endpoint.method.as_str()) {
            ("/hello", "GET") => Ok(json!({ "message": "Hello, World!" })),
            ("/echo", "POST") => {
                if let Some(body) = body {
                    Ok(body)
                } else {
                    Ok(json!({ "error": "No body provided" }))
                }
            }
            _ => Ok(json!({ "error": "Endpoint not found" })),
        }
    }
}
