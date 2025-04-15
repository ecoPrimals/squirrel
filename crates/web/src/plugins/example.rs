//! Example Plugin for Squirrel Web
//!
//! This module provides an example implementation of a WebPlugin 
//! to demonstrate how to build plugins for the Squirrel Web application.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use uuid::Uuid;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::plugins::{Plugin, PluginMetadata, PluginStatus, WebPlugin};
use crate::plugins::model::{WebRequest, WebResponse, WebEndpoint, WebComponent, ComponentType, HttpMethod};

/// An example plugin demonstrating how to implement the WebPlugin trait
pub struct ExamplePlugin {
    /// Plugin metadata
    metadata: PluginMetadata,
    /// Current plugin status
    status: PluginStatus,
    /// Counter for demo purposes
    counter: AtomicUsize,
}

impl ExamplePlugin {
    /// Create a new example plugin
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "example-plugin".to_string(),
                name: "Example Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "An example plugin demonstrating the plugin system".to_string(),
                author: "Squirrel Team".to_string(),
                repository: Some("https://github.com/squirrel/example-plugin".to_string()),
                license: Some("MIT".to_string()),
                tags: vec!["example".to_string(), "demo".to_string()],
            },
            status: PluginStatus::Active,
            counter: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl Plugin for ExamplePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    fn status(&self) -> PluginStatus {
        self.status
    }
    
    fn set_status(&mut self, status: PluginStatus) {
        self.status = status;
    }
}

#[async_trait]
impl WebPlugin for ExamplePlugin {
    fn get_endpoints(&self) -> Vec<WebEndpoint> {
        vec![
            WebEndpoint::new(
                "/api/example/hello".to_string(),
                HttpMethod::Get,
                "Returns a hello message".to_string(),
            ),
            WebEndpoint::new(
                "/api/example/counter".to_string(),
                HttpMethod::Get,
                "Gets the current counter value".to_string(),
            ),
            WebEndpoint::new(
                "/api/example/counter".to_string(),
                HttpMethod::Post,
                "Increments the counter value".to_string(),
            )
            .with_permission("counter.increment".to_string()),
            WebEndpoint::new(
                "/api/example/echo".to_string(),
                HttpMethod::Post,
                "Echoes back the request body".to_string(),
            ),
        ]
    }
    
    fn get_components(&self) -> Vec<WebComponent> {
        vec![
            WebComponent::new(
                "Example Widget".to_string(),
                ComponentType::Widget,
                "A simple example widget".to_string(),
            )
            .with_route("/dashboard".to_string())
            .with_priority(10)
            .with_icon("example-icon".to_string()),
            
            WebComponent::new(
                "Example Menu Item".to_string(),
                ComponentType::MenuItem,
                "An example menu item".to_string(),
            )
            .with_route("/example".to_string())
            .with_icon("example-menu".to_string()),
            
            WebComponent::new(
                "Example Settings Panel".to_string(),
                ComponentType::Panel,
                "Example settings panel".to_string(),
            )
            .with_route("/settings/example".to_string())
            .with_permission("admin".to_string()),
        ]
    }
    
    async fn handle_request(&self, request: WebRequest) -> Result<WebResponse> {
        match (request.path.as_str(), request.method) {
            ("/api/example/hello", HttpMethod::Get) => {
                Ok(WebResponse::ok().with_body(json!({
                    "message": "Hello from Example Plugin!",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                })))
            },
            ("/api/example/counter", HttpMethod::Get) => {
                Ok(WebResponse::ok().with_body(json!({
                    "counter": self.counter.load(Ordering::SeqCst),
                })))
            },
            ("/api/example/counter", HttpMethod::Post) => {
                // Update the counter using AtomicUsize
                self.counter.fetch_add(1, Ordering::SeqCst);
                
                Ok(WebResponse::ok().with_body(json!({
                    "counter": self.counter.load(Ordering::SeqCst),
                    "message": "Counter incremented",
                })))
            },
            ("/api/example/echo", HttpMethod::Post) => {
                // Handle the case where body might be None
                let body = request.body.unwrap_or(json!({}));
                Ok(WebResponse::ok().with_body(body))
            },
            _ => {
                Ok(WebResponse::not_found().with_body(json!({
                    "error": "Endpoint not found",
                    "path": request.path,
                    "method": format!("{:?}", request.method),
                })))
            }
        }
    }
    
    async fn get_component_markup(&self, component_id: Uuid, props: Value) -> Result<String> {
        // In a real plugin, this would generate proper HTML/JSX based on the component
        let component_name = self.get_components().iter()
            .find(|c| c.id == component_id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Unknown Component".to_string());
        
        let props_str = props.to_string();
        
        Ok(format!(r#"
            <div class="example-plugin-component">
                <h3>Example Plugin: {}</h3>
                <div class="content">
                    <p>This is a component from the Example Plugin.</p>
                    <pre>{}</pre>
                </div>
                <div class="footer">
                    <small>Powered by Squirrel Plugin System</small>
                </div>
            </div>
        "#, component_name, props_str))
    }
}

// Implement Clone for ExamplePlugin
impl Clone for ExamplePlugin {
    fn clone(&self) -> Self {
        Self {
            metadata: self.metadata.clone(),
            status: self.status,
            counter: AtomicUsize::new(self.counter.load(Ordering::SeqCst)),
        }
    }
} 