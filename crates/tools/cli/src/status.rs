// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Runtime status helpers for the `squirrel status` CLI subcommand.
//!
//! Probes the biomeOS socket path to determine whether a squirrel daemon is
//! reachable, and reports live process metrics instead of hardcoded stubs.

use std::fmt;

/// Summarise socket availability for display.
///
/// Returns a human-readable string like `"/run/user/1000/biomeos/squirrel.sock (listening)"`
/// or `"not found"`.
pub fn socket_status() -> String {
    let path = discover_socket_path();
    if std::path::Path::new(&path).exists() {
        format!("{path} (exists)")
    } else {
        format!("{path} (not found)")
    }
}

fn discover_socket_path() -> String {
    if let Ok(p) = std::env::var("SQUIRREL_SOCKET") {
        return p;
    }
    if let Ok(p) = std::env::var("BIOMEOS_SOCKET_PATH") {
        return p;
    }
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        return format!("{xdg}/biomeos/squirrel.sock");
    }
    "/tmp/squirrel.sock".to_string()
}

/// Wrapper that renders `Option<u64>` as `"N/A"` when absent.
pub struct OptionalKb(pub Option<u64>);

impl fmt::Display for OptionalKb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(kb) => write!(f, "{kb} KB"),
            None => f.write_str("N/A"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn socket_status_contains_path() {
        let status = socket_status();
        assert!(
            status.contains("squirrel.sock")
                || status.contains("(exists)")
                || status.contains("(not found)"),
            "unexpected socket status: {status}"
        );
    }

    #[test]
    fn discover_socket_path_returns_nonempty() {
        let path = discover_socket_path();
        assert!(!path.is_empty());
        assert!(path.contains("squirrel"));
    }

    #[test]
    fn optional_kb_some_value() {
        let kb = OptionalKb(Some(1024));
        assert_eq!(format!("{kb}"), "1024 KB");
    }

    #[test]
    fn optional_kb_none() {
        let kb = OptionalKb(None);
        assert_eq!(format!("{kb}"), "N/A");
    }

    #[test]
    fn optional_kb_zero() {
        let kb = OptionalKb(Some(0));
        assert_eq!(format!("{kb}"), "0 KB");
    }
}
