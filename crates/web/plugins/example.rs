//! Example plugin implementation
//!
//! This module provides an example implementation of a web plugin
//! to demonstrate the plugin architecture.

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde_json::{Value, json};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::plugins::core::{Plugin, PluginMetadata, PluginStatus, PluginState};
use crate::plugins::model::{
    WebPlugin, WebEndpoint, WebComponent, WebRequest, WebResponse, 
    HttpMethod, ComponentType, HttpStatus
};

/// Example plugin implementation
#[derive(Debug)]
pub struct ExamplePlugin {
    /// Plugin metadata
    metadata: PluginMetadata,
    /// Plugin state
    state: PluginState,
    /// Plugin data
    data: RwLock<HashMap<String, Value>>,
    /// Component ID map for quick lookup
    component_ids: HashMap<String, Uuid>,
}

impl ExamplePlugin {
    /// Create a new example plugin
    pub fn new() -> Self {
        // Create component IDs
        let greeting_id = Uuid::new_v4();
        let dashboard_id = Uuid::new_v4();
        
        let component_ids = HashMap::from([
            ("greeting".to_string(), greeting_id),
            ("dashboard".to_string(), dashboard_id),
        ]);
        
        Self {
            metadata: PluginMetadata::new(
                "Example Plugin",
                "1.0.0",
                "Example plugin for demonstration purposes",
                "DataScienceBioLab",
            )
            .with_capability("web.endpoints")
            .with_capability("web.components"),
            state: PluginState::new(),
            data: RwLock::new(HashMap::new()),
            component_ids,
        }
    }
    
    /// Get data item
    pub async fn get_data(&self, key: &str) -> Option<Value> {
        let data = self.data.read().await;
        data.get(key).cloned()
    }
    
    /// Set data item
    pub async fn set_data(&self, key: &str, value: Value) -> Result<()> {
        let mut data = self.data.write().await;
        data.insert(key.to_string(), value);
        Ok(())
    }
    
