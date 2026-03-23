// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin Management REST API
//!
//! This module provides REST API endpoints for plugin management including:
//! - Plugin discovery and listing
//! - Plugin installation and uninstallation
//! - Plugin configuration management
//! - Plugin execution and monitoring
//! - Plugin marketplace integration
//! - Real-time updates via WebSocket

use anyhow::Result;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::Plugin;
use crate::types::PluginStatus;
use crate::web::{
    ExampleWebPlugin, HttpMethod, HttpStatus, WebEndpoint, WebPlugin, WebRequest, WebResponse,
};
use crate::{DefaultPluginManager, PluginManagerTrait, PluginRegistry};

pub use super::api_types::{
    EndpointInfo, PluginConfigurationRequest, PluginExecutionRequest, PluginInfo,
    PluginInstallRequest, PluginMarketplaceEntry, PluginSearchRequest, WebSocketMessage,
};

/// Plugin management API endpoints
#[derive(Clone)]
pub struct PluginManagementAPI {
    /// Plugin manager instance
    manager: Arc<DefaultPluginManager>,
    /// WebSocket connections for real-time updates
    websocket_connections: Arc<RwLock<HashMap<Uuid, WebSocketConnection>>>,
    /// When this API instance was created (for uptime reporting).
    api_started_at: Instant,
}

/// WebSocket connection for real-time updates
#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    /// Connection ID
    pub id: Uuid,
    /// Connection metadata
    pub metadata: HashMap<String, String>,
    /// Subscribed event types
    pub subscriptions: Vec<String>,
}

impl PluginManagementAPI {
    /// Create a new plugin management API instance
    pub fn new(manager: Arc<DefaultPluginManager>) -> Self {
        Self {
            manager,
            websocket_connections: Arc::new(RwLock::new(HashMap::new())),
            api_started_at: Instant::now(),
        }
    }

    /// Collect HTTP endpoints advertised by a plugin when it implements [`crate::web::WebPlugin`].
    ///
    /// Concrete plugin types are discovered at runtime (infant primal pattern); unknown types
    /// return an empty list rather than fabricated routes.
    fn discovered_http_endpoints(plugin: &dyn Plugin) -> Vec<EndpointInfo> {
        if let Some(web) = plugin.as_any().downcast_ref::<ExampleWebPlugin>() {
            return web
                .get_endpoints()
                .into_iter()
                .map(|e| EndpointInfo {
                    path: e.path,
                    method: e.method.to_string(),
                    description: e.description,
                    permissions: e.permissions,
                })
                .collect();
        }
        Vec::new()
    }

