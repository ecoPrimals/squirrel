//! Default configuration values for AI tools
//!
//! This module provides environment-driven defaults to eliminate hardcoded values.

use std::env;
// Removed: use squirrel_mcp_config::get_service_endpoints;

/// Default AI service endpoints with environment override support
pub struct DefaultEndpoints;

impl DefaultEndpoints {
    /// Get Ollama endpoint from environment or default
    pub fn ollama_endpoint() -> String {
        env::var("OLLAMA_ENDPOINT").unwrap_or_else(|_| {
            let host = env::var("TOADSTOOL_HOST")
                .unwrap_or_else(|_| "localhost".to_string());
            format!("http://{}:11434", host)
        })
    }

    /// Get LlamaCpp endpoint from environment or default
    pub fn llamacpp_endpoint() -> String {
        env::var("LLAMACPP_ENDPOINT")
            .unwrap_or_else(|_| "http://127.0.0.1:8444".to_string())
    }

    /// Get MCP server endpoint from environment or default
    pub fn mcp_server_endpoint() -> String {
        env::var("MCP_SERVER_ENDPOINT").unwrap_or_else(|_| {
            let host = env::var("MCP_HOST")
                .unwrap_or_else(|_| "localhost".to_string());
            format!("{}:50051", host)
        })
    }

    /// Get general AI service host from environment or default
    pub fn ai_service_host() -> String {
        env::var("AI_SERVICE_HOST")
            .or_else(|_| env::var("MCP_HOST"))
            .unwrap_or_else(|_| "localhost".to_string())
    }

    /// Get development server host from environment or default
    pub fn dev_server_host() -> String {
        env::var("DEV_SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string())
    }

    /// Get service discovery port from environment or default
    pub fn service_discovery_port() -> u16 {
        env::var("SERVICE_DISCOVERY_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .unwrap_or(8080)
    }

    /// Get CLI MCP host from environment or default
    pub fn cli_mcp_host() -> String {
        env::var("CLI_MCP_HOST")
            .or_else(|_| env::var("MCP_HOST"))
            .unwrap_or_else(|_| "localhost".to_string())
    }

    /// Get WebSocket server URL from environment or default
    pub fn websocket_server_url() -> String {
        env::var("MCP_SERVER_URL").unwrap_or_else(|_| "ws://127.0.0.1:8080".to_string())
    }

    /// Get security service endpoint from environment (capability-based discovery)
    /// Note: This is a fallback - services should use capability discovery
    pub fn security_service_endpoint() -> String {
        env::var("SECURITY_SERVICE_ENDPOINT")
            .or_else(|_| env::var("SECURITY_AUTH_SERVICE_ENDPOINT"))
            .unwrap_or_else(|_| "http://localhost:8443".to_string())
    }

    /// Get Songbird endpoint from environment or default
    pub fn songbird_endpoint() -> String {
        env::var("SERVICE_MESH_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8500".to_string())
    }

    /// Get ToadStool endpoint from environment or default
    pub fn toadstool_endpoint() -> String {
        env::var("TOADSTOOL_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:9001".to_string())
    }

    /// Get network host from environment or default
    pub fn network_host() -> String {
        env::var("NETWORK_HOST").unwrap_or_else(|_| "127.0.0.1".to_string())
    }

    /// Build endpoint URL with host and port
    pub fn build_endpoint(host: &str, port: u16, scheme: &str) -> String {
        format!("{}://{}:{}", scheme, host, port)
    }

    /// Build health endpoint URL
    pub fn health_endpoint(base_url: &str) -> String {
        format!("{}/health", base_url.trim_end_matches('/'))
    }

    /// Build metrics endpoint URL
    pub fn metrics_endpoint(base_url: &str) -> String {
        format!("{}/metrics", base_url.trim_end_matches('/'))
    }

    /// Build admin endpoint URL
    pub fn admin_endpoint(base_url: &str) -> String {
        format!("{}/admin", base_url.trim_end_matches('/'))
    }
}

/// Environment variable documentation
pub const ENV_DOCS: &str = r#"
Environment Variables for AI Tools Configuration:

AI Services:
- OLLAMA_ENDPOINT: Ollama service endpoint (default: http://localhost:11434)
- LLAMACPP_ENDPOINT: LlamaCpp service endpoint (default: http://localhost:8080)
- AI_SERVICE_HOST: General AI service host (default: localhost)

MCP Protocol:
- MCP_SERVER_ENDPOINT: MCP server endpoint (default: localhost:50051)
- MCP_SERVER_URL: WebSocket server URL (default: ws://127.0.0.1:8080)
- CLI_MCP_HOST: CLI MCP host (default: localhost)

Security Services (Capability-Based Discovery):
- SECURITY_SERVICE_ENDPOINT: Security service main endpoint
- SECURITY_AUTH_SERVICE_ENDPOINT: Authentication service endpoint
- SECURITY_ENCRYPTION_SERVICE_ENDPOINT: Encryption service endpoint
- SECURITY_COMPLIANCE_SERVICE_ENDPOINT: Compliance service endpoint
- SECURITY_SERVICE_HOST: Security service host (default: localhost)
- SECURITY_AUTHENTICATION_PORT: Authentication service port (default: 8443)
- SECURITY_ENCRYPTION_PORT: Encryption service port (default: 8444)
- SECURITY_COMPLIANCE_PORT: Compliance service port (default: 8445)

ecoPrimals Services:
- SONGBIRD_ENDPOINT: Songbird orchestration service (default: http://localhost:8080)
- TOADSTOOL_ENDPOINT: ToadStool compute service (default: http://localhost:9001)

Network:
- DEV_SERVER_HOST: Development server host (default: 127.0.0.1)
- NETWORK_HOST: Network host for services (default: 127.0.0.1)
- SERVICE_DISCOVERY_PORT: Service discovery port (default: 8080)

Note: Security services use capability-based discovery. The endpoints above are fallbacks.
Services should discover security capabilities through the universal service registry.
"#;
