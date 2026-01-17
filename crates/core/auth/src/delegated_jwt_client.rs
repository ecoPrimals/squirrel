//! Delegated JWT Client - Capability-Based JWT Validation
//!
//! TRUE ecoBin Architecture via Capability Discovery:
//! - Squirrel discovers JWT validation capability (not hardcoded primal!)
//! - Currently: BearDog provides this capability (Security & Crypto Primal)
//! - Future: Any primal with JWT validation capability can provide it
//!
//! This eliminates ring v0.17 C dependency → 100% Pure Rust! 🦀

use crate::{AuthError, JwtClaims};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, error, info, warn};

/// JSON-RPC request for JWT validation
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: u64,
}

/// JSON-RPC response
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: u64,
}

/// JSON-RPC error
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

/// JWT validation parameters
#[derive(Debug, Serialize)]
struct JwtValidationParams {
    token: String,
    issuer: String,
    audience: String,
}

/// JWT validation result from BearDog
#[derive(Debug, Deserialize)]
struct JwtValidationResult {
    valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    claims: Option<JwtClaims>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

/// Delegated JWT Client - Communicates via Unix socket JSON-RPC with capability provider
///
/// # TRUE ecoBin Architecture via Capability Discovery
///
/// - Squirrel discovers "jwt.validate" capability at runtime
/// - Connects to whichever primal provides this capability
/// - Currently: BearDog (Security & Crypto Primal) provides it
/// - Future: Any primal with JWT capability can provide it
///
/// **Result**: Zero C dependencies in JWT path! Pure Rust! ✅
///
/// # Capability-Based Design
///
/// Instead of hardcoding "BearDog", we use capability discovery:
/// 1. Squirrel asks: "Who provides jwt.validate capability?"
/// 2. Registry responds: "/tmp/beardog-nat0.sock" (or any provider)
/// 3. Squirrel connects and validates JWT
///
/// # Example
///
/// ```rust,ignore
/// // Capability-based (good!)
/// let socket_path = discover_capability("jwt.validate").await?;
/// let client = DelegatedJwtClient::new(socket_path);
/// let claims = client.validate_token(token, "squirrel-mcp", "squirrel-mcp-api").await?;
/// ```
pub struct DelegatedJwtClient {
    socket_path: PathBuf,
    request_id: std::sync::atomic::AtomicU64,
    capability_provider: String, // Name of discovered provider (for logging)
}

impl DelegatedJwtClient {
    /// Create a new delegated JWT client
    ///
    /// # Arguments
    ///
    /// * `socket_path` - Path to capability provider's Unix socket
    ///
    /// # Capability Discovery
    ///
    /// The socket_path should come from capability discovery, not hardcoded!
    /// Example: `discover_capability("jwt.validate").await?`
    ///
    /// # Current Provider
    ///
    /// As of Jan 2026: BearDog provides jwt.validate capability
    pub fn new(socket_path: PathBuf) -> Self {
        info!("Initializing delegated JWT client (capability-based, TRUE ecoBin!)");
        debug!("JWT capability provider socket: {:?}", socket_path);

        Self {
            socket_path,
            request_id: std::sync::atomic::AtomicU64::new(1),
            capability_provider: "jwt-provider".to_string(), // Generic name
        }
    }

    /// Set the capability provider name (for logging/debugging)
    ///
    /// This is optional and used for better log messages.
    /// Example: "beardog-nat0" or "security-primal"
    pub fn with_provider_name(mut self, name: impl Into<String>) -> Self {
        self.capability_provider = name.into();
        self
    }

    /// Validate JWT token via capability provider
    ///
    /// # Arguments
    ///
    /// * `token` - JWT token to validate
    /// * `issuer` - Expected issuer (e.g., "squirrel-mcp")
    /// * `audience` - Expected audience (e.g., "squirrel-mcp-api")
    ///
    /// # Returns
    ///
    /// * `Ok(JwtClaims)` - Token is valid, returns parsed claims
    /// * `Err(AuthError)` - Token is invalid or capability provider unavailable
    ///
    /// # TRUE ecoBin via Capability Discovery
    ///
    /// This method delegates JWT validation to whichever primal provides
    /// the "jwt.validate" capability. Currently BearDog (Security & Crypto),
    /// but could be any primal with JWT capability in the future.
    ///
    /// The capability provider uses Pure Rust crypto (e.g., RustCrypto),
    /// eliminating ring C dependency from JWT validation!
    pub async fn validate_token(
        &self,
        token: &str,
        issuer: &str,
        audience: &str,
    ) -> Result<JwtClaims, AuthError> {
        debug!(
            "Delegating JWT validation to capability provider: {}",
            self.capability_provider
        );

        // Connect to capability provider's Unix socket
        let stream = UnixStream::connect(&self.socket_path).await.map_err(|e| {
            error!("Failed to connect to JWT capability provider socket: {}", e);
            AuthError::CapabilityProviderUnavailable(format!(
                "Could not connect to JWT provider at {:?}: {}",
                self.socket_path, e
            ))
        })?;

        debug!("Connected to JWT capability provider via Unix socket");

        // Split stream for reading and writing
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // Build JSON-RPC request for jwt.validate capability
        let request_id = self
            .request_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "jwt.validate".to_string(),
            params: serde_json::json!({
                "token": token,
                "issuer": issuer,
                "audience": audience,
            }),
            id: request_id,
        };

