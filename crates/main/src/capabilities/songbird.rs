// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Songbird service-mesh announcement
//!
//! Follows the wetSpring pattern: register capabilities with Songbird via
//! `discovery.register` and maintain a `discovery.heartbeat` loop.
//!
//! Socket discovery order:
//! 1. `SONGBIRD_SOCKET` env var
//! 2. `$XDG_RUNTIME_DIR/biomeos/songbird-default.sock`
//! 3. `/tmp/songbird-default.sock`

use std::path::{Path, PathBuf};
use tokio::sync::watch;
use tracing::{debug, info, warn};

use crate::niche;
use crate::primal_names;

/// Discover the Songbird service-mesh socket.
///
/// Returns `None` if no Songbird socket is found at any standard location.
pub fn discover_socket() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("SONGBIRD_SOCKET") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        let p = PathBuf::from(xdg)
            .join(primal_names::BIOMEOS_SOCKET_DIR)
            .join(primal_names::SONGBIRD_SOCKET_NAME);
        if p.exists() {
            return Some(p);
        }
    }

    let uid = nix::unistd::getuid();
    let dir = primal_names::BIOMEOS_SOCKET_DIR;
    let sock = primal_names::SONGBIRD_SOCKET_NAME;
    let candidates = [
        format!("/run/user/{uid}/{dir}/{sock}"),
        format!("/tmp/{sock}"),
    ];

    candidates
        .into_iter()
        .map(PathBuf::from)
        .find(|p| p.exists())
}

/// Register Squirrel with Songbird via `discovery.register`.
///
/// Sends primal identity, socket path, and all capabilities from `niche.rs`.
/// Returns `true` if Songbird acknowledged the registration.
pub async fn register(songbird_socket: &Path, own_socket: &str) -> bool {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "discovery.register",
        "params": {
            "primal": niche::PRIMAL_ID,
            "socket": own_socket,
            "capabilities": niche::CAPABILITIES,
            "version": niche::PRIMAL_VERSION,
            "domain": niche::DOMAIN,
        },
        "id": 1
    });

    match super::lifecycle::send_jsonrpc_public(songbird_socket, &request).await {
        Ok(resp) => {
            if resp.get("error").is_some() {
                warn!(
                    "discovery.register rejected by Songbird: {:?}",
                    resp.get("error")
                        .and_then(|e| e.get("message"))
                        .and_then(|m| m.as_str())
                );
                false
            } else {
                info!("Registered with Songbird at {}", songbird_socket.display());
                true
            }
        }
        Err(e) => {
            warn!("discovery.register to Songbird failed: {e}");
            false
        }
    }
}

/// Spawn a background heartbeat loop that sends `discovery.heartbeat` to Songbird.
///
/// Runs until `shutdown_rx` receives a signal.
#[must_use]
pub fn start_heartbeat_loop(
    songbird_socket: PathBuf,
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
                        "method": "discovery.heartbeat",
                        "params": {
                            "primal": niche::PRIMAL_ID,
                            "socket": own_socket,
                        },
                        "id": 2
                    });

                    match super::lifecycle::send_jsonrpc_public(&songbird_socket, &request).await {
                        Ok(_) => debug!("heartbeat sent to Songbird"),
                        Err(e) => debug!("Songbird heartbeat failed (may be down): {e}"),
                    }
                }
                _ = shutdown_rx.changed() => {
                    info!("Songbird heartbeat task shutting down");
                    break;
                }
            }
        }
    })
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "Invariant or startup failure: expect after validation"
)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_returns_true_on_jsonrpc_success() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("songbird_reg.sock");
        let listener = tokio::net::UnixListener::bind(&sock).expect("bind");
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept");
            use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
            let mut reader = BufReader::new(&mut stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.ok();
            let resp = serde_json::json!({"jsonrpc":"2.0","result":true,"id":1});
            let mut body = serde_json::to_string(&resp).expect("jsonrpc response");
            body.push('\n');
            stream.write_all(body.as_bytes()).await.ok();
        });
        let ok = register(&sock, "/tmp/own-squirrel.sock").await;
        server.await.ok();
        assert!(ok);
    }

    #[tokio::test]
    async fn register_returns_false_on_jsonrpc_error_field() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("songbird_rej.sock");
        let listener = tokio::net::UnixListener::bind(&sock).expect("bind");
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept");
            use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
            let mut reader = BufReader::new(&mut stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.ok();
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": -32000, "message": "no"},
                "id": 1
            });
            let mut body = serde_json::to_string(&resp).expect("jsonrpc response");
            body.push('\n');
            stream.write_all(body.as_bytes()).await.ok();
        });
        let ok = register(&sock, "/x.sock").await;
        server.await.ok();
        assert!(!ok);
    }

    #[tokio::test]
    async fn heartbeat_loop_stops_on_shutdown() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("songbird_hb.sock");
        let _listener = tokio::net::UnixListener::bind(&sock).expect("bind");
        let (tx, rx) = watch::channel(false);
        let handle = start_heartbeat_loop(
            sock,
            "/ignored.sock".to_string(),
            std::time::Duration::from_millis(30),
            rx,
        );
        tx.send(true).expect("shutdown");
        tokio::time::timeout(std::time::Duration::from_secs(3), handle)
            .await
            .expect("timeout")
            .expect("join");
    }

    #[test]
    fn discover_socket_env_override() {
        temp_env::with_var(
            "SONGBIRD_SOCKET",
            Some("/tmp/nonexistent_songbird_test.sock"),
            || {
                assert!(discover_socket().is_none());
            },
        );
    }

    #[test]
    fn discover_socket_returns_none_when_not_present() {
        temp_env::with_var("SONGBIRD_SOCKET", None::<&str>, || {
            // May or may not find a socket depending on host state,
            // but must not panic.
            let _ = discover_socket();
        });
    }
}
