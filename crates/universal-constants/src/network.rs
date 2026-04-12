// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Network Configuration - Infant Primal Pattern
//!
//! **Philosophy**: Zero hardcoded knowledge. All network configuration discovered at runtime.
//!
//! Following the infant primal pattern:
//! 1. Try environment variables first (`SERVICE_MESH` discovery)
//! 2. Fall back to OS-provided ports (dynamic allocation)
//! 3. Only use defaults as last resort (with warnings)
//!
//! # Migration from Hardcoding
//!
//! **OLD** (Hardcoded):
//! ```rust,ignore
//! const PORT: u16 = 8080;  // ❌ Hardcoded
//! ```
//!
//! **NEW** (Discovery-based):
//! ```rust,ignore
//! let port = get_service_port("websocket");  // ✅ Discovered
//! ```
//!
//! # Categories
//!
//! - **Discovery Functions**: Runtime port/address discovery
//! - **Fallback Defaults**: Used only when discovery fails (with warnings)
//! - **Helper Functions**: URL construction and utilities

use crate::config_helpers;

// ============================================================================
// Runtime Discovery Functions (Use These!)
// ============================================================================

/// Get service port via discovery (infant primal pattern)
///
/// Discovery order:
/// 1. Environment variable `{SERVICE}_PORT` (e.g., `WEBSOCKET_PORT`)
/// 2. Service mesh discovery (future: query service mesh)
/// 3. OS-allocated port (ephemeral port)
/// 4. Fallback default (with warning)
///
/// # Example
///
/// ```rust,ignore
/// // ✅ GOOD: Discovery-based
/// let port = get_service_port("websocket");
///
/// // ❌ BAD: Hardcoded
/// let port = 8080;
/// ```
#[must_use]
pub fn get_service_port(service: &str) -> u16 {
    // 1. Try environment variable (`mcp-tcp` → `MCP_TCP_PORT`)
    let normalized = service.replace('-', "_");
    let svc_upper = normalized.to_uppercase();
    if let Ok(port_str) = std::env::var(format!("{svc_upper}_PORT"))
        && let Ok(port) = port_str.parse::<u16>()
    {
        tracing::debug!("Using port from environment: {}={}", service, port);
        return port;
    }

    // 2. Service mesh discovery (future implementation)
    //    Intended pattern: Query service mesh (e.g. Consul, etcd) for service registration.
    //    Discovery flow: SERVICE_MESH env -> mesh client -> lookup(service) -> port.
    //    When implemented: if let Some(port) = query_service_mesh(service) { return port; }

    // 3. Use fallback (with warning)
    let key = service.to_lowercase().replace('-', "_");
    let fallback_port = match key.as_str() {
        "websocket" | "ws" | "primal" => 8080,
        "http" => 8081,
        "admin" => 8082,
        "security" => 8083,
        "storage" => 8084,
        "ui" => 3000,
        "service_mesh" | "mesh" => 8085,
        "compute" => 8086,
        "mcp_tcp" => 9000,
        "metrics" => 9090,
        "discovery" => 8500,
        _ => {
            tracing::warn!(
                "Unknown service '{}' - using dynamic port allocation recommended",
                service
            );
            0 // Let OS allocate
        }
    };

    if fallback_port > 0 {
        tracing::warn!(
            "Using fallback port for '{}': {} - set {}_PORT environment variable for production",
            service,
            fallback_port,
            svc_upper
        );
    }

    fallback_port
}

/// Get bind address via discovery (infant primal pattern)
///
/// Discovery order:
/// 1. Environment variable `BIND_ADDRESS`
/// 2. Service mesh discovery
/// 3. Fallback to localhost (with warning)
#[must_use]
pub fn get_bind_address() -> String {
    std::env::var("BIND_ADDRESS")
        .or_else(|_| std::env::var("PRIMAL_BIND_ADDRESS"))
        .unwrap_or_else(|_| {
            tracing::warn!(
                "Using fallback bind address: 127.0.0.1 - set BIND_ADDRESS for production"
            );
            "127.0.0.1".to_string()
        })
}

// ============================================================================
// Squirrel self + peer discovery (PRIMAL_IPC: zero compile-time peer ports)
// ============================================================================

