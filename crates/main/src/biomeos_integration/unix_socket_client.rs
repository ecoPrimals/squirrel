//! Unix Socket Client for Inter-Primal Communication
//!
//! **TRUE PRIMAL Compliant**: Port-free, secure, capability-based
//!
//! This module provides Unix socket-based clients for communicating with other
//! primals (Songbird, BearDog, biomeOS) following TRUE PRIMAL architecture.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────┐  Unix Socket   ┌─────────────┐
//! │   Squirrel  │ ────JSON-RPC──> │  Songbird   │
//! │             │ <──────────────  │  (Mesh)     │
//! └─────────────┘                 └─────────────┘
//!       │
//!       │ Unix Socket
//!       │
//!       ▼
//! ┌─────────────┐
//! │   BearDog   │
//! │  (Security) │
//! └─────────────┘
//! ```
//!
//! ## vs HTTP Client (DEPRECATED)
//!
//! ```rust
//! // ❌ OLD: HTTP-based (ecosystem_client.rs)
//! let client = reqwest::Client::new();
//! let response = client.post("http://localhost:8080/register").send().await?;
//!
//! // ✅ NEW: Unix socket-based
//! let client = UnixSocketClient::connect_to_songbird().await?;
//! let response = client.register_service(registration).await?;
//! ```
//!
//! ## Environment Variables
//!
//! - `SONGBIRD_SOCKET` - Songbird Unix socket path (default: auto-discover)
//! - `BEARDOG_SOCKET` - BearDog Unix socket path (default: auto-discover)  
//! - `BIOMEOS_SOCKET` - biomeOS Unix socket path (default: auto-discover)
//!
//! ## Discovery Priority
//!
//! 1. Explicit env var (`SONGBIRD_SOCKET`)
//! 2. XDG runtime dir (`/run/user/<uid>/songbird-<family>.sock`)
//! 3. biomeOS discovery service
//! 4. Error (no hardcoded fallback!)

use crate::error::PrimalError;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, info, warn};

/// Unix socket client for inter-primal communication
pub struct UnixSocketClient {
    /// Socket path
    socket_path: String,

    /// Connection stream (optional, connected on demand)
    stream: Option<UnixStream>,
}

impl UnixSocketClient {
    /// Create a new client for a specific socket path
    ///
    /// **Note**: This does NOT connect immediately. Use `connect()` or `send_request()`.
    pub fn new(socket_path: String) -> Self {
        Self {
            socket_path,
            stream: None,
        }
    }

    /// Connect by capability (TRUE PRIMAL - no hardcoded primal names!)
    ///
    /// ## Examples
    ///
    /// ```rust,ignore
    /// // ✅ Request by capability, not by primal name
    /// let orchestration = UnixSocketClient::connect_by_capability("orchestration").await?;
    /// let security = UnixSocketClient::connect_by_capability("security").await?;
    /// let storage = UnixSocketClient::connect_by_capability("storage").await?;
    ///
    /// // ❌ DON'T hardcode primal names
    /// // let songbird = connect_to_songbird().await?; // WRONG!
    /// ```
    ///
    /// ## Discovery Order
    ///
    /// 1. `{CAPABILITY}_SOCKET` env var (e.g., `ORCHESTRATION_SOCKET`)
    /// 2. Socket registry file (`/run/user/<uid>/socket-registry.json`)
    /// 3. Convention-based XDG path (legacy fallback)
    /// 4. Error (no hardcoded fallback!)
    pub async fn connect_by_capability(capability: &str) -> Result<Self, PrimalError> {
        let socket_path = Self::discover_socket_by_capability(capability)?;

        info!(
            "🔌 Connecting to '{}' capability via Unix socket: {}",
            capability, socket_path
        );

        let mut client = Self::new(socket_path);
        client.connect().await?;

        Ok(client)
    }

    /// Convenience: Connect to orchestration capability
    ///
    /// **Note**: This uses capability discovery, not hardcoded "Songbird"
    pub async fn connect_to_orchestration() -> Result<Self, PrimalError> {
        Self::connect_by_capability("orchestration").await
    }

