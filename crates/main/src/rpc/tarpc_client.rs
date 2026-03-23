// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! tarpc RPC Client Implementation
//!
//! Type-safe, high-performance binary RPC client using tarpc framework.
//! Provides automatic discovery and connection to Squirrel servers via
//! Universal Transport.
//!
//! ## Usage
//!
//! ```no_run
//! use squirrel::rpc::tarpc_client::SquirrelClient;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Connect to Squirrel via auto-discovery
//!     let client = SquirrelClient::connect("squirrel").await?;
//!
//!     // Make type-safe RPC calls
//!     let response = client.system_ping().await?;
//!     println!("Pong: {} at {}", response.pong, response.timestamp);
//!
//!     Ok(())
//! }
//! ```

// Note: This module is feature-gated via #[cfg(feature = "tarpc-rpc")] in mod.rs

use super::tarpc_service::{
    AnnounceCapabilitiesParams, AnnounceCapabilitiesResult, CapabilityDiscoverResult,
    DiscoveryPeersResult, HealthCheckResult, ListProvidersResult, PingResult, QueryAiParams,
    QueryAiResult, SquirrelRpcClient, SystemMetricsResult, ToolExecuteResult, ToolListResult,
};
use super::tarpc_transport::TarpcTransportAdapter;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tarpc::{client, context};
use universal_patterns::transport::UniversalTransport;

/// Ergonomic wrapper around tarpc SquirrelRpcClient
///
/// This client provides:
/// - Automatic service discovery
/// - Universal Transport support (Unix sockets, TCP, Named pipes)
/// - Type-safe RPC calls
/// - Deadline management
/// - Connection lifecycle management
pub struct SquirrelClient {
    /// Underlying tarpc client
    client: SquirrelRpcClient,

    /// Service name for reconnection
    service_name: String,

    /// Default timeout for RPC calls
    default_timeout: Duration,
}

impl SquirrelClient {
    /// Connect to a Squirrel server by service name.
    ///
    /// Performs full wateringHole `PRIMAL_IPC_PROTOCOL` compliant connection:
    /// 1. Discover server via Universal Transport auto-discovery
    /// 2. Negotiate protocol (tarpc preferred, JSON-RPC fallback)
    /// 3. Establish tarpc binary channel
    ///
    /// # Errors
    ///
    /// Returns an error if discovery fails, negotiation rejects tarpc, or
    /// the tarpc handshake cannot complete.
    pub async fn connect(service_name: &str) -> Result<Self> {
        let mut transport = UniversalTransport::connect_discovered(service_name)
            .await
            .context("Failed to discover and connect to service")?;

        use super::protocol::IpcProtocol;
        use super::protocol_negotiation::negotiate_client;

        let preferred = vec![IpcProtocol::Tarpc, IpcProtocol::JsonRpc];
        let selected = negotiate_client(&mut transport, preferred)
            .await
            .context("Protocol negotiation failed")?;

        if selected != IpcProtocol::Tarpc {
            anyhow::bail!(
                "Server does not support tarpc (negotiated {selected}); \
                 use IpcClient for JSON-RPC instead"
            );
        }

        Self::from_negotiated_transport(service_name, transport).await
    }

    /// Create a client from a transport that has already completed negotiation.
    ///
    /// Callers must ensure protocol negotiation selected tarpc before calling.
    pub async fn from_negotiated_transport(
        service_name: &str,
        transport: UniversalTransport,
    ) -> Result<Self> {
        let adapter = TarpcTransportAdapter::new(transport);

        use tokio_serde::formats::Bincode;
        let transport = tokio_serde::Framed::new(adapter, Bincode::default());

        let client = SquirrelRpcClient::new(client::Config::default(), transport).spawn();

        Ok(Self {
            client,
            service_name: service_name.to_string(),
            default_timeout: Duration::from_secs(30),
        })
    }

