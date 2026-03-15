// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Socket and endpoint discovery logic

use std::io::{self, Result as IoResult};
use std::path::PathBuf;

use crate::transport::types::IpcEndpoint;

/// Discover IPC endpoint for a service
///
/// Automatically discovers the correct endpoint, whether Unix socket,
/// Named pipe, or TCP fallback.
///
/// ## Isomorphic Discovery
///
/// Tries in order:
/// 1. Unix domain socket (optimal)
/// 2. Named pipe (Windows)
/// 3. TCP discovery file
///
/// # Arguments
///
/// * `service_name` - Name of the service to discover
///
/// # Returns
///
/// The discovered IPC endpoint
///
/// # Example
///
/// ```ignore,no_run
/// use universal_patterns::transport::discovery::discover_ipc_endpoint;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Discover endpoint (Unix socket OR TCP)
/// let endpoint = discover_ipc_endpoint("squirrel")?;
/// # Ok(())
/// # }
/// ```
pub fn discover_ipc_endpoint(service_name: &str) -> IoResult<IpcEndpoint> {
    // 1. Try Unix socket first (optimal)
    #[cfg(unix)]
    {
        let socket_paths = get_socket_paths(service_name);
        for path in socket_paths {
            if path.exists() {
                tracing::debug!("Discovered Unix socket: {}", path.display());
                return Ok(IpcEndpoint::UnixSocket(path));
            }
        }
    }

    // 2. Try Named Pipe (Windows)
    #[cfg(windows)]
    {
        let pipe_name = format!(r"\\.\pipe\{}", service_name);
        tracing::debug!("Trying Named Pipe: {}", pipe_name);
        return Ok(IpcEndpoint::NamedPipe(pipe_name));
    }

    // 3. Try TCP discovery file
    discover_tcp_endpoint(service_name)
}

/// Discover TCP endpoint from discovery file
///
/// Reads XDG-compliant discovery files to find TCP fallback endpoint.
pub fn discover_tcp_endpoint(service_name: &str) -> IoResult<IpcEndpoint> {
    let discovery_files = get_tcp_discovery_file_candidates(service_name);

    for file_path in discovery_files {
        if let Ok(contents) = std::fs::read_to_string(&file_path) {
            // Parse format: tcp:127.0.0.1:PORT
            if let Some(addr_str) = contents.trim().strip_prefix("tcp:") {
                if let Ok(addr) = addr_str.parse::<std::net::SocketAddr>() {
                    tracing::info!(
                        "📁 Discovered TCP endpoint: {} (from {})",
                        addr,
                        file_path.display()
                    );
                    return Ok(IpcEndpoint::TcpLocal(addr));
                }
            }
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!(
            "Could not discover IPC endpoint for {} (no Unix socket or TCP discovery file)",
            service_name
        ),
    ))
}

/// Get TCP discovery file candidates
///
/// Returns paths to check for TCP discovery files in XDG-compliant order.
pub fn get_tcp_discovery_file_candidates(service_name: &str) -> Vec<PathBuf> {
    let discovery_dirs = [
        std::env::var("XDG_RUNTIME_DIR").ok(),
        std::env::var("HOME")
            .ok()
            .map(|h| format!("{}/.local/share", h)),
        Some("/tmp".to_string()),
    ];

    discovery_dirs
        .iter()
        .filter_map(|d| d.as_ref())
        .map(|dir| PathBuf::from(format!("{}/{}-ipc-port", dir, service_name)))
        .collect()
}

/// Get potential Unix socket paths for a service
///
/// Returns likely paths where Unix sockets might exist.
#[cfg(unix)]
pub fn get_socket_paths(service_name: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // XDG_RUNTIME_DIR/biomeos/{service}.sock (ecosystem convention)
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        paths.push(PathBuf::from(format!(
            "{}/{}/{}.sock",
            runtime_dir,
            universal_constants::network::BIOMEOS_SOCKET_SUBDIR,
            service_name
        )));
    }

    // /tmp/biomeos/{service}.sock (ecosystem fallback)
    paths.push(PathBuf::from(format!(
        "{}/{}.sock",
        universal_constants::network::BIOMEOS_SOCKET_FALLBACK_DIR,
        service_name
    )));

    paths
}