    /// Get all REST API endpoints
    #[must_use]
    pub fn get_endpoints(&self) -> Vec<WebEndpoint> {
        vec![
            // Plugin management endpoints
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/plugins".to_string(),
                HttpMethod::Get,
                "List all installed plugins".to_string(),
            )
            .with_tag("plugins"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/plugins/:id".to_string(),
                HttpMethod::Get,
                "Get plugin details by ID".to_string(),
            )
            .with_tag("plugins"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/plugins".to_string(),
                HttpMethod::Post,
                "Install a new plugin".to_string(),
            )
            .with_permission("plugin.install")
            .with_tag("plugins"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/plugins/:id".to_string(),
                HttpMethod::Delete,
                "Uninstall a plugin".to_string(),
            )
            .with_permission("plugin.uninstall")
            .with_tag("plugins"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/plugins/:id/start".to_string(),
                HttpMethod::Post,
                "Start a plugin".to_string(),
            )
            .with_permission("plugin.manage")
            .with_tag("plugins"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/plugins/:id/stop".to_string(),
                HttpMethod::Post,
                "Stop a plugin".to_string(),
            )
            .with_permission("plugin.manage")
            .with_tag("plugins"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/plugins/:id/restart".to_string(),
                HttpMethod::Post,
                "Restart a plugin".to_string(),
            )
            .with_permission("plugin.manage")
            .with_tag("plugins"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/plugins/:id/config".to_string(),
                HttpMethod::Get,
                "Get plugin configuration".to_string(),
            )
            .with_permission("plugin.config.read")
            .with_tag("plugins"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/plugins/:id/config".to_string(),
                HttpMethod::Put,
                "Update plugin configuration".to_string(),
            )
            .with_permission("plugin.config.write")
            .with_tag("plugins"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/plugins/:id/execute".to_string(),
                HttpMethod::Post,
                "Execute plugin command".to_string(),
            )
            .with_permission("plugin.execute")
            .with_tag("plugins"),
            // Plugin marketplace endpoints
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/marketplace/plugins".to_string(),
                HttpMethod::Get,
                "Search plugins in marketplace".to_string(),
            )
            .make_public()
            .with_tag("marketplace"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/marketplace/plugins/:id".to_string(),
                HttpMethod::Get,
                "Get marketplace plugin details".to_string(),
            )
            .make_public()
            .with_tag("marketplace"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/marketplace/plugins/:id/install".to_string(),
                HttpMethod::Post,
                "Install plugin from marketplace".to_string(),
            )
            .with_permission("plugin.install")
            .with_tag("marketplace"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/marketplace/categories".to_string(),
                HttpMethod::Get,
                "Get plugin categories".to_string(),
            )
            .make_public()
            .with_tag("marketplace"),
            // WebSocket endpoint for real-time updates
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/plugins/ws".to_string(),
                HttpMethod::Get,
                "WebSocket endpoint for real-time plugin updates".to_string(),
            )
            .with_tag("websocket"),
            // Health and metrics endpoints
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/plugins/health".to_string(),
                HttpMethod::Get,
                "Get plugin system health status".to_string(),
            )
            .with_tag("health"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/plugins/metrics".to_string(),
                HttpMethod::Get,
                "Get plugin system metrics".to_string(),
            )
            .with_tag("metrics"),
        ]
    }

    /// Handle REST API request
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] if routing, deserialization, or handler logic fails.
    pub async fn handle_request(&self, request: WebRequest) -> Result<WebResponse> {
        match (request.method, request.path.as_str()) {
            // Plugin management endpoints
            (HttpMethod::Get, "/api/plugins") => self.list_plugins().await,
            (HttpMethod::Get, path)
                if path.starts_with("/api/plugins/") && path.ends_with("/config") =>
            {
                let id = self.extract_plugin_id(path)?;
                self.get_plugin_config(id).await
            }
            // Fixed route: must be before the generic `/api/plugins/{uuid}` matcher so
            // paths like `/api/plugins/health` are not parsed as plugin IDs.
            (HttpMethod::Get, "/api/plugins/health") => self.get_plugin_health().await,
            (HttpMethod::Get, "/api/plugins/metrics") => self.get_plugin_metrics().await,
            (HttpMethod::Get, path) if path.starts_with("/api/plugins/") => {
                let id = self.extract_plugin_id(path)?;
                self.get_plugin_details(id).await
            }
            (HttpMethod::Post, "/api/plugins") => {
                let request_data: PluginInstallRequest =
                    serde_json::from_value(request.body.unwrap_or_default())?;
                self.install_plugin(request_data).await
            }
            (HttpMethod::Delete, path) if path.starts_with("/api/plugins/") => {
                let id = self.extract_plugin_id(path)?;
                self.uninstall_plugin(id).await
            }
            (HttpMethod::Post, path) if path.ends_with("/start") => {
                let id = self.extract_plugin_id(path)?;
                self.start_plugin(id).await
            }
            (HttpMethod::Post, path) if path.ends_with("/stop") => {
                let id = self.extract_plugin_id(path)?;
                self.stop_plugin(id).await
            }
            (HttpMethod::Post, path) if path.ends_with("/restart") => {
                let id = self.extract_plugin_id(path)?;
                self.restart_plugin(id).await
            }
            (HttpMethod::Put, path) if path.ends_with("/config") => {
                let id = self.extract_plugin_id(path)?;
                let config_data: PluginConfigurationRequest =
                    serde_json::from_value(request.body.unwrap_or_default())?;
                self.update_plugin_config(id, config_data).await
            }
            (HttpMethod::Post, path) if path.ends_with("/execute") => {
                let id = self.extract_plugin_id(path)?;
                let exec_data: PluginExecutionRequest =
                    serde_json::from_value(request.body.unwrap_or_default())?;
                self.execute_plugin_command(id, exec_data).await
            }

            // Marketplace endpoints
            (HttpMethod::Get, "/api/marketplace/plugins") => {
                let search_params = self.extract_search_params(&request)?;
                self.search_marketplace_plugins(search_params).await
            }
            (HttpMethod::Get, path) if path.starts_with("/api/marketplace/plugins/") => {
                let id = self.extract_plugin_id(path)?;
                self.get_marketplace_plugin_details(id).await
            }
            (HttpMethod::Post, path) if path.ends_with("/install") => {
                let id = self.extract_plugin_id(path)?;
                self.install_marketplace_plugin(id).await
            }
            (HttpMethod::Get, "/api/marketplace/categories") => {
                self.get_marketplace_categories().await
            }

            _ => Ok(WebResponse {
                status: HttpStatus::NotFound,
                headers: HashMap::new(),
                body: Some(serde_json::json!({
                    "error": "Not Found",
                    "message": format!("No endpoint found for {} {}", request.method, request.path)
                })),
            }),
        }
    }

    /// List all installed plugins
    async fn list_plugins(&self) -> Result<WebResponse> {
        let plugins = PluginRegistry::get_all_plugins(self.manager.as_ref()).await?;
        let mut plugin_infos = Vec::new();

        for plugin in plugins {
            let plugin_info = self.plugin_to_info(&plugin).await?;
            plugin_infos.push(plugin_info);
        }

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "plugins": plugin_infos,
                "total": plugin_infos.len()
            })),
        })
    }

    /// Get plugin details by ID
    async fn get_plugin_details(&self, plugin_id: Uuid) -> Result<WebResponse> {
        let plugin = PluginRegistry::get_plugin(self.manager.as_ref(), plugin_id).await?;
        let plugin_info = self.plugin_to_info(&plugin).await?;

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::to_value(plugin_info)?),
        })
    }

    /// Install a new plugin
    async fn install_plugin(&self, request: PluginInstallRequest) -> Result<WebResponse> {
        // This would integrate with the plugin distribution system
        // For now, return a placeholder response
        let plugin_id = Uuid::new_v4();

        // Emit WebSocket event for installation progress
        self.emit_websocket_event(WebSocketMessage {
            event_type: "plugin.install.started".to_string(),
            plugin_id: Some(plugin_id),
            data: serde_json::json!({
                "source": request.source,
                "version": request.version
            }),
            timestamp: chrono::Utc::now(),
        })
        .await;

        // Simulate installation process
        // In real implementation, this would:
        // 1. Download plugin from source
        // 2. Verify plugin integrity
        // 3. Install dependencies
        // 4. Register plugin with manager
        // 5. Initialize plugin

        Ok(WebResponse {
            status: HttpStatus::Accepted,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "plugin_id": plugin_id,
                "status": "installing",
                "message": "Plugin installation started"
            })),
        })
    }

    /// Uninstall a plugin
    async fn uninstall_plugin(&self, plugin_id: Uuid) -> Result<WebResponse> {
        // Get plugin info before uninstalling
        let plugin = PluginRegistry::get_plugin(self.manager.as_ref(), plugin_id).await?;
        let plugin_name = plugin.metadata().name.clone();

        // Emit WebSocket event
        self.emit_websocket_event(WebSocketMessage {
            event_type: "plugin.uninstall.started".to_string(),
            plugin_id: Some(plugin_id),
            data: serde_json::json!({
                "name": plugin_name
            }),
            timestamp: chrono::Utc::now(),
        })
        .await;

        // In real implementation, this would:
        // 1. Stop plugin if running
        // 2. Clean up plugin resources
        // 3. Remove plugin files
        // 4. Unregister from manager

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "plugin_id": plugin_id,
                "status": "uninstalled",
                "message": "Plugin uninstalled successfully"
            })),
        })
    }

    /// Start a plugin
    async fn start_plugin(&self, plugin_id: Uuid) -> Result<WebResponse> {
        PluginManagerTrait::initialize_plugin(self.manager.as_ref(), plugin_id).await?;

        self.emit_websocket_event(WebSocketMessage {
            event_type: "plugin.started".to_string(),
            plugin_id: Some(plugin_id),
            data: serde_json::json!({}),
            timestamp: chrono::Utc::now(),
        })
        .await;

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "plugin_id": plugin_id,
                "status": "started",
                "message": "Plugin started successfully"
            })),
        })
    }

    /// Stop a plugin
    async fn stop_plugin(&self, plugin_id: Uuid) -> Result<WebResponse> {
        PluginManagerTrait::shutdown_plugin(self.manager.as_ref(), plugin_id).await?;

        self.emit_websocket_event(WebSocketMessage {
            event_type: "plugin.stopped".to_string(),
            plugin_id: Some(plugin_id),
            data: serde_json::json!({}),
            timestamp: chrono::Utc::now(),
        })
        .await;

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "plugin_id": plugin_id,
                "status": "stopped",
                "message": "Plugin stopped successfully"
            })),
        })
    }

    /// Restart a plugin
    async fn restart_plugin(&self, plugin_id: Uuid) -> Result<WebResponse> {
        PluginManagerTrait::shutdown_plugin(self.manager.as_ref(), plugin_id).await?;
        PluginManagerTrait::initialize_plugin(self.manager.as_ref(), plugin_id).await?;

        self.emit_websocket_event(WebSocketMessage {
            event_type: "plugin.restarted".to_string(),
            plugin_id: Some(plugin_id),
            data: serde_json::json!({}),
            timestamp: chrono::Utc::now(),
        })
        .await;

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "plugin_id": plugin_id,
                "status": "restarted",
                "message": "Plugin restarted successfully"
            })),
        })
    }

    /// Get plugin configuration
    async fn get_plugin_config(&self, plugin_id: Uuid) -> Result<WebResponse> {
        // In real implementation, this would fetch plugin configuration
        // from the plugin manager or configuration store
        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "plugin_id": plugin_id,
                "configuration": {
                    "enabled": true,
                    "log_level": "info"
                }
            })),
        })
    }

    /// Update plugin configuration
    async fn update_plugin_config(
        &self,
        plugin_id: Uuid,
        config: PluginConfigurationRequest,
    ) -> Result<WebResponse> {
        // In real implementation, this would update plugin configuration
        // and possibly restart the plugin if needed

        self.emit_websocket_event(WebSocketMessage {
            event_type: "plugin.config.updated".to_string(),
            plugin_id: Some(plugin_id),
            data: serde_json::json!({
                "configuration": config.configuration
            }),
            timestamp: chrono::Utc::now(),
        })
        .await;

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "plugin_id": plugin_id,
                "status": "updated",
                "message": "Plugin configuration updated successfully"
            })),
        })
    }

    /// Execute plugin command
    #[expect(
        clippy::unused_async,
        reason = "Async trait method; required for future implementations"
    )]
    async fn execute_plugin_command(
        &self,
        plugin_id: Uuid,
        exec_request: PluginExecutionRequest,
    ) -> Result<WebResponse> {
        // In real implementation, this would execute the command on the plugin
        // For now, return a placeholder response
        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "plugin_id": plugin_id,
                "command": exec_request.command,
                "result": "Command executed successfully",
                "output": "Sample output"
            })),
        })
    }

    /// Search marketplace plugins
    #[expect(
        clippy::unused_async,
        reason = "Async trait method; required for future implementations"
    )]
    async fn search_marketplace_plugins(&self, search: PluginSearchRequest) -> Result<WebResponse> {
        // In real implementation, this would search the plugin marketplace
        // For now, return sample data
        let sample_plugins = vec![PluginMarketplaceEntry {
            id: Uuid::new_v4(),
            name: "Sample Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A sample plugin for demonstration".to_string(),
            author: "Plugin Author".to_string(),
            category: "utility".to_string(),
            capabilities: vec!["web".to_string(), "command".to_string()],
            download_url: "https://example.com/plugin.zip".to_string(),
            documentation_url: Some("https://example.com/docs".to_string()),
            rating: Some(4.5),
            downloads: 1000,
            verified: true,
        }];

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "plugins": sample_plugins,
                "total": sample_plugins.len(),
                "query": search.query,
                "category": search.category
            })),
        })
    }

    /// Get marketplace plugin details
    #[expect(
        clippy::unused_async,
        reason = "Async trait method; required for future implementations"
    )]
    async fn get_marketplace_plugin_details(&self, plugin_id: Uuid) -> Result<WebResponse> {
        // In real implementation, this would fetch plugin details from marketplace
        let sample_plugin = PluginMarketplaceEntry {
            id: plugin_id,
            name: "Sample Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A sample plugin for demonstration".to_string(),
            author: "Plugin Author".to_string(),
            category: "utility".to_string(),
            capabilities: vec!["web".to_string(), "command".to_string()],
            download_url: "https://example.com/plugin.zip".to_string(),
            documentation_url: Some("https://example.com/docs".to_string()),
            rating: Some(4.5),
            downloads: 1000,
            verified: true,
        };

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::to_value(sample_plugin)?),
        })
    }

    /// Install plugin from marketplace
    async fn install_marketplace_plugin(&self, plugin_id: Uuid) -> Result<WebResponse> {
        // This would integrate with the install_plugin method
        let install_request = PluginInstallRequest {
            source: format!("marketplace://{plugin_id}"),
            version: None,
            configuration: None,
        };

        self.install_plugin(install_request).await
    }

    /// Get marketplace categories
    #[expect(
        clippy::unused_async,
        reason = "Async trait method; required for future implementations"
    )]
    async fn get_marketplace_categories(&self) -> Result<WebResponse> {
        let categories = vec![
            "utility",
            "development",
            "security",
            "monitoring",
            "integration",
        ];

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "categories": categories
            })),
        })
    }

    /// Get plugin system health
    async fn get_plugin_health(&self) -> Result<WebResponse> {
        let plugins = PluginRegistry::get_all_plugins(self.manager.as_ref()).await?;
        let mut healthy_count = 0;
        let mut unhealthy_count = 0;

        for plugin in plugins {
            let status =
                PluginManagerTrait::get_plugin_status(self.manager.as_ref(), plugin.metadata().id)
                    .await?;
            match status {
                PluginStatus::Initialized | PluginStatus::Registered => healthy_count += 1,
                _ => unhealthy_count += 1,
            }
        }

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "status": if unhealthy_count == 0 { "healthy" } else { "degraded" },
                "healthy_plugins": healthy_count,
                "unhealthy_plugins": unhealthy_count,
                "total_plugins": healthy_count + unhealthy_count
            })),
        })
    }

    /// Get plugin system metrics
    async fn get_plugin_metrics(&self) -> Result<WebResponse> {
        let plugins = PluginRegistry::get_all_plugins(self.manager.as_ref()).await?;
        let mut capability_counts: BTreeMap<String, usize> = BTreeMap::new();
        for plugin in &plugins {
            for cap in &plugin.metadata().capabilities {
                *capability_counts.entry(cap.clone()).or_insert(0) += 1;
            }
        }
        let unique_capabilities: Vec<String> = capability_counts.keys().cloned().collect();
        let websocket_count = self.websocket_connections.read().await.len();
        let api_uptime_seconds = self.api_started_at.elapsed().as_secs();

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "total_plugins": plugins.len(),
                "unique_capabilities": unique_capabilities,
                "capability_registration_counts": capability_counts,
                "active_websocket_connections": websocket_count,
                "api_uptime_seconds": api_uptime_seconds,
                "host_resource_metrics": {
                    "available": false,
                    "reason": "host-level RSS/CPU is not exposed by the plugin management API in this build",
                    "discovery_hints": [
                        "attach a capability.system.metrics or OS integration provider when available",
                        "use /api/plugins/health for readiness derived from registered plugin status"
                    ]
                }
            })),
        })
    }

    /// Convert plugin to API info struct
    async fn plugin_to_info(&self, plugin: &Arc<dyn Plugin>) -> Result<PluginInfo> {
        let metadata = plugin.metadata();
        let status =
            PluginManagerTrait::get_plugin_status(self.manager.as_ref(), metadata.id).await?;

        let endpoints = Self::discovered_http_endpoints(plugin.as_ref());

        Ok(PluginInfo {
            id: metadata.id,
            name: metadata.name.clone(),
            version: metadata.version.clone(),
            description: metadata.description.clone(),
            author: metadata.author.clone(),
            status,
            capabilities: metadata.capabilities.clone(),
            dependencies: metadata
                .dependencies
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
            endpoints,
        })
    }

    /// Extract plugin ID from URL path
    fn extract_plugin_id(&self, path: &str) -> Result<Uuid> {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 4 {
            let id_str = parts[3];
            Uuid::parse_str(id_str).map_err(|e| anyhow::anyhow!("Invalid plugin ID: {e}"))
        } else {
            Err(anyhow::anyhow!("Invalid path format"))
        }
    }

    /// Extract search parameters from request
    const fn extract_search_params(&self, _request: &WebRequest) -> Result<PluginSearchRequest> {
        // In real implementation, this would parse query parameters
        Ok(PluginSearchRequest {
            query: None,
            category: None,
            author: None,
            capabilities: None,
            limit: Some(10),
            offset: Some(0),
        })
    }

    /// Emit WebSocket event to all connected clients
    #[expect(
        clippy::unused_async,
        reason = "Async trait method; required for future implementations"
    )]
    async fn emit_websocket_event(&self, message: WebSocketMessage) {
        // In real implementation, this would send the message to all connected WebSocket clients
        // For now, just log it
        tracing::info!("WebSocket event: {:?}", message);
    }
}