/// TCP port for **this** Squirrel primal only (`SQUIRREL_PORT`, then `SQUIRREL_SERVER_PORT`).
///
/// Other primals must not be resolved here — use [`discover_peer_http_origin`] with capability
/// ports from [`crate::deployment::ports`] or Unix socket discovery.
#[must_use]
pub fn squirrel_primal_port() -> u16 {
    std::env::var("SQUIRREL_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| config_helpers::get_port("SQUIRREL_SERVER_PORT", 9010))
}

/// Host/port pair discovered at runtime (env, registry, or IPC) — not hardcoded per primal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredEndpoint {
    /// Hostname or IP address.
    pub host: String,
    /// TCP port (`0` means unknown; resolve via registry/socket before connecting).
    pub port: u16,
}

impl DiscoveredEndpoint {
    /// `http://{host}:{port}` origin (caller should reject `port == 0` if invalid).
    #[must_use]
    pub fn http_origin(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

/// Resolve a peer HTTP base URL without embedding another primal’s port in source.
///
/// Resolution order:
/// 1. `endpoint_env` (full URL) if set and non-empty
/// 2. `http://{host}:{port}` with host from `host_env` (fallback `fallback_host`) and port from
///    `port_env` (fallback `fallback_port`, typically from [`crate::deployment::ports`])
#[must_use]
pub fn discover_peer_http_origin(
    endpoint_env: &str,
    host_env: &str,
    port_env: &str,
    fallback_host: &str,
    fallback_port: u16,
) -> String {
    if let Ok(url) = std::env::var(endpoint_env)
        && !url.is_empty()
    {
        return url;
    }
    let host = config_helpers::get_host(host_env, fallback_host);
    let port = config_helpers::get_port(port_env, fallback_port);
    format!("http://{host}:{port}")
}

// ============================================================================
// Fallback Defaults (Use get_service_port() instead!)
// ============================================================================
//
// These port constants are FALLBACKS for when runtime discovery is unavailable:
// - Environment variables (e.g. SQUIRREL_PORT, WEBSOCKET_PORT) take precedence
// - Service mesh / capability-based discovery is preferred when available
// - Use get_service_port(service) for env-aware, discovery-aware resolution
// - These defaults should only be used in dev/testing or when discovery fails
//
// ============================================================================

/// Fallback bind address (use `get_bind_address()` instead)
///
/// **Deprecated**: Use `get_bind_address()` for runtime discovery
#[deprecated(
    since = "3.0.0",
    note = "Use get_bind_address() for runtime discovery instead of hardcoded constant"
)]
pub const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1";

/// Localhost IPv4 address (informational only)
pub const LOCALHOST_IPV4: &str = "127.0.0.1";

/// Default localhost hostname (informational only)
pub const DEFAULT_LOCALHOST: &str = "localhost";

/// Fallback WebSocket port (use `get_service_port("websocket")` instead)
///
/// **Deprecated**: Use `get_service_port("websocket")` for runtime discovery
#[deprecated(
    since = "3.0.0",
    note = "Use get_service_port(\"websocket\") for runtime discovery"
)]
pub const DEFAULT_WEBSOCKET_PORT: u16 = 8080;

/// Fallback HTTP port (use `get_service_port("http")` instead)
///
/// **Deprecated**: Use `get_service_port("http")` for runtime discovery
#[deprecated(since = "3.0.0", note = "Use get_service_port(\"http\")")]
pub const DEFAULT_HTTP_PORT: u16 = 8081;

/// Fallback admin port (use `get_service_port("admin")` instead)
///
/// **Deprecated**: Use `get_service_port("admin")` for runtime discovery
#[deprecated(since = "3.0.0", note = "Use get_service_port(\"admin\")")]
pub const DEFAULT_ADMIN_PORT: u16 = 8082;

/// Default TCP port for metrics and Prometheus scrape endpoints (`METRICS_PORT` fallback).
pub const DEFAULT_METRICS_LISTEN_PORT: u16 = 9090;

/// Fallback metrics port (use `get_service_port("metrics")` instead)
///
/// **Deprecated**: Use `get_service_port("metrics")` for runtime discovery
#[deprecated(since = "3.0.0", note = "Use get_service_port(\"metrics\")")]
pub const DEFAULT_METRICS_PORT: u16 = DEFAULT_METRICS_LISTEN_PORT;

