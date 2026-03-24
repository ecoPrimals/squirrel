// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Route handler implementations for plugin management, marketplace, and health.

use anyhow::Result;
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

use crate::plugin::Plugin;
use crate::types::PluginStatus;
use std::sync::Arc;

use crate::web::{ExampleWebPlugin, HttpStatus, WebPlugin, WebResponse};
use crate::{PluginManagerTrait, PluginRegistry};

use super::super::api_types::{
    EndpointInfo, PluginConfigurationRequest, PluginExecutionRequest, PluginInfo,
    PluginInstallRequest, PluginMarketplaceEntry, PluginSearchRequest, WebSocketMessage,
};
use super::PluginManagementAPI;

impl PluginManagementAPI {
    /// List all installed plugins
    pub(super) async fn list_plugins(&self) -> Result<WebResponse> {
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
    pub(super) async fn get_plugin_details(&self, plugin_id: Uuid) -> Result<WebResponse> {
        let plugin = PluginRegistry::get_plugin(self.manager.as_ref(), plugin_id).await?;
        let plugin_info = self.plugin_to_info(&plugin).await?;

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::to_value(plugin_info)?),
        })
    }

    /// Install a new plugin
    pub(super) async fn install_plugin(
        &self,
        request: PluginInstallRequest,
    ) -> Result<WebResponse> {
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
    pub(super) async fn uninstall_plugin(&self, plugin_id: Uuid) -> Result<WebResponse> {
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
    pub(super) async fn start_plugin(&self, plugin_id: Uuid) -> Result<WebResponse> {
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
    pub(super) async fn stop_plugin(&self, plugin_id: Uuid) -> Result<WebResponse> {
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
    pub(super) async fn restart_plugin(&self, plugin_id: Uuid) -> Result<WebResponse> {
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
    pub(super) async fn get_plugin_config(&self, plugin_id: Uuid) -> Result<WebResponse> {
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
    pub(super) async fn update_plugin_config(
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
    #[allow(
        clippy::unused_async,
        reason = "Async trait method; required for future implementations"
    )]
    pub(super) async fn execute_plugin_command(
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
    #[allow(
        clippy::unused_async,
        reason = "Async trait method; required for future implementations"
    )]
    pub(super) async fn search_marketplace_plugins(
        &self,
        search: PluginSearchRequest,
    ) -> Result<WebResponse> {
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
    #[allow(
        clippy::unused_async,
        reason = "Async trait method; required for future implementations"
    )]
    pub(super) async fn get_marketplace_plugin_details(
        &self,
        plugin_id: Uuid,
    ) -> Result<WebResponse> {
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
    pub(super) async fn install_marketplace_plugin(&self, plugin_id: Uuid) -> Result<WebResponse> {
        // This would integrate with the install_plugin method
        let install_request = PluginInstallRequest {
            source: format!("marketplace://{plugin_id}"),
            version: None,
            configuration: None,
        };

        self.install_plugin(install_request).await
    }

    /// Get marketplace categories
    #[allow(
        clippy::unused_async,
        reason = "Async trait method; required for future implementations"
    )]
    pub(super) async fn get_marketplace_categories(&self) -> Result<WebResponse> {
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
    pub(super) async fn get_plugin_health(&self) -> Result<WebResponse> {
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
    pub(super) async fn get_plugin_metrics(&self) -> Result<WebResponse> {
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
    pub(super) async fn plugin_to_info(&self, plugin: &Arc<dyn Plugin>) -> Result<PluginInfo> {
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

    /// Collect HTTP endpoints advertised by a plugin when it implements [`crate::web::WebPlugin`].
    ///
    /// Concrete plugin types are discovered at runtime (infant primal pattern); unknown types
    /// return an empty list rather than fabricated routes.
    pub(super) fn discovered_http_endpoints(plugin: &dyn Plugin) -> Vec<EndpointInfo> {
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
}
