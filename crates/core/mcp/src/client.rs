//! MCP Client API
//!
//! This module provides a high-level client API for interacting with the Machine Context Protocol.
//! The client has been split into focused modules for better organization and maintainability.
//!
//! ## Architecture
//!
//! The client is composed of several focused modules:
//!
//! * **config**: Client configuration management
//! * **connection**: Connection state and transport management
//! * **event**: Event handling and subscription
//! * **session**: Session management and message processing
//!
//! ## Usage
//!
//! ```rust,no_run
//! use squirrel_mcp::client::{MCPClient, ClientConfig};
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create and configure client
//!     let mut client = MCPClient::new(
//!         ClientConfig::new()
//!             .with_server_address("localhost:9000")
//!             .with_request_timeout(std::time::Duration::from_secs(30))
//!     );
//!     
//!     // Connect to server
//!     client.connect().await?;
//!     
//!     // Send commands and handle responses
//!     let response = client.send_command_with_content(
//!         "get_status",
//!         json!({"detail_level": "full"})
//!     ).await?;
//!     
//!     // Subscribe to events
//!     let mut events = client.subscribe_to_events().await;
//!     
//!     // Clean disconnect
//!     client.disconnect().await?;
//!     
//!     Ok(())
//! }
//! ```

// Re-export all client functionality from the client module
pub use self::client::*;

mod client; 