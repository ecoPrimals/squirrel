// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use std::path::{Path, PathBuf};

/// XDG-compliant socket path discovery
pub(super) fn discover_socket(service_id: &str) -> PathBuf {
    let sock_name = format!("{service_id}.sock");

    // Try XDG_RUNTIME_DIR first
    if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
        let path = Path::new(&xdg_runtime).join("biomeos").join(&sock_name);
        if path.exists() {
            return path;
        }
    }

    // Fallback: /tmp/biomeos/{service_id}.sock (ecosystem convention)
    PathBuf::from(universal_constants::network::BIOMEOS_SOCKET_FALLBACK_DIR).join(sock_name)
}
