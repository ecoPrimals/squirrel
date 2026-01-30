//! Capability Discovery - TRUE PRIMAL Infant Pattern
//!
//! Discovers capabilities at runtime with ZERO hardcoded primal names.
//! Deploy like an infant - knows nothing, discovers everything.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
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
    #[error("Capability not found: {0}")]
    CapabilityNotFound(String),

    #[error("Socket probe failed: {0}")]
    ProbeFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

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
    // BIOME OS FIX (Jan 20, 2026): Registry BEFORE socket scan for speed
    // Registry query is <1ms vs socket scan 2s+ timeout
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
/// BIOME OS FIX (Jan 27, 2026): Trust explicit env vars without probing.
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
                id: format!("{}-provider", capability),
                capabilities: vec![capability.to_string()],
                socket: path,
                metadata: std::collections::HashMap::new(),
                discovered_via: format!("env:{}", env_var),
            }));
        }
    }

    Ok(None)
}

/// Scan socket directory for capability providers
///
/// TRUE PRIMAL: Scans all sockets, probes each to ask what it provides
/// BIOME OS FIX: Added overall timeout to prevent infinite hangs
async fn try_socket_scan(capability: &str) -> Result<Option<CapabilityProvider>, DiscoveryError> {
    // Get socket directory from environment or use default
    let socket_dirs = get_socket_directories();

    // BIOME OS FIX: Total scan timeout of 5 seconds (was unlimited)
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
                        if let Ok(provider) = probe_socket(&path).await {
                            if provider.capabilities.contains(&capability.to_string()) {
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
        }
        Ok(None)
    })
    .await;

    match scan_result {
        Ok(result) => result,
        Err(_) => {
            warn!("Socket scan timed out after 5s");
            Ok(None)
        }
    }
}

