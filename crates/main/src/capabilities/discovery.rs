// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Capability Discovery - TRUE PRIMAL Infant Pattern
//!
//! Discovers capabilities at runtime with ZERO hardcoded primal names.
//! Deploy like an infant - knows nothing, discovers everything.
//!
//! ## Discovery Protocol
//!
//! This module sends `{"method":"capability.discover"}` JSON-RPC probes
//! to sockets during scanning. Any primal that responds with its capabilities
//! list can be discovered. Squirrel's own JSON-RPC server also handles
//! this method (see `jsonrpc_server.rs` - `handle_discover_capabilities` for `capability.discover`),
//! making Squirrel discoverable by other primals.
//!
//! ## Songbird Alignment (Feb 9, 2026)
//!
//! Once Songbird implements its own `discover_capabilities` handler,
//! Squirrel will auto-discover Songbird's `http.request` capability
//! without needing the `HTTP_REQUEST_PROVIDER_SOCKET` env var bypass.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Discovered capability provider
///
/// Represents a service discovered at runtime that provides capabilities.
/// NO knowledge of what primal this is - only what it can do!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityProvider {
    /// Runtime-assigned ID (NOT a primal name!)
    pub id: String,

    /// Capabilities this provider offers
    pub capabilities: Vec<String>,

    /// How to reach it (Unix socket path)
    pub socket: PathBuf,

    /// Optional metadata from discovery
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// Discovery method used
    #[serde(default)]
    pub discovered_via: String,
}

/// Discovery error types
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    /// The requested capability was not found.
    #[error("Capability not found: {0}")]
    CapabilityNotFound(String),

    /// Socket probe operation failed.
    #[error("Socket probe failed: {0}")]
    ProbeFailed(String),

    /// I/O error during discovery.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing error when reading capability metadata.
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    /// No socket directory was accessible for discovery.
    #[error("No socket directory accessible")]
    NoSocketDirectory,
}

/// Discover provider for a specific capability
///
/// TRUE PRIMAL: Discovers at runtime, NO hardcoded primal names!
///
/// # Example
/// ```no_run
/// # use squirrel::capabilities::discovery::discover_capability;
/// # async fn example() -> anyhow::Result<()> {
/// // Discover who provides crypto signing (could be anyone!)
/// let crypto_provider = discover_capability("crypto.signing").await?;
///
/// // We have NO IDEA what primal this is - we only know it can sign
/// println!("Found crypto provider at: {:?}", crypto_provider.socket);
/// # Ok(())
/// # }
/// ```
pub async fn discover_capability(capability: &str) -> Result<CapabilityProvider, DiscoveryError> {
    info!("🔍 Discovering capability: {}", capability);

    // Method 1: Explicit environment variable (instant)
    if let Some(provider) = try_explicit_env(capability).await? {
        info!("✅ Found {} via environment variable", capability);
        return Ok(provider);
    }

    // Method 2: Query capability registry (instant, event-driven!)
    // Registry query first: <1ms vs socket scan 2s+ timeout
    if let Some(provider) = try_registry_query(capability).await? {
        info!("✅ Found {} via capability registry", capability);
        return Ok(provider);
    }

    // Method 3: Scan socket directory (slow fallback)
    // Only used if registry unavailable (dev/testing)
    if let Some(provider) = try_socket_scan(capability).await? {
        info!("✅ Found {} via socket scan", capability);
        return Ok(provider);
    }

    warn!("❌ Capability not found: {}", capability);
    Err(DiscoveryError::CapabilityNotFound(capability.to_string()))
}

