// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! JSON-RPC method dispatch table and deprecated legacy handler.
//!
//! Extracted from [`super::jsonrpc_server`] for module size management.

use anyhow::{Context, Result};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tracing::{debug, warn};

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

    /// Handle a client connection (LEGACY — kept for backward compatibility)
    ///
    /// New code should use `handle_universal_connection()` instead.
    #[deprecated(note = "Use handle_universal_connection() with UniversalTransport instead")]
    #[expect(dead_code, reason = "deprecated legacy path; kept for fallback")]
    async fn handle_connection<S>(&self, stream: S) -> Result<()>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    debug!("Client disconnected");
                    break;
                }
                Ok(_) => {
                    if let Some(response_json) = self.handle_request_or_batch(&line).await {
                        let mut out = response_json;
                        out.push('\n');
                        reader
                            .get_mut()
                            .write_all(out.as_bytes())
                            .await
                            .context("Failed to write JSON-RPC response (legacy)")?;
                        reader
                            .get_mut()
                            .flush()
                            .await
                            .context("Failed to flush JSON-RPC response (legacy)")?;
                    }
                }
                Err(e) => {
                    warn!("Error reading from socket: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }
}
