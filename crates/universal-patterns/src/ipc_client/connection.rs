// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use std::path::Path;

use tokio::net::UnixStream;
use tokio::time::{Duration, timeout};

use super::types::{IpcClientError, IpcErrorPhase};

/// Connect to the Unix socket with a bounded wait (connection phase).
pub(super) async fn connect_unix_stream(
    socket_path: &Path,
    connection_timeout: Duration,
) -> Result<UnixStream, anyhow::Error> {
    timeout(connection_timeout, UnixStream::connect(socket_path))
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
