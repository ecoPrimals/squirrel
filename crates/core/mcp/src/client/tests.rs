// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! MCP Client Tests
//!
//! Test suite for the MCP client module covering initialization and configuration.
//!
//! NOTE: Transport-level tests (connect, disconnect, send, reconnect, timeout,
//! concurrent requests, error handling, edge cases) require a mock transport
//! layer. These will be added when the transport abstraction supports test doubles.

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::time::Duration;

    /// Helper: Create test client with default configuration
    fn create_test_client() -> MCPClient {
        let config = ClientConfig::default();
        MCPClient::new(config)
    }

    // ========== Initialization Tests ==========

    #[test]
    fn test_client_creation() {
        let client = create_test_client();
        // Client should be created in disconnected state
        assert!(!client.is_connected());
    }

    #[test]
    fn test_client_with_custom_config() {
        let config = ClientConfig {
            timeout: Duration::from_secs(30),
            retry_attempts: 5,
            ..Default::default()
        };
        let client = MCPClient::new(config);
        assert!(!client.is_connected());
    }

    #[test]
    fn test_client_default_config() {
        let config = ClientConfig::default();
        // Default config should have reasonable values
        assert!(config.timeout.as_secs() > 0);
        assert!(config.retry_attempts > 0);
    }
}