    /// Create a client from an existing Universal Transport (legacy, no negotiation).
    #[deprecated(
        since = "0.1.0",
        note = "Use connect() for protocol-negotiated connections"
    )]
    pub async fn from_transport(service_name: &str, transport: UniversalTransport) -> Result<Self> {
        Self::from_negotiated_transport(service_name, transport).await
    }

    /// Set default timeout for RPC calls
    pub const fn set_default_timeout(&mut self, timeout: Duration) {
        self.default_timeout = timeout;
    }

    /// Get the service name
    #[must_use]
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    /// Create a context with default timeout
    fn create_context(&self) -> context::Context {
        let mut ctx = context::current();
        ctx.deadline = std::time::Instant::now() + self.default_timeout;
        ctx
    }

    /// Ping the server (connectivity test)
    pub async fn system_ping(&self) -> Result<PingResult> {
        let ctx = self.create_context();
        self.client
            .system_ping(ctx)
            .await
            .context("Ping RPC failed")
    }

    /// Get server health status
    pub async fn system_health(&self) -> Result<HealthCheckResult> {
        let ctx = self.create_context();
        self.client
            .system_health(ctx)
            .await
            .context("Health RPC failed")
    }

    /// List available AI providers
    pub async fn ai_list_providers(&self) -> Result<ListProvidersResult> {
        let ctx = self.create_context();
        self.client
            .ai_list_providers(ctx)
            .await
            .context("List providers RPC failed")
    }

    /// Query AI with a prompt
    pub async fn ai_query(
        &self,
        prompt: impl Into<String>,
        model: Option<String>,
        max_tokens: Option<usize>,
        temperature: Option<f64>,
    ) -> Result<QueryAiResult> {
        let params = QueryAiParams {
            prompt: prompt.into(),
            model: model.map(|s| Arc::from(s.as_str())),
            max_tokens,
            temperature,
        };

        let ctx = self.create_context();
        self.client
            .ai_query(ctx, params)
            .await
            .context("Query AI RPC failed")
    }

    /// Announce service capabilities
    pub async fn capability_announce(
        &self,
        capabilities: Vec<String>,
        primal: Option<String>,
        socket_path: Option<String>,
        tools: Option<Vec<String>>,
    ) -> Result<AnnounceCapabilitiesResult> {
        let params = AnnounceCapabilitiesParams {
            capabilities,
            primal,
            socket_path,
            tools,
            sub_federations: None,
            genetic_families: None,
        };

        let ctx = self.create_context();
        self.client
            .capability_announce(ctx, params)
            .await
            .context("Announce capabilities RPC failed")
    }

    /// Discover peer services
    pub async fn discovery_peers(&self) -> Result<DiscoveryPeersResult> {
        let ctx = self.create_context();
        self.client
            .discovery_peers(ctx)
            .await
            .context("Discover peers RPC failed")
    }

    /// Execute a tool
    pub async fn tool_execute(
        &self,
        tool: impl AsRef<str>,
        args: HashMap<String, serde_json::Value>,
    ) -> Result<ToolExecuteResult> {
        let ctx = self.create_context();
        self.client
            .tool_execute(ctx, tool.as_ref().to_string(), args)
            .await
            .context("Execute tool RPC failed")
    }

    /// Report own capabilities (capability.discover)
    pub async fn capability_discover(&self) -> Result<CapabilityDiscoverResult> {
        let ctx = self.create_context();
        self.client
            .capability_discover(ctx)
            .await
            .context("Capability discover RPC failed")
    }

    /// Get server metrics
    pub async fn system_metrics(&self) -> Result<SystemMetricsResult> {
        let ctx = self.create_context();
        self.client
            .system_metrics(ctx)
            .await
            .context("Metrics RPC failed")
    }

    /// List available tools
    pub async fn tool_list(&self) -> Result<ToolListResult> {
        let ctx = self.create_context();
        self.client
            .tool_list(ctx)
            .await
            .context("Tool list RPC failed")
    }
}

