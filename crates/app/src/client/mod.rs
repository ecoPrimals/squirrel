//! Application API client module
//!
//! This module provides API client functionality for the Squirrel application.

use std::sync::Arc;

/// Client for communicating with the MCP API
#[derive(Debug)]
pub struct McpClient {
    /// The API endpoint URL
    api_url: String,
    // More implementation details will be added as needed
}

impl McpClient {
    /// Creates a new MCP client
    pub fn new() -> Self {
        Self {
            api_url: "http://localhost:8080".to_string(),
        }
    }
    
    /// Get a shared reference to this client
    pub fn as_shared(&self) -> Arc<Self> {
        Arc::new(Self::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_client_creation() {
        let client = McpClient::new();
        assert!(std::mem::size_of_val(&client) > 0);
    }
} 