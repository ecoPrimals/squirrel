// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

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
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::types::PluginStatus;
use crate::web::{HttpMethod, HttpStatus, WebEndpoint, WebRequest, WebResponse};
use crate::Plugin;
use crate::{DefaultPluginManager, PluginManagerTrait, PluginRegistry};

/// Plugin management API endpoints
#[derive(Clone)]
pub struct PluginManagementAPI {
    /// Plugin manager instance
    manager: Arc<DefaultPluginManager>,
    /// WebSocket connections for real-time updates
    websocket_connections: Arc<RwLock<HashMap<Uuid, WebSocketConnection>>>,
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

/// Plugin API request/response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub status: PluginStatus,
    pub capabilities: Vec<String>,
    pub dependencies: Vec<String>,
    pub endpoints: Vec<EndpointInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointInfo {
    pub path: String,
    pub method: String,
    pub description: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstallRequest {
    pub source: String,
    pub version: Option<String>,
    pub configuration: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfigurationRequest {
    pub configuration: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginExecutionRequest {
    pub command: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSearchRequest {
    pub query: Option<String>,
    pub category: Option<String>,
    pub author: Option<String>,
    pub capabilities: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMarketplaceEntry {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub category: String,
    pub capabilities: Vec<String>,
    pub download_url: String,
    pub documentation_url: Option<String>,
    pub rating: Option<f64>,
    pub downloads: u64,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub event_type: String,
    pub plugin_id: Option<Uuid>,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl PluginManagementAPI {
    /// Create a new plugin management API instance
    pub fn new(manager: Arc<DefaultPluginManager>) -> Self {
        Self {
            manager,
            websocket_connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get all REST API endpoints
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

            // Health and metrics endpoints
            (HttpMethod::Get, "/api/plugins/health") => self.get_plugin_health().await,
            (HttpMethod::Get, "/api/plugins/metrics") => self.get_plugin_metrics().await,

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
            source: format!("marketplace://{}", plugin_id),
            version: None,
            configuration: None,
        };

        self.install_plugin(install_request).await
    }

    /// Get marketplace categories
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

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "total_plugins": plugins.len(),
                "memory_usage": "50MB", // Placeholder
                "cpu_usage": "2%",      // Placeholder
                "active_connections": 5, // Placeholder
                "uptime": "24h 30m"     // Placeholder
            })),
        })
    }

    /// Convert plugin to API info struct
    async fn plugin_to_info(&self, plugin: &Arc<dyn Plugin>) -> Result<PluginInfo> {
        let metadata = plugin.metadata();
        let status =
            PluginManagerTrait::get_plugin_status(self.manager.as_ref(), metadata.id).await?;

        // Get endpoints if plugin implements WebPlugin
        let endpoints = Vec::new(); // Placeholder - in real implementation, would check if plugin implements WebPlugin

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
                .map(|id| id.to_string())
                .collect(),
            endpoints,
        })
    }

    /// Extract plugin ID from URL path
    fn extract_plugin_id(&self, path: &str) -> Result<Uuid> {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 4 {
            let id_str = parts[3];
            Uuid::parse_str(id_str).map_err(|e| anyhow::anyhow!("Invalid plugin ID: {}", e))
        } else {
            Err(anyhow::anyhow!("Invalid path format"))
        }
    }

    /// Extract search parameters from request
    fn extract_search_params(&self, _request: &WebRequest) -> Result<PluginSearchRequest> {
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
    pub fn new(api: Arc<PluginManagementAPI>) -> Self {
        Self { api }
    }

    /// Handle WebSocket connection
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

        let mut connections = self.api.websocket_connections.write().await;
        connections.insert(connection_id, connection);

        Ok(())
    }

    /// Handle WebSocket disconnection
    pub async fn handle_disconnection(&self, connection_id: Uuid) -> Result<()> {
        let mut connections = self.api.websocket_connections.write().await;
        connections.remove(&connection_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::DefaultPluginManager;
    use crate::state::MemoryStateManager;

    #[tokio::test]
    async fn test_plugin_management_api_creation() {
        let state_manager = Arc::new(MemoryStateManager::new());
        let manager = Arc::new(DefaultPluginManager::new(state_manager));
        let api = PluginManagementAPI::new(manager);

        let endpoints = api.get_endpoints();
        assert!(!endpoints.is_empty());
        assert!(endpoints.iter().any(|ep| ep.path == "/api/plugins"));
    }

    #[tokio::test]
    async fn test_plugin_id_extraction() {
        let state_manager = Arc::new(MemoryStateManager::new());
        let manager = Arc::new(DefaultPluginManager::new(state_manager));
        let api = PluginManagementAPI::new(manager);

        let plugin_id = Uuid::new_v4();
        let path = format!("/api/plugins/{}", plugin_id);

        let extracted_id = api.extract_plugin_id(&path).unwrap();
        assert_eq!(extracted_id, plugin_id);
    }
}