    /// Handle greeting request
    async fn handle_greeting(&self, request: &WebRequest) -> Result<WebResponse> {
        let name = match &request.body {
            Some(body) => body.get("name").and_then(|n| n.as_str()).unwrap_or("World"),
            None => "World",
        };
        
        let response = json!({
            "message": format!("Hello, {}!", name),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        Ok(WebResponse::ok_with_body(response))
    }
    
    /// Handle data get request
    async fn handle_data_get(&self, request: &WebRequest) -> Result<WebResponse> {
        let key = request
            .route_params
            .get("key")
            .ok_or_else(|| anyhow!("Missing key parameter"))?;
        
        match self.get_data(key).await {
            Some(value) => Ok(WebResponse::ok_with_body(value)),
            None => Ok(WebResponse::not_found()),
        }
    }
    
    /// Handle data set request
    async fn handle_data_set(&self, request: &WebRequest) -> Result<WebResponse> {
        let key = request
            .route_params
            .get("key")
            .ok_or_else(|| anyhow!("Missing key parameter"))?;
        
        let value = request
            .body
            .clone()
            .ok_or_else(|| anyhow!("Missing request body"))?;
        
        self.set_data(key, value).await?;
        
        Ok(WebResponse::created())
    }
    
    /// Generate greeting component markup
    fn generate_greeting_markup(&self, props: &Value) -> Result<String> {
        let name = props.get("name").and_then(|n| n.as_str()).unwrap_or("World");
        
        let markup = format!(
            r#"<div class="greeting-component">
                <h2>Hello, {name}!</h2>
                <p>Welcome to the Example Plugin</p>
                <p class="timestamp">{}</p>
            </div>"#,
            chrono::Utc::now().to_rfc3339()
        );
        
        Ok(markup)
    }
    
    /// Generate dashboard component markup
    fn generate_dashboard_markup(&self, props: &Value) -> Result<String> {
        let title = props.get("title").and_then(|t| t.as_str()).unwrap_or("Dashboard");
        
        let markup = format!(
            r#"<div class="dashboard-component">
                <h2>{title}</h2>
                <div class="dashboard-content">
                    <div class="dashboard-item">
                        <h3>Stats</h3>
                        <p>Items: 42</p>
                        <p>Users: 7</p>
                    </div>
                    <div class="dashboard-item">
                        <h3>Activity</h3>
                        <p>Last updated: {}</p>
                    </div>
                </div>
            </div>"#,
            chrono::Utc::now().to_rfc3339()
        );
        
        Ok(markup)
    }
}

#[async_trait]
impl Plugin for ExamplePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn status(&self) -> PluginStatus {
        self.state.status().await
    }
    
    async fn initialize(&self) -> Result<()> {
        self.state.set_status(PluginStatus::Initializing).await;
        
        // Initialize plugin data
        let mut data = self.data.write().await;
        data.insert("version".to_string(), json!(self.metadata.version));
        data.insert("initialized_at".to_string(), json!(chrono::Utc::now().to_rfc3339()));
        
        self.state.set_status(PluginStatus::Ready).await;
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        self.state.set_status(PluginStatus::ShuttingDown).await;
        
        // Perform cleanup
        let mut data = self.data.write().await;
        data.insert("shutdown_at".to_string(), json!(chrono::Utc::now().to_rfc3339()));
        
        self.state.set_status(PluginStatus::Disabled).await;
        Ok(())
    }
}

#[async_trait]
impl WebPlugin for ExamplePlugin {
    fn get_endpoints(&self) -> Vec<WebEndpoint> {
        vec![
            WebEndpoint::new(
                "/api/example/greeting",
                HttpMethod::Post,
                "Get a greeting message",
            )
            .with_public(true)
            .with_tag("example")
            .with_tag("greeting"),
            
            WebEndpoint::new(
                "/api/example/data/:key",
                HttpMethod::Get,
                "Get data by key",
            )
            .with_permission("example.data.read")
            .with_tag("example")
            .with_tag("data"),
            
            WebEndpoint::new(
                "/api/example/data/:key",
                HttpMethod::Post,
                "Set data by key",
            )
            .with_permission("example.data.write")
            .with_tag("example")
            .with_tag("data"),
        ]
    }
    
    fn get_components(&self) -> Vec<WebComponent> {
        vec![
            WebComponent::new(
                "Greeting",
                "A greeting component",
                ComponentType::UI,
            )
            .with_property("name", json!("World"))
            .with_id(self.component_ids["greeting"])
            .with_permission("example.components.view")
            .with_icon("wave"),
            
            WebComponent::new(
                "Dashboard",
                "A dashboard component",
                ComponentType::Dashboard,
            )
            .with_property("title", json!("Example Dashboard"))
            .with_id(self.component_ids["dashboard"])
            .with_route("/dashboard")
            .with_permission("example.components.view")
            .with_priority(10)
            .with_icon("chart-bar"),
        ]
    }
    
    async fn handle_request(&self, request: WebRequest) -> Result<WebResponse> {
        match (request.method, request.path.as_str()) {
            (HttpMethod::Post, "/api/example/greeting") => {
                self.handle_greeting(&request).await
            },
            (HttpMethod::Get, path) if path.starts_with("/api/example/data/") => {
                self.handle_data_get(&request).await
            },
            (HttpMethod::Post, path) if path.starts_with("/api/example/data/") => {
                self.handle_data_set(&request).await
            },
            _ => {
                Ok(WebResponse::not_found())
            }
        }
    }
    
    async fn get_component_markup(&self, component_id: Uuid, props: Value) -> Result<String> {
        if component_id == self.component_ids["greeting"] {
            self.generate_greeting_markup(&props)
        } else if component_id == self.component_ids["dashboard"] {
            self.generate_dashboard_markup(&props)
        } else {
            Err(anyhow!("Unknown component ID: {}", component_id))
        }
    }
}

impl WebComponent {
    /// Helper method to set the ID directly (for testing)
    fn with_id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }
} 