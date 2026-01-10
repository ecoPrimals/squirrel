//! Unix Socket Utilities
//!
//! Helper functions for Unix socket management.

use std::path::Path;
use tracing::{info, warn};

/// Get the default socket path for a Squirrel instance
#[must_use]
pub fn get_socket_path(node_id: &str) -> String {
    format!("/tmp/squirrel-{node_id}.sock")
}

/// Get the node ID from environment or generate one
#[must_use]
pub fn get_node_id() -> String {
    std::env::var("SQUIRREL_NODE_ID").unwrap_or_else(|_| {
        warn!("SQUIRREL_NODE_ID not set, using hostname");
        hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "squirrel".to_string())
    })
}

/// Clean up socket file on shutdown
pub fn cleanup_socket(socket_path: &str) {
    if Path::new(socket_path).exists() {
        info!("🧹 Cleaning up socket: {}", socket_path);
        if let Err(e) = std::fs::remove_file(socket_path) {
            warn!("⚠️ Failed to remove socket: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_socket_path() {
        let path = get_socket_path("test-node");
        assert_eq!(path, "/tmp/squirrel-test-node.sock");
    }

    #[test]
    fn test_get_node_id_default() {
        // Clear env var for test
        std::env::remove_var("SQUIRREL_NODE_ID");

        let node_id = get_node_id();
        assert!(!node_id.is_empty());
    }

    #[test]
    fn test_get_node_id_from_env() {
        std::env::set_var("SQUIRREL_NODE_ID", "custom-node");

        let node_id = get_node_id();
        assert_eq!(node_id, "custom-node");

        std::env::remove_var("SQUIRREL_NODE_ID");
    }
}