/// Try to discover via explicit environment variable
///
/// Format: CAPABILITY_NAME_PROVIDER_SOCKET=/path/to/socket
/// Example: CRYPTO_SIGNING_PROVIDER_SOCKET=/tmp/provider.sock
///
/// Trust explicit env vars without probing.
/// Not all primals implement discover_capabilities, and operators know
/// what they're configuring. Skip the probe and trust the env var.
async fn try_explicit_env(capability: &str) -> Result<Option<CapabilityProvider>, DiscoveryError> {
    let env_var = format!(
        "{}_PROVIDER_SOCKET",
        capability.to_uppercase().replace('.', "_")
    );

    if let Ok(socket_path) = std::env::var(&env_var) {
        let path = PathBuf::from(&socket_path);

        // Verify socket exists
        if path.exists() {
            info!(
                "✅ Found {} via env var {} = {}",
                capability, env_var, socket_path
            );

            // Trust the env var - operator knows what they're doing
            // Skip probe since not all primals support discover_capabilities
            return Ok(Some(CapabilityProvider {
                id: format!("{capability}-provider"),
                capabilities: vec![capability.to_string()],
                socket: path,
                metadata: std::collections::HashMap::new(),
                discovered_via: format!("env:{env_var}"),
            }));
        }
    }

    Ok(None)
}

/// Scan socket directory for capability providers
///
/// TRUE PRIMAL: Scans all sockets, probes each to ask what it provides.
/// Uses overall timeout to prevent infinite hangs during socket scanning.
async fn try_socket_scan(capability: &str) -> Result<Option<CapabilityProvider>, DiscoveryError> {
    // Get socket directory from environment or use default
    let socket_dirs = get_socket_directories();

    // Total scan timeout of 5 seconds
    let scan_result = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        for socket_dir in socket_dirs {
            debug!("Scanning socket directory: {:?}", socket_dir);

            if let Ok(mut entries) = fs::read_dir(&socket_dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();

                    // Only check Unix sockets
                    if is_unix_socket(&path).await {
                        debug!("Probing socket: {:?}", path);

                        // Probe each socket to see what it provides
                        // Errors are logged but don't stop the scan
                        if let Ok(provider) = probe_socket(&path).await
                            && provider.capabilities.contains(&capability.to_string())
                        {
                            return Ok::<Option<CapabilityProvider>, DiscoveryError>(Some(
                                CapabilityProvider {
                                    discovered_via: format!("scan:{}", socket_dir.display()),
                                    ..provider
                                },
                            ));
                        }
                    }
                }
            }
        }
        Ok(None)
    })
    .await;

    if let Ok(result) = scan_result {
        result
    } else {
        warn!("Socket scan timed out after 5s");
        Ok(None)
    }
}

/// Query capability registry if available
///
/// TRUE PRIMAL: Even the registry is discovered, not hardcoded!
/// Now uses Neural API for semantic capability routing.
async fn try_registry_query(
    capability: &str,
) -> Result<Option<CapabilityProvider>, DiscoveryError> {
    // Try Neural API first (NUCLEUS-compliant semantic routing)
    // Neural API provides capability.discover for finding providers
    info!("🧠 Checking Neural API for capability: {}", capability);

    let neural_api_socket = std::env::var("NEURAL_API_SOCKET").ok().or_else(|| {
        let uid = nix::unistd::getuid();
        let dir = crate::primal_names::BIOMEOS_SOCKET_DIR;
        let sock = crate::primal_names::NEURAL_API_SOCKET_NAME;
        let paths = [
            format!("/tmp/{sock}"),
            format!("/run/user/{uid}/{dir}/{sock}"),
        ];
        paths.into_iter().find(|p| Path::new(p).exists())
    });

    if let Some(socket_path) = neural_api_socket {
        info!("🧠 Found Neural API socket: {}", socket_path);
        let registry_path = PathBuf::from(&socket_path);

        if registry_path.exists() {
            info!("🧠 Querying Neural API at: {:?}", registry_path);

            // Connect to Neural API and query capability
            match query_registry(&registry_path, capability).await {
                Ok(provider) => {
                    info!(
                        "✅ Neural API found provider for {}: {:?}",
                        capability, provider.socket
                    );
                    return Ok(Some(CapabilityProvider {
                        discovered_via: "neural_api".to_string(),
                        ..provider
                    }));
                }
                Err(e) => {
                    warn!("⚠️  Neural API query failed for {}: {}", capability, e);
                }
            }
        }
    } else {
        debug!("Neural API socket not found");
    }

    // Fallback: Legacy registry (for backward compatibility)
    if let Ok(registry_socket) = std::env::var("CAPABILITY_REGISTRY_SOCKET") {
        let registry_path = PathBuf::from(registry_socket);

        if registry_path.exists() {
            debug!(
                "Querying legacy capability registry at: {:?}",
                registry_path
            );

            if let Ok(provider) = query_registry(&registry_path, capability).await {
                return Ok(Some(CapabilityProvider {
                    discovered_via: "registry".to_string(),
                    ..provider
                }));
            }
        }
    }

    Ok(None)
}