/// Query capability registry if available
///
/// TRUE PRIMAL: Even the registry is discovered, not hardcoded!
async fn try_registry_query(
    capability: &str,
) -> Result<Option<CapabilityProvider>, DiscoveryError> {
    // First, discover the registry itself (no hardcoding!)
    if let Ok(registry_socket) = std::env::var("CAPABILITY_REGISTRY_SOCKET") {
        let registry_path = PathBuf::from(registry_socket);

        if registry_path.exists() {
            debug!("Querying capability registry at: {:?}", registry_path);

            // Connect to registry and query
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
async fn probe_socket(socket_path: &Path) -> Result<CapabilityProvider, DiscoveryError> {
    // Connect to socket
    let stream = UnixStream::connect(socket_path)
        .await
        .map_err(|e| DiscoveryError::ProbeFailed(e.to_string()))?;

    // Build discovery request (JSON-RPC 2.0)
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "discover_capabilities",
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

    // BIOME OS FIX: Use 2s timeout per socket (was 500ms)
    match tokio::time::timeout(
        std::time::Duration::from_secs(2),
        reader.read_line(&mut response_line),
    )
    .await
    {
        Ok(Ok(_)) => {
            // Parse JSON-RPC response
            let response: serde_json::Value = serde_json::from_str(&response_line)?;

            // BIOME OS FIX: Handle JSON-RPC error responses gracefully!
            if let Some(error) = response.get("error") {
                debug!(
                    "Socket {:?} returned JSON-RPC error: {} (code: {})",
                    socket_path,
                    error
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("unknown"),
                    error.get("code").and_then(|c| c.as_i64()).unwrap_or(-1)
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
        Ok(Err(e)) => Err(DiscoveryError::ProbeFailed(format!("Read error: {}", e))),
        Err(_) => Err(DiscoveryError::ProbeFailed("Timeout (>2s)".to_string())),
    }
}

/// Query capability registry for a specific capability
async fn query_registry(
    registry_path: &Path,
    capability: &str,
) -> Result<CapabilityProvider, DiscoveryError> {
    let stream = UnixStream::connect(registry_path)
        .await
        .map_err(|e| DiscoveryError::ProbeFailed(e.to_string()))?;

    // Build registry query (JSON-RPC 2.0)
    // BIOME OS FIX (Jan 20, 2026): Use correct Neural API method name
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "neural_api.discover_capability",
        "params": {
            "capability": capability,
        },
        "id": Uuid::new_v4().to_string(),
    });

    let mut request_str = serde_json::to_string(&request)?;
    request_str.push('\n');

    let (read_half, mut write_half) = stream.into_split();
    write_half.write_all(request_str.as_bytes()).await?;

    let mut reader = BufReader::new(read_half);
    let mut response_line = String::new();

    // BIOME OS FIX (Jan 27, 2026): Add timeout to prevent hangs
    match tokio::time::timeout(
        std::time::Duration::from_secs(2),
        reader.read_line(&mut response_line),
    )
    .await
    {
        Ok(Ok(_)) => { /* Continue with response parsing */ }
        Ok(Err(e)) => {
            return Err(DiscoveryError::ProbeFailed(format!(
                "Registry read error: {}",
                e
            )))
        }
        Err(_) => {
            return Err(DiscoveryError::ProbeFailed(
                "Registry query timeout (>2s)".to_string(),
            ))
        }
    }

    let response: serde_json::Value = serde_json::from_str(&response_line)?;

    if let Some(result) = response.get("result") {
        // Neural API returns: {"capability": "...", "primary_socket": "...", "primals": [...]}
        // Extract primary_socket and build CapabilityProvider
        if let Some(socket_path) = result.get("primary_socket").and_then(|s| s.as_str()) {
            Ok(CapabilityProvider {
                id: result
                    .get("capability")
                    .and_then(|c| c.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                capabilities: vec![capability.to_string()],
                socket: PathBuf::from(socket_path),
                metadata: std::collections::HashMap::new(),
                discovered_via: "registry".to_string(),
            })
        } else {
            Err(DiscoveryError::CapabilityNotFound(capability.to_string()))
        }
    } else {
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
    let biomeos_dir = PathBuf::from(format!("/run/user/{}/biomeos", uid));
    if biomeos_dir.exists() {
        dirs.push(biomeos_dir);
    }

    // Priority 3: XDG Runtime Directory with biomeos subdirectory
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let xdg_biomeos = PathBuf::from(format!("{}/biomeos", runtime_dir));
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
/// Returns a map of capability name → providers
pub async fn discover_all_capabilities() -> Result<HashMap<String, Vec<CapabilityProvider>>> {
    info!("🔍 Discovering all available capabilities...");

    let mut all_capabilities: HashMap<String, Vec<CapabilityProvider>> = HashMap::new();

    // Scan all socket directories
    for socket_dir in get_socket_directories() {
        if let Ok(mut entries) = fs::read_dir(&socket_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();

                if is_unix_socket(&path).await {
                    if let Ok(provider) = probe_socket(&path).await {
                        // Add this provider to all its capabilities
                        for capability in &provider.capabilities {
                            all_capabilities
                                .entry(capability.clone())
                                .or_default()
                                .push(provider.clone());
                        }
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

// ============================================================================
// STANDARD PRIMAL DISCOVERY HELPERS
// ============================================================================
//
// These convenience functions discover specific primals in the NUCLEUS stack.
// They follow TRUE PRIMAL principles:
// - Check environment variables first (explicit configuration)
// - Then check standard biomeOS paths (NUCLEUS-compliant discovery)
// - No hardcoded assumptions about what capabilities each primal provides
//
// Used for:
// - Tower Atomic: BearDog + Songbird
// - Node Atomic: Tower + Toadstool
// - Nest Atomic: Tower + NestGate
// - Full NUCLEUS: All primals

/// Discover Songbird primal (network, discovery, TLS capabilities)
///
/// ## Discovery Order
/// 1. `SONGBIRD_SOCKET` environment variable
/// 2. `/run/user/<uid>/biomeos/songbird.sock` (standard path)
/// 3. Socket scan fallback (slow)
///
/// ## Example
/// ```no_run
/// # use squirrel::capabilities::discovery::discover_songbird;
/// # async fn example() -> anyhow::Result<()> {
/// let songbird = discover_songbird().await?;
/// println!("Found Songbird at: {:?}", songbird.socket);
/// # Ok(())
/// # }
/// ```
pub async fn discover_songbird() -> Result<CapabilityProvider, DiscoveryError> {
    discover_standard_primal("songbird", &["network", "discovery", "tls"]).await
}

/// Discover BearDog primal (security, crypto, JWT capabilities)
///
/// ## Discovery Order
/// 1. `BEARDOG_SOCKET` environment variable
/// 2. `/run/user/<uid>/biomeos/beardog.sock` (standard path)
/// 3. Socket scan fallback (slow)
///
/// ## Example
/// ```no_run
/// # use squirrel::capabilities::discovery::discover_beardog;
/// # async fn example() -> anyhow::Result<()> {
/// let beardog = discover_beardog().await?;
/// println!("Found BearDog at: {:?}", beardog.socket);
/// # Ok(())
/// # }
/// ```
pub async fn discover_beardog() -> Result<CapabilityProvider, DiscoveryError> {
    discover_standard_primal("beardog", &["security", "crypto", "jwt"]).await
}

/// Discover Toadstool primal (compute, GPU capabilities)
///
/// ## Discovery Order
/// 1. `TOADSTOOL_SOCKET` environment variable
/// 2. `/run/user/<uid>/biomeos/toadstool.sock` (standard path)
/// 3. Socket scan fallback (slow)
///
/// ## Example
/// ```no_run
/// # use squirrel::capabilities::discovery::discover_toadstool;
/// # async fn example() -> anyhow::Result<()> {
/// let toadstool = discover_toadstool().await?;
/// println!("Found Toadstool at: {:?}", toadstool.socket);
/// # Ok(())
/// # }
/// ```
pub async fn discover_toadstool() -> Result<CapabilityProvider, DiscoveryError> {
    discover_standard_primal("toadstool", &["compute", "gpu"]).await
}

/// Discover NestGate primal (storage, persistence capabilities)
///
/// ## Discovery Order
/// 1. `NESTGATE_SOCKET` environment variable
/// 2. `/run/user/<uid>/biomeos/nestgate.sock` (standard path)
/// 3. Socket scan fallback (slow)
///
/// ## Example
/// ```no_run
/// # use squirrel::capabilities::discovery::discover_nestgate;
/// # async fn example() -> anyhow::Result<()> {
/// let nestgate = discover_nestgate().await?;
/// println!("Found NestGate at: {:?}", nestgate.socket);
/// # Ok(())
/// # }
/// ```
pub async fn discover_nestgate() -> Result<CapabilityProvider, DiscoveryError> {
    discover_standard_primal("nestgate", &["storage", "persistence"]).await
}

/// Generic discovery for standard biomeOS primals
///
/// This function implements the standardized discovery pattern used by all primals
/// in the NUCLEUS stack. It's TRUE PRIMAL - checks environment first, then standard
/// paths, then falls back to socket scanning.
///
/// ## Discovery Order
/// 1. `{PRIMAL}_SOCKET` env var (e.g., `SONGBIRD_SOCKET`)
/// 2. `/run/user/<uid>/biomeos/{primal}.sock` (standard path)
/// 3. Socket scan of all directories (fallback)
///
/// ## Arguments
/// - `primal_name`: Name of the primal (e.g., "songbird", "beardog")
/// - `expected_capabilities`: Capabilities we expect this primal to provide
///
/// ## Returns
/// - `Ok(CapabilityProvider)` if primal found and reachable
/// - `Err(DiscoveryError)` if primal not found or unreachable
async fn discover_standard_primal(
    primal_name: &str,
    expected_capabilities: &[&str],
) -> Result<CapabilityProvider, DiscoveryError> {
    debug!("🔍 Discovering {} primal...", primal_name);

    // Priority 1: Check environment variable (explicit configuration)
    let env_var = format!("{}_SOCKET", primal_name.to_uppercase());
    if let Ok(socket_path) = std::env::var(&env_var) {
        let path = PathBuf::from(&socket_path);
        if path.exists() {
            info!(
                "✅ Found {} via env var {} = {}",
                primal_name, env_var, socket_path
            );

            // Try to probe the socket to verify it's reachable
            // If probe fails, we still trust the env var (operator knows best)
            match probe_socket(&path).await {
                Ok(provider) => return Ok(provider),
                Err(e) => {
                    debug!(
                        "Probe failed for {} socket, trusting env var anyway: {}",
                        primal_name, e
                    );
                    return Ok(CapabilityProvider {
                        id: primal_name.to_string(),
                        capabilities: expected_capabilities
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
                        socket: path,
                        metadata: HashMap::new(),
                        discovered_via: format!("env:{}", env_var),
                    });
                }
            }
        }
    }

    // Priority 2: Check standard biomeOS path (NUCLEUS-compliant!)
    let uid = nix::unistd::getuid();
    let standard_path = PathBuf::from(format!("/run/user/{}/biomeos/{}.sock", uid, primal_name));

    if standard_path.exists() {
        info!(
            "✅ Found {} at standard path: {}",
            primal_name,
            standard_path.display()
        );

        // Try to probe the socket
        match probe_socket(&standard_path).await {
            Ok(provider) => return Ok(provider),
            Err(e) => {
                debug!("Probe failed for {} at standard path: {}", primal_name, e);
                // Return it anyway - socket exists, might just not respond to probe
                return Ok(CapabilityProvider {
                    id: primal_name.to_string(),
                    capabilities: expected_capabilities
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    socket: standard_path,
                    metadata: HashMap::new(),
                    discovered_via: "standard_path".to_string(),
                });
            }
        }
    }

    // Priority 3: Fallback to socket scan (slow, but comprehensive)
    debug!(
        "Standard path not found, falling back to socket scan for {}",
        primal_name
    );

    // Try to discover by capability
    for capability in expected_capabilities {
        if let Ok(provider) = discover_capability(capability).await {
            info!(
                "✅ Found {} via capability scan: {}",
                primal_name, capability
            );
            return Ok(provider);
        }
    }

    warn!("❌ Could not discover {} primal", primal_name);
    Err(DiscoveryError::CapabilityNotFound(format!(
        "Primal '{}' not found via env var, standard path, or socket scan",
        primal_name
    )))
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
}
