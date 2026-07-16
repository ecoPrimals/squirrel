// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use std::io;
use std::path::Path;

use tokio::time::Duration;

use crate::transport::{TransportEndpoint, TransportStream, connect_transport_with_timeout};

use super::types::{IpcClientError, IpcErrorPhase};

/// Connect to the IPC endpoint with a bounded wait (connection phase).
///
/// On Unix, `endpoint` is treated as a Unix socket path.
/// On non-Unix, `endpoint` is parsed as a TCP `host:port` address (localhost fallback).
pub(super) async fn connect_ipc_stream(
    endpoint: &Path,
    connection_timeout: Duration,
) -> Result<TransportStream, anyhow::Error> {
    connect_transport_with_timeout(
        &TransportEndpoint::uds(endpoint.to_string_lossy()),
        connection_timeout,
    )
    .await
    .map_err(|e| {
        if e.kind() == io::ErrorKind::TimedOut {
            IpcClientError::Timeout {
                phase: IpcErrorPhase::Connect,
                duration: connection_timeout,
            }
        } else {
            IpcClientError::Io {
                phase: IpcErrorPhase::Connect,
                source: e,
            }
        }
    })
    .map_err(Into::into)
}
