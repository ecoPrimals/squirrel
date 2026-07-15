// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use std::path::Path;

use tokio::time::{Duration, timeout};

use super::types::{IpcClientError, IpcErrorPhase};

/// Platform stream returned by [`connect_ipc_stream`].
///
/// On Unix this wraps a `UnixStream`; on Windows a `TcpStream` (TCP localhost
/// fallback — named pipe support can be added when the Windows target matures).
#[cfg(unix)]
pub(super) type PlatformStream = tokio::net::UnixStream;
#[cfg(not(unix))]
pub(super) type PlatformStream = tokio::net::TcpStream;

/// Connect to the IPC endpoint with a bounded wait (connection phase).
///
/// On Unix, `endpoint` is treated as a Unix socket path.
/// On non-Unix, `endpoint` is parsed as a TCP `host:port` address (localhost fallback).
pub(super) async fn connect_ipc_stream(
    endpoint: &Path,
    connection_timeout: Duration,
) -> Result<PlatformStream, anyhow::Error> {
    #[cfg(unix)]
    {
        timeout(connection_timeout, PlatformStream::connect(endpoint))
            .await
            .map_err(|_| IpcClientError::Timeout {
                phase: IpcErrorPhase::Connect,
                duration: connection_timeout,
            })?
            .map_err(|e| IpcClientError::Io {
                phase: IpcErrorPhase::Connect,
                source: e,
            })
            .map_err(Into::into)
    }
    #[cfg(not(unix))]
    {
        let addr = endpoint
            .to_str()
            .and_then(|s| s.parse::<std::net::SocketAddr>().ok())
            .unwrap_or_else(|| std::net::SocketAddr::from(([127, 0, 0, 1], 9200)));
        timeout(connection_timeout, PlatformStream::connect(addr))
            .await
            .map_err(|_| IpcClientError::Timeout {
                phase: IpcErrorPhase::Connect,
                duration: connection_timeout,
            })?
            .map_err(|e| IpcClientError::Io {
                phase: IpcErrorPhase::Connect,
                source: e,
            })
            .map_err(Into::into)
    }
}
