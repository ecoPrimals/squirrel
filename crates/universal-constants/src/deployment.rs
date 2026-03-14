// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Deployment configuration with environment variable support
//!
//! This module provides functions to retrieve configuration values from
//! environment variables with sensible defaults for development.
//!
//! # Philosophy
//!
//! All deployment-specific values should be configurable via environment variables
//! while providing sensible defaults for local development. This enables:
//! - Multi-environment deployments without code changes
//! - Container-friendly configuration
//! - Dynamic service discovery
//! - Spec-compliant zero-hardcoding architecture
//!
//! # Usage
//!
//! ```rust
//! use universal_constants::deployment::{ports, services, hosts};
//!
//! // Get MCP server port (from env or default 8443)
//! let port = ports::mcp_server();
//!
//! // Get security service name (from env or default "beardog")
//! let security = services::security();
//!
//! // Build endpoint
//! let endpoint = format!("http://{}:{}", hosts::default(), port);
//! ```

/// Port configuration with environment variable support
pub mod ports {
    use std::env;

    /// MCP server port
    ///
    /// **Environment**: `MCP_SERVER_PORT`\
    /// **Default**: `8443`\
    /// **Usage**: Primary MCP protocol server
    #[must_use]
    pub fn mcp_server() -> u16 {
        env::var("MCP_SERVER_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8443)
    }

    /// Service mesh orchestration port
    ///
    /// **Environment**: `SERVICE_MESH_PORT`\
    /// **Default**: `8444`\
    /// **Usage**: Service mesh coordination (capability-based discovery)
    /// **Note**: Discovers actual service mesh at runtime, no hardcoded primal names
    #[must_use]
    pub fn service_mesh() -> u16 {
        env::var("SERVICE_MESH_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8444)
    }

    /// Security service port
    ///
    /// **Environment**: `SECURITY_SERVICE_PORT`\
    /// **Default**: `8443`\
    /// **Usage**: Security and authentication service (capability-based discovery)
    /// **Note**: Discovers actual security provider at runtime
    #[must_use]
    pub fn security_service() -> u16 {
        env::var("SECURITY_SERVICE_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8443)
    }

    /// Storage service port
    ///
    /// **Environment**: `STORAGE_SERVICE_PORT`\
    /// **Default**: `8445`\
    /// **Usage**: Storage and data management (capability-based discovery)
    /// **Note**: Discovers actual storage provider at runtime
    #[must_use]
    pub fn storage_service() -> u16 {
        env::var("STORAGE_SERVICE_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8445)
    }

    /// Compute service port
    ///
    /// **Environment**: `COMPUTE_SERVICE_PORT`\
    /// **Default**: `8446`\
    /// **Usage**: Compute and execution service (capability-based discovery)
    /// **Note**: Discovers actual compute provider at runtime
    #[must_use]
    pub fn compute_service() -> u16 {
        env::var("COMPUTE_SERVICE_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8446)
    }

    /// API gateway port
    ///
    /// **Environment**: `API_GATEWAY_PORT`\
    /// **Default**: `8080`\
    /// **Usage**: Public-facing API gateway
    #[must_use]
    pub fn api_gateway() -> u16 {
        env::var("API_GATEWAY_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8080)
    }

    /// WebSocket server port
    ///
    /// **Environment**: `WEBSOCKET_PORT`\
    /// **Default**: `8448`\
    /// **Usage**: WebSocket transport layer
    #[must_use]
    pub fn websocket() -> u16 {
        env::var("WEBSOCKET_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8448)
    }

    /// Metrics/monitoring port
    ///
    /// **Environment**: `METRICS_PORT`\
    /// **Default**: `9090`\
    /// **Usage**: Prometheus metrics endpoint
    #[must_use]
    pub fn metrics() -> u16 {
        env::var("METRICS_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(9090)
    }

    /// Health check port
    ///
    /// **Environment**: `HEALTH_PORT`\
    /// **Default**: `9091`\
    /// **Usage**: Health check endpoint
    #[must_use]
    pub fn health() -> u16 {
        env::var("HEALTH_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(9091)
    }

    /// CLI MCP server port
    ///
    /// **Environment**: `CLI_MCP_PORT`\
    /// **Default**: `9000`\
    /// **Usage**: CLI MCP server
    #[must_use]
    pub fn cli_mcp() -> u16 {
        env::var("CLI_MCP_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(9000)
    }

    /// `PostgreSQL` database port
    ///
    /// **Environment**: `POSTGRES_PORT` or `DATABASE_PORT`\
    /// **Default**: `5432`\
    /// **Usage**: `PostgreSQL` database connections
    #[must_use]
    pub fn postgres() -> u16 {
        env::var("POSTGRES_PORT")
            .or_else(|_| env::var("DATABASE_PORT"))
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5432)
    }
}

/// Service name configuration with environment variable support
pub mod services {
    use std::env;

    /// Security service name (capability: security)
    ///
    /// **Environment**: `SECURITY_SERVICE_NAME`\
    /// **Default**: `"security"`\
    /// **Usage**: Capability-based service discovery for security
    #[must_use]
    pub fn security() -> String {
        env::var("SECURITY_SERVICE_NAME").unwrap_or_else(|_| "security".to_string())
    }

    /// Orchestration service name (capability: orchestration)
    ///
    /// **Environment**: `ORCHESTRATION_SERVICE_NAME`\
    /// **Default**: `"service-mesh"`\
    /// **Usage**: Capability-based service discovery for orchestration
    #[must_use]
    pub fn orchestration() -> String {
        env::var("ORCHESTRATION_SERVICE_NAME").unwrap_or_else(|_| "service-mesh".to_string())
    }

    /// Storage service name (capability: storage)
    ///
    /// **Environment**: `STORAGE_SERVICE_NAME`\
    /// **Default**: `"storage"`\
    /// **Usage**: Capability-based service discovery for storage
    #[must_use]
    pub fn storage() -> String {
        env::var("STORAGE_SERVICE_NAME").unwrap_or_else(|_| "storage".to_string())
    }

    /// Compute service name (capability: compute)
    ///
    /// **Environment**: `COMPUTE_SERVICE_NAME`\
    /// **Default**: `"compute"`\
    /// **Usage**: Capability-based service discovery for compute
    #[must_use]
    pub fn compute() -> String {
        env::var("COMPUTE_SERVICE_NAME").unwrap_or_else(|_| "compute".to_string())
    }

    /// AI service name (capability: ai)
    ///
    /// **Environment**: `AI_SERVICE_NAME`\
    /// **Default**: `"ai"`\
    /// **Usage**: Capability-based service discovery for AI
    #[must_use]
    pub fn ai() -> String {
        env::var("AI_SERVICE_NAME").unwrap_or_else(|_| "ai".to_string())
    }
}

/// Host configuration with environment variable support
pub mod hosts {
    use std::env;

    /// Default host for development
    ///
    /// **Environment**: `SQUIRREL_HOST`\
    /// **Default**: `"localhost"`\
    /// **Usage**: Default hostname for local development
    #[must_use]
    pub fn default() -> String {
        env::var("SQUIRREL_HOST").unwrap_or_else(|_| "localhost".to_string())
    }

    /// Localhost address (127.0.0.1)
    ///
    /// **Usage**: Explicit localhost IP for development
    #[must_use]
    pub fn localhost() -> String {
        "127.0.0.1".to_string()
    }

    /// All interfaces address (0.0.0.0)
    ///
    /// **Usage**: Bind to all network interfaces (production)
    #[must_use]
    pub fn all_interfaces() -> String {
        "0.0.0.0".to_string()
    }

    /// MCP server host
    ///
    /// **Environment**: `MCP_SERVER_HOST`\
    /// **Default**: Same as `default()`\
    /// **Usage**: MCP server hostname
    #[must_use]
    pub fn mcp_server() -> String {
        env::var("MCP_SERVER_HOST").unwrap_or_else(|_| default())
    }

    /// Service mesh/orchestration service host
    ///
    /// **Environment**: `SERVICE_MESH_HOST`\
    /// **Default**: Same as `default()`\
    /// **Usage**: Service mesh/orchestration hostname (capability-based)
    #[must_use]
    pub fn service_mesh() -> String {
        env::var("SERVICE_MESH_HOST").unwrap_or_else(|_| default())
    }

    /// Security service host
    ///
    /// **Environment**: `SECURITY_SERVICE_HOST`\
    /// **Default**: Same as `default()`\
    /// **Usage**: Security service hostname (capability-based)
    #[must_use]
    pub fn security_service() -> String {
        env::var("SECURITY_SERVICE_HOST").unwrap_or_else(|_| default())
    }

    /// Storage service host
    ///
    /// **Environment**: `STORAGE_SERVICE_HOST`\
    /// **Default**: Same as `default()`\
    /// **Usage**: Storage service hostname (capability-based)
    #[must_use]
    pub fn storage_service() -> String {
        env::var("STORAGE_SERVICE_HOST").unwrap_or_else(|_| default())
    }

    /// Compute service host
    ///
    /// **Environment**: `COMPUTE_SERVICE_HOST`\
    /// **Default**: Same as `default()`\
    /// **Usage**: Compute service hostname (capability-based)
    #[must_use]
    pub fn compute_service() -> String {
        env::var("COMPUTE_SERVICE_HOST").unwrap_or_else(|_| default())
    }
}

/// Endpoint builders combining hosts, ports, and services
pub mod endpoints {
    use std::env;

    use super::{hosts, ports, services};

    /// Build MCP server endpoint
    ///
    /// **Format**: `http://{host}:{port}`\
    /// **Example**: `http://localhost:8443`
    #[must_use]
    pub fn mcp_server() -> String {
        let host = hosts::mcp_server();
        let port = ports::mcp_server();
        format!("http://{host}:{port}")
    }

    /// Build service mesh endpoint
    ///
    /// **Format**: `http://{host}:{port}/{service}`\
    /// **Example**: `http://localhost:8444/service-mesh`
    #[must_use]
    pub fn service_mesh() -> String {
        let host = hosts::service_mesh();
        let port = ports::service_mesh();
        let svc = services::orchestration();
        format!("http://{host}:{port}/{svc}")
    }

    /// Build security service endpoint
    ///
    /// **Format**: `http://{host}:{port}/{service}`\
    /// **Example**: `http://localhost:8445/security`
    #[must_use]
    pub fn security_service() -> String {
        let host = hosts::security_service();
        let port = ports::security_service();
        let svc = services::security();
        format!("http://{host}:{port}/{svc}")
    }

    /// Build storage service endpoint
    ///
    /// **Format**: `http://{host}:{port}/{service}`\
    /// **Example**: `http://localhost:8446/storage`
    #[must_use]
    pub fn storage_service() -> String {
        let host = hosts::storage_service();
        let port = ports::storage_service();
        let svc = services::storage();
        format!("http://{host}:{port}/{svc}")
    }

    /// Build compute service endpoint
    ///
    /// **Format**: `http://{host}:{port}/{service}`\
    /// **Example**: `http://localhost:8447/compute`
    #[must_use]
    pub fn compute_service() -> String {
        let host = hosts::compute_service();
        let port = ports::compute_service();
        let svc = services::compute();
        format!("http://{host}:{port}/{svc}")
    }

    /// Get `BiomeOS` UI endpoint (default: <http://localhost:3000>)
    ///
    /// **Environment**: `BIOMEOS_UI_ENDPOINT`\
    /// **Default**: <http://localhost:3000>\
    /// **Example**: Frontend web UI endpoint
    #[must_use]
    pub fn biomeos_ui() -> String {
        let host = hosts::default();
        env::var("BIOMEOS_UI_ENDPOINT").unwrap_or_else(|_| format!("http://{host}:3000"))
    }

    /// Get Ollama endpoint (default: <http://localhost:11434>)
    ///
    /// **Environment**: `OLLAMA_ENDPOINT`\
    /// **Default**: <http://localhost:11434>\
    /// **Example**: Local Ollama AI model server
    #[must_use]
    pub fn ollama() -> String {
        let host = hosts::default();
        env::var("OLLAMA_ENDPOINT").unwrap_or_else(|_| format!("http://{host}:11434"))
    }

    /// Build WebSocket endpoint
    ///
    /// **Format**: `ws://{host}:{port}`\
    /// **Example**: `ws://localhost:8448`
    #[must_use]
    pub fn websocket() -> String {
        let host = hosts::default();
        let port = ports::websocket();
        format!("ws://{host}:{port}")
    }

    /// Build metrics endpoint
    ///
    /// **Format**: `http://{host}:{port}/metrics`\
    /// **Example**: `http://localhost:9090/metrics`
    #[must_use]
    pub fn metrics() -> String {
        let host = hosts::default();
        let port = ports::metrics();
        format!("http://{host}:{port}/metrics")
    }

    /// Build health check endpoint
    ///
    /// **Format**: `http://{host}:{port}/health`\
    /// **Example**: `http://localhost:9091/health`
    #[must_use]
    pub fn health() -> String {
        let host = hosts::default();
        let port = ports::health();
        format!("http://{host}:{port}/health")
    }
}

/// Configuration validation helpers
pub mod validation {
    use std::net::{IpAddr, ToSocketAddrs};

    /// Validate that a port is in valid range (1-65535)
    #[must_use]
    pub const fn is_valid_port(port: u16) -> bool {
        port > 0
    }

    /// Validate that a hostname is resolvable
    #[must_use]
    pub fn is_resolvable_host(host: &str) -> bool {
        // Check if it's a valid IP address
        if host.parse::<IpAddr>().is_ok() {
            return true;
        }

        // Try to resolve as hostname
        format!("{host}:80").to_socket_addrs().is_ok()
    }

    /// Validate a complete endpoint URL (basic validation)
    #[must_use]
    pub fn is_valid_endpoint(endpoint: &str) -> bool {
        // Basic validation: check for protocol and structure
        endpoint.starts_with("http://")
            || endpoint.starts_with("https://")
            || endpoint.starts_with("ws://")
            || endpoint.starts_with("wss://")
    }
}

#[cfg(test)]
mod tests {
    use super::{endpoints, hosts, ports, services, validation};

    #[test]
    fn test_default_ports() {
        // Test default port values (capability-based)
        assert_eq!(ports::mcp_server(), 8443);
        assert_eq!(ports::security_service(), 8443);
        assert_eq!(ports::service_mesh(), 8444);
        assert_eq!(ports::storage_service(), 8445);
        assert_eq!(ports::compute_service(), 8446);
        assert_eq!(ports::api_gateway(), 8080);
    }

    #[test]
    fn test_default_services() {
        // Updated to expect capability names, not primal names
        assert_eq!(services::security(), "security");
        assert_eq!(services::orchestration(), "service-mesh");
        assert_eq!(services::storage(), "storage");
        assert_eq!(services::compute(), "compute");
        assert_eq!(services::ai(), "ai");
    }

    #[test]
    fn test_default_hosts() {
        assert_eq!(hosts::default(), "localhost");
        assert_eq!(hosts::mcp_server(), "localhost");
    }

    #[test]
    fn test_endpoint_builders() {
        assert!(endpoints::mcp_server().starts_with("http://"));
        assert!(endpoints::websocket().starts_with("ws://"));
        assert!(endpoints::security_service().contains("security"));
        assert!(endpoints::service_mesh().contains("service-mesh"));
    }

    #[test]
    fn test_port_validation() {
        assert!(validation::is_valid_port(8080));
        assert!(validation::is_valid_port(1));
        assert!(validation::is_valid_port(65535));
    }

    #[test]
    fn test_host_validation() {
        assert!(validation::is_resolvable_host("127.0.0.1"));
        assert!(validation::is_resolvable_host("localhost"));
    }

    #[test]
    fn test_invalid_port_zero() {
        assert!(!validation::is_valid_port(0));
    }

    #[test]
    fn test_endpoint_validation() {
        assert!(validation::is_valid_endpoint("http://localhost:8080"));
        assert!(validation::is_valid_endpoint("https://api.example.com"));
        assert!(validation::is_valid_endpoint("ws://localhost:8080"));
        assert!(validation::is_valid_endpoint("wss://secure.example.com"));
        assert!(!validation::is_valid_endpoint("ftp://example.com"));
        assert!(!validation::is_valid_endpoint("invalid"));
    }

    #[test]
    fn test_ports_websocket_health_cli_postgres() {
        assert_eq!(ports::websocket(), 8448);
        assert_eq!(ports::health(), 9091);
        assert_eq!(ports::cli_mcp(), 9000);
        assert_eq!(ports::postgres(), 5432);
    }

    #[test]
    fn test_hosts_localhost_all_interfaces() {
        assert_eq!(hosts::localhost(), "127.0.0.1");
        assert_eq!(hosts::all_interfaces(), "0.0.0.0");
    }

    #[test]
    fn test_endpoints_biomeos_ollama_metrics_health() {
        assert!(endpoints::biomeos_ui().starts_with("http://"));
        assert!(endpoints::ollama().contains("11434"));
        assert!(endpoints::metrics().contains("/metrics"));
        assert!(endpoints::health().contains("/health"));
    }
}
