// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

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
//!     let response = client.ping().await?;
//!     println!("Response: {}", response);
//!     
//!     Ok(())
//! }
//! ```

// Note: This module is feature-gated via #[cfg(feature = "tarpc-rpc")] in mod.rs

use super::tarpc_service::*;
use super::tarpc_transport::TarpcTransportAdapter;
use anyhow::{Context, Result};
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
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use squirrel::rpc::tarpc_client::SquirrelClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let client = SquirrelClient::connect("squirrel").await?;
    /// let health = client.health().await?;
    /// println!("Status: {}", health.status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(service_name: &str) -> Result<Self> {
        // Connect via Universal Transport auto-discovery
        let transport = UniversalTransport::connect_discovered(service_name)
            .await
            .context("Failed to discover and connect to service")?;

        Self::from_transport(service_name, transport).await
    }

    /// Create a client from an existing Universal Transport
    ///
    /// Useful for custom connection scenarios or testing.
    ///
    /// # Arguments
    ///
    /// * `service_name` - Service name (for reconnection)
    /// * `transport` - Connected Universal Transport
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
    ///
    /// # Arguments
    ///
    /// * `timeout` - Default timeout duration
    pub fn set_default_timeout(&mut self, timeout: Duration) {
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
    ///
    /// # Returns
    ///
    /// Pong response with server info
    pub async fn ping(&self) -> Result<String> {
        let ctx = self.create_context();
        self.client.ping(ctx).await.context("Ping RPC failed")
    }

    /// Get server health status
    ///
    /// # Returns
    ///
    /// Health check result with metrics
    pub async fn health(&self) -> Result<HealthCheckResult> {
        let ctx = self.create_context();
        self.client.health(ctx).await.context("Health RPC failed")
    }

    /// List available AI providers
    ///
    /// # Returns
    ///
    /// List of providers with status and capabilities
    pub async fn list_providers(&self) -> Result<ListProvidersResult> {
        let ctx = self.create_context();
        self.client
            .list_providers(ctx)
            .await
            .context("List providers RPC failed")
    }

    /// Query AI with a prompt
    ///
    /// # Arguments
    ///
    /// * `prompt` - The prompt to send
    /// * `model` - Optional model name
    /// * `max_tokens` - Optional max tokens
    /// * `temperature` - Optional temperature (0.0-1.0)
    ///
    /// # Returns
    ///
    /// AI response with metadata
    pub async fn query_ai(
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
            .query_ai(ctx, params)
            .await
            .context("Query AI RPC failed")
    }

    /// Announce service capabilities
    ///
    /// # Arguments
    ///
    /// * `service` - Service name
    /// * `capabilities` - List of capabilities
    /// * `metadata` - Additional metadata
    pub async fn announce_capabilities(
        &self,
        service: impl AsRef<str>,
        capabilities: impl IntoIterator<Item = impl AsRef<str>>,
        metadata: std::collections::HashMap<String, String>,
    ) -> Result<AnnounceCapabilitiesResult> {
        let params = AnnounceCapabilitiesParams {
            service: Arc::from(service.as_ref()),
            capabilities: capabilities
                .into_iter()
                .map(|c| Arc::from(c.as_ref()))
                .collect(),
            metadata,
        };

        let ctx = self.create_context();
        self.client
            .announce_capabilities(ctx, params)
            .await
            .context("Announce capabilities RPC failed")
    }

    /// Discover peer services
    ///
    /// # Returns
    ///
    /// List of discovered peer service names
    pub async fn discover_peers(&self) -> Result<Vec<String>> {
        let ctx = self.create_context();
        self.client
            .discover_peers(ctx)
            .await
            .context("Discover peers RPC failed")
    }

    /// Execute a tool
    ///
    /// # Arguments
    ///
    /// * `tool` - Tool name
    /// * `args` - Tool arguments
    ///
    /// # Returns
    ///
    /// Tool execution result
    pub async fn execute_tool(
        &self,
        tool: impl AsRef<str>,
        args: std::collections::HashMap<String, String>,
    ) -> Result<String> {
        let ctx = self.create_context();
        self.client
            .execute_tool(ctx, Arc::from(tool.as_ref()), args)
            .await
            .context("Execute tool RPC failed")
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
    pub fn timeout(mut self, timeout: Duration) -> Self {
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

    // Note: Full integration tests with actual connections are in
    // tarpc_integration_tests.rs and require a running server
}