/// Default TCP port for Consul-style service discovery listens (`discovery` service fallback).
pub const DEFAULT_DISCOVERY_LISTEN_PORT: u16 = 8500;

/// Fallback discovery port (use `get_service_port("discovery")` instead)
///
/// **Deprecated**: Use `get_service_port("discovery")` for runtime discovery
#[deprecated(since = "3.0.0", note = "Use get_service_port(\"discovery\")")]
pub const DEFAULT_DISCOVERY_PORT: u16 = DEFAULT_DISCOVERY_LISTEN_PORT;

/// Legacy security port constant — prefer [`crate::deployment::ports::security_service`].
///
/// Squirrel must not treat this as another primal’s fixed port; capability discovery applies.
pub const DEFAULT_SECURITY_PORT: u16 = 8443;

/// Default TCP listen port for CLI MCP server (`MCP_PORT` fallback in `squirrel-cli`).
///
/// Aligns with [`crate::deployment::ports::service_mesh`] default when coordinating mesh discovery.
pub const DEFAULT_CLI_MCP_PORT: u16 = 8444;

/// Default port for Ollama local inference API (`OLLAMA_PORT` fallback).
///
/// See [`crate::deployment::ports::ollama`].
pub const DEFAULT_OLLAMA_PORT: u16 = 11434;

/// Default base port for medium-memory biomeOS agent deployments.
pub const DEFAULT_AGENT_DEPLOY_BASE_PORT_MEDIUM: u16 = 8085;

/// Default base port for high-memory biomeOS agent deployments.
pub const DEFAULT_AGENT_DEPLOY_BASE_PORT_HIGH: u16 = 8090;

/// Legacy gRPC port constant — gRPC fully removed, kept only for config deserialization compat
pub const DEFAULT_GRPC_PORT: u16 = 50051;

/// Bind to all network interfaces (0.0.0.0)
///
/// Use for production servers that accept external connections.
pub const BIND_ALL_INTERFACES: &str = "0.0.0.0";

/// Fallback orchestration/ecosystem API port (discovery, registration)
///
/// **Deprecated**: Use `universal_constants::deployment::ports::service_mesh()` for
/// env-aware, capability-based port resolution instead.
#[deprecated(
    since = "3.0.0",
    note = "Use deployment::ports::service_mesh() for capability-based discovery"
)]
pub const DEFAULT_SONGBIRD_PORT: u16 = 8001;

/// Fallback Squirrel main server port
pub const DEFAULT_SQUIRREL_SERVER_PORT: u16 = 9010;

/// Default port for JSON-RPC / MCP API server (HTTP)
///
/// Used when binding the main API server (e.g. squirrel-mcp-server).
pub const DEFAULT_JSON_RPC_PORT: u16 = 8080;

/// Default port for `BiomeOS` ecosystem API
///
/// Used for `BiomeOS` AI, MCP, context, health, and metrics endpoints.
pub const DEFAULT_BIOMEOS_PORT: u16 = 5000;

/// Default bind address for API server (all interfaces + JSON-RPC port)
///
/// Format: "0.0.0.0:{port}". Use when starting HTTP API servers.
#[must_use]
pub fn default_api_bind_addr() -> String {
    format!("{BIND_ALL_INTERFACES}:{DEFAULT_JSON_RPC_PORT}")
}

// ============================================================================
// Unix Socket Path Constants (XDG ecosystem convention)
// ============================================================================

/// Subdirectory under `$XDG_RUNTIME_DIR` for all biomeos primal sockets.
///
/// Ecosystem convention (ludoSpring, airSpring, squirrel):
///   Primary: `$XDG_RUNTIME_DIR/biomeos/{primal}.sock`
///   Fallback: `/tmp/biomeos/{primal}.sock`
pub const BIOMEOS_SOCKET_SUBDIR: &str = "biomeos";

/// Fallback base directory when `$XDG_RUNTIME_DIR` is not available.
///
/// Used on systems without a user session manager (containers, CI).
pub const BIOMEOS_SOCKET_FALLBACK_DIR: &str = "/tmp/biomeos";

