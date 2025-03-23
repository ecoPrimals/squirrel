//! Web plugin module
//!
//! This module provides functionality for web plugins.

use std::fmt::Debug;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

use crate::plugin::Plugin;

/// Web endpoint metadata
#[derive(Clone, Debug)]
pub struct WebEndpoint {
    /// Endpoint path
    pub path: String,
    
    /// HTTP method
    pub method: String,
    
    /// Endpoint description
    pub description: String,
    
    /// Required permissions
    pub permissions: Vec<String>,
}

/// Web UI component
#[derive(Clone, Debug)]
pub struct WebComponent {
    /// Component ID
    pub id: String,
    
    /// Component name
    pub name: String,
    
    /// Component description
    pub description: String,
    
    /// Component type
    pub component_type: String,
    
    /// Component properties
    pub properties: Value,
}

/// Web plugin trait
#[async_trait]
pub trait WebPlugin: Plugin {
    /// Get web endpoints
    fn get_endpoints(&self) -> Vec<WebEndpoint>;
    
    /// Handle web request
    async fn handle_request(&self, path: &str, method: &str, body: Value) -> Result<Value>;
    
    /// Get web components
    fn get_components(&self) -> Vec<WebComponent>;
    
    /// Get component markup
    async fn get_component_markup(&self, component_id: &str, props: Value) -> Result<String>;
    
    /// Check if plugin supports endpoint
    fn supports_endpoint(&self, path: &str, method: &str) -> bool {
        self.get_endpoints().iter().any(|e| e.path == path && e.method == method)
    }
    
    /// Check if plugin supports component
    fn supports_component(&self, component_id: &str) -> bool {
        self.get_components().iter().any(|c| c.id == component_id)
    }
    
    /// Get plugin capabilities
    fn get_capabilities(&self) -> Vec<String> {
        self.metadata().capabilities.clone()
    }
} 