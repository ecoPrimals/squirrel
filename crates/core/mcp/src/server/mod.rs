//! MCP Server Module
//!
//! This module provides a high-level server API for the Machine Context Protocol.
//! It has been refactored for better maintainability with clear separation of concerns.
//!
//! # Module Organization
//!
//! - **config**: Server configuration types
//! - **handlers**: Command and connection handler traits
//! - **connection**: Client connection management
//! - **adapters**: Handler adapters for routing integration
//! - **core**: Main MCPServer implementation (in server.rs)
//!
//! # Examples
//!
//! ```no_run
//! use squirrel_mcp::server::{MCPServer, ServerConfig, CommandHandler};
//! use squirrel_mcp::message::{Message, MessageType};
//! use squirrel_mcp::error::Result;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Create server with default configuration
//!     let mut server = MCPServer::new(ServerConfig::default());
//!     
//!     // Start the server
//!     server.start().await?;
//!     
//!     // Run indefinitely
//!     tokio::signal::ctrl_c().await?;
//!     
//!     // Gracefully stop the server
//!     server.stop().await?;
//!     
//!     Ok(())
//! }
//! ```

mod config;
mod handlers;
mod connection;
mod adapters;

// Re-export public API
pub use config::{ServerConfig, ServerState};
pub use handlers::{CommandHandler, ConnectionHandler};
pub use connection::ClientConnection;
pub use adapters::{RouterCommandHandler, CommandHandlerAdapter};

// The core MCPServer implementation remains in the parent server.rs file
// This allows for a clean separation while maintaining backward compatibility