/// Get the XDG-compliant socket directory for biomeos primals.
///
/// Returns `$XDG_RUNTIME_DIR/biomeos` if the env var is set,
/// otherwise falls back to `/tmp/biomeos`.
#[must_use]
pub fn get_socket_dir() -> std::path::PathBuf {
    std::env::var("XDG_RUNTIME_DIR").map_or_else(
        |_| std::path::PathBuf::from(BIOMEOS_SOCKET_FALLBACK_DIR),
        |xdg| std::path::PathBuf::from(xdg).join(BIOMEOS_SOCKET_SUBDIR),
    )
}

/// Get the full socket path for a named primal service.
///
/// Discovery order:
/// 1. `{SERVICE}_SOCKET` env var (e.g. `SQUIRREL_SOCKET`)
/// 2. `$XDG_RUNTIME_DIR/biomeos/{service}-{family_id}.sock` (with `FAMILY_ID`)
/// 3. `$XDG_RUNTIME_DIR/biomeos/{service}.sock` (no `FAMILY_ID` set)
/// 4. `/tmp/biomeos/{service}.sock` (fallback)
///
/// Per `PRIMAL_IPC_PROTOCOL.md`, the standard socket name includes `FAMILY_ID`
/// when set: `<primal>-${FAMILY_ID}.sock`. This allows multiple primal instances
/// (different families) to coexist.
#[must_use]
pub fn get_socket_path(service: &str) -> std::path::PathBuf {
    let env_key = format!("{}_SOCKET", service.to_uppercase());
    if let Ok(path) = std::env::var(&env_key) {
        return std::path::PathBuf::from(path);
    }
    let filename = match std::env::var("FAMILY_ID") {
        Ok(family_id) if !family_id.is_empty() => format!("{service}-{family_id}.sock"),
        _ => format!("{service}.sock"),
    };
    get_socket_dir().join(filename)
}

/// Tiered Unix socket resolution for capability clients (AI, crypto, security provider, etc.).
///
/// Aligns with `crates/main/src/rpc/unix_socket` (`SocketConfig` / `get_socket_path` tiers):
///
/// 1. `primary_env` if set and non-empty (e.g. `AI_CAPABILITY_SOCKET`, `CRYPTO_CAPABILITY_SOCKET`)
/// 2. `SQUIRREL_SOCKET`
/// 3. `BIOMEOS_SOCKET_PATH`
/// 4. `PRIMAL_SOCKET` with `SQUIRREL_FAMILY_ID` suffix (default family: `default`)
/// 5. `$XDG_RUNTIME_DIR/biomeos/{stem}[-{FAMILY_ID}].sock`, else `/tmp/biomeos/...`
///
/// `biomeos_socket_stem` is the basename without `.sock` (e.g. `ai-provider`, `crypto-provider`).
#[must_use]
pub fn resolve_capability_unix_socket(
    primary_env: &str,
    biomeos_socket_stem: &str,
) -> std::path::PathBuf {
    use std::path::PathBuf;

    if let Ok(p) = std::env::var(primary_env)
        && !p.is_empty()
    {
        return PathBuf::from(p);
    }
    if let Ok(p) = std::env::var("SQUIRREL_SOCKET")
        && !p.is_empty()
    {
        return PathBuf::from(p);
    }
    if let Ok(p) = std::env::var("BIOMEOS_SOCKET_PATH")
        && !p.is_empty()
    {
        return PathBuf::from(p);
    }
    if let Ok(generic) = std::env::var("PRIMAL_SOCKET")
        && !generic.is_empty()
    {
        let family_id =
            std::env::var("SQUIRREL_FAMILY_ID").unwrap_or_else(|_| "default".to_string());
        return PathBuf::from(format!("{generic}-{family_id}"));
    }

    let filename = match std::env::var("FAMILY_ID") {
        Ok(fid) if !fid.is_empty() => format!("{biomeos_socket_stem}-{fid}.sock"),
        _ => format!("{biomeos_socket_stem}.sock"),
    };
    get_socket_dir().join(filename)
}

/// Generic environment variable name for a primal's socket path.
///
/// Pattern absorbed from sweetGrass v0.7.17: `{PRIMAL_NAME}_SOCKET`.
/// Any primal works without code changes — no per-primal constants needed.
///
/// # Example
///
/// ```rust,ignore
/// let key = socket_env_var("rhizocrypt"); // "RHIZOCRYPT_SOCKET"
/// ```
#[must_use]
pub fn socket_env_var(primal_name: &str) -> String {
    format!("{}_SOCKET", primal_name.to_uppercase())
}

