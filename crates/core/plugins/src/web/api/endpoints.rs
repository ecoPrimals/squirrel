// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! HTTP endpoint catalog for the plugin management API.

use uuid::Uuid;

use crate::web::{HttpMethod, WebEndpoint};

use super::PluginManagementAPI;

impl PluginManagementAPI {
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
}
