// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! IPC transport for universal adapters — JSON-RPC over Unix domain sockets.

use std::path::{Path, PathBuf};

use serde_json::Value;
use universal_patterns::ipc_client::IpcClient;

use super::registry::ServiceInfo;
use crate::error::PrimalError;

/// Resolve the Unix socket path for a capability provider.
///
/// Priority:
/// 1. `unix://` endpoint on the discovered service
/// 2. Absolute `.sock` path on the discovered service
/// 3. `socket_path` in service metadata extensions
/// 4. Tiered env-based resolution (`{CAPABILITY}_SOCKET` → XDG biomeOS path)
pub fn resolve_provider_socket(
    service: &ServiceInfo,
    capability: &str,
) -> Result<PathBuf, PrimalError> {
    for endpoint in &service.endpoints {
        let url = endpoint.as_ref();
        if let Some(path) = url.strip_prefix("unix://") {
            return Ok(PathBuf::from(path));
        }
        if url.starts_with('/')
            && std::path::Path::new(url)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
        {
            return Ok(PathBuf::from(url));
        }
    }

    if let Some(socket) = service.metadata.get("socket_path").and_then(|v| v.as_str()) {
        return Ok(PathBuf::from(socket));
    }

    let (env_var, stem) = capability_socket_config(capability)?;
    Ok(universal_constants::network::resolve_capability_unix_socket(env_var, stem))
}

/// Send a JSON-RPC 2.0 request to a capability provider over Unix socket.
pub async fn send_rpc_request(
    socket: &Path,
    method: &str,
    params: Option<Value>,
) -> Result<Value, PrimalError> {
    let client = IpcClient::new(socket);
    let params = params.unwrap_or(Value::Null);

    client.call(method, &params).await.map_err(|e| {
        PrimalError::OperationFailed(format!("IPC RPC '{method}' at {}: {e}", socket.display()))
    })
}

fn capability_socket_config(capability: &str) -> Result<(&'static str, &'static str), PrimalError> {
    match capability {
        "compute" => Ok(("COMPUTE_SOCKET", "compute")),
        "storage" => Ok(("STORAGE_SOCKET", "storage")),
        "security" => Ok(("SECURITY_SOCKET", "security")),
        "defense" | "defense.anomaly" | "security.anomaly" => {
            Ok(("DEFENSE_SOCKET", "defense-provider"))
        }
        "orchestration" => Ok(("ORCHESTRATION_SOCKET", "orchestration")),
        other => Err(PrimalError::OperationFailed(format!(
            "No socket resolution config for capability '{other}'"
        ))),
    }
}