/// Generic environment variable name for a primal's network address.
///
/// Pattern absorbed from sweetGrass v0.7.17: `{PRIMAL_NAME}_ADDRESS`.
///
/// # Example
///
/// ```rust,ignore
/// let key = address_env_var("rhizocrypt"); // "RHIZOCRYPT_ADDRESS"
/// ```
#[must_use]
pub fn address_env_var(primal_name: &str) -> String {
    format!("{}_ADDRESS", primal_name.to_uppercase())
}

// ============================================================================
// URL Templates
// ============================================================================

/// Default localhost HTTP URL template
///
/// Format: "http://localhost:{port}"
pub const LOCALHOST_HTTP_TEMPLATE: &str = "http://localhost:{}";

/// Default localhost WebSocket URL template
///
/// Format: "ws://localhost:{port}"
pub const LOCALHOST_WS_TEMPLATE: &str = "ws://localhost:{}";

// ============================================================================
// Endpoint Paths
// ============================================================================

/// Health check endpoint path
pub const HEALTH_ENDPOINT: &str = "/health";

/// Metrics endpoint path
pub const METRICS_ENDPOINT: &str = "/metrics";

/// Admin endpoint path
pub const ADMIN_ENDPOINT: &str = "/admin";

/// WebSocket endpoint path
pub const WS_ENDPOINT: &str = "/ws";

/// Service discovery endpoint path
pub const DISCOVERY_ENDPOINT: &str = "/discovery";

/// Registration endpoint path
pub const REGISTRATION_ENDPOINT: &str = "/register";

// ============================================================================
// Helper Functions
// ============================================================================

/// Get port from environment variable or use default
///
/// **Deprecated**: Use `get_service_port()` for better discovery pattern
#[must_use]
#[deprecated(
    since = "3.0.0",
    note = "Use get_service_port(service_name) for infant primal pattern"
)]
pub fn get_port_from_env(env_var: &str, default: u16) -> u16 {
    std::env::var(env_var)
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or_else(|| {
            if default > 0 {
                tracing::warn!(
                    "Using fallback port {} - set {} environment variable",
                    default,
                    env_var
                );
            }
            default
        })
}

/// Construct HTTP URL from components
#[must_use]
pub fn http_url(host: &str, port: u16, path: &str) -> String {
    if path.is_empty() {
        format!("http://{host}:{port}")
    } else {
        format!("http://{host}:{port}{path}")
    }
}

#[cfg(test)]
mod tests {
    #[expect(deprecated)]
    use super::{
        ADMIN_ENDPOINT, DEFAULT_LOCALHOST, DISCOVERY_ENDPOINT, HEALTH_ENDPOINT,
        LOCALHOST_HTTP_TEMPLATE, LOCALHOST_IPV4, LOCALHOST_WS_TEMPLATE, METRICS_ENDPOINT,
        REGISTRATION_ENDPOINT, WS_ENDPOINT, get_bind_address, get_port_from_env, get_service_port,
        http_url,
    };

    #[test]
    fn test_addresses() {
        assert_eq!(get_bind_address(), "127.0.0.1");
        assert_eq!(DEFAULT_LOCALHOST, "localhost");
        assert_eq!(LOCALHOST_IPV4, "127.0.0.1");
    }

    #[test]
    fn test_ports_all_services() {
        assert_eq!(get_service_port("websocket"), 8080);
        assert_eq!(get_service_port("ws"), 8080);
        assert_eq!(get_service_port("http"), 8081);
        assert_eq!(get_service_port("admin"), 8082);
        assert_eq!(get_service_port("security"), 8083);
        assert_eq!(get_service_port("storage"), 8084);
        assert_eq!(get_service_port("ui"), 3000);
        assert_eq!(get_service_port("service_mesh"), 8085);
        assert_eq!(get_service_port("mesh"), 8085);
        assert_eq!(get_service_port("compute"), 8086);
        assert_eq!(get_service_port("metrics"), 9090);
        assert_eq!(get_service_port("discovery"), 8500);
    }

    #[test]
    fn test_unknown_service_port() {
        assert_eq!(get_service_port("unknown_service_xyz"), 0);
    }

