//! Plugin adapter module
//!
//! This module provides adapter implementations to bridge between legacy and modern
//! web plugin interfaces, ensuring bidirectional compatibility.

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;
use tokio::sync::RwLock;

use crate::plugins::core::{Plugin, PluginMetadata, PluginStatus, PluginState};
use crate::plugins::model::{
    WebPlugin, WebEndpoint, WebComponent, WebRequest, WebResponse, 
    HttpMethod, ComponentType, HttpStatus
};

// Legacy plugin interfaces from the existing implementation
use crate::plugins as legacy;

/// Adapter for legacy web plugins
///
/// This adapter allows legacy web plugins to be used with the modern plugin system.
#[derive(Debug)]
pub struct LegacyWebPluginAdapter {
    /// The legacy plugin being adapted
    plugin: Arc<Box<dyn legacy::WebPlugin>>,
    /// Plugin metadata
    metadata: PluginMetadata,
    /// Plugin state
    state: PluginState,
    /// Endpoints for this plugin
    endpoints: Vec<WebEndpoint>,
    /// Components for this plugin
    components: Vec<WebComponent>,
}

impl LegacyWebPluginAdapter {
    /// Create a new adapter for a legacy plugin
    pub fn new(plugin: Box<dyn legacy::WebPlugin>) -> Self {
        // Convert legacy metadata to modern metadata
        let legacy_metadata = plugin.metadata();
        let metadata = PluginMetadata {
            id: legacy_metadata.id,
            name: legacy_metadata.name.clone(),
            version: legacy_metadata.version.clone(),
            description: legacy_metadata.description.clone(),
            author: legacy_metadata.author.clone(),
            capabilities: legacy_metadata.capabilities.clone(),
            dependencies: legacy_metadata.dependencies.clone(),
        };
        
        // Convert legacy endpoints to modern endpoints
        let legacy_endpoints = plugin.get_endpoints();
        let endpoints = legacy_endpoints.iter().map(|e| {
            let mut endpoint = WebEndpoint::new(
                e.path.clone(),
                convert_legacy_http_method(e.method),
                format!("Legacy endpoint: {}", e.path),
            );
            
            for permission in &e.permissions {
                endpoint = endpoint.with_permission(permission.clone());
            }
            
            endpoint
        }).collect();
        
        // Convert legacy components to modern components
        let components = Vec::new(); // Legacy doesn't have complex components
        
        Self {
            plugin: Arc::new(plugin),
            metadata,
            state: PluginState::new(),
            endpoints,
            components,
        }
    }
}

#[async_trait]
impl Plugin for LegacyWebPluginAdapter {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn status(&self) -> PluginStatus {
        self.state.status().await
    }
    
    async fn initialize(&self) -> Result<()> {
        self.state.set_status(PluginStatus::Initializing).await;
        match self.plugin.initialize().await {
            Ok(()) => {
                self.state.set_status(PluginStatus::Ready).await;
                Ok(())
            },
            Err(e) => {
                self.state.set_status(PluginStatus::Error).await;
                Err(e)
            }
        }
    }
    
    async fn shutdown(&self) -> Result<()> {
        self.state.set_status(PluginStatus::ShuttingDown).await;
        match self.plugin.shutdown().await {
            Ok(()) => {
                self.state.set_status(PluginStatus::Disabled).await;
                Ok(())
            },
            Err(e) => {
                self.state.set_status(PluginStatus::Error).await;
                Err(e)
            }
        }
    }
}

#[async_trait]
impl WebPlugin for LegacyWebPluginAdapter {
    fn get_endpoints(&self) -> Vec<WebEndpoint> {
        self.endpoints.clone()
    }
    
    fn get_components(&self) -> Vec<WebComponent> {
        self.components.clone()
    }
    
    async fn handle_request(&self, request: WebRequest) -> Result<WebResponse> {
        // Find the matching endpoint in legacy format
        let legacy_endpoint = self.plugin.get_endpoints().iter()
            .find(|e| {
                e.path == request.path && 
                convert_legacy_http_method(e.method) == request.method
            })
            .ok_or_else(|| anyhow!("Legacy endpoint not found: {}", request.path))?;
        
        // Check permissions
        for permission in &legacy_endpoint.permissions {
            if !request.permissions.contains(permission) {
                return Ok(WebResponse::forbidden());
            }
        }
        
        // Call the legacy handler
        let response = self.plugin.handle_web_endpoint(
            legacy_endpoint,
            request.body,
        ).await?;
        
        // Convert to modern response
        Ok(WebResponse::ok_with_body(response))
    }
    
