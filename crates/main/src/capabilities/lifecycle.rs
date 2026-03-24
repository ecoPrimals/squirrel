// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! biomeOS Lifecycle Integration
//!
//! Implements the healthSpring lifecycle pattern:
//! - `lifecycle.register` on startup (when biomeOS socket found)
//! - `lifecycle.status` heartbeat every 30s
//! - Socket file cleanup on SIGTERM
//!
//! All communication uses Unix socket JSON-RPC — no HTTP.

use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::watch;
use tracing::{debug, info, warn};

use crate::niche;
use crate::primal_names;

/// Discover the biomeOS orchestrator socket.
///
/// Checks standard locations without hardcoding a primal name.
pub fn find_biomeos_socket() -> Option<PathBuf> {
    // Explicit env override
    if let Ok(path) = std::env::var("BIOMEOS_SOCKET") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    let uid = nix::unistd::getuid();
    let dir = primal_names::BIOMEOS_SOCKET_DIR;
    let candidates = [
        format!(
            "/run/user/{uid}/{dir}/{}",
            primal_names::BIOMEOS_SOCKET_NAME
        ),
        format!(
            "/run/user/{uid}/{dir}/{}",
            primal_names::NEURAL_API_SOCKET_NAME
        ),
        format!("/tmp/{}", primal_names::BIOMEOS_SOCKET_NAME),
        format!("/tmp/{}", primal_names::NEURAL_API_SOCKET_NAME),
    ];

    candidates
        .into_iter()
        .map(PathBuf::from)
        .find(|p| p.exists())
}

/// Send `lifecycle.register` to a biomeOS socket.
///
/// Returns `true` if the registration was acknowledged.
pub async fn register_with_biomeos(
    biomeos_socket: &Path,
    own_socket: &str,
    capabilities: &[&str],
) -> bool {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "lifecycle.register",
        "params": {
            "primal": niche::PRIMAL_ID,
            "version": niche::PRIMAL_VERSION,
            "socket": own_socket,
            "capabilities": capabilities,
            "domain": niche::DOMAIN,
        },
        "id": 1
    });

    match send_jsonrpc_public(biomeos_socket, &request).await {
        Ok(resp) => {
            if resp.get("error").is_some() {
                warn!(
                    "lifecycle.register rejected: {:?}",
                    resp.get("error")
                        .and_then(|e| e.get("message"))
                        .and_then(|m| m.as_str())
                );
                false
            } else {
                info!("Registered with biomeOS at {}", biomeos_socket.display());
                true
            }
        }
        Err(e) => {
            warn!("lifecycle.register failed: {e}");
            false
        }
    }
}

/// Spawn a background heartbeat task that sends `lifecycle.status` every `interval`.
///
/// The task runs until `shutdown_rx` receives a signal.
#[must_use]
pub fn spawn_heartbeat(
    biomeos_socket: PathBuf,
    own_socket: String,
    interval: std::time::Duration,
    mut shutdown_rx: watch::Receiver<bool>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let request = serde_json::json!({
                        "jsonrpc": "2.0",
                        "method": "lifecycle.status",
                        "params": {
                            "primal": niche::PRIMAL_ID,
                            "socket": own_socket,
                            "status": "healthy",
                        },
                        "id": 1
                    });

                    match send_jsonrpc_public(&biomeos_socket, &request).await {
                        Ok(_) => debug!("heartbeat sent to biomeOS"),
                        Err(e) => debug!("heartbeat failed (biomeOS may be down): {e}"),
                    }
                }
                _ = shutdown_rx.changed() => {
                    info!("heartbeat task shutting down");
                    break;
                }
            }
        }
    })
}

/// Clean up the socket file. Safe to call even if the file doesn't exist.
pub fn cleanup_socket(socket_path: &str) {
    if let Err(e) = std::fs::remove_file(socket_path)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        warn!("Failed to remove socket {socket_path}: {e}");
    }
}

