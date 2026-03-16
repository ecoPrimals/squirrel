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
    QueryAiResult, SquirrelRpc, SquirrelRpcClient, SystemMetricsResult, ToolExecuteResult,
    ToolListResult,
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
    /// Connect to a Squirrel server by service name
    ///
    /// Uses Universal Transport's auto-discovery to locate and connect
    /// to the server. Supports Unix sockets, TCP, and Named pipes.
    ///
    /// # Arguments
    ///
    /// * `service_name` - Service name (e.g., "squirrel")
    ///
    /// # Returns
    ///
    /// Connected client ready for RPC calls
    pub async fn connect(service_name: &str) -> Result<Self> {
        // Connect via Universal Transport auto-discovery
        let transport = UniversalTransport::connect_discovered(service_name)
            .await
            .context("Failed to discover and connect to service")?;

        Self::from_transport(service_name, transport).await
    }

    /// Create a client from an existing Universal Transport
    pub async fn from_transport(service_name: &str, transport: UniversalTransport) -> Result<Self> {
        // Wrap transport in tarpc adapter
        let adapter = TarpcTransportAdapter::new(transport);

        // Create tarpc transport with bincode serialization
        use tokio_serde::formats::Bincode;
        let transport = tokio_serde::Framed::new(adapter, Bincode::default());

        // Create tarpc client with default config
        let client = SquirrelRpcClient::new(client::Config::default(), transport).spawn();

        Ok(Self {
            client,
            service_name: service_name.to_string(),
            default_timeout: Duration::from_secs(30),
        })
    }

    /// Set default timeout for RPC calls
    pub const fn set_default_timeout(&mut self, timeout: Duration) {
        self.default_timeout = timeout;
    }

    /// Get the service name
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    /// Create a context with default timeout
    fn create_context(&self) -> context::Context {
        let mut ctx = context::current();
        ctx.deadline = std::time::SystemTime::now() + self.default_timeout;
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
    pub const fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Build and connect the client
    pub async fn connect(self) -> Result<SquirrelClient> {
        let mut client = SquirrelClient::connect(&self.service_name).await?;
        client.set_default_timeout(self.timeout);
        Ok(client)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
