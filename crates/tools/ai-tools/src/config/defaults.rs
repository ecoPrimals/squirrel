// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Default configuration values for AI tools
//!
//! This module provides environment-driven defaults to eliminate hardcoded values.

use std::env;

use universal_constants::config_helpers;
use universal_constants::deployment::ports;
use universal_constants::network::{LOCALHOST_IPV4, get_service_port};

/// Default AI service endpoints with environment override support
pub struct DefaultEndpoints;

impl DefaultEndpoints {
    /// Get local AI server endpoint from environment or default
    ///
    /// Agnostic: works with any OpenAI-compatible local server (Ollama, llama.cpp, vLLM, etc.)
    /// Checks `LOCAL_AI_ENDPOINT` first, then falls back to legacy vendor-specific env vars.
    #[must_use]
    pub fn local_server_endpoint() -> String {
        env::var("LOCAL_AI_ENDPOINT")
            .or_else(|_| env::var("OLLAMA_ENDPOINT"))
            .or_else(|_| env::var("LLAMACPP_ENDPOINT"))
            .unwrap_or_else(|_| {
                let host = env::var("LOCAL_AI_HOST")
                    .or_else(|_| env::var("TOADSTOOL_HOST"))
                    .unwrap_or_else(|_| config_helpers::get_host("LOCAL_AI_HOST", "localhost"));
                let port = env::var("LOCAL_AI_PORT")
                    .or_else(|_| env::var("OLLAMA_PORT"))
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or_else(ports::ollama);
                format!("http://{host}:{port}")
            })
    }

    /// Backward-compatible alias
    #[must_use]
    pub fn ollama_endpoint() -> String {
        Self::local_server_endpoint()
    }

    /// Backward-compatible alias
    #[must_use]
    pub fn llamacpp_endpoint() -> String {
        Self::local_server_endpoint()
    }

    /// Get MCP server endpoint from environment or default
    #[must_use]
    pub fn mcp_server_endpoint() -> String {
        env::var("MCP_SERVER_ENDPOINT").unwrap_or_else(|_| {
            let host = env::var("MCP_HOST")
                .unwrap_or_else(|_| config_helpers::get_host("MCP_HOST", "localhost"));
            let port = config_helpers::get_port(
                "MCP_GRPC_PORT",
                universal_constants::network::DEFAULT_GRPC_PORT,
            );
            format!("{host}:{port}")
        })
    }

    /// Get general AI service host from environment or default
    #[must_use]
    pub fn ai_service_host() -> String {
        env::var("AI_SERVICE_HOST")
            .or_else(|_| env::var("MCP_HOST"))
            .unwrap_or_else(|_| "localhost".to_string())
    }

    /// Get development server host from environment or default
    #[must_use]
    pub fn dev_server_host() -> String {
        env::var("DEV_SERVER_HOST")
            .unwrap_or_else(|_| config_helpers::get_host("DEV_SERVER_HOST", LOCALHOST_IPV4))
    }

