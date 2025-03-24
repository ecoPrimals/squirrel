//! Plugin adapter module
//!
//! This module provides adapters for bridging between the legacy and modern plugin systems.

use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;
use tracing::{debug, error, info};

use crate::plugins_legacy as legacy;
use crate::plugins::{Plugin, PluginMetadata, PluginStatus, WebPlugin};
use crate::plugins::model::{WebRequest, WebResponse, WebEndpoint, WebComponent, ComponentType, HttpMethod};
use crate::plugin_adapter::{convert_http_method, convert_legacy_endpoint};

/// Adapter for using legacy plugins with the modern registry
pub struct LegacyWebPluginAdapter {
    /// The legacy plugin
    plugin: Arc<Box<dyn legacy::WebPlugin>>,
    /// Metadata for the modern plugin system
    metadata: PluginMetadata,
    /// Status of the plugin
    status: PluginStatus,
}

impl Clone for LegacyWebPluginAdapter {
    fn clone(&self) -> Self {
        Self {
            plugin: Arc::clone(&self.plugin),
            metadata: self.metadata.clone(),
            status: self.status,
        }
    }
}

impl LegacyWebPluginAdapter {
    /// Create a new adapter for a legacy plugin
    pub fn new(plugin: Box<dyn legacy::WebPlugin>) -> Self {
        let legacy_metadata = plugin.metadata();
        
        let metadata = PluginMetadata {
            id: legacy_metadata.id.to_string(),
            name: legacy_metadata.name.clone(),
            version: legacy_metadata.version.clone(),
            description: legacy_metadata.description.clone(),
            author: legacy_metadata.author.clone(),
            repository: None,
            license: None,
            tags: legacy_metadata.capabilities.clone(),
        };
        
        Self {
            plugin: Arc::new(plugin),
            metadata,
            status: PluginStatus::Active,
        }
    }
    
    /// Convert a legacy WebEndpoint to a modern WebEndpoint
    fn convert_endpoint(&self, endpoint: &legacy::WebEndpoint) -> WebEndpoint {
        let mut new_endpoint = WebEndpoint::new(
            endpoint.path.clone(),
            convert_http_method(endpoint.method),
            format!("Legacy endpoint: {}", endpoint.path),
        );
        
        for permission in &endpoint.permissions {
            new_endpoint = new_endpoint.with_permission(permission.clone());
        }
        
        new_endpoint
    }
    
    /// Convert a legacy WebComponent to a modern WebComponent
    fn convert_component(&self, component: &legacy::WebComponent) -> WebComponent {
        // Map legacy component type to modern ComponentType
        let component_type = match component.component_type.as_str() {
            "widget" => ComponentType::Widget,
            "menu" => ComponentType::MenuItem,
            "dashboard" => ComponentType::Dashboard,
            "panel" => ComponentType::Panel,
            "modal" => ComponentType::Modal,
            "form" => ComponentType::Form,
            _ => ComponentType::Custom,
        };
        
        let mut new_component = WebComponent::new(
            component.name.clone(),
            component_type,
            format!("Legacy component: {}", component.name),
        );
        
        // Set route if mount point exists
        if !component.mount_point.is_empty() {
            new_component = new_component.with_route(component.mount_point.clone());
        }
        
        new_component
    }
}

#[async_trait]
impl Plugin for LegacyWebPluginAdapter {
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
impl WebPlugin for LegacyWebPluginAdapter {
    fn get_endpoints(&self) -> Vec<WebEndpoint> {
        self.plugin.get_endpoints()
            .into_iter()
            .map(|e| self.convert_endpoint(&e))
            .collect()
    }
    
    fn get_components(&self) -> Vec<WebComponent> {
        self.plugin.get_components()
            .into_iter()
            .map(|c| self.convert_component(&c))
            .collect()
    }
    
    async fn handle_request(&self, request: WebRequest) -> Result<WebResponse> {
        // Convert to legacy format
        let legacy_endpoint = legacy::WebEndpoint {
            path: request.path.clone(),
            method: match request.method {
                HttpMethod::Get => legacy::HttpMethod::Get,
                HttpMethod::Post => legacy::HttpMethod::Post,
                HttpMethod::Put => legacy::HttpMethod::Put,
                HttpMethod::Delete => legacy::HttpMethod::Delete,
                HttpMethod::Patch => legacy::HttpMethod::Patch,
                HttpMethod::Options => legacy::HttpMethod::Options,
                HttpMethod::Head => legacy::HttpMethod::Head,
            },
            permissions: Vec::new(),
        };
        
        // Call the legacy plugin
        let result = self.plugin.handle_web_endpoint(&legacy_endpoint, request.body).await;
        
        // Convert the result to modern format
        match result {
            Ok(value) => Ok(WebResponse::ok().with_body(value)),
            Err(err) => {
                error!("Error from legacy plugin: {}", err);
                Ok(WebResponse::internal_server_error().with_body(serde_json::json!({
                    "error": err.to_string(),
                })))
            }
        }
    }
    
    async fn get_component_markup(&self, _id: Uuid, _props: Value) -> Result<String> {
        // Legacy plugins don't have a direct equivalent for get_component_markup
        // Provide a placeholder implementation
        Ok("<div class=\"legacy-component-placeholder\">Legacy component (markup not supported)</div>".to_string())
    }
} 