// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Web plugin module
//!
//! This module provides functionality for web plugins.
//!
//! The web plugin system is being migrated to a unified architecture.
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
//! - `PluginManagementAPI`: REST API for plugin management and marketplace integration

// Public submodules
pub mod adapter;
pub mod api;
pub mod api_types;
pub mod component;
pub mod dashboard;
pub mod endpoint;
pub mod example;
pub mod http;
pub mod marketplace;
pub mod registry;
pub mod request;
pub mod routing;

// Re-exports for easier usage
pub use adapter::{LegacyWebPluginAdapter, NewWebPluginAdapter};
pub use api::{PluginManagementAPI, PluginWebSocketHandler};
pub use api_types::{
    EndpointInfo, PluginConfigurationRequest, PluginExecutionRequest, PluginInfo,
    PluginInstallRequest, PluginMarketplaceEntry, PluginSearchRequest, WebSocketMessage,
};
pub use component::{ComponentType, WebComponent};
pub use dashboard::{DashboardConfig, DashboardOverview, PluginDashboard};
pub use endpoint::WebEndpoint;
pub use example::ExampleWebPlugin;
pub use http::{HttpMethod, HttpStatus};
pub use marketplace::{MarketplacePlugin, MarketplaceSearchCriteria, PluginMarketplaceClient};
pub use registry::WebPluginRegistry;
pub use request::{WebRequest, WebResponse};
pub use routing::Route;

/// Alias for plugin route definitions (compatibility).
pub type WebPluginRoute = Route;
/// Alias for plugin UI components (compatibility).
pub type WebPluginComponent = WebComponent;
/// Alias for plugin HTTP endpoints (compatibility).
pub type WebPluginEndpoint = WebEndpoint;

// Re-export WebPluginExt as an alias for WebPlugin
pub use WebPlugin as WebPluginExt;

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
        self.get_endpoints()
            .iter()
            .any(|e| e.path == path && e.method == method)
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
}

/// Plugin Management Web Interface
///
/// This struct provides a complete web interface for plugin management,
/// including REST API endpoints, marketplace integration, and real-time updates.
pub struct PluginManagementInterface {
    /// Plugin management API
    pub api: PluginManagementAPI,
    /// WebSocket handler
    pub websocket_handler: PluginWebSocketHandler,
    /// Web plugin registry
    pub registry: WebPluginRegistry,
    /// Marketplace client
    pub marketplace: PluginMarketplaceClient,
    /// Dashboard component
    pub dashboard: PluginDashboard,
}

impl PluginManagementInterface {
    /// Create a new plugin management interface
    pub const fn new(
        api: PluginManagementAPI,
        websocket_handler: PluginWebSocketHandler,
        registry: WebPluginRegistry,
        marketplace: PluginMarketplaceClient,
        dashboard: PluginDashboard,
    ) -> Self {
        Self {
            api,
            websocket_handler,
            registry,
            marketplace,
            dashboard,
        }
    }

    /// Get all management endpoints
    pub fn get_management_endpoints(&self) -> Vec<WebEndpoint> {
        let mut endpoints = self.api.get_endpoints();

        // Add marketplace endpoints
        endpoints.extend(self.marketplace.get_endpoints());

        // Add dashboard endpoints
        endpoints.extend(self.dashboard.get_endpoints());

        // Add additional management endpoints
        endpoints.extend(vec![
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/plugins/logs".to_string(),
                HttpMethod::Get,
                "Get plugin system logs".to_string(),
            )
            .with_permission("plugin.logs.read")
            .with_tag("logs"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/plugins/logs/:id".to_string(),
                HttpMethod::Get,
                "Get logs for specific plugin".to_string(),
            )
            .with_permission("plugin.logs.read")
            .with_tag("logs"),
        ]);

        endpoints
    }

    /// Handle management request
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] if a delegated dashboard, marketplace, or API handler fails.
    pub async fn handle_management_request(&self, request: WebRequest) -> Result<WebResponse> {
        match request.path.as_str() {
            // Dashboard endpoints
            path if path.starts_with("/api/dashboard/") => {
                self.dashboard.handle_request(request).await
            }
            // Marketplace endpoints
            path if path.starts_with("/api/marketplace/") => {
                self.marketplace.handle_request(request).await
            }
            // Log endpoints
            path if path.starts_with("/api/plugins/logs") => self.get_plugin_logs(path).await,
            // Default to API handler
            _ => self.api.handle_request(request).await,
        }
    }

    /// Get plugin logs
    #[allow(
        clippy::unused_async,
        reason = "Async trait method; required for future implementations"
    )]
    async fn get_plugin_logs(&self, _path: &str) -> Result<WebResponse> {
        use std::collections::HashMap;

        let logs = vec![
            serde_json::json!({
                "timestamp": "2024-01-20T10:30:00Z",
                "level": "INFO",
                "message": "Plugin initialized successfully",
                "plugin": "Security Scanner"
            }),
            serde_json::json!({
                "timestamp": "2024-01-20T10:29:45Z",
                "level": "DEBUG",
                "message": "Loading plugin configuration",
                "plugin": "Security Scanner"
            }),
        ];

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "logs": logs,
                "total": logs.len(),
                "page": 1,
                "per_page": 50
            })),
        })
    }
}

/// Web plugin factory for creating plugin management interfaces
pub struct PluginManagementFactory;

impl PluginManagementFactory {
    /// Create a complete plugin management interface
    pub fn create_interface(
        manager: std::sync::Arc<crate::DefaultPluginManager>,
        registry: std::sync::Arc<dyn crate::registry::PluginRegistry>,
    ) -> PluginManagementInterface {
        let api = PluginManagementAPI::new(manager.clone());
        let websocket_handler = PluginWebSocketHandler::new(std::sync::Arc::new(api.clone()));
        let web_registry = WebPluginRegistry::new(registry);
        let marketplace = PluginMarketplaceClient::new(manager.clone());
        let dashboard = PluginDashboard::new(manager);

        PluginManagementInterface::new(api, websocket_handler, web_registry, marketplace, dashboard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DefaultPluginManager;
    use crate::registry::PluginRegistry;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_plugin_management_interface_creation() {
        let manager = Arc::new(DefaultPluginManager::new());
        let registry = manager.clone() as Arc<dyn PluginRegistry>;

        let interface = PluginManagementFactory::create_interface(manager, registry);

        let endpoints = interface.get_management_endpoints();
        assert!(!endpoints.is_empty());

        // Check that key endpoints are present
        assert!(endpoints.iter().any(|ep| ep.path == "/api/plugins"));
        assert!(
            endpoints
                .iter()
                .any(|ep| ep.path == "/api/dashboard/overview")
        );
        assert!(
            endpoints
                .iter()
                .any(|ep| ep.path == "/api/marketplace/search")
        );
    }

    #[tokio::test]
    async fn test_interface_components() {
        let manager = Arc::new(DefaultPluginManager::new());
        let registry = manager.clone() as Arc<dyn PluginRegistry>;

        let interface = PluginManagementFactory::create_interface(manager, registry);

        // Test that all components are properly initialized
        let endpoints = interface.get_management_endpoints();

        // Should have API endpoints
        assert!(
            endpoints
                .iter()
                .any(|ep| ep.path.starts_with("/api/plugins"))
        );

        // Should have dashboard endpoints
        assert!(
            endpoints
                .iter()
                .any(|ep| ep.path.starts_with("/api/dashboard/"))
        );

        // Should have marketplace endpoints
        assert!(
            endpoints
                .iter()
                .any(|ep| ep.path.starts_with("/api/marketplace/"))
        );
    }
}
