// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! tarpc RPC Server Implementation
//!
//! High-performance binary RPC server using tarpc framework.
//! Delegates to JsonRpcServer handlers for consistency with JSON-RPC.
//! Provides the same functionality as JSON-RPC but with:
//! - Type safety
//! - Binary serialization
//! - Lower latency
//! - Smaller payloads
//! - Cascading cancellation
//! - Deadline propagation

// Note: This module is feature-gated via #[cfg(feature = "tarpc-rpc")] in mod.rs

use super::tarpc_service::{
    AnnounceCapabilitiesResult, CapabilityDiscoverResult, DiscoveryPeersResult, HealthCheckResult,
    ListProvidersResult, PeerInfo, PingResult, ProviderInfo, QueryAiResult, SquirrelRpc,
    SystemMetricsResult, ToolExecuteResult, ToolListEntry, ToolListResult, ToolSource,
};
use anyhow::Result;
use futures::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tarpc::{server, server::Channel};
use tracing::info;
use universal_patterns::transport::UniversalTransport;

/// tarpc RPC Server
///
/// Implements the SquirrelRpc service trait by delegating to JsonRpcServer handlers.
#[derive(Clone)]
pub struct TarpcRpcServer {
    /// JSON-RPC server for delegation
    pub(crate) jsonrpc: Arc<super::JsonRpcServer>,
}

impl TarpcRpcServer {
    /// Create tarpc server that delegates to the given JSON-RPC server
    #[must_use]
    pub const fn from_jsonrpc(jsonrpc: Arc<super::JsonRpcServer>) -> Self {
        Self { jsonrpc }
    }

    /// Handle a single tarpc connection
    ///
    /// This method sets up a tarpc server channel for a single client connection
    /// over Universal Transport (Unix socket, TCP, or Named pipe).
    pub async fn handle_connection(self, transport: UniversalTransport) -> Result<()> {
        info!("🔌 tarpc: New connection accepted");

        // Bridge UniversalTransport (AsyncRead + AsyncWrite) to Stream + Sink
        use super::tarpc_transport::TarpcTransportAdapter;
        let transport_adapter = TarpcTransportAdapter::new(transport);

        // Create tarpc transport using tokio-serde for serialization
        use tokio_serde::formats::Bincode;
        let transport = tokio_serde::Framed::new(transport_adapter, Bincode::default());

        // Create tarpc server channel
        let server = server::BaseChannel::with_defaults(transport);

        // Serve requests using this server's trait implementation
        server
            .execute(self.serve())
            .for_each(|response| async move {
                tokio::spawn(response);
            })
            .await;

        info!("🔌 tarpc: Connection closed");
        Ok(())
    }

    /// Convert JSON-RPC result to tarpc type, or return error result
    pub(crate) fn json_to_query_result(v: &serde_json::Value) -> QueryAiResult {
        QueryAiResult {
            response: v
                .get("response")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            provider: Arc::from(v.get("provider").and_then(|x| x.as_str()).unwrap_or("none")),
            model: Arc::from(v.get("model").and_then(|x| x.as_str()).unwrap_or("none")),
            tokens_used: v
                .get("tokens_used")
                .and_then(serde_json::Value::as_u64)
                .map(|u| u as usize),
            latency_ms: v
                .get("latency_ms")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
            success: v
                .get("success")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        }
    }

    pub(crate) fn json_to_list_providers_result(v: &serde_json::Value) -> ListProvidersResult {
        let providers: Vec<ProviderInfo> = v
            .get("providers")
            .and_then(|x| x.as_array())
            .map_or(&[] as &[serde_json::Value], |a| a.as_slice())
            .iter()
            .filter_map(|p| {
                Some(ProviderInfo {
                    id: Arc::from(p.get("id")?.as_str()?),
                    name: Arc::from(p.get("name")?.as_str()?),
                    models: p
                        .get("models")?
                        .as_array()?
                        .iter()
                        .filter_map(|m| m.as_str().map(Arc::from))
                        .collect(),
                    capabilities: p
                        .get("capabilities")?
                        .as_array()?
                        .iter()
                        .filter_map(|c| c.as_str().map(Arc::from))
                        .collect(),
                    online: p.get("online")?.as_bool()?,
                    avg_latency_ms: p.get("avg_latency_ms")?.as_f64(),
                    cost_tier: Arc::from(p.get("cost_tier")?.as_str()?),
                })
            })
            .collect();
        ListProvidersResult {
            total: v
                .get("total")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as usize,
            providers,
        }
    }

    pub(crate) fn json_to_announce_result(v: &serde_json::Value) -> AnnounceCapabilitiesResult {
        AnnounceCapabilitiesResult {
            success: v
                .get("success")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            message: v
                .get("message")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            announced_at: v
                .get("announced_at")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            tools_registered: v
                .get("tools_registered")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as usize,
        }
    }