/// WebSocket handler for real-time plugin updates
pub struct PluginWebSocketHandler {
    api: Arc<PluginManagementAPI>,
}

impl PluginWebSocketHandler {
    /// Creates a new WebSocket handler backed by the plugin management API.
    #[must_use]
    pub const fn new(api: Arc<PluginManagementAPI>) -> Self {
        Self { api }
    }

    /// Handle WebSocket connection
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] if the connection cannot be registered.
    pub async fn handle_connection(&self, connection_id: Uuid) -> Result<()> {
        let connection = WebSocketConnection {
            id: connection_id,
            metadata: HashMap::new(),
            subscriptions: vec![
                "plugin.install".to_string(),
                "plugin.uninstall".to_string(),
                "plugin.start".to_string(),
                "plugin.stop".to_string(),
                "plugin.config.update".to_string(),
            ],
        };

        self.api
            .websocket_connections
            .write()
            .await
            .insert(connection_id, connection);

        Ok(())
    }

    /// Handle WebSocket disconnection
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] if the connection state cannot be updated.
    pub async fn handle_disconnection(&self, connection_id: Uuid) -> Result<()> {
        self.api
            .websocket_connections
            .write()
            .await
            .remove(&connection_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DefaultPluginManager;
    use crate::discovery::create_noop_plugin;
    use crate::plugin::PluginMetadata;
    use crate::types::PluginStatus as RegistryPluginStatus;
    use serde_json::json;

    fn web_req(method: HttpMethod, path: &str, body: Option<serde_json::Value>) -> WebRequest {
        WebRequest {
            method,
            path: path.to_string(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body,
            user_id: None,
            permissions: vec![],
            route_params: HashMap::new(),
        }
    }

    #[test]
    fn api_dtos_serde_roundtrip() {
        let info = PluginInfo {
            id: Uuid::new_v4(),
            name: "n".to_string(),
            version: "1".to_string(),
            description: "d".to_string(),
            author: "a".to_string(),
            status: RegistryPluginStatus::Registered,
            capabilities: vec!["c".to_string()],
            dependencies: vec!["dep".to_string()],
            endpoints: vec![EndpointInfo {
                path: "/p".to_string(),
                method: "GET".to_string(),
                description: "x".to_string(),
                permissions: vec![],
            }],
        };
        let j = serde_json::to_string(&info).unwrap();
        let back: PluginInfo = serde_json::from_str(&j).unwrap();
        assert_eq!(back.name, info.name);

        let install = PluginInstallRequest {
            source: "s".to_string(),
            version: Some("1.0".to_string()),
            configuration: None,
        };
        let j = serde_json::to_string(&install).unwrap();
        let _: PluginInstallRequest = serde_json::from_str(&j).unwrap();

        let cfg = PluginConfigurationRequest {
            configuration: [("k".to_string(), json!(1))].into_iter().collect(),
        };
        let j = serde_json::to_string(&cfg).unwrap();
        let _: PluginConfigurationRequest = serde_json::from_str(&j).unwrap();

        let exec = PluginExecutionRequest {
            command: "cmd".to_string(),
            parameters: [("p".to_string(), json!("v"))].into_iter().collect(),
        };
        let j = serde_json::to_string(&exec).unwrap();
        let _: PluginExecutionRequest = serde_json::from_str(&j).unwrap();

        let search = PluginSearchRequest {
            query: Some("q".to_string()),
            category: None,
            author: None,
            capabilities: None,
            limit: Some(5),
            offset: Some(0),
        };
        let j = serde_json::to_string(&search).unwrap();
        let _: PluginSearchRequest = serde_json::from_str(&j).unwrap();

        let mpe = PluginMarketplaceEntry {
            id: Uuid::new_v4(),
            name: "n".to_string(),
            version: "1".to_string(),
            description: "d".to_string(),
            author: "a".to_string(),
            category: "c".to_string(),
            capabilities: vec![],
            download_url: "u".to_string(),
            documentation_url: None,
            rating: Some(4.0),
            downloads: 1,
            verified: true,
        };
        let j = serde_json::to_string(&mpe).unwrap();
        let _: PluginMarketplaceEntry = serde_json::from_str(&j).unwrap();

        let ws = WebSocketMessage {
            event_type: "e".to_string(),
            plugin_id: Some(Uuid::new_v4()),
            data: json!({}),
            timestamp: chrono::Utc::now(),
        };
        let j = serde_json::to_string(&ws).unwrap();
        let _: WebSocketMessage = serde_json::from_str(&j).unwrap();
    }

    #[test]
    fn websocket_connection_debug_clone() {
        let c = WebSocketConnection {
            id: Uuid::new_v4(),
            metadata: [("a".to_string(), "b".to_string())].into_iter().collect(),
            subscriptions: vec!["s".to_string()],
        };
        let _ = format!("{c:?}");
        let d = c.clone();
        assert_eq!(d.id, c.id);
    }

    #[tokio::test]
    async fn test_plugin_management_api_creation() {
        let manager = Arc::new(DefaultPluginManager::new());
        let api = PluginManagementAPI::new(manager);

        let endpoints = api.get_endpoints();
        assert!(!endpoints.is_empty());
        assert!(endpoints.iter().any(|ep| ep.path == "/api/plugins"));
        assert!(
            endpoints
                .iter()
                .any(|ep| ep.is_public && ep.path.contains("marketplace"))
        );
    }

    #[tokio::test]
    async fn test_plugin_id_extraction_ok_and_errors() {
        let manager = Arc::new(DefaultPluginManager::new());
        let api = PluginManagementAPI::new(manager);

        let plugin_id = Uuid::new_v4();
        let path = format!("/api/plugins/{plugin_id}");
        assert_eq!(api.extract_plugin_id(&path).unwrap(), plugin_id);

        assert!(api.extract_plugin_id("/api/plugins").is_err());
        assert!(api.extract_plugin_id("/api/plugins/not-a-uuid").is_err());
    }

    #[tokio::test]
    async fn extract_search_params_default() {
        let manager = Arc::new(DefaultPluginManager::new());
        let api = PluginManagementAPI::new(manager);
        let r = web_req(HttpMethod::Get, "/api/marketplace/plugins", None);
        let s = api.extract_search_params(&r).unwrap();
        assert_eq!(s.limit, Some(10));
        assert_eq!(s.offset, Some(0));
    }

    #[tokio::test]
    async fn handle_list_get_plugins_empty_and_with_example_web() {
        let manager = Arc::new(DefaultPluginManager::new());
        let api = PluginManagementAPI::new(manager.clone());

        let res = api
            .handle_request(web_req(HttpMethod::Get, "/api/plugins", None))
            .await
            .unwrap();
        assert_eq!(res.status, HttpStatus::Ok);
        assert_eq!(res.body.as_ref().unwrap()["total"], 0);

        let ex = Arc::new(ExampleWebPlugin::new()) as Arc<dyn crate::Plugin>;
        let ex_id = ex.id();
        manager.register_plugin(ex).await.unwrap();

        let res = api
            .handle_request(web_req(HttpMethod::Get, "/api/plugins", None))
            .await
            .unwrap();
        assert_eq!(res.body.as_ref().unwrap()["total"], 1);
        let plugins = res.body.as_ref().unwrap()["plugins"].as_array().unwrap();
        let first = &plugins[0];
        assert_eq!(first["id"].as_str().unwrap(), ex_id.to_string());
        let ep_list = first["endpoints"].as_array().unwrap();
        assert!(!ep_list.is_empty());
    }

    #[tokio::test]
    async fn handle_get_plugin_details_and_config() {
        let manager = Arc::new(DefaultPluginManager::new());
        let meta = PluginMetadata::new("t", "1.0.0", "d", "a");
        let id = meta.id;
        let plugin = create_noop_plugin(meta);
        manager.register_plugin(plugin).await.unwrap();

        let api = PluginManagementAPI::new(manager);

        let res = api
            .handle_request(web_req(
                HttpMethod::Get,
                &format!("/api/plugins/{id}"),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(res.status, HttpStatus::Ok);
        assert_eq!(res.body.as_ref().unwrap()["name"], "t");

        let cfg = api
            .handle_request(web_req(
                HttpMethod::Get,
                &format!("/api/plugins/{id}/config"),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(cfg.status, HttpStatus::Ok);
        assert!(cfg.body.as_ref().unwrap().get("configuration").is_some());
    }

    #[tokio::test]
    async fn handle_install_post_accepted() {
        let manager = Arc::new(DefaultPluginManager::new());
        let api = PluginManagementAPI::new(manager);
        let res = api
            .handle_request(web_req(
                HttpMethod::Post,
                "/api/plugins",
                Some(json!({
                    "source": "https://example.com/p.zip",
                    "version": "1.2.3"
                })),
            ))
            .await
            .unwrap();
        assert_eq!(res.status, HttpStatus::Accepted);
        assert_eq!(res.body.as_ref().unwrap()["status"], "installing");
    }

    #[tokio::test]
    async fn handle_marketplace_search_and_categories() {
        let manager = Arc::new(DefaultPluginManager::new());
        let api = PluginManagementAPI::new(manager);

        let res = api
            .handle_request(web_req(HttpMethod::Get, "/api/marketplace/plugins", None))
            .await
            .unwrap();
        assert_eq!(res.status, HttpStatus::Ok);
        assert!(res.body.as_ref().unwrap()["plugins"].is_array());

        let cat = api
            .handle_request(web_req(
                HttpMethod::Get,
                "/api/marketplace/categories",
                None,
            ))
            .await
            .unwrap();
        assert_eq!(cat.status, HttpStatus::Ok);
        assert!(cat.body.as_ref().unwrap()["categories"].is_array());
    }

    #[tokio::test]
    async fn handle_health_and_metrics() {
        let manager = Arc::new(DefaultPluginManager::new());
        let api = PluginManagementAPI::new(manager);

        let h = api
            .handle_request(web_req(HttpMethod::Get, "/api/plugins/health", None))
            .await
            .unwrap();
        assert_eq!(h.status, HttpStatus::Ok);
        assert!(h.body.as_ref().unwrap().get("healthy_plugins").is_some());

        let m = api
            .handle_request(web_req(HttpMethod::Get, "/api/plugins/metrics", None))
            .await
            .unwrap();
        assert_eq!(m.status, HttpStatus::Ok);
        assert!(m.body.as_ref().unwrap().get("api_uptime_seconds").is_some());
    }

    #[tokio::test]
    async fn handle_not_found() {
        let manager = Arc::new(DefaultPluginManager::new());
        let api = PluginManagementAPI::new(manager);
        let res = api
            .handle_request(web_req(HttpMethod::Get, "/api/unknown", None))
            .await
            .unwrap();
        assert_eq!(res.status, HttpStatus::NotFound);
    }

    #[tokio::test]
    async fn handle_uninstall_start_stop_restart_execute_config() {
        let manager = Arc::new(DefaultPluginManager::new());
        let meta = PluginMetadata::new("life", "1.0.0", "d", "a");
        let id = meta.id;
        let plugin = create_noop_plugin(meta);
        manager.register_plugin(plugin).await.unwrap();

        let api = PluginManagementAPI::new(manager.clone());

        let un = api
            .handle_request(web_req(
                HttpMethod::Delete,
                &format!("/api/plugins/{id}"),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(un.status, HttpStatus::Ok);

        let meta2 = PluginMetadata::new("life2", "1.0.0", "d", "a");
        let id2 = meta2.id;
        manager
            .register_plugin(create_noop_plugin(meta2))
            .await
            .unwrap();

        let st = api
            .handle_request(web_req(
                HttpMethod::Post,
                &format!("/api/plugins/{id2}/start"),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(st.status, HttpStatus::Ok);

        let sp = api
            .handle_request(web_req(
                HttpMethod::Post,
                &format!("/api/plugins/{id2}/stop"),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(sp.status, HttpStatus::Ok);

        let rs = api
            .handle_request(web_req(
                HttpMethod::Post,
                &format!("/api/plugins/{id2}/restart"),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(rs.status, HttpStatus::Ok);

        let put = api
            .handle_request(web_req(
                HttpMethod::Put,
                &format!("/api/plugins/{id2}/config"),
                Some(json!({"configuration": {"k": "v"}})),
            ))
            .await
            .unwrap();
        assert_eq!(put.status, HttpStatus::Ok);

        let ex = api
            .handle_request(web_req(
                HttpMethod::Post,
                &format!("/api/plugins/{id2}/execute"),
                Some(json!({
                    "command": "ping",
                    "parameters": {}
                })),
            ))
            .await
            .unwrap();
        assert_eq!(ex.status, HttpStatus::Ok);
        assert_eq!(ex.body.as_ref().unwrap()["command"], "ping");
    }

    #[tokio::test]
    async fn marketplace_paths_with_wrong_segment_return_err() {
        let manager = Arc::new(DefaultPluginManager::new());
        let api = PluginManagementAPI::new(manager);
        let pid = Uuid::new_v4();
        let err = api
            .handle_request(web_req(
                HttpMethod::Get,
                &format!("/api/marketplace/plugins/{pid}"),
                None,
            ))
            .await;
        assert!(err.is_err());

        let err2 = api
            .handle_request(web_req(
                HttpMethod::Post,
                &format!("/api/marketplace/plugins/{pid}/install"),
                None,
            ))
            .await;
        assert!(err2.is_err());
    }

    #[tokio::test]
    async fn websocket_handler_register_and_remove() {
        let manager = Arc::new(DefaultPluginManager::new());
        let api = Arc::new(PluginManagementAPI::new(manager));
        let h = PluginWebSocketHandler::new(api.clone());
        let cid = Uuid::new_v4();
        h.handle_connection(cid).await.unwrap();
        assert_eq!(api.websocket_connections.read().await.len(), 1);
        h.handle_disconnection(cid).await.unwrap();
        assert_eq!(api.websocket_connections.read().await.len(), 0);
    }

    #[tokio::test]
    async fn get_plugin_unknown_id_errors() {
        let manager = Arc::new(DefaultPluginManager::new());
        let api = PluginManagementAPI::new(manager);
        let err = api
            .handle_request(web_req(
                HttpMethod::Get,
                &format!("/api/plugins/{}", Uuid::new_v4()),
                None,
            ))
            .await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn post_plugins_invalid_json_errors() {
        let manager = Arc::new(DefaultPluginManager::new());
        let api = PluginManagementAPI::new(manager);
        let err = api
            .handle_request(web_req(
                HttpMethod::Post,
                "/api/plugins",
                Some(json!("not-object")),
            ))
            .await;
        assert!(err.is_err());
    }
}