    /// Get service discovery port from environment or default
    #[must_use]
    pub fn service_discovery_port() -> u16 {
        env::var("SERVICE_DISCOVERY_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or_else(|| get_service_port("discovery"))
    }

    /// Get CLI MCP host from environment or default
    #[must_use]
    pub fn cli_mcp_host() -> String {
        env::var("CLI_MCP_HOST")
            .or_else(|_| env::var("MCP_HOST"))
            .unwrap_or_else(|_| "localhost".to_string())
    }

    /// Get WebSocket server URL from environment or default
    ///
    /// Multi-tier resolution:
    /// 1. `MCP_SERVER_URL` (full URL override)
    /// 2. `MCP_SERVER_PORT` (port override)
    /// 3. Default: <ws://127.0.0.1> with port from `get_service_port("websocket")`
    #[must_use]
    pub fn websocket_server_url() -> String {
        env::var("MCP_SERVER_URL").unwrap_or_else(|_| {
            let port = env::var("MCP_SERVER_PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or_else(|| get_service_port("websocket"));
            format!("ws://{LOCALHOST_IPV4}:{port}")
        })
    }

    /// Get security service endpoint from environment (capability-based discovery)
    /// Note: This is a fallback - services should use capability discovery
    ///
    /// Multi-tier resolution:
    /// 1. `SECURITY_SERVICE_ENDPOINT` (full endpoint)
    /// 2. `SECURITY_AUTH_SERVICE_ENDPOINT` (alt full endpoint)
    /// 3. `SECURITY_AUTHENTICATION_PORT` (port override)
    /// 4. Fallback: `SECURITY_SERVICE_PORT` / capability default from [`ports::security_service`]
    #[must_use]
    pub fn security_service_endpoint() -> String {
        env::var("SECURITY_SERVICE_ENDPOINT")
            .or_else(|_| env::var("SECURITY_AUTH_SERVICE_ENDPOINT"))
            .unwrap_or_else(|_| {
                let host = config_helpers::get_host("SECURITY_SERVICE_HOST", "localhost");
                let port = env::var("SECURITY_AUTHENTICATION_PORT")
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or_else(|| {
                        config_helpers::get_port("SECURITY_SERVICE_PORT", ports::security_service())
                    });
                format!("http://{host}:{port}")
            })
    }

    /// Get ecosystem registry (service mesh) endpoint from environment or default
    ///
    /// Env vars are runtime config; `SONGBIRD_ENDPOINT` / `SONGBIRD_PORT` remain backward-compatible.
    /// Code treats this as ecosystem role endpoint, not hardcoded primal identity.
    ///
    /// Multi-tier resolution:
    /// 1. `SERVICE_MESH_ENDPOINT` (capability-based)
    /// 2. `SONGBIRD_ENDPOINT` (legacy full URL)
    /// 3. `SERVICE_MESH_PORT` or `SONGBIRD_PORT` (port override)
    /// 4. Fallback: [`ports::service_mesh`] when resolving port via `config_helpers::get_port`
    #[must_use]
    pub fn service_mesh_endpoint() -> String {
        env::var("SERVICE_MESH_ENDPOINT")
            .or_else(|_| env::var("SONGBIRD_ENDPOINT"))
            .unwrap_or_else(|_| {
                let host = config_helpers::get_host("SERVICE_MESH_HOST", "localhost");
                let port = env::var("SERVICE_MESH_PORT")
                    .or_else(|_| env::var("SONGBIRD_PORT"))
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or_else(|| {
                        config_helpers::get_port("SERVICE_MESH_PORT", ports::service_mesh())
                    });
                format!("http://{host}:{port}")
            })
    }

    /// Get `ToadStool` endpoint from environment or default
    ///
    /// Multi-tier resolution:
    /// 1. `TOADSTOOL_ENDPOINT` (full endpoint)
    /// 2. `TOADSTOOL_PORT` (port override)
    /// 3. Fallback: [`ports::compute_service`] (compute capability — no compile-time primal port)
    #[must_use]
    pub fn toadstool_endpoint() -> String {
        env::var("TOADSTOOL_ENDPOINT").unwrap_or_else(|_| {
            let host = config_helpers::get_host("TOADSTOOL_HOST", "localhost");
            let port = config_helpers::get_port("TOADSTOOL_PORT", ports::compute_service());
            format!("http://{host}:{port}")
        })
    }

    /// Get network host from environment or default
    #[must_use]
    pub fn network_host() -> String {
        env::var("NETWORK_HOST")
            .unwrap_or_else(|_| config_helpers::get_host("NETWORK_HOST", LOCALHOST_IPV4))
    }

    /// Build endpoint URL with host and port
    #[must_use]
    pub fn build_endpoint(host: &str, port: u16, scheme: &str) -> String {
        format!("{scheme}://{host}:{port}")
    }

    /// Build health endpoint URL
    #[must_use]
    pub fn health_endpoint(base_url: &str) -> String {
        format!("{}/health", base_url.trim_end_matches('/'))
    }

    /// Build metrics endpoint URL
    #[must_use]
    pub fn metrics_endpoint(base_url: &str) -> String {
        format!("{}/metrics", base_url.trim_end_matches('/'))
    }

    /// Build admin endpoint URL
    #[must_use]
    pub fn admin_endpoint(base_url: &str) -> String {
        format!("{}/admin", base_url.trim_end_matches('/'))
    }
}

/// Environment variable documentation
pub const ENV_DOCS: &str = r"
Environment Variables for AI Tools Configuration:

AI Services (Vendor-Agnostic):
- LOCAL_AI_ENDPOINT: Local AI server endpoint (default: http://localhost:11434)
- LOCAL_AI_HOST: Local AI server host (default: localhost)
- AI_SERVICE_HOST: General AI service host (default: localhost)
- OLLAMA_ENDPOINT: Backward-compatible alias for LOCAL_AI_ENDPOINT
- LLAMACPP_ENDPOINT: Backward-compatible alias for LOCAL_AI_ENDPOINT

MCP Protocol:
- MCP_SERVER_ENDPOINT: MCP server endpoint (host from MCP_HOST; port from MCP_GRPC_PORT or DEFAULT_GRPC_PORT)
- MCP_GRPC_PORT: gRPC-style MCP listener port (legacy compat)
- MCP_SERVER_URL: WebSocket server URL (default: ws://127.0.0.1 with port from WEBSOCKET_PORT)
- CLI_MCP_HOST: CLI MCP host (default: localhost)

Security Services (Capability-Based Discovery):
- SECURITY_SERVICE_ENDPOINT: Security service main endpoint
- SECURITY_AUTH_SERVICE_ENDPOINT: Authentication service endpoint
- SECURITY_ENCRYPTION_SERVICE_ENDPOINT: Encryption service endpoint
- SECURITY_COMPLIANCE_SERVICE_ENDPOINT: Compliance service endpoint
- SECURITY_SERVICE_HOST: Security service host (default: localhost)
- SECURITY_AUTHENTICATION_PORT: Authentication service port (fallback via SECURITY_SERVICE_PORT / capability defaults)
- SECURITY_ENCRYPTION_PORT: Encryption service port (env-driven)
- SECURITY_COMPLIANCE_PORT: Compliance service port (env-driven)

ecoPrimals Services (ecosystem role endpoints, not primal identity):
- SERVICE_MESH_ENDPOINT: Ecosystem registry / service mesh (primary)
- SONGBIRD_ENDPOINT: Ecosystem registry (legacy alias for full URL)
- SONGBIRD_PORT / SERVICE_MESH_PORT: Port overrides (fallback from deployment capability defaults)
- TOADSTOOL_ENDPOINT: Compute capability endpoint (full URL)
- TOADSTOOL_PORT: Port override (fallback: COMPUTE_SERVICE_PORT / capability default)

Network:
- DEV_SERVER_HOST: Development server host (default: 127.0.0.1)
- NETWORK_HOST: Network host for services (default: 127.0.0.1)
- SERVICE_DISCOVERY_PORT: Service discovery port (default: 8080)

Note: Security services use capability-based discovery. The endpoints above are fallbacks.
Services should discover security capabilities through the universal service registry.
";

#[cfg(test)]
mod tests {
    use super::DefaultEndpoints;

    #[test]
    fn build_endpoint_https() {
        assert_eq!(
            DefaultEndpoints::build_endpoint("10.0.0.1", 443, "https"),
            "https://10.0.0.1:443"
        );
    }

    #[test]
    fn health_metrics_admin_trim_slash() {
        let base = "http://localhost:8080/";
        assert_eq!(
            DefaultEndpoints::health_endpoint(base),
            "http://localhost:8080/health"
        );
        assert_eq!(
            DefaultEndpoints::metrics_endpoint(base),
            "http://localhost:8080/metrics"
        );
        assert_eq!(
            DefaultEndpoints::admin_endpoint(base),
            "http://localhost:8080/admin"
        );
    }

    #[test]
    fn endpoint_aliases_match_local() {
        temp_env::with_vars_unset(
            [
                "LOCAL_AI_ENDPOINT",
                "OLLAMA_ENDPOINT",
                "LLAMACPP_ENDPOINT",
                "LOCAL_AI_HOST",
                "LOCAL_AI_PORT",
                "OLLAMA_PORT",
            ],
            || {
                let a = DefaultEndpoints::local_server_endpoint();
                let b = DefaultEndpoints::ollama_endpoint();
                let c = DefaultEndpoints::llamacpp_endpoint();
                assert_eq!(a, b);
                assert_eq!(b, c);
            },
        );
    }

    #[test]
    fn local_server_endpoint_full_url_override() {
        temp_env::with_var("LOCAL_AI_ENDPOINT", Some("http://custom:9999"), || {
            assert_eq!(
                DefaultEndpoints::local_server_endpoint(),
                "http://custom:9999"
            );
        });
    }

    #[test]
    fn websocket_url_respects_mcp_server_url() {
        temp_env::with_var("MCP_SERVER_URL", Some("ws://edge/ws"), || {
            assert_eq!(DefaultEndpoints::websocket_server_url(), "ws://edge/ws");
        });
    }

    #[test]
    fn env_docs_is_non_empty() {
        assert!(!super::ENV_DOCS.is_empty());
        assert!(super::ENV_DOCS.contains("LOCAL_AI_ENDPOINT"));
    }

    #[test]
    fn mcp_server_endpoint_uses_env() {
        temp_env::with_var("MCP_SERVER_ENDPOINT", Some("grpc://custom:9"), || {
            assert_eq!(DefaultEndpoints::mcp_server_endpoint(), "grpc://custom:9");
        });
    }

    #[test]
    fn ai_service_host_falls_back() {
        temp_env::with_vars_unset(["AI_SERVICE_HOST", "MCP_HOST"], || {
            assert_eq!(DefaultEndpoints::ai_service_host(), "localhost");
        });
        temp_env::with_var("MCP_HOST", Some("mcp.example"), || {
            assert_eq!(DefaultEndpoints::ai_service_host(), "mcp.example");
        });
    }

    #[test]
    fn dev_server_host_default_uses_localhost_constant() {
        temp_env::with_vars_unset(["DEV_SERVER_HOST"], || {
            let h = DefaultEndpoints::dev_server_host();
            assert!(!h.is_empty());
        });
    }

    #[test]
    fn websocket_url_uses_mcp_server_port() {
        temp_env::with_vars_unset(["MCP_SERVER_URL"], || {
            temp_env::with_var("MCP_SERVER_PORT", Some("7777"), || {
                let u = DefaultEndpoints::websocket_server_url();
                assert!(u.contains("7777"), "got {u}");
            });
        });
    }

    #[test]
    fn security_service_endpoint_respects_full_url() {
        temp_env::with_var(
            "SECURITY_SERVICE_ENDPOINT",
            Some("https://sec.example/full"),
            || {
                assert_eq!(
                    DefaultEndpoints::security_service_endpoint(),
                    "https://sec.example/full"
                );
            },
        );
    }

    #[test]
    fn service_mesh_endpoint_respects_service_mesh_endpoint_env() {
        temp_env::with_var("SERVICE_MESH_ENDPOINT", Some("http://mesh:1"), || {
            assert_eq!(DefaultEndpoints::service_mesh_endpoint(), "http://mesh:1");
        });
    }

    #[test]
    fn toadstool_endpoint_respects_full_url() {
        temp_env::with_var("TOADSTOOL_ENDPOINT", Some("http://toad:2"), || {
            assert_eq!(DefaultEndpoints::toadstool_endpoint(), "http://toad:2");
        });
    }

    #[test]
    fn cli_mcp_host_prefers_cli_env() {
        temp_env::with_var("CLI_MCP_HOST", Some("cli-host"), || {
            assert_eq!(DefaultEndpoints::cli_mcp_host(), "cli-host");
        });
    }
}