/// Probe a socket to discover what capabilities it provides
///
/// Sends a JSON-RPC discovery request and parses the response
pub async fn probe_socket(socket_path: &Path) -> Result<CapabilityProvider, DiscoveryError> {
    // Connect to socket
    let stream = UnixStream::connect(socket_path)
        .await
        .map_err(|e| DiscoveryError::ProbeFailed(e.to_string()))?;

    // Build discovery request (JSON-RPC 2.0)
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "capability.discover",
        "params": {},
        "id": Uuid::new_v4().to_string(),
    });

    // Send request
    let mut request_str = serde_json::to_string(&request)?;
    request_str.push('\n');

    let (read_half, mut write_half) = stream.into_split();
    write_half
        .write_all(request_str.as_bytes())
        .await
        .map_err(|e| DiscoveryError::ProbeFailed(e.to_string()))?;

    // Read response (with timeout)
    let mut reader = BufReader::new(read_half);
    let mut response_line = String::new();

    // 2s timeout per socket probe
    match tokio::time::timeout(
        std::time::Duration::from_secs(2),
        reader.read_line(&mut response_line),
    )
    .await
    {
        Ok(Ok(_)) => {
            // Parse JSON-RPC response
            let response: serde_json::Value = serde_json::from_str(&response_line)?;

            // Handle JSON-RPC error responses gracefully
            if let Some(error) = response.get("error") {
                debug!(
                    "Socket {:?} returned JSON-RPC error: {} (code: {})",
                    socket_path,
                    error
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("unknown"),
                    error
                        .get("code")
                        .and_then(serde_json::Value::as_i64)
                        .unwrap_or(-1)
                );
                // Socket doesn't support discover_capabilities method - return error
                return Err(DiscoveryError::ProbeFailed(
                    "Method not supported".to_string(),
                ));
            }

            if let Some(result) = response.get("result") {
                let capabilities: Vec<String> = serde_json::from_value(
                    result.get("capabilities").cloned().unwrap_or_default(),
                )?;

                let metadata: HashMap<String, String> =
                    serde_json::from_value(result.get("metadata").cloned().unwrap_or_default())?;

                // Generate ID from socket name (no primal name knowledge!)
                let id = socket_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                Ok(CapabilityProvider {
                    id,
                    capabilities,
                    socket: socket_path.to_path_buf(),
                    metadata,
                    discovered_via: "probe".to_string(),
                })
            } else {
                Err(DiscoveryError::ProbeFailed(
                    "No result in response".to_string(),
                ))
            }
        }
        Ok(Err(e)) => Err(DiscoveryError::ProbeFailed(format!("Read error: {e}"))),
        Err(_) => Err(DiscoveryError::ProbeFailed("Timeout (>2s)".to_string())),
    }
}