    /// Convenience: Connect to security capability
    ///
    /// **Note**: This uses capability discovery, not hardcoded "BearDog"
    pub async fn connect_to_security() -> Result<Self, PrimalError> {
        Self::connect_by_capability("security").await
    }

    /// Convenience: Connect to core capability (biomeOS)
    ///
    /// **Note**: This uses capability discovery
    pub async fn connect_to_core() -> Result<Self, PrimalError> {
        Self::connect_by_capability("core").await
    }

    /// Establish the Unix socket connection
    async fn connect(&mut self) -> Result<(), PrimalError> {
        if !Path::new(&self.socket_path).exists() {
            return Err(PrimalError::NetworkError(format!(
                "Unix socket does not exist: {}. Is the service running?",
                self.socket_path
            )));
        }

        let stream = UnixStream::connect(&self.socket_path).await.map_err(|e| {
            PrimalError::NetworkError(format!(
                "Failed to connect to Unix socket {}: {}",
                self.socket_path, e
            ))
        })?;

        debug!("✅ Connected to Unix socket: {}", self.socket_path);
        self.stream = Some(stream);

        Ok(())
    }

    /// Send a JSON-RPC request and receive response
    ///
    /// ## JSON-RPC 2.0 Format
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "register_service",
    ///   "params": { ... },
    ///   "id": 1
    /// }
    /// ```
    pub async fn send_request(
        &mut self,
        method: &str,
        params: JsonValue,
    ) -> Result<JsonValue, PrimalError> {
        // Ensure we're connected
        if self.stream.is_none() {
            self.connect().await?;
        }

        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| PrimalError::Internal("Stream not connected".to_string()))?;

        // Build JSON-RPC 2.0 request
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let request_str = serde_json::to_string(&request).map_err(|e| {
            PrimalError::SerializationError(format!("Failed to serialize request: {}", e))
        })?;

        debug!("📤 Sending JSON-RPC request: {}", method);

        // Send request (newline-delimited)
        stream
            .write_all(format!("{}\n", request_str).as_bytes())
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Failed to write to socket: {}", e)))?;

        stream
            .flush()
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Failed to flush socket: {}", e)))?;

        // Read response
        let mut reader = BufReader::new(stream);
        let mut response_str = String::new();

        reader
            .read_line(&mut response_str)
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Failed to read from socket: {}", e)))?;

        debug!(
            "📥 Received JSON-RPC response ({} bytes)",
            response_str.len()
        );

        // Parse JSON-RPC response
        let response: JsonRpcResponse = serde_json::from_str(&response_str).map_err(|e| {
            PrimalError::SerializationError(format!("Failed to parse response: {}", e))
        })?;

        if let Some(error) = response.error {
            return Err(PrimalError::Internal(format!(
                "JSON-RPC error {}: {}",
                error.code, error.message
            )));
        }

