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

mod endpoints;
mod handlers;
mod websocket;

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::DefaultPluginManager;
use crate::web::{HttpMethod, HttpStatus, WebRequest, WebResponse};

pub use super::api_types::{
    EndpointInfo, PluginConfigurationRequest, PluginExecutionRequest, PluginInfo,
    PluginInstallRequest, PluginMarketplaceEntry, PluginSearchRequest, WebSocketMessage,
};
pub use websocket::{PluginWebSocketHandler, WebSocketConnection};

/// Plugin management API endpoints
#[derive(Clone)]
pub struct PluginManagementAPI {
    /// Plugin manager instance
    pub(super) manager: Arc<DefaultPluginManager>,
    /// WebSocket connections for real-time updates
    pub(super) websocket_connections: Arc<RwLock<HashMap<Uuid, WebSocketConnection>>>,
    /// When this API instance was created (for uptime reporting).
    pub(super) api_started_at: Instant,
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

    /// Extract plugin ID from URL path
    pub(super) fn extract_plugin_id(&self, path: &str) -> Result<Uuid> {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 4 {
            let id_str = parts[3];
            Uuid::parse_str(id_str).map_err(|e| anyhow::anyhow!("Invalid plugin ID: {e}"))
        } else {
            Err(anyhow::anyhow!("Invalid path format"))
        }
    }

    /// Extract search parameters from request
    pub(super) const fn extract_search_params(
        &self,
        _request: &WebRequest,
    ) -> Result<PluginSearchRequest> {
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
    #[allow(
        clippy::unused_async,
        reason = "Async trait method; required for future implementations"
    )]
    pub(super) async fn emit_websocket_event(&self, message: WebSocketMessage) {
        // In real implementation, this would send the message to all connected WebSocket clients
        // For now, just log it
        tracing::info!("WebSocket event: {:?}", message);
    }
}

#[cfg(test)]
#[path = "api_tests.rs"]
mod tests;
