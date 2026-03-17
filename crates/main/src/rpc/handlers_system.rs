// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! System, Discovery, and Lifecycle domain JSON-RPC handlers.
//!
//! - `system.health`, `system.metrics`, `system.ping`
//! - `discovery.peers`
//! - `lifecycle.register`, `lifecycle.status`

use super::jsonrpc_server::{JsonRpcError, JsonRpcServer, error_codes};
use super::types::HealthCheckResponse;
use serde_json::Value;
use tracing::{debug, info, warn};

impl JsonRpcServer {
    // -- System domain -------------------------------------------------------

    /// Handle `system.health` / `health` method (full health report).
    pub(crate) async fn handle_health(&self) -> Result<Value, JsonRpcError> {
        debug!("health check");

        let metrics = self.metrics.read().await;

        let response = HealthCheckResponse {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: metrics.uptime_seconds(),
            active_providers: if let Some(router) = &self.ai_router {
                router.provider_count().await
            } else {
                0
            },
            requests_processed: metrics.requests_handled,
            avg_response_time_ms: metrics.avg_response_time_ms(),
        };

        serde_json::to_value(response).map_err(|e| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Serialization error: {e}"),
            data: None,
        })
    }

    /// Handle `health.liveness` — PRIMAL_IPC_PROTOCOL v3.0.
    ///
    /// Minimal check: process is alive and can respond to JSON-RPC.
    /// Absorbed from sweetGrass v0.7.19 / petalTongue v1.6.6.
    pub(crate) async fn handle_health_liveness(&self) -> Result<Value, JsonRpcError> {
        debug!("health.liveness probe");
        Ok(serde_json::json!({
            "alive": true,
            "version": env!("CARGO_PKG_VERSION"),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))
    }

    /// Handle `health.readiness` — PRIMAL_IPC_PROTOCOL v3.0.
    ///
    /// Checks whether the primal is ready to serve requests. Verifies
    /// critical subsystems are initialized (AI router, capability registry).
    pub(crate) async fn handle_health_readiness(&self) -> Result<Value, JsonRpcError> {
        debug!("health.readiness probe");

        let ai_ready = if let Some(router) = &self.ai_router {
            router.provider_count().await > 0
        } else {
            false
        };

        let cap_count = self.capability_registry.method_names().len();
        let ready = cap_count > 0;

        Ok(serde_json::json!({
            "ready": ready,
            "checks": {
                "capability_registry": cap_count > 0,
                "ai_router": ai_ready,
            },
            "version": env!("CARGO_PKG_VERSION"),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))
    }

    /// Handle `system.metrics` / `metrics` method
    pub(crate) async fn handle_metrics(&self) -> Result<Value, JsonRpcError> {
        debug!("metrics request");

        let metrics = self.metrics.read().await;

        let response = serde_json::json!({
            "requests_handled": metrics.requests_handled,
            "errors": metrics.errors,
            "uptime_seconds": metrics.uptime_seconds(),
            "avg_response_time_ms": metrics.avg_response_time_ms(),
            "success_rate": if metrics.requests_handled > 0 {
                (metrics.requests_handled - metrics.errors) as f64 / metrics.requests_handled as f64
            } else {
                1.0
            }
        });

        Ok(response)
    }

    /// Handle `system.ping` method
    pub(crate) async fn handle_ping(&self) -> Result<Value, JsonRpcError> {
        debug!("ping");

        Ok(serde_json::json!({
            "pong": true,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "version": env!("CARGO_PKG_VERSION")
        }))
    }

    // -- Discovery domain ----------------------------------------------------

    /// Handle `discovery.peers` — scan sockets for running primals.
    pub(crate) async fn handle_discover_peers(
        &self,
        _params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        info!("discover_peers request");

        let peers = match crate::capabilities::discovery::discover_all_capabilities().await {
            Ok(capabilities_map) => {
                let mut seen_sockets = std::collections::HashSet::new();
                let mut peer_list = Vec::new();

                for providers in capabilities_map.values() {
                    for provider in providers {
                        if seen_sockets.insert(provider.socket.clone()) {
                            peer_list.push(serde_json::json!({
                                "id": provider.id,
                                "socket": provider.socket.display().to_string(),
                                "capabilities": provider.capabilities,
                                "discovered_via": provider.discovered_via,
                            }));
                        }
                    }
                }

                peer_list
            }
            Err(e) => {
                warn!("Peer discovery scan failed: {e}");
                Vec::new()
            }
        };

        let response = serde_json::json!({
            "peers": peers,
            "total": peers.len(),
            "discovery_method": "socket_scan"
        });

        Ok(response)
    }

    // -- Lifecycle domain (biomeOS) ------------------------------------------

    /// Handle `lifecycle.register` — acknowledge registration requests.
    pub(crate) async fn handle_lifecycle_register(&self) -> Result<Value, JsonRpcError> {
        debug!("lifecycle.register request");

        Ok(serde_json::json!({
            "success": true,
            "message": format!("{} registered", self.service_name),
            "version": env!("CARGO_PKG_VERSION"),
        }))
    }

    /// Handle `lifecycle.status` — heartbeat status report
    pub(crate) async fn handle_lifecycle_status(&self) -> Result<Value, JsonRpcError> {
        debug!("lifecycle.status request");

        let metrics = self.metrics.read().await;

        Ok(serde_json::json!({
            "status": "healthy",
            "version": env!("CARGO_PKG_VERSION"),
            "uptime_seconds": metrics.uptime_seconds(),
            "service": self.service_name,
        }))
    }
}
