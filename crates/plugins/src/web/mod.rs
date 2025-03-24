//! Web plugin module
//!
//! This module provides functionality for web plugins.
//!
//! The web plugin system is being migrated to a unified architecture.
//! For migration instructions, see the guide at `crates/plugins/docs/WEB_PLUGIN_MIGRATION.md`.
//!
//! ## Migration Status
//!
//! The web plugin system is currently in Phase 1 of migration, which means:
//!
//! - Both old and new APIs are supported via adapters
//! - New plugins should use the new API
//! - Existing plugins can continue to work with the compatibility adapter
//!
//! ## Bidirectional Compatibility
//!
//! The web plugin system supports bidirectional compatibility:
//!
//! - `LegacyWebPluginAdapter`: Allows legacy plugins to work with the new system without modification
//! - `NewWebPluginAdapter`: Allows new plugins to work with legacy systems
//!
//! This bidirectional approach is critical for the plugin silo team, enabling gradual migration
//! while supporting continuous development and deployment across mixed environments.
//!
//! ## Key Components
//!
//! - `WebPlugin`: The main trait for implementing web plugins
//! - `WebEndpoint`: Represents an HTTP endpoint provided by a plugin
//! - `WebComponent`: Represents a UI component provided by a plugin
//! - `WebRequest`/`WebResponse`: Structured request/response objects
//! - `HttpMethod`/`HttpStatus`: Enums for HTTP methods and status codes
//! - `LegacyWebPluginAdapter`: Adapter for using legacy plugins with the new system
//! - `NewWebPluginAdapter`: Adapter for using new plugins with legacy systems
//! - `WebPluginRegistry`: Registry for managing web plugins
//! - `Route`: Utility for handling path parameters in routes

// Public submodules
pub mod http;
pub mod request;
pub mod component;
pub mod endpoint;
pub mod registry;
pub mod adapter;
pub mod example;
pub mod tests;
pub mod routing;

// Re-exports for easier usage
pub use http::{HttpMethod, HttpStatus};
pub use request::{WebRequest, WebResponse};
pub use component::{WebComponent, ComponentType};
pub use endpoint::WebEndpoint;
pub use registry::WebPluginRegistry;
pub use adapter::{LegacyWebPluginAdapter, NewWebPluginAdapter};
pub use example::ExampleWebPlugin;
pub use routing::Route;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

use crate::plugin::Plugin;

/// Web plugin trait
#[async_trait]
pub trait WebPlugin: Plugin {
    /// Get web endpoints provided by this plugin
    fn get_endpoints(&self) -> Vec<WebEndpoint>;
    
    /// Handle web request
    /// 
    /// This method is called when a request matches one of the plugin's endpoints
    async fn handle_request(&self, request: WebRequest) -> Result<WebResponse>;
    
    /// Get web components provided by this plugin
    fn get_components(&self) -> Vec<WebComponent>;
    
    /// Get component markup
    /// 
    /// This method is called to render a component with the given properties
    async fn get_component_markup(&self, component_id: Uuid, props: Value) -> Result<String>;
    
    /// Check if plugin supports the given endpoint
    fn supports_endpoint(&self, path: &str, method: HttpMethod) -> bool {
        self.get_endpoints().iter().any(|e| e.path == path && e.method == method)
    }
    
    /// Check if plugin supports the given component
    fn supports_component(&self, component_id: &Uuid) -> bool {
        self.get_components().iter().any(|c| c.id == *component_id)
    }
    
    /// Get plugin capabilities
    fn get_capabilities(&self) -> Vec<String> {
        self.metadata().capabilities.clone()
    }
    
    /// Check if the plugin has the 'web' capability
    fn has_web_capability(&self) -> bool {
        self.get_capabilities().contains(&"web".to_string())
    }
    
    /// Check if the plugin has a specific capability
    fn has_capability(&self, capability: &str) -> bool {
        self.get_capabilities().contains(&capability.to_string())
    }
} 