    pub(crate) fn json_to_health_result(v: &serde_json::Value) -> HealthCheckResult {
        let tier = v
            .get("tier")
            .and_then(|t| serde_json::from_value::<super::types::HealthTier>(t.clone()).ok())
            .unwrap_or(super::types::HealthTier::Alive);
        HealthCheckResult {
            tier,
            alive: v
                .get("alive")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            ready: v
                .get("ready")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            healthy: v
                .get("healthy")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            status: v
                .get("status")
                .and_then(|x| x.as_str())
                .unwrap_or("unknown")
                .to_string(),
            version: v
                .get("version")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            uptime_seconds: v
                .get("uptime_seconds")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
            active_providers: v
                .get("active_providers")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as usize,
            requests_processed: v
                .get("requests_processed")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
            avg_response_time_ms: v
                .get("avg_response_time_ms")
                .and_then(serde_json::Value::as_f64),
        }
    }

    pub(crate) fn json_to_ping_result(v: &serde_json::Value) -> PingResult {
        PingResult {
            pong: v
                .get("pong")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true),
            timestamp: v
                .get("timestamp")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            version: v
                .get("version")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
        }
    }

    pub(crate) fn json_to_discovery_peers_result(v: &serde_json::Value) -> DiscoveryPeersResult {
        let peers: Vec<PeerInfo> = v
            .get("peers")
            .and_then(|x| x.as_array())
            .map_or(&[] as &[serde_json::Value], |a| a.as_slice())
            .iter()
            .filter_map(|p| {
                Some(PeerInfo {
                    id: p.get("id")?.as_str()?.to_string(),
                    socket: p.get("socket")?.as_str()?.to_string(),
                    capabilities: p
                        .get("capabilities")?
                        .as_array()?
                        .iter()
                        .filter_map(|c| c.as_str().map(String::from))
                        .collect(),
                    discovered_via: p.get("discovered_via")?.as_str().unwrap_or("").to_string(),
                })
            })
            .collect();
        DiscoveryPeersResult {
            total: v
                .get("total")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as usize,
            peers,
            discovery_method: v
                .get("discovery_method")
                .and_then(|x| x.as_str())
                .unwrap_or("socket_scan")
                .to_string(),
        }
    }

    pub(crate) fn json_to_tool_execute_result(v: &serde_json::Value) -> ToolExecuteResult {
        ToolExecuteResult {
            tool: v
                .get("tool")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            success: v
                .get("success")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            output: v
                .get("output")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            error: v.get("error").and_then(|x| x.as_str()).map(String::from),
            timestamp: v
                .get("timestamp")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
        }
    }

    pub(crate) fn json_to_tool_list_result(v: &serde_json::Value) -> ToolListResult {
        let tools: Vec<ToolListEntry> = v
            .get("tools")
            .and_then(|x| x.as_array())
            .map_or(&[] as &[serde_json::Value], |a| a.as_slice())
            .iter()
            .filter_map(|t| {
                let source = match t.get("source") {
                    Some(s) => {
                        // Serde serializes Remote as {"Remote": {"primal": "x"}}
                        let primal = s
                            .get("Remote")
                            .and_then(|r| r.get("primal"))
                            .and_then(|p| p.as_str())
                            .or_else(|| s.get("primal").and_then(|p| p.as_str()));
                        match primal {
                            Some(p) => ToolSource::Remote {
                                primal: p.to_string(),
                            },
                            None => ToolSource::Builtin,
                        }
                    }
                    None => ToolSource::Builtin,
                };
                Some(ToolListEntry {
                    name: t.get("name")?.as_str()?.to_string(),
                    description: t.get("description")?.as_str()?.to_string(),
                    domain: t.get("domain")?.as_str()?.to_string(),
                    source,
                    input_schema: t.get("input_schema").cloned(),
                })
            })
            .collect();
        ToolListResult {
            total: v
                .get("total")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as usize,
            tools,
        }
    }

    pub(crate) fn json_to_capability_discover_result(
        v: &serde_json::Value,
    ) -> CapabilityDiscoverResult {
        let metadata: HashMap<String, String> = v
            .get("metadata")
            .and_then(|x| x.as_object())
            .map(|m| m.iter().map(|(k, v)| (k.clone(), v.to_string())).collect())
            .unwrap_or_default();
        CapabilityDiscoverResult {
            primal: v
                .get("primal")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            capabilities: v
                .get("capabilities")
                .and_then(|x| x.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|c| c.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            version: v
                .get("version")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            metadata,
        }
    }

    pub(crate) fn json_to_system_metrics_result(v: &serde_json::Value) -> SystemMetricsResult {
        SystemMetricsResult {
            requests_handled: v
                .get("requests_handled")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
            errors: v
                .get("errors")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
            uptime_seconds: v
                .get("uptime_seconds")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
            avg_response_time_ms: v
                .get("avg_response_time_ms")
                .and_then(serde_json::Value::as_f64),
            success_rate: v
                .get("success_rate")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(1.0),
        }
    }
}

#[cfg(test)]
#[path = "tarpc_server_tests.rs"]
mod tests;