/// Query capability registry for a specific capability
///
/// Uses Neural API's `capability.discover` method for semantic routing.
/// This is the TRUE PRIMAL way - primals don't know about each other,
/// they just discover capabilities via Neural API.
async fn query_registry(
    registry_path: &Path,
    capability: &str,
) -> Result<CapabilityProvider, DiscoveryError> {
    let stream = UnixStream::connect(registry_path)
        .await
        .map_err(|e| DiscoveryError::ProbeFailed(e.to_string()))?;

    // Build capability.discover query (Neural API semantic routing)
    // NUCLEUS FIX (Feb 3, 2026): Use correct method name for Neural API
    // NOTE: Neural API requires integer IDs, not string UUIDs
    use std::sync::atomic::{AtomicU64, Ordering};
    static REQUEST_ID: AtomicU64 = AtomicU64::new(1);
    let id = REQUEST_ID.fetch_add(1, Ordering::SeqCst);

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "capability.discover",
        "params": {
            "capability": capability,
        },
        "id": id,
    });

    let mut request_str = serde_json::to_string(&request)?;
    request_str.push('\n');

    let (read_half, mut write_half) = stream.into_split();
    write_half.write_all(request_str.as_bytes()).await?;

    let mut reader = BufReader::new(read_half);
    let mut response_line = String::new();

    // Timeout to prevent hangs on unresponsive sockets
    match tokio::time::timeout(
        std::time::Duration::from_secs(2),
        reader.read_line(&mut response_line),
    )
    .await
    {
        Ok(Ok(_)) => { /* Continue with response parsing */ }
        Ok(Err(e)) => {
            return Err(DiscoveryError::ProbeFailed(format!(
                "Registry read error: {e}"
            )));
        }
        Err(_) => {
            return Err(DiscoveryError::ProbeFailed(
                "Registry query timeout (>2s)".to_string(),
            ));
        }
    }

    debug!("Neural API raw response: {}", response_line.trim());

    let response: serde_json::Value = serde_json::from_str(&response_line).map_err(|e| {
        warn!("Failed to parse Neural API response: {}", e);
        DiscoveryError::Json(e)
    })?;

    // Check for errors
    if let Some(error) = response.get("error") {
        warn!("Neural API returned error for {}: {:?}", capability, error);
        return Err(DiscoveryError::CapabilityNotFound(capability.to_string()));
    }

    if let Some(result) = response.get("result") {
        // Neural API returns: {"capability": "...", "primary_socket": "...", "primals": [...]}
        // Extract primary_socket and build CapabilityProvider
        if let Some(socket_path) = result.get("primary_socket").and_then(|s| s.as_str()) {
            info!("✅ Neural API discovered {} at {}", capability, socket_path);
            Ok(CapabilityProvider {
                id: result
                    .get("capability")
                    .and_then(|c| c.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                capabilities: vec![capability.to_string()],
                socket: PathBuf::from(socket_path),
                metadata: std::collections::HashMap::new(),
                discovered_via: "neural_api".to_string(),
            })
        } else {
            warn!("Neural API result has no primary_socket: {:?}", result);
            Err(DiscoveryError::CapabilityNotFound(capability.to_string()))
        }
    } else {
        warn!("Neural API response has no result field: {:?}", response);
        Err(DiscoveryError::CapabilityNotFound(capability.to_string()))
    }
}

/// Get socket directories to scan
/// Get socket directories to scan, prioritizing biomeOS standard locations
///
/// ## Priority Order (for NUCLEUS-compliant discovery)
///
/// 1. `SOCKET_SCAN_DIR` env var (explicit override)
/// 2. `/run/user/<uid>/biomeos/` (STANDARD biomeOS path - highest priority!)
/// 3. `$XDG_RUNTIME_DIR/biomeos/` (XDG-compliant standard path)
/// 4. `/run/user/<uid>/` (fallback for old socket locations)
/// 5. `/tmp/` and `/var/run/` (dev/testing fallback)
///
/// This order enables:
/// - Tower Atomic discovery (BearDog + Songbird)
/// - Node Atomic discovery (Tower + Toadstool)
/// - Nest Atomic discovery (Tower + NestGate)
/// - Full NUCLEUS discovery (all primals)
fn get_socket_directories() -> Vec<PathBuf> {
    let mut dirs = vec![];

    // Priority 1: Check explicit environment variable override
    if let Ok(dir) = std::env::var("SOCKET_SCAN_DIR") {
        dirs.push(PathBuf::from(dir));
    }

    // Priority 2: Standard biomeOS socket directory (NUCLEUS-compliant!)
    // This is where BearDog, Songbird, NestGate, Toadstool sockets live
    let uid = nix::unistd::getuid();
    let biomeos_dir = PathBuf::from(format!("/run/user/{uid}/biomeos"));
    if biomeos_dir.exists() {
        dirs.push(biomeos_dir);
    }

    // Priority 3: XDG Runtime Directory with biomeos subdirectory
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let xdg_biomeos = PathBuf::from(format!("{runtime_dir}/biomeos"));
        if xdg_biomeos.exists() {
            dirs.push(xdg_biomeos);
        }
        // Also check XDG root (for backward compatibility)
        dirs.push(PathBuf::from(runtime_dir));
    }

    // Priority 4: Fallback to standard temp/run directories (dev/testing)
    dirs.push(PathBuf::from("/tmp"));
    dirs.push(PathBuf::from("/var/run"));

    debug!("Socket scan directories (in order): {:?}", dirs);
    dirs
}

