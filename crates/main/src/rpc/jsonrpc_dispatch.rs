// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! JSON-RPC method dispatch table.
//!
//! Extracted from [`super::jsonrpc_server`] for module size management.

use serde_json::Value;

use super::jsonrpc_server::JsonRpcServer;
use super::jsonrpc_types::{JsonRpcError, normalize_method};

impl JsonRpcServer {
    /// Dispatch a validated JSON-RPC method name (after `normalize_method`).
    pub(crate) async fn dispatch_jsonrpc_method(
        &self,
        original_method: &str,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        let method = normalize_method(original_method);
        match method {
            // Inference domain — CANONICAL per SEMANTIC_METHOD_NAMING_STANDARD v2.0 §7
            "inference.complete" | "ai.query" | "ai.complete" | "ai.chat" => {
                self.handle_inference_complete(params).await
            }
            "inference.embed" => self.handle_inference_embed(params).await,
            "inference.models" => self.handle_inference_models(params).await,
            "inference.register_provider" => self.handle_inference_register_provider(params).await,
            "inference.unregister_provider" => {
                self.handle_inference_unregister_provider(params).await
            }
            "ai.list_providers" => self.handle_list_providers(params).await,

            // Capabilities domain — SEMANTIC_METHOD_NAMING_STANDARD v2.1
            "capabilities.announce" | "capability.announce" => {
                self.handle_announce_capabilities(params).await
            }
            "capabilities.discover" | "capability.discover" => {
                self.handle_discover_capabilities().await
            }
            "capabilities.list" | "capability.list" | "primal.capabilities" => {
                self.handle_capability_list().await
            }

            // Identity domain — CAPABILITY_BASED_DISCOVERY_STANDARD v1.0
            "identity.get" => self.handle_identity_get().await,

            // Health domain — PRIMAL_IPC_PROTOCOL v3.0 (canonical)
            "health.check" | "system.health" | "system.status" => self.handle_health().await,
            "health.liveness" => self.handle_health_liveness().await,
            "health.readiness" => self.handle_health_readiness().await,

            // System domain — backward-compat
            "system.metrics" => self.handle_metrics().await,
            "system.ping" => self.handle_ping().await,

            // Discovery domain
            "discovery.peers" | "discovery.list" => self.handle_discover_peers(params).await,

            // Tool domain
            "tool.execute" => self.handle_execute_tool(params).await,
            "tool.list" => self.handle_list_tools().await,

            // Context domain
            "context.create" => self.handle_context_create(params).await,
            "context.update" => self.handle_context_update(params).await,
            "context.summarize" => self.handle_context_summarize(params).await,

            // Provider domain — spring capability registration
            "provider.register" => self.handle_provider_register(params).await,
            "provider.list" => self.handle_provider_list().await,
            "provider.deregister" => self.handle_provider_deregister(params).await,

            // Lifecycle domain — biomeOS registration
            "lifecycle.register" => self.handle_lifecycle_register().await,
            "lifecycle.status" => self.handle_lifecycle_status().await,

            // Graph domain — primalSpring BYOB coordination
            "graph.parse" => self.handle_graph_parse(params).await,
            "graph.validate" => self.handle_graph_validate(params).await,

            // Method not found
            _ => Err(self.method_not_found(original_method)),
        }
    }
}