/// Install a SIGTERM handler that cleans up the socket and sends shutdown signal.
///
/// Returns a `watch::Sender` that fires when shutdown is requested (Ctrl+C or SIGTERM).
#[must_use]
pub fn install_signal_handlers(
    socket_path: String,
) -> (watch::Sender<bool>, tokio::task::JoinHandle<()>) {
    let (shutdown_tx, _) = watch::channel(false);
    let tx_clone = shutdown_tx.clone();

    let handle = tokio::spawn(async move {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{SignalKind, signal};
            let mut sigterm = match signal(SignalKind::terminate()) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Failed to setup SIGTERM handler: {e}");
                    return;
                }
            };
            let mut sigint = match signal(SignalKind::interrupt()) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Failed to setup SIGINT handler: {e}");
                    return;
                }
            };

            tokio::select! {
                _ = sigterm.recv() => {
                    info!("SIGTERM received — cleaning up");
                }
                _ = sigint.recv() => {
                    info!("SIGINT received — cleaning up");
                }
            }
        }

        #[cfg(not(unix))]
        {
            if let Err(e) = tokio::signal::ctrl_c().await {
                tracing::error!("Failed to setup Ctrl-C handler: {e}");
                return;
            }
            info!("Ctrl+C received — cleaning up");
        }

        cleanup_socket(&socket_path);
        let _ = tx_clone.send(true);
    });

    (shutdown_tx, handle)
}

/// Send a single JSON-RPC request over a Unix socket and read the response.
///
/// Used by both `lifecycle` (biomeOS) and `songbird` announcement modules.
pub(crate) async fn send_jsonrpc_public(
    socket: &Path,
    request: &serde_json::Value,
) -> anyhow::Result<serde_json::Value> {
    let stream = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        UnixStream::connect(socket),
    )
    .await??;

    let mut line = serde_json::to_string(request)?;
    line.push('\n');

    let (reader, mut writer) = tokio::io::split(stream);
    writer.write_all(line.as_bytes()).await?;
    writer.flush().await?;

    let mut buf = BufReader::new(reader);
    let mut resp_line = String::new();
    tokio::time::timeout(
        std::time::Duration::from_secs(2),
        buf.read_line(&mut resp_line),
    )
    .await??;

    let response: serde_json::Value = serde_json::from_str(resp_line.trim())?;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_biomeos_socket_env_override() {
        temp_env::with_var(
            "BIOMEOS_SOCKET",
            Some("/tmp/nonexistent_biomeos_test.sock"),
            || {
                assert!(find_biomeos_socket().is_none());
            },
        );
    }

    #[test]
    fn test_cleanup_socket_nonexistent() {
        cleanup_socket("/tmp/nonexistent_socket_test_12345.sock");
        // Should not panic
    }

    #[tokio::test]
    async fn test_signal_handler_creation() {
        let (tx, _handle) = install_signal_handlers("/tmp/test_signal.sock".to_string());
        // Just verify it creates without panicking
        let _ = tx.send(true);
    }

    #[tokio::test]
    async fn register_with_biomeos_success() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock_path = dir.path().join("biome.sock");
        let listener = tokio::net::UnixListener::bind(&sock_path).expect("bind");

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept");
            use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
            let mut reader = BufReader::new(&mut stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.expect("read");
            let resp = serde_json::json!({"jsonrpc":"2.0","result":{},"id":1});
            let mut body = serde_json::to_string(&resp).expect("should succeed");
            body.push('\n');
            stream.write_all(body.as_bytes()).await.expect("write");
        });

        let ok = register_with_biomeos(&sock_path, "/tmp/own.sock", &["cap.a", "cap.b"]).await;
        server.await.expect("join server task");
        assert!(ok);
    }

    #[tokio::test]
    async fn register_with_biomeos_jsonrpc_error_returns_false() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock_path = dir.path().join("biome2.sock");
        let listener = tokio::net::UnixListener::bind(&sock_path).expect("bind");

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept");
            use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
            let mut reader = BufReader::new(&mut stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.expect("read");
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": -32000, "message": "rejected"},
                "id": 1
            });
            let mut body = serde_json::to_string(&resp).expect("should succeed");
            body.push('\n');
            stream.write_all(body.as_bytes()).await.expect("write");
        });

        let ok = register_with_biomeos(&sock_path, "/x", &[]).await;
        server.await.expect("server");
        assert!(!ok);
    }

    #[tokio::test]
    async fn send_jsonrpc_public_connection_refused() {
        let p = std::path::Path::new("/tmp/nonexistent_squirrel_lifecycle_sock_99999.sock");
        let req = serde_json::json!({"jsonrpc":"2.0","method":"ping","id":1});
        let err = send_jsonrpc_public(p, &req).await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn heartbeat_stops_after_shutdown_signal() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("hb_bio.sock");
        let _listener = tokio::net::UnixListener::bind(&sock).expect("bind");

        let (tx, rx) = watch::channel(false);
        let handle = spawn_heartbeat(
            sock,
            "/ignored.sock".to_string(),
            std::time::Duration::from_millis(40),
            rx,
        );

        tx.send(true).expect("shutdown");
        tokio::time::timeout(std::time::Duration::from_secs(3), handle)
            .await
            .expect("timeout")
            .expect("join");
    }
}