        // Serialize and send request
        let request_json = serde_json::to_string(&request)
            .map_err(|e| AuthError::internal_error(format!("JSON serialization error: {}", e)))?;

        debug!("Sending JWT validation request to capability provider");
        writer
            .write_all(request_json.as_bytes())
            .await
            .map_err(|e| {
                error!("Failed to write to capability provider socket: {}", e);
                AuthError::CapabilityProviderUnavailable(format!("Write error: {}", e))
            })?;

        writer.write_all(b"\n").await.map_err(|e| {
            error!(
                "Failed to write newline to capability provider socket: {}",
                e
            );
            AuthError::CapabilityProviderUnavailable(format!("Write error: {}", e))
        })?;

        writer.flush().await.map_err(|e| {
            error!("Failed to flush capability provider socket: {}", e);
            AuthError::CapabilityProviderUnavailable(format!("Flush error: {}", e))
        })?;

        // Read response
        let mut response_line = String::new();
        reader.read_line(&mut response_line).await.map_err(|e| {
            error!("Failed to read from capability provider socket: {}", e);
            AuthError::CapabilityProviderUnavailable(format!("Read error: {}", e))
        })?;

        debug!("Received JWT validation response from capability provider");

        // Parse JSON-RPC response
        let response: JsonRpcResponse = serde_json::from_str(&response_line).map_err(|e| {
            error!("Failed to parse capability provider response: {}", e);
            AuthError::internal_error(format!("Parse error: {}", e))
        })?;

        // Check for JSON-RPC error
        if let Some(error) = response.error {
            warn!(
                "Capability provider returned error: {} (code: {})",
                error.message, error.code
            );
            return Err(AuthError::CapabilityProviderError(error.message));
        }

        // Parse validation result
        let result: JwtValidationResult =
            serde_json::from_value(response.result.ok_or_else(|| {
                error!("Capability provider response missing result field");
                AuthError::InvalidResponse
            })?)
            .map_err(|e| {
                error!("Failed to parse JWT validation result: {}", e);
                AuthError::internal_error(format!("Result parse error: {}", e))
            })?;

        // Check validation status
        if !result.valid {
            let reason = result
                .reason
                .unwrap_or_else(|| "Unknown reason".to_string());
            warn!("JWT validation failed: {}", reason);
            return Err(AuthError::InvalidToken);
        }

        // Extract claims
        let claims = result.claims.ok_or_else(|| {
            error!("Capability provider returned valid=true but no claims");
            AuthError::InvalidResponse
        })?;

        info!(
            "JWT validation successful via {} (user: {})",
            self.capability_provider, claims.username
        );
        Ok(claims)
    }

