// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Request and response DTOs for the Plugin Management API.

use crate::types::PluginStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Plugin metadata returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Unique plugin identifier.
    pub id: Uuid,
    /// Plugin display name.
    pub name: String,
    /// Semantic version string.
    pub version: String,
    /// Human-readable description.
    pub description: String,
    /// Plugin author or maintainer.
    pub author: String,
    /// Current plugin status (active, disabled, etc.).
    pub status: PluginStatus,
    /// Capabilities this plugin provides.
    pub capabilities: Vec<String>,
    /// IDs of plugins this one depends on.
    pub dependencies: Vec<String>,
    /// HTTP endpoints exposed by the plugin.
    pub endpoints: Vec<EndpointInfo>,
}

/// Metadata for a plugin HTTP endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointInfo {
    /// URL path (e.g. "/api/status").
    pub path: String,
    /// HTTP method (GET, POST, etc.).
    pub method: String,
    /// Description of what the endpoint does.
    pub description: String,
    /// Required permissions to access.
    pub permissions: Vec<String>,
}

/// Request to install a plugin from a source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstallRequest {
    /// Source URL or registry identifier.
    pub source: String,
    /// Optional version constraint.
    pub version: Option<String>,
    /// Initial configuration values.
    pub configuration: Option<HashMap<String, serde_json::Value>>,
}

/// Request to update plugin configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfigurationRequest {
    /// New configuration key-value pairs.
    pub configuration: HashMap<String, serde_json::Value>,
}

/// Request to execute a plugin command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginExecutionRequest {
    /// Command name to execute.
    pub command: String,
    /// Command parameters.
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Request to search the plugin marketplace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSearchRequest {
    /// Free-text search query.
    pub query: Option<String>,
    /// Filter by category.
    pub category: Option<String>,
    /// Filter by author.
    pub author: Option<String>,
    /// Filter by required capabilities.
    pub capabilities: Option<Vec<String>>,
    /// Maximum results to return.
    pub limit: Option<usize>,
    /// Offset for pagination.
    pub offset: Option<usize>,
}

/// Entry from the plugin marketplace listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMarketplaceEntry {
    /// Unique marketplace entry ID.
    pub id: Uuid,
    /// Plugin name.
    pub name: String,
    /// Plugin version.
    pub version: String,
    /// Plugin description.
    pub description: String,
    /// Plugin author.
    pub author: String,
    /// Marketplace category.
    pub category: String,
    /// Capabilities provided.
    pub capabilities: Vec<String>,
    /// URL to download the plugin package.
    pub download_url: String,
    /// URL to plugin documentation.
    pub documentation_url: Option<String>,
    /// User rating (if available).
    pub rating: Option<f64>,
    /// Download count.
    pub downloads: u64,
    /// Whether the plugin is verified by the marketplace.
    pub verified: bool,
}

/// WebSocket event message for plugin updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    /// Event type (e.g. "`plugin_loaded`", "`status_changed`").
    pub event_type: String,
    /// Plugin ID if event is plugin-specific.
    pub plugin_id: Option<Uuid>,
    /// Event payload data.
    pub data: serde_json::Value,
    /// When the event occurred.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