/// Builder for SquirrelClient with custom configuration
pub struct SquirrelClientBuilder {
    service_name: String,
    timeout: Duration,
}

impl SquirrelClientBuilder {
    /// Create a new client builder
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            timeout: Duration::from_secs(30),
        }
    }

    /// Set custom timeout
    #[must_use]
    pub const fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Build and connect the client with protocol negotiation.
    ///
    /// # Errors
    ///
    /// Returns an error if discovery, negotiation, or connection fails.
    pub async fn connect(self) -> Result<SquirrelClient> {
        let mut client = SquirrelClient::connect(&self.service_name).await?;
        client.set_default_timeout(self.timeout);
        Ok(client)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::tarpc_service::{AnnounceCapabilitiesParams, QueryAiParams, QueryAiResult};
    use std::sync::Arc;

    #[test]
    fn test_client_builder() {
        let builder = SquirrelClientBuilder::new("test-service").timeout(Duration::from_secs(60));

        assert_eq!(builder.service_name, "test-service");
        assert_eq!(builder.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_default_timeout() {
        let builder = SquirrelClientBuilder::new("test");
        assert_eq!(builder.timeout, Duration::from_secs(30));
    }

    #[test]
    fn query_ai_params_bincode_roundtrip() {
        let params = QueryAiParams {
            prompt: "hello".to_string(),
            model: Some(Arc::from("gpt-test")),
            max_tokens: Some(256),
            temperature: Some(0.7),
        };
        let bytes = bincode::serialize(&params).expect("bincode serialize");
        let back: QueryAiParams = bincode::deserialize(&bytes).expect("bincode deserialize");
        assert_eq!(back.prompt, "hello");
        assert_eq!(
            back.model.as_ref().map(std::convert::AsRef::as_ref),
            Some("gpt-test")
        );
        assert_eq!(back.max_tokens, Some(256));
        assert_eq!(back.temperature, Some(0.7));
    }

    #[test]
    fn announce_capabilities_params_bincode_roundtrip() {
        let params = AnnounceCapabilitiesParams {
            capabilities: vec!["cap.a".to_string()],
            primal: Some("p1".to_string()),
            socket_path: Some("/tmp/s".to_string()),
            tools: Some(vec!["t1".to_string()]),
            sub_federations: Some(vec!["sub".to_string()]),
            genetic_families: None,
        };
        let bytes = bincode::serialize(&params).expect("bincode");
        let back: AnnounceCapabilitiesParams = bincode::deserialize(&bytes).expect("de");
        assert_eq!(back.capabilities, params.capabilities);
        assert_eq!(back.primal, params.primal);
        assert_eq!(back.socket_path, params.socket_path);
    }

    #[test]
    fn query_ai_result_bincode_roundtrip() {
        let res = QueryAiResult {
            response: "r".to_string(),
            provider: Arc::from("prov"),
            model: Arc::from("mod"),
            tokens_used: Some(10),
            latency_ms: 5,
            success: true,
        };
        let bytes = bincode::serialize(&res).expect("bincode");
        let back: QueryAiResult = bincode::deserialize(&bytes).expect("de");
        assert_eq!(back.response, "r");
        assert_eq!(back.provider.as_ref(), "prov");
        assert_eq!(back.latency_ms, 5);
        assert!(back.success);
    }

    #[test]
    fn capability_announce_params_shape_matches_client_usage() {
        let params = AnnounceCapabilitiesParams {
            capabilities: vec!["x".to_string()],
            primal: Some("primal".to_string()),
            socket_path: None,
            tools: None,
            sub_federations: None,
            genetic_families: None,
        };
        assert_eq!(params.capabilities.len(), 1);
        assert_eq!(params.primal.as_deref(), Some("primal"));
    }

    #[test]
    fn client_builder_chaining_preserves_service_name() {
        let b = SquirrelClientBuilder::new("my-svc").timeout(std::time::Duration::from_secs(99));
        assert_eq!(b.service_name, "my-svc");
        assert_eq!(b.timeout, std::time::Duration::from_secs(99));
    }

    #[tokio::test]
    async fn connect_fails_when_service_not_discoverable() {
        let err = SquirrelClient::connect("nonexistent-squirrel-service-xyz-999")
            .await
            .err()
            .expect("discovery should fail");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("discover") || msg.contains("Failed") || msg.contains("connect"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn result_types_bincode_roundtrip() {
        use crate::rpc::tarpc_service::{
            CapabilityDiscoverResult, DiscoveryPeersResult, HealthCheckResult,
            LifecycleRegisterResult, LifecycleStatusResult, PeerInfo, PingResult,
            SystemMetricsResult, ToolExecuteResult, ToolListEntry, ToolListResult, ToolSource,
        };
        use std::collections::HashMap;

        let ping = PingResult {
            pong: true,
            timestamp: "t".to_string(),
            version: "v".to_string(),
        };
        let b = bincode::serialize(&ping).unwrap();
        let back: PingResult = bincode::deserialize(&b).unwrap();
        assert!(back.pong);

        let health = HealthCheckResult {
            status: "ok".to_string(),
            version: "1".to_string(),
            uptime_seconds: 3,
            active_providers: 1,
            requests_processed: 2,
            avg_response_time_ms: Some(1.5),
        };
        let b = bincode::serialize(&health).unwrap();
        let _: HealthCheckResult = bincode::deserialize(&b).unwrap();

        let m = SystemMetricsResult {
            requests_handled: 1,
            errors: 0,
            uptime_seconds: 2,
            avg_response_time_ms: None,
            success_rate: 0.99,
        };
        let b = bincode::serialize(&m).unwrap();
        let _: SystemMetricsResult = bincode::deserialize(&b).unwrap();

        let cap = CapabilityDiscoverResult {
            primal: "sq".to_string(),
            capabilities: vec!["a".to_string()],
            version: "0".to_string(),
            metadata: HashMap::from([("k".to_string(), "v".to_string())]),
        };
        let b = bincode::serialize(&cap).unwrap();
        let _: CapabilityDiscoverResult = bincode::deserialize(&b).unwrap();

        let peers = DiscoveryPeersResult {
            peers: vec![PeerInfo {
                id: "i".to_string(),
                socket: "/s".to_string(),
                capabilities: vec!["c".to_string()],
                discovered_via: "x".to_string(),
            }],
            total: 1,
            discovery_method: "d".to_string(),
        };
        let b = bincode::serialize(&peers).unwrap();
        let _: DiscoveryPeersResult = bincode::deserialize(&b).unwrap();

        let te = ToolExecuteResult {
            tool: "t".to_string(),
            success: true,
            output: "o".to_string(),
            error: None,
            timestamp: "ts".to_string(),
        };
        let b = bincode::serialize(&te).unwrap();
        let _: ToolExecuteResult = bincode::deserialize(&b).unwrap();

        let tl = ToolListResult {
            tools: vec![ToolListEntry {
                name: "n".to_string(),
                description: "d".to_string(),
                domain: "dom".to_string(),
                source: ToolSource::Remote {
                    primal: "p".to_string(),
                },
                input_schema: None,
            }],
            total: 1,
        };
        let b = bincode::serialize(&tl).unwrap();
        let _: ToolListResult = bincode::deserialize(&b).unwrap();

        let lr = LifecycleRegisterResult {
            success: true,
            message: "m".to_string(),
        };
        let b = bincode::serialize(&lr).unwrap();
        let _: LifecycleRegisterResult = bincode::deserialize(&b).unwrap();

        let ls = LifecycleStatusResult {
            status: "s".to_string(),
            version: "v".to_string(),
            uptime_seconds: 0,
        };
        let b = bincode::serialize(&ls).unwrap();
        let _: LifecycleStatusResult = bincode::deserialize(&b).unwrap();
    }
}
