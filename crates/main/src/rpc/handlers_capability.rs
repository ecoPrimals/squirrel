// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Capability domain JSON-RPC handlers — `capability.announce`, `capability.discover`,
//! `capability.list`.

use super::jsonrpc_server::{JsonRpcError, JsonRpcServer, error_codes};
use super::types::{AnnounceCapabilitiesRequest, AnnounceCapabilitiesResponse};
use crate::niche;
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, info};

impl JsonRpcServer {
    /// Handle `capability.announce` — register remote tools for routing.
    pub(crate) async fn handle_announce_capabilities(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        let request: AnnounceCapabilitiesRequest = self.parse_params(params)?;

        info!(
            "announce_capabilities - {} capabilities from {:?}",
            request.capabilities.len(),
            request.primal
        );

        let mut tools_registered = 0usize;

        if let (Some(primal), Some(socket_path)) = (&request.primal, &request.socket_path) {
            let tools: Vec<Arc<str>> = match &request.tools {
                Some(t) if !t.is_empty() => t.iter().map(|s| Arc::from(s.as_str())).collect(),
                _ => request
                    .capabilities
                    .iter()
                    .map(|s| Arc::from(s.as_str()))
                    .collect(),
            };
            tools_registered = tools.len();

            let capabilities: Vec<Arc<str>> = request
                .capabilities
                .iter()
                .map(|s| Arc::from(s.as_str()))
                .collect();

            let announced = super::types::AnnouncedPrimal {
                primal: Arc::from(primal.as_str()),
                socket_path: Arc::from(socket_path.as_str()),
                capabilities,
                tools,
                announced_at: chrono::Utc::now(),
            };

            let mut registry = self.announced_tools.write().await;
            for tool_name in &announced.tools {
                info!(
                    "Registered remote tool '{}' -> {} at {}",
                    tool_name, primal, socket_path
                );
                registry.insert(Arc::clone(tool_name), announced.clone());
            }
        }

        let response = AnnounceCapabilitiesResponse {
            success: true,
            message: format!("Acknowledged {} capabilities", request.capabilities.len()),
            announced_at: chrono::Utc::now().to_rfc3339(),
            tools_registered,
        };

        serde_json::to_value(response).map_err(|e| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Serialization error: {e}"),
            data: None,
        })
    }

    /// Handle `capabilities.list` — Wire Standard L1/L2 compliant.
    ///
    /// Response envelope per CAPABILITY_WIRE_STANDARD v1.0:
    /// - `methods`: flat string array of all callable JSON-RPC methods (primary routing signal)
    /// - `provided_capabilities`: structured grouping for L3 composability
    /// - `consumed_capabilities`: cross-primal dependencies for composition validation
    /// - `cost_estimates` / `operation_dependencies`: AI planner metadata (L3)
    pub(crate) async fn handle_capability_list(&self) -> Result<Value, JsonRpcError> {
        debug!("capabilities.list request (Wire Standard L2)");

        let methods: Vec<&str> = niche::CAPABILITIES.to_vec();

        let domains: Vec<&str> = methods
            .iter()
            .filter_map(|c| c.split('.').next())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();

        let provided_capabilities: Vec<Value> = domains
            .iter()
            .map(|&domain| {
                let domain_methods: Vec<&str> = methods
                    .iter()
                    .filter(|m| m.starts_with(domain) && m.contains('.'))
                    .copied()
                    .collect();
                serde_json::json!({
                    "type": domain,
                    "methods": domain_methods,
                    "version": niche::PRIMAL_VERSION,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "primal": niche::PRIMAL_ID,
            "version": niche::PRIMAL_VERSION,
            "methods": methods,
            "provided_capabilities": provided_capabilities,
            "consumed_capabilities": niche::CONSUMED_CAPABILITIES,
            "cost_estimates": niche::cost_estimates_json(),
            "operation_dependencies": niche::operation_dependencies(),
            "protocol": "jsonrpc-2.0",
            "transport": ["uds", "tcp"],
        }))
    }

    /// Handle `capability.discover` — return capabilities for socket scanning.
    pub(crate) async fn handle_discover_capabilities(&self) -> Result<Value, JsonRpcError> {
        debug!("discover_capabilities request");

        let mut capabilities: Vec<&str> = self.capability_registry.method_names();

        if let Some(router) = &self.ai_router
            && router.provider_count().await > 0
        {
            if !capabilities.contains(&"ai.inference") {
                capabilities.push("ai.inference");
            }
            if !capabilities.contains(&"ai.text_generation") {
                capabilities.push("ai.text_generation");
            }
        }

        let reg = &self.capability_registry.primal;
        let response = serde_json::json!({
            "primal": reg.name,
            "capabilities": capabilities,
            "version": reg.version,
            "metadata": {
                "transport": reg.transport,
                "protocol": reg.protocol,
                "service_name": self.service_name,
                "domain": reg.domain,
            },
            "cost_estimates": niche::cost_estimates_json(),
            "operation_dependencies": niche::operation_dependencies(),
            "consumed_capabilities": niche::CONSUMED_CAPABILITIES,
        });

        Ok(response)
    }
}

#[cfg(test)]
mod direct_tests {
    use crate::rpc::JsonRpcServer;
    use serde_json::json;

    #[tokio::test]
    async fn announce_with_primal_socket_and_tools_counts_tools() {
        let server = JsonRpcServer::new("/tmp/cap-announce.sock".to_string());
        let params = Some(json!({
            "capabilities": ["ai.inference"],
            "primal": "peer-a",
            "socket_path": "/tmp/peer-a.sock",
            "tools": ["a.tool", "b.tool"]
        }));
        let v = server
            .handle_announce_capabilities(params)
            .await
            .expect("should succeed");
        assert_eq!(
            v.get("tools_registered")
                .and_then(serde_json::Value::as_u64),
            Some(2)
        );
    }
}
