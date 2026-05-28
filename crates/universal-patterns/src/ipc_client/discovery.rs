// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use std::path::{Path, PathBuf};

/// XDG-compliant socket path discovery
pub(super) fn discover_socket(service_id: &str) -> PathBuf {
    let sock_name = format!("{service_id}.sock");

    if let Ok(xdg_runtime) = std::env::var(universal_constants::env_vars::sys::XDG_RUNTIME_DIR) {
        let path = Path::new(&xdg_runtime)
            .join(universal_constants::network::BIOMEOS_SOCKET_SUBDIR)
            .join(&sock_name);
        if path.exists() {
            return path;
        }
    }

    PathBuf::from(universal_constants::network::BIOMEOS_SOCKET_FALLBACK_DIR).join(sock_name)
}