    async fn get_component_markup(&self, component_id: Uuid, props: Value) -> Result<String> {
        // Legacy system doesn't have complex components with markup
        Err(anyhow!("Component markup not supported by legacy plugin"))
    }
}

/// Adapter for modern web plugins
///
/// This adapter allows modern web plugins to be used with the legacy plugin system.
#[derive(Debug)]
pub struct NewWebPluginAdapter<T: WebPlugin + ?Sized> {
    /// The modern plugin being adapted
    plugin: Arc<T>,
}

impl<T: WebPlugin + ?Sized> NewWebPluginAdapter<T> {
    /// Create a new adapter for a modern plugin
    pub fn new(plugin: Arc<T>) -> Self {
        Self {
            plugin,
        }
    }
}

#[async_trait]
impl<T: WebPlugin + 'static> legacy::Plugin for NewWebPluginAdapter<T> {
    fn metadata(&self) -> &legacy::PluginMetadata {
        // This is a workaround since we can't directly convert between types
        // In a real implementation, we'd need to create a legacy metadata object
        static EMPTY_METADATA: legacy::PluginMetadata = legacy::PluginMetadata {
            id: Uuid::nil(),
            name: String::new(),
            version: String::new(),
            description: String::new(),
            author: String::new(),
            capabilities: Vec::new(),
            dependencies: Vec::new(),
        };
        
        &EMPTY_METADATA
    }
    
    async fn initialize(&self) -> Result<()> {
        self.plugin.initialize().await
    }
    
    async fn shutdown(&self) -> Result<()> {
        self.plugin.shutdown().await
    }
}

#[async_trait]
impl<T: WebPlugin + 'static> legacy::WebPlugin for NewWebPluginAdapter<T> {
    fn get_endpoints(&self) -> Vec<legacy::WebEndpoint> {
        let modern_endpoints = self.plugin.get_endpoints();
        
        modern_endpoints.iter().map(|e| {
            legacy::WebEndpoint {
                path: e.path.clone(),
                method: convert_modern_http_method(e.method),
                permissions: e.permissions.clone(),
            }
        }).collect()
    }
    
    async fn handle_web_endpoint(&self, endpoint: &legacy::WebEndpoint, data: Option<Value>) -> Result<Value> {
        // Create a modern request
        let request = WebRequest::new(
            endpoint.path.clone(),
            convert_legacy_http_method(endpoint.method),
        ).with_body(data.unwrap_or(serde_json::json!({})));
        
        // Handle with modern plugin
        let response = self.plugin.handle_request(request).await?;
        
        // Extract response body
        response.body.ok_or_else(|| anyhow!("Response has no body"))
    }
}

/// Convert from legacy HTTP method to modern HTTP method
fn convert_legacy_http_method(method: legacy::HttpMethod) -> HttpMethod {
    match method {
        legacy::HttpMethod::Get => HttpMethod::Get,
        legacy::HttpMethod::Post => HttpMethod::Post,
        legacy::HttpMethod::Put => HttpMethod::Put,
        legacy::HttpMethod::Delete => HttpMethod::Delete,
        legacy::HttpMethod::Patch => HttpMethod::Patch,
        legacy::HttpMethod::Options => HttpMethod::Options,
        legacy::HttpMethod::Head => HttpMethod::Head,
    }
}

/// Convert from modern HTTP method to legacy HTTP method
fn convert_modern_http_method(method: HttpMethod) -> legacy::HttpMethod {
    match method {
        HttpMethod::Get => legacy::HttpMethod::Get,
        HttpMethod::Post => legacy::HttpMethod::Post,
        HttpMethod::Put => legacy::HttpMethod::Put,
        HttpMethod::Delete => legacy::HttpMethod::Delete,
        HttpMethod::Patch => legacy::HttpMethod::Patch,
        HttpMethod::Options => legacy::HttpMethod::Options,
        HttpMethod::Head => legacy::HttpMethod::Head,
    }
} 