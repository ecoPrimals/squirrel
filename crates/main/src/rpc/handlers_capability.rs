// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Capability domain JSON-RPC handlers — `capability.announce`, `capability.discover`,
//! `capability.list`.

use super::jsonrpc_server::{JsonRpcError, JsonRpcServer, error_codes};
use super::types::{AnnounceCapabilitiesRequest, AnnounceCapabilitiesResponse};
use crate::niche;
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, info, warn};

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
                capabilities: capabilities.clone(),
                tools: tools.clone(),
                announced_at: chrono::Utc::now(),
            };

            let mut registry = self.announced_tools.write().await;
            for tool_name in &tools {
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

    /// Handle `capability.list` — per-method cost, dependency, and schema info.
    ///
    /// Richer than `capability.discover`: returns structured per-operation
    /// data that biomeOS PathwayLearner and other primals use for scheduling.
    pub(crate) async fn handle_capability_list(&self) -> Result<Value, JsonRpcError> {
        debug!("capability.list request");

        let mut methods = serde_json::Map::new();
        let costs = niche::cost_estimates_json();
        let deps = niche::operation_dependencies();

        let cost_map = costs.as_object();
        let dep_map = deps.as_object();

        for cap in niche::CAPABILITIES {
            let mut entry = serde_json::Map::new();
            if let Some(cm) = cost_map
                && let Some(cost) = cm.get(*cap)
            {
                entry.insert("cost".to_string(), cost.clone());
            }
            if let Some(dm) = dep_map
                && let Some(dep) = dm.get(*cap)
            {
                entry.insert("depends_on".to_string(), dep.clone());
            }
            methods.insert(cap.to_string(), Value::Object(entry));
        }

        let capabilities: Vec<&str> = niche::CAPABILITIES.to_vec();

        let domains: Vec<&str> = capabilities
            .iter()
            .filter_map(|c| c.split('.').next())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();

        Ok(serde_json::json!({
            "primal": niche::PRIMAL_ID,
            "version": niche::PRIMAL_VERSION,
            "domain": niche::DOMAIN,
            "capabilities": capabilities,
            "domains": domains,
            "locality": {
                "local": capabilities,
                "external": niche::CONSUMED_CAPABILITIES,
            },
            "methods": methods,
            "consumed_capabilities": niche::CONSUMED_CAPABILITIES,
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