/// Write TCP discovery file for client discovery
///
/// Writes an XDG-compliant discovery file when using TCP fallback,
/// enabling clients to automatically discover the TCP endpoint.
///
/// ## Discovery File Format
///
/// Format: `tcp:127.0.0.1:PORT`
///
/// ## XDG-Compliant Paths
///
/// Tries in order:
/// 1. `$XDG_RUNTIME_DIR/{service}-ipc-port`
/// 2. `$HOME/.local/share/{service}-ipc-port`
/// 3. `/tmp/{service}-ipc-port`
pub fn write_tcp_discovery_file(service_name: &str, addr: &std::net::SocketAddr) -> IoResult<()> {
    use std::io::Write;

    // XDG-compliant discovery directories (in order of preference)
    let discovery_dirs = [
        std::env::var("XDG_RUNTIME_DIR").ok(),
        std::env::var("HOME")
            .ok()
            .map(|h| format!("{}/.local/share", h)),
        Some("/tmp".to_string()),
    ];

    for dir in discovery_dirs.iter().filter_map(|d| d.as_ref()) {
        let discovery_file = format!("{}/{}-ipc-port", dir, service_name);

        match std::fs::File::create(&discovery_file) {
            Ok(mut file) => {
                // Write format: tcp:127.0.0.1:PORT
                writeln!(file, "tcp:{}", addr)?;
                tracing::debug!("   TCP discovery file: {}", discovery_file);
                return Ok(());
            }
            Err(e) => {
                tracing::debug!("   Could not write to {}: {}", discovery_file, e);
                continue;
            }
        }
    }

    Err(io::Error::new(
        io::ErrorKind::PermissionDenied,
        "Could not write discovery file to any XDG-compliant directory",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- get_tcp_discovery_file_candidates tests ---
    #[test]
    fn test_get_tcp_discovery_file_candidates_returns_paths() {
        let candidates = get_tcp_discovery_file_candidates("test-service");
        // Should have at least /tmp candidate
        assert!(!candidates.is_empty());

        // All candidates should end with the service name pattern
        for path in &candidates {
            assert!(
                path.to_string_lossy().contains("test-service-ipc-port"),
                "Path should contain service name: {:?}",
                path
            );
        }
    }

    #[test]
    fn test_get_tcp_discovery_file_candidates_includes_tmp() {
        let candidates = get_tcp_discovery_file_candidates("myservice");
        let has_tmp = candidates
            .iter()
            .any(|p| p.to_string_lossy().starts_with("/tmp/"));
        assert!(has_tmp, "Should include /tmp/ fallback");
    }

    #[test]
    fn test_get_tcp_discovery_file_candidates_different_services() {
        let candidates_a = get_tcp_discovery_file_candidates("service-a");
        let candidates_b = get_tcp_discovery_file_candidates("service-b");

        // Should produce different paths
        assert_ne!(candidates_a.first().unwrap(), candidates_b.first().unwrap(),);
    }

    // --- get_socket_paths tests (unix only) ---
    #[cfg(unix)]
    #[test]
    fn test_get_socket_paths_returns_paths() {
        let paths = get_socket_paths("test-service");
        assert!(!paths.is_empty());

        // All paths should end with .sock
        for path in &paths {
            assert!(
                path.to_string_lossy().ends_with(".sock"),
                "Path should end with .sock: {:?}",
                path
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_get_socket_paths_includes_tmp() {
        let paths = get_socket_paths("myservice");
        let has_tmp = paths
            .iter()
            .any(|p| p.to_string_lossy().starts_with("/tmp/"));
        assert!(has_tmp, "Should include /tmp/ path");
    }

    #[cfg(unix)]
    #[test]
    fn test_get_socket_paths_uses_biomeos_convention() {
        std::env::remove_var("XDG_RUNTIME_DIR");
        let paths = get_socket_paths("myservice");
        let has_biomeos = paths
            .iter()
            .any(|p| p.to_string_lossy().contains("biomeos"));
        assert!(
            has_biomeos,
            "Should include biomeos path per ecosystem convention"
        );
    }

    // --- discover_ipc_endpoint tests ---
    #[test]
    fn test_discover_ipc_endpoint_nonexistent_service() {
        let result = discover_ipc_endpoint("nonexistent-service-xyz-12345");
        // Should fail since no socket or discovery file exists
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    // --- discover_tcp_endpoint tests ---
    #[test]
    fn test_discover_tcp_endpoint_nonexistent() {
        let result = discover_tcp_endpoint("nonexistent-service-abc-99999");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound);
    }

    // --- write_tcp_discovery_file + discover roundtrip ---
    #[test]
    fn test_write_and_discover_tcp_endpoint() {
        let service_name = "test-discovery-roundtrip";
        let addr: std::net::SocketAddr = "127.0.0.1:54321".parse().unwrap();

        // Write discovery file
        let write_result = write_tcp_discovery_file(service_name, &addr);
        assert!(write_result.is_ok(), "Failed to write discovery file");

        // Discover it
        let discover_result = discover_tcp_endpoint(service_name);
        assert!(discover_result.is_ok(), "Failed to discover endpoint");

        let endpoint = discover_result.unwrap();
        match endpoint {
            IpcEndpoint::TcpLocal(discovered_addr) => {
                assert_eq!(discovered_addr, addr);
            }
            other => panic!("Expected TcpLocal, got {:?}", other),
        }

        // Clean up - remove discovery files
        let candidates = get_tcp_discovery_file_candidates(service_name);
        for path in candidates {
            let _ = std::fs::remove_file(path);
        }
    }
}