    #[test]
    fn test_service_port_matches_legacy_defaults() {
        // Validates get_service_port returns same values as legacy constants
        assert_eq!(get_service_port("websocket"), 8080);
        assert_eq!(get_service_port("http"), 8081);
        assert_eq!(get_service_port("admin"), 8082);
        assert_eq!(get_service_port("metrics"), 9090);
        assert_eq!(get_service_port("discovery"), 8500);
    }

    #[test]
    fn test_default_api_bind_addr() {
        assert_eq!(super::default_api_bind_addr(), "0.0.0.0:8080");
        assert_eq!(super::DEFAULT_JSON_RPC_PORT, 8080);
        assert_eq!(super::DEFAULT_BIOMEOS_PORT, 5000);
        assert_eq!(super::DEFAULT_OLLAMA_PORT, 11434);
        assert_eq!(super::DEFAULT_CLI_MCP_PORT, 8444);
        assert_eq!(super::DEFAULT_AGENT_DEPLOY_BASE_PORT_MEDIUM, 8085);
        assert_eq!(super::DEFAULT_AGENT_DEPLOY_BASE_PORT_HIGH, 8090);
        assert_eq!(super::DEFAULT_METRICS_LISTEN_PORT, 9090);
        assert_eq!(super::DEFAULT_DISCOVERY_LISTEN_PORT, 8500);
    }

    #[test]
    fn test_endpoint_paths() {
        assert_eq!(HEALTH_ENDPOINT, "/health");
        assert_eq!(METRICS_ENDPOINT, "/metrics");
        assert_eq!(ADMIN_ENDPOINT, "/admin");
        assert_eq!(WS_ENDPOINT, "/ws");
        assert_eq!(DISCOVERY_ENDPOINT, "/discovery");
        assert_eq!(REGISTRATION_ENDPOINT, "/register");
    }

    #[test]
    fn test_url_templates() {
        assert_eq!(
            LOCALHOST_HTTP_TEMPLATE.replace("{}", "8080"),
            "http://localhost:8080"
        );
        assert_eq!(
            LOCALHOST_WS_TEMPLATE.replace("{}", "8080"),
            "ws://localhost:8080"
        );
    }

    #[test]
    fn test_http_url_helper() {
        assert_eq!(http_url("localhost", 8080, ""), "http://localhost:8080");
        assert_eq!(
            http_url("localhost", 8080, "/api"),
            "http://localhost:8080/api"
        );
        assert_eq!(
            http_url("10.0.0.1", 9090, "/health"),
            "http://10.0.0.1:9090/health"
        );
    }