/// Check if path is a Unix socket
async fn is_unix_socket(path: &Path) -> bool {
    if let Ok(metadata) = fs::metadata(path).await {
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            return metadata.file_type().is_socket();
        }

        #[cfg(not(unix))]
        {
            // On non-Unix, check file extension as fallback
            return path.extension().and_then(|s| s.to_str()) == Some("sock");
        }
    }

    false
}

/// Discover all available capabilities in the environment
///
/// Returns a map of capability name → providers (Arc for zero-copy sharing)
pub async fn discover_all_capabilities() -> Result<HashMap<String, Vec<Arc<CapabilityProvider>>>> {
    info!("🔍 Discovering all available capabilities...");

    let mut all_capabilities: HashMap<String, Vec<Arc<CapabilityProvider>>> = HashMap::new();

    // Scan all socket directories
    for socket_dir in get_socket_directories() {
        if let Ok(mut entries) = fs::read_dir(&socket_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();

                if is_unix_socket(&path).await
                    && let Ok(provider) = probe_socket(&path).await
                {
                    let provider = Arc::new(provider);
                    for capability in &provider.capabilities {
                        all_capabilities
                            .entry(capability.clone())
                            .or_default()
                            .push(Arc::clone(&provider));
                    }
                }
            }
        }
    }

    info!(
        "✅ Discovery complete: {} capabilities found",
        all_capabilities.len()
    );

    Ok(all_capabilities)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_provider_serialization() {
        let provider = CapabilityProvider {
            id: "test-provider".to_string(),
            capabilities: vec!["crypto.signing".to_string()],
            socket: PathBuf::from("/tmp/test.sock"),
            metadata: HashMap::new(),
            discovered_via: "test".to_string(),
        };

        let json = serde_json::to_string(&provider).unwrap();
        let deserialized: CapabilityProvider = serde_json::from_str(&json).unwrap();

        assert_eq!(provider.id, deserialized.id);
        assert_eq!(provider.capabilities, deserialized.capabilities);
    }

    #[test]
    fn test_env_var_formatting() {
        let capability = "crypto.signing";
        let env_var = format!(
            "{}_PROVIDER_SOCKET",
            capability.to_uppercase().replace('.', "_")
        );
        assert_eq!(env_var, "CRYPTO_SIGNING_PROVIDER_SOCKET");
    }

    #[tokio::test]
    async fn test_socket_directories() {
        let dirs = get_socket_directories();
        assert!(!dirs.is_empty());
        assert!(dirs.contains(&PathBuf::from("/tmp")));
    }

    #[test]
    fn discovery_error_display() {
        let e = DiscoveryError::CapabilityNotFound("cap.x".to_string());
        assert!(format!("{e}").contains("cap.x"));
        let e2 = DiscoveryError::ProbeFailed("read".to_string());
        assert!(format!("{e2}").contains("read"));
    }

    #[test]
    #[serial_test::serial]
    fn discover_capability_returns_env_provider_without_probe() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("fake.sock");
        std::fs::File::create(&path).expect("touch");
        temp_env::with_var(
            "CRYPTO_SIGNING_PROVIDER_SOCKET",
            Some(path.to_str().expect("utf8")),
            || {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("rt")
                    .block_on(async {
                        let p = discover_capability("crypto.signing")
                            .await
                            .expect("discovered");
                        assert_eq!(p.socket, path);
                        assert!(p.discovered_via.starts_with("env:"));
                    });
            },
        );
    }

    #[test]
    #[serial_test::serial]
    fn get_socket_directories_respects_socket_scan_dir_override() {
        let dir = tempfile::tempdir().expect("tempdir");
        temp_env::with_var(
            "SOCKET_SCAN_DIR",
            Some(dir.path().to_str().expect("utf8")),
            || {
                let dirs = get_socket_directories();
                assert_eq!(dirs.first().map(|p| p.as_path()), Some(dir.path()));
            },
        );
    }

    #[tokio::test]
    async fn probe_socket_success_parses_capabilities() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock_path = dir.path().join("cap.sock");
        let listener = tokio::net::UnixListener::bind(&sock_path).expect("bind");

        let cap_name = "probe.test.cap";
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
            let mut stream = stream;
            let mut line = String::new();
            let mut reader = BufReader::new(&mut stream);
            reader.read_line(&mut line).await.expect("read");
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "id": "1",
                "result": {
                    "capabilities": [cap_name],
                    "metadata": { "k": "v" }
                }
            });
            let mut out = serde_json::to_string(&resp).unwrap();
            out.push('\n');
            stream.write_all(out.as_bytes()).await.expect("write");
            stream.flush().await.expect("flush");
        });

        tokio::time::sleep(std::time::Duration::from_millis(30)).await;

        let p = probe_socket(&sock_path).await.expect("probe");
        assert!(p.capabilities.contains(&cap_name.to_string()));
        assert_eq!(p.discovered_via, "probe");
        assert_eq!(p.metadata.get("k").map(String::as_str), Some("v"));
    }

    #[tokio::test]
    async fn probe_socket_jsonrpc_error_returns_probe_failed() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock_path = dir.path().join("err.sock");
        let listener = tokio::net::UnixListener::bind(&sock_path).expect("bind");

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
            let mut stream = stream;
            let mut line = String::new();
            let mut reader = BufReader::new(&mut stream);
            reader.read_line(&mut line).await.expect("read");
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "error": { "code": -32601, "message": "nope" }
            });
            let mut out = serde_json::to_string(&resp).unwrap();
            out.push('\n');
            stream.write_all(out.as_bytes()).await.expect("write");
            stream.flush().await.expect("flush");
        });

        tokio::time::sleep(std::time::Duration::from_millis(30)).await;

        let err = probe_socket(&sock_path).await.unwrap_err();
        match err {
            DiscoveryError::ProbeFailed(m) => assert!(m.contains("Method not supported")),
            _ => panic!("expected ProbeFailed, got {err:?}"),
        }
    }

    #[test]
    #[serial_test::serial]
    fn discover_capability_not_found_without_env_or_registry() {
        let dir = tempfile::tempdir().expect("tempdir");
        temp_env::with_vars(
            [
                ("SOCKET_SCAN_DIR", Some(dir.path().to_str().expect("utf8"))),
                ("NEURAL_API_SOCKET", None::<&str>),
                ("CAPABILITY_REGISTRY_SOCKET", None::<&str>),
            ],
            || {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("rt")
                    .block_on(async {
                        let err =
                            discover_capability("zzzz.nonexistent.capability.discovery.test.99999")
                                .await
                                .expect_err("expected not found");
                        match err {
                            DiscoveryError::CapabilityNotFound(c) => {
                                assert!(c.contains("zzzz.nonexistent"));
                            }
                            _ => panic!("unexpected {err:?}"),
                        }
                    });
            },
        );
    }

    #[tokio::test]
    async fn discover_all_capabilities_returns_ok_map() {
        let map = discover_all_capabilities().await.expect("ok");
        let _ = map.len();
    }
}
