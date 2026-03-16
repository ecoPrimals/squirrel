// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Default configuration values for AI tools
//!
//! This module provides environment-driven defaults to eliminate hardcoded values.

use std::env;

use universal_constants::network::{LOCALHOST_IPV4, get_service_port};

/// Default AI service endpoints with environment override support
pub struct DefaultEndpoints;

impl DefaultEndpoints {
    /// Get local AI server endpoint from environment or default
    ///
    /// Agnostic: works with any OpenAI-compatible local server (Ollama, llama.cpp, vLLM, etc.)
    /// Checks LOCAL_AI_ENDPOINT first, then falls back to legacy vendor-specific env vars.
    pub fn local_server_endpoint() -> String {
        env::var("LOCAL_AI_ENDPOINT")
            .or_else(|_| env::var("OLLAMA_ENDPOINT"))
            .or_else(|_| env::var("LLAMACPP_ENDPOINT"))
            .unwrap_or_else(|_| {
                let host = env::var("LOCAL_AI_HOST")
                    .or_else(|_| env::var("TOADSTOOL_HOST"))
                    .unwrap_or_else(|_| "localhost".to_string());
                format!("http://{}:11434", host)
            })
    }

    /// Backward-compatible alias
    pub fn ollama_endpoint() -> String {
        Self::local_server_endpoint()
    }

    /// Backward-compatible alias
    pub fn llamacpp_endpoint() -> String {
        Self::local_server_endpoint()
    }

    /// Get MCP server endpoint from environment or default
    pub fn mcp_server_endpoint() -> String {
        env::var("MCP_SERVER_ENDPOINT").unwrap_or_else(|_| {
            let host = env::var("MCP_HOST").unwrap_or_else(|_| "localhost".to_string());
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
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or_else(|| get_service_port("discovery"))
    }

    /// Get CLI MCP host from environment or default
    pub fn cli_mcp_host() -> String {
        env::var("CLI_MCP_HOST")
            .or_else(|_| env::var("MCP_HOST"))
            .unwrap_or_else(|_| "localhost".to_string())
    }

    /// Get WebSocket server URL from environment or default
    ///
    /// Multi-tier resolution:
    /// 1. MCP_SERVER_URL (full URL override)
    /// 2. MCP_SERVER_PORT (port override)
    /// 3. Default: ws://127.0.0.1 with port from `get_service_port("websocket")`
    pub fn websocket_server_url() -> String {
        env::var("MCP_SERVER_URL").unwrap_or_else(|_| {
            let port = env::var("MCP_SERVER_PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or_else(|| get_service_port("websocket"));
            format!("ws://{}:{port}", LOCALHOST_IPV4)
        })
    }

    /// Get security service endpoint from environment (capability-based discovery)
    /// Note: This is a fallback - services should use capability discovery
    ///
    /// Multi-tier resolution:
    /// 1. SECURITY_SERVICE_ENDPOINT (full endpoint)
    /// 2. SECURITY_AUTH_SERVICE_ENDPOINT (alt full endpoint)
    /// 3. SECURITY_AUTHENTICATION_PORT (port override)
    /// 4. Default: http://localhost:8443
    pub fn security_service_endpoint() -> String {
        env::var("SECURITY_SERVICE_ENDPOINT")
            .or_else(|_| env::var("SECURITY_AUTH_SERVICE_ENDPOINT"))
            .unwrap_or_else(|_| {
                let port = env::var("SECURITY_AUTHENTICATION_PORT")
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(8443); // Default security auth port
                format!("http://localhost:{}", port)
            })
    }

    /// Get ecosystem registry (service mesh) endpoint from environment or default
    ///
    /// Env vars are runtime config - SONGBIRD_ENDPOINT is backward-compatible.
    /// Code treats this as ecosystem role endpoint, not hardcoded primal identity.
    ///
    /// Multi-tier resolution:
    /// 1. SERVICE_MESH_ENDPOINT (capability-based)
    /// 2. SONGBIRD_ENDPOINT (legacy env var, runtime config)
    /// 3. SONGBIRD_PORT (port override)
    /// 4. Default: http://localhost:8500
    pub fn songbird_endpoint() -> String {
        env::var("SERVICE_MESH_ENDPOINT")
            .or_else(|_| env::var("SONGBIRD_ENDPOINT"))
            .unwrap_or_else(|_| {
                let port = env::var("SONGBIRD_PORT")
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(8500); // Default Songbird service mesh port
                format!("http://localhost:{}", port)
            })
    }

    /// Get ToadStool endpoint from environment or default
    ///
    /// Multi-tier resolution:
    /// 1. TOADSTOOL_ENDPOINT (full endpoint)
    /// 2. TOADSTOOL_PORT (port override)
    /// 3. Default: http://localhost:9001
    pub fn toadstool_endpoint() -> String {
        env::var("TOADSTOOL_ENDPOINT").unwrap_or_else(|_| {
            let port = env::var("TOADSTOOL_PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(9001); // Default ToadStool compute port
            format!("http://localhost:{}", port)
        })
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

AI Services (Vendor-Agnostic):
- LOCAL_AI_ENDPOINT: Local AI server endpoint (default: http://localhost:11434)
- LOCAL_AI_HOST: Local AI server host (default: localhost)
- AI_SERVICE_HOST: General AI service host (default: localhost)
- OLLAMA_ENDPOINT: Backward-compatible alias for LOCAL_AI_ENDPOINT
- LLAMACPP_ENDPOINT: Backward-compatible alias for LOCAL_AI_ENDPOINT

MCP Protocol:
- MCP_SERVER_ENDPOINT: MCP server endpoint (default: localhost:50051)
- MCP_SERVER_URL: WebSocket server URL (default: ws://127.0.0.1 with port from WEBSOCKET_PORT)
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

ecoPrimals Services (ecosystem role endpoints, not primal identity):
- SERVICE_MESH_ENDPOINT: Ecosystem registry / service mesh (primary)
- SONGBIRD_ENDPOINT: Ecosystem registry (legacy, default: http://localhost:8500)
- SONGBIRD_PORT: Songbird port override (default: 8500)
- TOADSTOOL_ENDPOINT: ToadStool compute service (default: http://localhost:9001)
- TOADSTOOL_PORT: ToadStool port override (default: 9001)

Network:
- DEV_SERVER_HOST: Development server host (default: 127.0.0.1)
- NETWORK_HOST: Network host for services (default: 127.0.0.1)
- SERVICE_DISCOVERY_PORT: Service discovery port (default: 8080)

Note: Security services use capability-based discovery. The endpoints above are fallbacks.
Services should discover security capabilities through the universal service registry.
"#;