    #[test]
    #[expect(
        deprecated,
        reason = "Tests deprecated path for backward compatibility"
    )]
    fn test_get_port_from_env() {
        assert_eq!(get_port_from_env("NONEXISTENT_PORT_XYZ", 1234), 1234);
    }

    #[test]
    fn test_biomeos_socket_constants() {
        assert_eq!(super::BIOMEOS_SOCKET_SUBDIR, "biomeos");
        assert_eq!(super::BIOMEOS_SOCKET_FALLBACK_DIR, "/tmp/biomeos");
    }

    #[test]
    fn test_get_socket_dir_fallback() {
        temp_env::with_var_unset("XDG_RUNTIME_DIR", || {
            let dir = super::get_socket_dir();
            assert_eq!(dir, std::path::PathBuf::from("/tmp/biomeos"));
        });
    }

    #[test]
    fn test_get_socket_path_fallback() {
        temp_env::with_vars_unset(["XDG_RUNTIME_DIR", "SQUIRREL_SOCKET"], || {
            let path = super::get_socket_path("squirrel");
            assert_eq!(path, std::path::PathBuf::from("/tmp/biomeos/squirrel.sock"));
        });
    }

    #[test]
    fn test_get_socket_path_env_override() {
        temp_env::with_var("TESTPRIMAL_SOCKET", Some("/custom/path.sock"), || {
            let path = super::get_socket_path("testprimal");
            assert_eq!(path, std::path::PathBuf::from("/custom/path.sock"));
        });
    }

    #[test]
    fn test_socket_env_var() {
        assert_eq!(super::socket_env_var("rhizocrypt"), "RHIZOCRYPT_SOCKET");
        assert_eq!(super::socket_env_var("sweetGrass"), "SWEETGRASS_SOCKET");
        assert_eq!(super::socket_env_var("squirrel"), "SQUIRREL_SOCKET");
    }

    #[test]
    fn test_address_env_var() {
        assert_eq!(super::address_env_var("rhizocrypt"), "RHIZOCRYPT_ADDRESS");
        assert_eq!(super::address_env_var("squirrel"), "SQUIRREL_ADDRESS");
    }

    #[test]
    fn test_squirrel_primal_port_env() {
        temp_env::with_var("SQUIRREL_PORT", Some("7777"), || {
            assert_eq!(super::squirrel_primal_port(), 7777);
        });
    }

    #[test]
    fn test_squirrel_primal_port_fallback() {
        temp_env::with_vars_unset(["SQUIRREL_PORT", "SQUIRREL_SERVER_PORT"], || {
            assert_eq!(super::squirrel_primal_port(), 9010);
        });
    }

    #[test]
    fn test_discover_peer_http_origin_from_env() {
        temp_env::with_var("PEER_EP", Some("http://registry.example:9"), || {
            let url = super::discover_peer_http_origin(
                "PEER_EP",
                "PEER_HOST",
                "PEER_PORT",
                "localhost",
                8080,
            );
            assert_eq!(url, "http://registry.example:9");
        });
    }

    #[test]
    fn test_discover_peer_http_origin_built() {
        temp_env::with_vars_unset(["PEER2_EP", "PEER2_HOST", "PEER2_PORT"], || {
            let url = super::discover_peer_http_origin(
                "PEER2_EP",
                "PEER2_HOST",
                "PEER2_PORT",
                "localhost",
                8444,
            );
            assert_eq!(url, "http://localhost:8444");
        });
    }

    #[test]
    fn test_resolve_capability_unix_socket_primary_env() {
        temp_env::with_var("AI_CAPABILITY_SOCKET", Some("/custom/ai.sock"), || {
            let p = super::resolve_capability_unix_socket("AI_CAPABILITY_SOCKET", "ai-provider");
            assert_eq!(p, std::path::PathBuf::from("/custom/ai.sock"));
        });
    }

    #[test]
    fn test_resolve_capability_unix_socket_squirrel_overrides_biomeos() {
        temp_env::with_vars(
            [
                ("AI_CAPABILITY_SOCKET", None::<&str>),
                ("SQUIRREL_SOCKET", Some("/run/squirrel.sock")),
                ("BIOMEOS_SOCKET_PATH", Some("/run/biomeos.sock")),
            ],
            || {
                let p =
                    super::resolve_capability_unix_socket("AI_CAPABILITY_SOCKET", "ai-provider");
                assert_eq!(p, std::path::PathBuf::from("/run/squirrel.sock"));
            },
        );
    }

    #[test]
    fn test_resolve_capability_unix_socket_biomeos_tier() {
        temp_env::with_vars(
            [
                ("AI_CAPABILITY_SOCKET", None::<&str>),
                ("SQUIRREL_SOCKET", None::<&str>),
                ("BIOMEOS_SOCKET_PATH", Some("/run/neural.sock")),
                ("PRIMAL_SOCKET", None::<&str>),
            ],
            || {
                let p =
                    super::resolve_capability_unix_socket("AI_CAPABILITY_SOCKET", "ai-provider");
                assert_eq!(p, std::path::PathBuf::from("/run/neural.sock"));
            },
        );
    }

    #[test]
    fn test_resolve_capability_unix_socket_xdg_fallback() {
        temp_env::with_vars(
            [
                ("AI_CAPABILITY_SOCKET", None::<&str>),
                ("SQUIRREL_SOCKET", None::<&str>),
                ("BIOMEOS_SOCKET_PATH", None::<&str>),
                ("PRIMAL_SOCKET", None::<&str>),
                ("FAMILY_ID", None::<&str>),
                ("XDG_RUNTIME_DIR", None::<&str>),
            ],
            || {
                let p =
                    super::resolve_capability_unix_socket("AI_CAPABILITY_SOCKET", "ai-provider");
                assert_eq!(p, std::path::PathBuf::from("/tmp/biomeos/ai-provider.sock"));
            },
        );
    }
}