    /// Request JWT signing secret from capability provider
    ///
    /// This is used if Squirrel needs to generate JWTs locally (e.g., for dev/testing).
    /// In production, the capability provider should generate JWTs directly.
    ///
    /// # Arguments
    ///
    /// * `purpose` - Purpose of the secret (e.g., "jwt-signing-key")
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` - JWT signing secret
    /// * `Err(AuthError)` - Capability provider unavailable or error
    ///
    /// # Capability Discovery
    ///
    /// Uses the "secret.get" capability from the same provider.
    /// Currently provided by BearDog (Security & Crypto Primal).
    pub async fn request_jwt_secret(&self, purpose: &str) -> Result<Vec<u8>, AuthError> {
        debug!(
            "Requesting JWT secret from capability provider: {}",
            purpose
        );

        // Connect to capability provider Unix socket
        let stream = UnixStream::connect(&self.socket_path).await.map_err(|e| {
            error!("Failed to connect to capability provider socket: {}", e);
            AuthError::CapabilityProviderUnavailable(format!("Connection error: {}", e))
        })?;

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // Build JSON-RPC request
        let request_id = self
            .request_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "secret.get".to_string(),
            params: serde_json::json!({
                "purpose": purpose,
                "requester": "squirrel-mcp",
            }),
            id: request_id,
        };

        // Send request
        let request_json = serde_json::to_string(&request)
            .map_err(|e| AuthError::internal_error(format!("JSON serialization error: {}", e)))?;

        writer
            .write_all(request_json.as_bytes())
            .await
            .map_err(|e| AuthError::CapabilityProviderUnavailable(format!("Write error: {}", e)))?;

        writer
            .write_all(b"\n")
            .await
            .map_err(|e| AuthError::CapabilityProviderUnavailable(format!("Write error: {}", e)))?;

        writer
            .flush()
            .await
            .map_err(|e| AuthError::CapabilityProviderUnavailable(format!("Flush error: {}", e)))?;

        // Read response
        let mut response_line = String::new();
        reader
            .read_line(&mut response_line)
            .await
            .map_err(|e| AuthError::CapabilityProviderUnavailable(format!("Read error: {}", e)))?;

        // Parse response
        let response: JsonRpcResponse = serde_json::from_str(&response_line)
            .map_err(|e| AuthError::internal_error(format!("Parse error: {}", e)))?;

        if let Some(error) = response.error {
            warn!(
                "Capability provider secret request failed: {}",
                error.message
            );
            return Err(AuthError::CapabilityProviderError(error.message));
        }

        let result = response.result.ok_or_else(|| AuthError::InvalidResponse)?;

        // Extract secret (base64-encoded)
        let secret_base64: String = serde_json::from_value(result["secret"].clone())
            .map_err(|e| AuthError::internal_error(format!("Secret parse error: {}", e)))?;

        let secret = base64::decode(&secret_base64)
            .map_err(|e| AuthError::internal_error(format!("Base64 decode error: {}", e)))?;

        info!(
            "Received JWT secret from capability provider ({} bytes)",
            secret.len()
        );
        Ok(secret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegated_client_creation() {
        let client = DelegatedJwtClient::new("/tmp/jwt-provider.sock".into());
        assert_eq!(client.socket_path, PathBuf::from("/tmp/jwt-provider.sock"));
    }

    #[test]
    fn test_capability_provider_name() {
        // ✅ Provider name should come from discovery
        let discovered_name = "security-provider-xyz"; // From capability discovery
        let client = DelegatedJwtClient::new("/tmp/jwt-provider.sock".into())
            .with_provider_name(discovered_name);
        assert_eq!(client.capability_provider, discovered_name);

        // ❌ Don't hardcode ecosystem-specific names like "beardog-nat0"
        // (nat0 is BirdSong P2P family tag - ecosystem knowledge!)
    }

    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "jwt.validate".to_string(),
            params: serde_json::json!({
                "token": "test-token",
                "issuer": "squirrel-mcp",
                "audience": "squirrel-mcp-api",
            }),
            id: 1,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("jwt.validate"));
        assert!(json.contains("test-token"));
    }

    #[test]
    fn test_jwt_validation_result_deserialization() {
        let json = r#"{
            "valid": true,
            "claims": {
                "sub": "user-123",
                "username": "alice",
                "roles": ["admin"],
                "session_id": "session-456",
                "iat": 1737158400,
                "exp": 1737244800,
                "nbf": 1737158400,
                "iss": "squirrel-mcp",
                "aud": "squirrel-mcp-api",
                "jti": "jwt-789"
            }
        }"#;

        let result: JwtValidationResult = serde_json::from_str(json).unwrap();
        assert!(result.valid);
        assert!(result.claims.is_some());

        let claims = result.claims.unwrap();
        assert_eq!(claims.username, "alice");
        assert_eq!(claims.roles, vec!["admin"]);
    }

    // Integration tests would require a running capability provider
    // These are marked as ignored by default
    #[tokio::test]
    #[ignore]
    async fn test_validate_token_integration() {
        // This test requires a capability provider to be running
        // In real usage, socket path and name come from capability discovery!

        // ✅ In production: From discovery
        // let capability = discover_capability("jwt.validate").await?;
        // let client = DelegatedJwtClient::new(capability.socket_path)
        //     .with_provider_name(&capability.provider_name);

        // For this test: Hardcoded for demonstration only
        // (In real code, ALWAYS use discovery!)
        let client = DelegatedJwtClient::new("/tmp/jwt-capability-provider.sock".into())
            .with_provider_name("test-provider"); // Would be from discovery

        // In real test, we'd get a valid token from the provider first
        let token = "valid-test-token";
        let result = client
            .validate_token(token, "squirrel-mcp", "squirrel-mcp-api")
            .await;

        // Would assert based on capability provider's response
        match result {
            Ok(claims) => {
                assert!(!claims.username.is_empty());
            }
            Err(AuthError::CapabilityProviderUnavailable(_)) => {
                // Expected if capability provider not running
                println!("JWT capability provider not available for integration test");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}
