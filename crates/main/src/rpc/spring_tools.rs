// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Spring MCP Tool Discovery — discover and aggregate `mcp.tools.list` from domain springs.
//!
//! Springs (healthSpring, wetSpring, airSpring, etc.) expose their tools via
//! `mcp.tools.list` JSON-RPC calls. This module discovers those tools at
//! runtime via the biomeOS socket registry and merges them into Squirrel's
//! `tool.list` response.

use crate::discovery::mechanisms::socket_registry::SocketRegistryDiscovery;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// How long to cache discovered spring tools before re-querying.
const SPRING_TOOL_CACHE_TTL: Duration = Duration::from_secs(60);

/// A tool definition discovered from a remote spring.
///
/// Aligned with biomeOS `McpToolDefinition` (V251) for interop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpringToolDef {
    /// Tool name (e.g., `"health.check"`, `"weather.forecast"`)
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Domain prefix (derived from tool name)
    pub domain: String,
    /// JSON Schema for input parameters (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
    /// Version of the tool (from spring manifest, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Primal that provides this tool (discovered at runtime)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primal: Option<String>,
}

/// Cached spring tools with TTL.
struct CachedSpringTools {
    tools: Vec<(SpringToolDef, Arc<str>)>,
    fetched_at: Instant,
}

/// Discovers and caches MCP tools from domain springs.
pub struct SpringToolDiscovery {
    cache: DashMap<(), CachedSpringTools>,
    cache_ttl: Duration,
}

impl SpringToolDiscovery {
    /// Create a new spring tool discovery instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
            cache_ttl: SPRING_TOOL_CACHE_TTL,
        }
    }

    /// Discover tools from all springs in the socket registry.
    ///
    /// Returns `(tool_def, socket_path)` pairs for routing.
    pub async fn discover_spring_tools(&self) -> Vec<(SpringToolDef, Arc<str>)> {
        if let Some(cached) = self.cache.get(&())
            && cached.fetched_at.elapsed() < self.cache_ttl
        {
            return cached.tools.clone();
        }

        let tools = self.fetch_spring_tools().await;

        self.cache.insert(
            (),
            CachedSpringTools {
                tools: tools.clone(),
                fetched_at: Instant::now(),
            },
        );

        tools
    }

    /// Fetch tools from all discovered springs via `mcp.tools.list`.
    async fn fetch_spring_tools(&self) -> Vec<(SpringToolDef, Arc<str>)> {
        let registry = SocketRegistryDiscovery::new();
        let all_services = match registry.discover_all() {
            Ok(services) => services,
            Err(e) => {
                warn!("Failed to read socket registry for spring tool discovery: {e}");
                return Vec::new();
            }
        };

        let mut all_tools = Vec::new();

        for service in &all_services {
            let socket_path = service
                .endpoint
                .strip_prefix("unix://")
                .unwrap_or(&service.endpoint);

            match self.query_spring_tools(socket_path).await {
                Ok(tools) => {
                    let socket_arc: Arc<str> = Arc::from(socket_path);
                    for tool in tools {
                        all_tools.push((tool, Arc::clone(&socket_arc)));
                    }
                }
                Err(e) => {
                    debug!(
                        "Spring at {} does not support mcp.tools.list: {e}",
                        socket_path
                    );
                }
            }
        }

        all_tools
    }

    /// Query a single spring for its tools via `mcp.tools.list`.
    async fn query_spring_tools(
        &self,
        socket_path: &str,
    ) -> Result<Vec<SpringToolDef>, Box<dyn std::error::Error + Send + Sync>> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        let stream = tokio::time::timeout(Duration::from_secs(2), UnixStream::connect(socket_path))
            .await??;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "mcp.tools.list",
            "params": {},
            "id": 1
        });

        let mut request_line = serde_json::to_string(&request)?;
        request_line.push('\n');

        let (reader, mut writer) = tokio::io::split(stream);
        writer.write_all(request_line.as_bytes()).await?;
        writer.flush().await?;

        let mut buf_reader = BufReader::new(reader);
        let mut response_line = String::new();

        tokio::time::timeout(
            Duration::from_secs(2),
            buf_reader.read_line(&mut response_line),
        )
        .await??;

        let response: serde_json::Value = serde_json::from_str(response_line.trim())?;

        let result = universal_patterns::extract_rpc_result(&response)
            .map_err(|e| format!("RPC error: {e}"))?;

        let tools: Vec<SpringToolDef> = if let Some(tools_array) = result.get("tools") {
            serde_json::from_value(tools_array.clone()).unwrap_or_default()
        } else {
            serde_json::from_value(result).unwrap_or_default()
        };

        Ok(tools)
    }

    /// Build a tool-name-to-socket-path routing map from discovered spring tools.
    pub async fn build_routing_table(&self) -> std::collections::HashMap<String, Arc<str>> {
        self.discover_spring_tools()
            .await
            .into_iter()
            .map(|(tool, socket)| (tool.name, socket))
            .collect()
    }

    /// Clear the cache (e.g., after a spring announces new capabilities).
    pub fn invalidate_cache(&self) {
        self.cache.clear();
    }
}

impl Default for SpringToolDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_tool_def_serialization() {
        let tool = SpringToolDef {
            name: "health.check".to_string(),
            description: "Check system health".to_string(),
            domain: "health".to_string(),
            input_schema: Some(serde_json::json!({"type": "object"})),
            version: Some("1.0.0".to_string()),
            primal: Some("healthSpring".to_string()),
        };

        let json = serde_json::to_string(&tool).unwrap();
        let decoded: SpringToolDef = serde_json::from_str(&json).unwrap();
        assert_eq!(tool, decoded);
    }

    #[test]
    fn spring_tool_def_without_schema() {
        let tool = SpringToolDef {
            name: "weather.forecast".to_string(),
            description: "Get weather forecast".to_string(),
            domain: "weather".to_string(),
            input_schema: None,
            version: None,
            primal: None,
        };

        let json = serde_json::to_string(&tool).unwrap();
        assert!(!json.contains("input_schema"));
    }

    #[test]
    fn spring_tool_discovery_creation() {
        let discovery = SpringToolDiscovery::new();
        assert_eq!(discovery.cache_ttl, SPRING_TOOL_CACHE_TTL);
    }

    #[tokio::test]
    async fn discover_returns_empty_when_no_registry() {
        let discovery = SpringToolDiscovery::new();
        let tools = discovery.discover_spring_tools().await;
        assert!(tools.is_empty());
    }

    #[tokio::test]
    async fn routing_table_empty_without_springs() {
        let discovery = SpringToolDiscovery::new();
        let table = discovery.build_routing_table().await;
        assert!(table.is_empty());
    }
}