        response
            .result
            .ok_or_else(|| PrimalError::Internal("JSON-RPC response missing result".to_string()))
    }

    /// Register service with Songbird (via Unix socket)
    ///
    /// Replaces the HTTP-based `EcosystemClient::register_service_with_songbird()`
    pub async fn register_with_songbird(
        &mut self,
        registration: ServiceRegistration,
    ) -> Result<RegistrationResponse, PrimalError> {
        let params = serde_json::to_value(registration)
            .map_err(|e| PrimalError::SerializationError(e.to_string()))?;

        let result = self.send_request("register_service", params).await?;

        serde_json::from_value(result).map_err(|e| PrimalError::SerializationError(e.to_string()))
    }

    /// Send heartbeat to Songbird
    pub async fn send_heartbeat(&mut self, health_status: HealthStatus) -> Result<(), PrimalError> {
        let params = serde_json::to_value(health_status)
            .map_err(|e| PrimalError::SerializationError(e.to_string()))?;

        self.send_request("heartbeat", params).await?;
        Ok(())
    }

    /// Deregister from Songbird
    pub async fn deregister(&mut self, service_id: &str) -> Result<(), PrimalError> {
        let params = serde_json::json!({ "service_id": service_id });
        self.send_request("deregister_service", params).await?;
        Ok(())
    }

    /// Discover socket by capability (TRUE PRIMAL compliant!)
    ///
    /// ## Discovery Order
    ///
    /// 1. **Capability-based env var**: `{CAPABILITY}_SOCKET`
    ///    - `ORCHESTRATION_SOCKET=/run/user/1000/songbird-nat0.sock`
    ///    - `SECURITY_SOCKET=/run/user/1000/beardog-nat0.sock`
    ///    
    /// 2. **Socket registry file**: `/run/user/<uid>/socket-registry.json`
    ///    ```json
    ///    {
    ///      "orchestration": "/run/user/1000/songbird-nat0.sock",
    ///      "security": "/run/user/1000/beardog-nat0.sock"
    ///    }
    ///    ```
    ///
    /// 3. **Legacy compatibility**: Convention-based XDG paths
    ///    - Only if registry file doesn't exist
    ///    - Maps common capabilities to legacy names
    ///
    /// 4. **Error**: No hardcoded fallback!
    pub(crate) fn discover_socket_by_capability(capability: &str) -> Result<String, PrimalError> {
        // 1. Capability-based environment variable (highest priority)
        let env_var = format!("{}_SOCKET", capability.to_uppercase());
        if let Ok(socket_path) = std::env::var(&env_var) {
            debug!("Using {} for '{}' capability", env_var, capability);
            return Ok(socket_path);
        }

        // 2. Socket registry file (maintained by orchestration layer)
        if let Some(registry_path) = Self::get_socket_registry(capability)? {
            debug!(
                "Found '{}' in socket registry: {}",
                capability, registry_path
            );
            return Ok(registry_path);
        }

        // 3. Legacy compatibility - convert capability to conventional service name
        // DEPRECATED: This should be replaced by registry in production
        warn!(
            "Using legacy socket discovery for '{}'. \
             Consider setting {} or creating a socket registry.",
            capability, env_var
        );

        if let Some(legacy_socket) = Self::legacy_capability_to_socket(capability) {
            return Ok(legacy_socket);
        }

        // 4. Error - no hardcoded fallback!
        Err(PrimalError::Configuration(format!(
            "Socket for capability '{}' not found. \
             Options: \n\
             1. Set {}_SOCKET environment variable\n\
             2. Create socket registry at /run/user/<uid>/socket-registry.json\n\
             3. Ensure the primal providing '{}' capability is running",
            capability,
            capability.to_uppercase(),
            capability
        )))
    }

    /// Read socket registry file (maintained by orchestration layer)
    ///
    /// Format: `/run/user/<uid>/socket-registry.json`
    /// ```json
    /// {
    ///   "orchestration": "/run/user/1000/songbird-nat0.sock",
    ///   "security": "/run/user/1000/beardog-nat0.sock",
    ///   "storage": "/run/user/1000/nestgate-nat0.sock",
    ///   "compute": "/run/user/1000/toadstool-nat0.sock"
    /// }
    /// ```
    pub(crate) fn get_socket_registry(capability: &str) -> Result<Option<String>, PrimalError> {
        let uid = nix::unistd::getuid();
        let registry_path = format!("/run/user/{}/socket-registry.json", uid);

        if !Path::new(&registry_path).exists() {
            debug!("Socket registry not found: {}", registry_path);
            return Ok(None);
        }

        let content = std::fs::read_to_string(&registry_path)
            .map_err(|e| PrimalError::Internal(format!("Failed to read socket registry: {}", e)))?;

        let registry: std::collections::HashMap<String, String> = serde_json::from_str(&content)
            .map_err(|e| {
                PrimalError::Internal(format!("Failed to parse socket registry: {}", e))
            })?;

        Ok(registry.get(capability).cloned())
    }

    /// Legacy capability-to-socket mapping (DEPRECATED)
    ///
    /// This provides backward compatibility for conventional socket names.
    /// In production, use socket registry or explicit env vars.
    pub(crate) fn legacy_capability_to_socket(capability: &str) -> Option<String> {
        let family_id = std::env::var("FAMILY_ID")
            .or_else(|_| std::env::var("SQUIRREL_FAMILY_ID"))
            .unwrap_or_else(|_| "default".to_string());

        let uid = nix::unistd::getuid();
        let xdg_runtime = format!("/run/user/{}", uid);

        if !Path::new(&xdg_runtime).exists() {
            return None;
        }

        // Map capability to conventional service name (for legacy support only)
        let service_name = match capability {
            "orchestration" | "coordination" | "mesh" => "songbird",
            "security" | "authentication" | "authorization" => "beardog",
            "storage" | "data" | "persistence" => "nestgate",
            "compute" | "execution" | "processing" => "toadstool",
            "core" | "nucleus" | "biome" => "biomeos",
            "ai" | "intelligence" | "inference" => "squirrel",
            _ => {
                warn!("Unknown capability '{}' - no legacy mapping", capability);
                return None;
            }
        };

        Some(format!(
            "{}/{}-{}.sock",
            xdg_runtime, service_name, family_id
        ))
    }
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<JsonValue>,
    error: Option<JsonRpcError>,
    id: serde_json::Value,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<JsonValue>,
}

