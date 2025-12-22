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

use std::env;

/// Port configuration with environment variable support
pub mod ports {
    use super::*;

    /// MCP server port
    ///
    /// **Environment**: `MCP_SERVER_PORT`  
    /// **Default**: `8443`  
    /// **Usage**: Primary MCP protocol server
    pub fn mcp_server() -> u16 {
        env::var("MCP_SERVER_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8443)
    }

    /// Service mesh orchestration port
    ///
    /// **Environment**: `SERVICE_MESH_PORT`  
    /// **Default**: `8444`  
    /// **Usage**: Service mesh coordination (capability-based discovery)
    /// **Note**: Discovers actual service mesh at runtime, no hardcoded primal names
    pub fn service_mesh() -> u16 {
        env::var("SERVICE_MESH_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8444)
    }

    /// Security service port
    ///
    /// **Environment**: `SECURITY_SERVICE_PORT`  
    /// **Default**: `8443`  
    /// **Usage**: Security and authentication service (capability-based discovery)
    /// **Note**: Discovers actual security provider at runtime
    pub fn security_service() -> u16 {
        env::var("SECURITY_SERVICE_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8443)
    }

    /// Storage service port
    ///
    /// **Environment**: `STORAGE_SERVICE_PORT`  
    /// **Default**: `8445`  
    /// **Usage**: Storage and data management (capability-based discovery)
    /// **Note**: Discovers actual storage provider at runtime
    pub fn storage_service() -> u16 {
        env::var("STORAGE_SERVICE_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8445)
    }

    /// Compute service port
    ///
    /// **Environment**: `COMPUTE_SERVICE_PORT`  
    /// **Default**: `8446`  
    /// **Usage**: Compute and execution service (capability-based discovery)
    /// **Note**: Discovers actual compute provider at runtime
    pub fn compute_service() -> u16 {
        env::var("COMPUTE_SERVICE_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8446)
    }

    // ========================================================================
    // DEPRECATED ALIASES (Backward Compatibility)
    // ========================================================================
    // These will be removed in future versions. Use capability-based names above.

    /// DEPRECATED: Use `security_service()` instead
    #[deprecated(note = "Use security_service() - capability-based discovery")]
    pub fn beardog() -> u16 {
        security_service()
    }

    /// DEPRECATED: Use `service_mesh()` instead
    #[deprecated(note = "Use service_mesh() - capability-based discovery")]
    pub fn songbird() -> u16 {
        service_mesh()
    }

    /// DEPRECATED: Use `storage_service()` instead
    #[deprecated(note = "Use storage_service() - capability-based discovery")]
    pub fn nestgate() -> u16 {
        storage_service()
    }

    /// DEPRECATED: Use `compute_service()` instead
    #[deprecated(note = "Use compute_service() - capability-based discovery")]
    pub fn toadstool() -> u16 {
        compute_service()
    }

    /// API gateway port
    ///
    /// **Environment**: `API_GATEWAY_PORT`  
    /// **Default**: `8080`  
    /// **Usage**: Public-facing API gateway
    pub fn api_gateway() -> u16 {
        env::var("API_GATEWAY_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8080)
    }

    /// WebSocket server port
    ///
    /// **Environment**: `WEBSOCKET_PORT`  
    /// **Default**: `8448`  
    /// **Usage**: WebSocket transport layer
    pub fn websocket() -> u16 {
        env::var("WEBSOCKET_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8448)
    }

    /// Metrics/monitoring port
    ///
    /// **Environment**: `METRICS_PORT`  
    /// **Default**: `9090`  
    /// **Usage**: Prometheus metrics endpoint
    pub fn metrics() -> u16 {
        env::var("METRICS_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(9090)
    }

    /// Health check port
    ///
    /// **Environment**: `HEALTH_PORT`  
    /// **Default**: `9091`  
    /// **Usage**: Health check endpoint
    pub fn health() -> u16 {
        env::var("HEALTH_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(9091)
    }

    /// CLI MCP server port
    ///
    /// **Environment**: `CLI_MCP_PORT`  
    /// **Default**: `9000`  
    /// **Usage**: CLI MCP server
    pub fn cli_mcp() -> u16 {
        env::var("CLI_MCP_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(9000)
    }
}

/// Service name configuration with environment variable support
pub mod services {
    use super::*;

    /// Security service name (capability: security)
    ///
    /// **Environment**: `SECURITY_SERVICE_NAME`  
    /// **Default**: `"security"`  
    /// **Usage**: Capability-based service discovery for security
    pub fn security() -> String {
        env::var("SECURITY_SERVICE_NAME").unwrap_or_else(|_| "security".to_string())
    }

    /// Orchestration service name (capability: orchestration)
    ///
    /// **Environment**: `ORCHESTRATION_SERVICE_NAME`  
    /// **Default**: `"service-mesh"`  
    /// **Usage**: Capability-based service discovery for orchestration
    pub fn orchestration() -> String {
        env::var("ORCHESTRATION_SERVICE_NAME").unwrap_or_else(|_| "service-mesh".to_string())
    }

    /// Storage service name (capability: storage)
    ///
    /// **Environment**: `STORAGE_SERVICE_NAME`  
    /// **Default**: `"storage"`  
    /// **Usage**: Capability-based service discovery for storage
    pub fn storage() -> String {
        env::var("STORAGE_SERVICE_NAME").unwrap_or_else(|_| "storage".to_string())
    }

    /// Compute service name (capability: compute)
    ///
    /// **Environment**: `COMPUTE_SERVICE_NAME`  
    /// **Default**: `"compute"`  
    /// **Usage**: Capability-based service discovery for compute
    pub fn compute() -> String {
        env::var("COMPUTE_SERVICE_NAME").unwrap_or_else(|_| "compute".to_string())
    }

    /// AI service name (capability: ai)
    ///
    /// **Environment**: `AI_SERVICE_NAME`  
    /// **Default**: `"ai"`  
    /// **Usage**: Capability-based service discovery for AI
    pub fn ai() -> String {
        env::var("AI_SERVICE_NAME").unwrap_or_else(|_| "ai".to_string())
    }

    // Deprecated primal name functions for backward compatibility
    #[deprecated(note = "Use security() which now returns capability name, not primal name")]
    pub fn beardog() -> String {
        "beardog".to_string()
    }

    #[deprecated(note = "Use orchestration() which now returns capability name, not primal name")]
    pub fn songbird() -> String {
        "songbird".to_string()
    }

    #[deprecated(note = "Use storage() which now returns capability name, not primal name")]
    pub fn nestgate() -> String {
        "nestgate".to_string()
    }

    #[deprecated(note = "Use compute() which now returns capability name, not primal name")]
    pub fn toadstool() -> String {
        "toadstool".to_string()
    }

    #[deprecated(note = "Use ai() which now returns capability name, not primal name")]
    pub fn squirrel() -> String {
        "squirrel".to_string()
    }
}

/// Host configuration with environment variable support
pub mod hosts {
    use super::*;

    /// Default host for development
    ///
    /// **Environment**: `SQUIRREL_HOST`  
    /// **Default**: `"localhost"`  
    /// **Usage**: Default hostname for local development
    pub fn default() -> String {
        env::var("SQUIRREL_HOST").unwrap_or_else(|_| "localhost".to_string())
    }

    /// Localhost address (127.0.0.1)
    ///
    /// **Usage**: Explicit localhost IP for development
    pub fn localhost() -> String {
        "127.0.0.1".to_string()
    }

    /// All interfaces address (0.0.0.0)
    ///
    /// **Usage**: Bind to all network interfaces (production)
    pub fn all_interfaces() -> String {
        "0.0.0.0".to_string()
    }

    /// MCP server host
    ///
    /// **Environment**: `MCP_SERVER_HOST`  
    /// **Default**: Same as `default()`  
    /// **Usage**: MCP server hostname
    pub fn mcp_server() -> String {
        env::var("MCP_SERVER_HOST").unwrap_or_else(|_| default())
    }

    /// Service mesh/orchestration service host
    ///
    /// **Environment**: `SERVICE_MESH_HOST`  
    /// **Default**: Same as `default()`  
    /// **Usage**: Service mesh/orchestration hostname (capability-based)
    pub fn service_mesh() -> String {
        env::var("SERVICE_MESH_HOST").unwrap_or_else(|_| default())
    }

    /// Security service host
    ///
    /// **Environment**: `SECURITY_SERVICE_HOST`  
    /// **Default**: Same as `default()`  
    /// **Usage**: Security service hostname (capability-based)
    pub fn security_service() -> String {
        env::var("SECURITY_SERVICE_HOST").unwrap_or_else(|_| default())
    }

    /// Storage service host
    ///
    /// **Environment**: `STORAGE_SERVICE_HOST`  
    /// **Default**: Same as `default()`  
    /// **Usage**: Storage service hostname (capability-based)
    pub fn storage_service() -> String {
        env::var("STORAGE_SERVICE_HOST").unwrap_or_else(|_| default())
    }

    /// Compute service host
    ///
    /// **Environment**: `COMPUTE_SERVICE_HOST`  
    /// **Default**: Same as `default()`  
    /// **Usage**: Compute service hostname (capability-based)
    pub fn compute_service() -> String {
        env::var("COMPUTE_SERVICE_HOST").unwrap_or_else(|_| default())
    }

    // Deprecated aliases for backward compatibility
    #[deprecated(note = "Use service_mesh() - capability-based discovery")]
    pub fn songbird() -> String {
        env::var("SONGBIRD_HOST").unwrap_or_else(|_| default())
    }

    #[deprecated(note = "Use security_service() - capability-based discovery")]
    pub fn beardog() -> String {
        env::var("BEARDOG_HOST").unwrap_or_else(|_| default())
    }

    #[deprecated(note = "Use storage_service() - capability-based discovery")]
    pub fn nestgate() -> String {
        env::var("NESTGATE_HOST").unwrap_or_else(|_| default())
    }

    #[deprecated(note = "Use compute_service() - capability-based discovery")]
    pub fn toadstool() -> String {
        env::var("TOADSTOOL_HOST").unwrap_or_else(|_| default())
    }
}

/// Endpoint builders combining hosts, ports, and services
pub mod endpoints {
    use super::*;

    /// Build MCP server endpoint
    ///
    /// **Format**: `http://{host}:{port}`  
    /// **Example**: `http://localhost:8443`
    pub fn mcp_server() -> String {
        format!("http://{}:{}", hosts::mcp_server(), ports::mcp_server())
    }

    /// Build service mesh endpoint
    ///
    /// **Format**: `http://{host}:{port}/{service}`  
    /// **Example**: `http://localhost:8444/service-mesh`
    pub fn service_mesh() -> String {
        format!(
            "http://{}:{}/{}",
            hosts::service_mesh(),
            ports::service_mesh(),
            services::orchestration()
        )
    }

    /// Build security service endpoint
    ///
    /// **Format**: `http://{host}:{port}/{service}`  
    /// **Example**: `http://localhost:8445/security`
    pub fn security_service() -> String {
        format!(
            "http://{}:{}/{}",
            hosts::security_service(),
            ports::security_service(),
            services::security()
        )
    }

    /// Build storage service endpoint
    ///
    /// **Format**: `http://{host}:{port}/{service}`  
    /// **Example**: `http://localhost:8446/storage`
    pub fn storage_service() -> String {
        format!(
            "http://{}:{}/{}",
            hosts::storage_service(),
            ports::storage_service(),
            services::storage()
        )
    }

    /// Build compute service endpoint
    ///
    /// **Format**: `http://{host}:{port}/{service}`  
    /// **Example**: `http://localhost:8447/compute`
    pub fn compute_service() -> String {
        format!(
            "http://{}:{}/{}",
            hosts::compute_service(),
            ports::compute_service(),
            services::compute()
        )
    }

    // Deprecated aliases for backward compatibility
    #[deprecated(note = "Use service_mesh() - capability-based discovery")]
    #[allow(deprecated)]
    pub fn songbird() -> String {
        format!(
            "http://{}:{}/{}",
            hosts::songbird(),
            ports::songbird(),
            services::orchestration()
        )
    }

    #[deprecated(note = "Use security_service() - capability-based discovery")]
    #[allow(deprecated)]
    pub fn beardog() -> String {
        format!(
            "http://{}:{}/{}",
            hosts::beardog(),
            ports::beardog(),
            services::security()
        )
    }

    #[deprecated(note = "Use storage_service() - capability-based discovery")]
    #[allow(deprecated)]
    pub fn nestgate() -> String {
        format!(
            "http://{}:{}/{}",
            hosts::nestgate(),
            ports::nestgate(),
            services::storage()
        )
    }

    #[deprecated(note = "Use compute_service() - capability-based discovery")]
    #[allow(deprecated)]
    pub fn toadstool() -> String {
        format!(
            "http://{}:{}/{}",
            hosts::toadstool(),
            ports::toadstool(),
            services::compute()
        )
    }

    /// Get BiomeOS UI endpoint (default: http://localhost:3000)
    ///
    /// **Environment**: `BIOMEOS_UI_ENDPOINT`  
    /// **Default**: `http://localhost:3000`  
    /// **Example**: Frontend web UI endpoint
    pub fn biomeos_ui() -> String {
        env::var("BIOMEOS_UI_ENDPOINT")
            .unwrap_or_else(|_| format!("http://{}:3000", hosts::default()))
    }

    /// Get Ollama endpoint (default: http://localhost:11434)
    ///
    /// **Environment**: `OLLAMA_ENDPOINT`  
    /// **Default**: `http://localhost:11434`  
    /// **Example**: Local Ollama AI model server
    pub fn ollama() -> String {
        env::var("OLLAMA_ENDPOINT").unwrap_or_else(|_| format!("http://{}:11434", hosts::default()))
    }

    /// Build WebSocket endpoint
    ///
    /// **Format**: `ws://{host}:{port}`  
    /// **Example**: `ws://localhost:8448`
    pub fn websocket() -> String {
        format!("ws://{}:{}", hosts::default(), ports::websocket())
    }

    /// Build metrics endpoint
    ///
    /// **Format**: `http://{host}:{port}/metrics`  
    /// **Example**: `http://localhost:9090/metrics`
    pub fn metrics() -> String {
        format!("http://{}:{}/metrics", hosts::default(), ports::metrics())
    }

    /// Build health check endpoint
    ///
    /// **Format**: `http://{host}:{port}/health`  
    /// **Example**: `http://localhost:9091/health`
    pub fn health() -> String {
        format!("http://{}:{}/health", hosts::default(), ports::health())
    }
}

/// Configuration validation helpers
pub mod validation {
    use std::net::{IpAddr, ToSocketAddrs};

    /// Validate that a port is in valid range (1-65535)
    pub fn is_valid_port(port: u16) -> bool {
        port > 0
    }

    /// Validate that a hostname is resolvable
    pub fn is_resolvable_host(host: &str) -> bool {
        // Check if it's a valid IP address
        if host.parse::<IpAddr>().is_ok() {
            return true;
        }

        // Try to resolve as hostname
        format!("{}:80", host).to_socket_addrs().is_ok()
    }

    /// Validate a complete endpoint URL (basic validation)
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
    use super::*;

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
    #[allow(deprecated)]
    fn test_endpoint_builders() {
        assert!(endpoints::mcp_server().starts_with("http://"));
        assert!(endpoints::websocket().starts_with("ws://"));
        // Updated to test new capability-based endpoint
        assert!(endpoints::security_service().contains("security"));
        // Deprecated endpoint still works but uses deprecated hosts/ports
        let beardog_endpoint = endpoints::beardog();
        assert!(beardog_endpoint.starts_with("http://"));
        // The deprecated beardog() endpoint is now composed of deprecated functions
        // that still use the old names, so it will contain "security" from services::security()
        // which now returns "security" instead of "beardog"
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
}