/// Service registration (simplified from ecosystem_client.rs)
#[derive(Debug, Serialize)]
pub struct ServiceRegistration {
    pub service_id: String,
    pub primal_type: String,
    pub capabilities: Vec<String>,
    pub socket_path: String,
    pub metadata: serde_json::Value,
}

/// Registration response
#[derive(Debug, Deserialize)]
pub struct RegistrationResponse {
    pub success: bool,
    pub service_id: String,
    pub message: String,
}

/// Health status
#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub service_id: String,
    pub status: String,
    pub timestamp: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_capability_discovery() {
        std::env::set_var("FAMILY_ID", "nat0");

        let socket = UnixSocketClient::legacy_capability_to_socket("orchestration");

        if let Some(path) = socket {
            assert!(path.contains("songbird-nat0.sock"));
        }

        std::env::remove_var("FAMILY_ID");
    }

    #[test]
    fn test_explicit_override() {
        std::env::set_var("ORCHESTRATION_SOCKET", "/custom/path.sock");

        let socket = UnixSocketClient::discover_socket_by_capability("orchestration").unwrap();
        assert_eq!(socket, "/custom/path.sock");

        std::env::remove_var("ORCHESTRATION_SOCKET");
    }

    // ============================================================================
    // COMPREHENSIVE TEST SUITE
    // ============================================================================

    #[test]
    fn test_multiple_capability_types() {
        std::env::set_var("FAMILY_ID", "test");

        let capabilities = vec![
            ("orchestration", "songbird"),
            ("security", "beardog"),
            ("storage", "nestgate"),
            ("compute", "toadstool"),
        ];

        for (capability, expected_service) in capabilities {
            let result = UnixSocketClient::legacy_capability_to_socket(capability);
            assert!(result.is_some());
            assert!(result.unwrap().contains(expected_service));
        }

        std::env::remove_var("FAMILY_ID");
    }

    #[tokio::test]
    async fn test_missing_socket() {
        std::env::set_var("ORCHESTRATION_SOCKET", "/nonexistent.sock");

        let result = UnixSocketClient::connect_by_capability("orchestration").await;
        assert!(result.is_err());

        std::env::remove_var("ORCHESTRATION_SOCKET");
    }

    #[tokio::test]
    async fn test_empty_capability_name() {
        let result = UnixSocketClient::discover_socket_by_capability("");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unknown_capability() {
        std::env::remove_var("UNKNOWN_SOCKET");
        std::env::remove_var("FAMILY_ID");

        let result = UnixSocketClient::discover_socket_by_capability("unknown");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_discovery_performance() {
        std::env::set_var("ORCHESTRATION_SOCKET", "/test/perf.sock");

        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = UnixSocketClient::discover_socket_by_capability("orchestration");
        }
        let duration = start.elapsed();

        // Should be fast (< 100ms for 1000 iterations)
        assert!(duration.as_millis() < 100);

        std::env::remove_var("ORCHESTRATION_SOCKET");
    }
}
