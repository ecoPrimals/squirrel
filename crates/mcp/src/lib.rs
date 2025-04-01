//! # Machine Context Protocol (MCP)
//!
//! The Machine Context Protocol (MCP) is a comprehensive system for secure communication
//! and context management between different components of the Squirrel platform. It provides
//! a standardized way for components to exchange messages, maintain state, and coordinate
//! operations in a distributed environment.
//!
//! ## Core Features
//!
//! * **Message-based communication**: Structured message format for command execution, event
//!   notification, and data exchange between components.
//!
//! * **Security and authentication**: Built-in security features including encryption,
//!   authentication, and role-based access control (RBAC).
//!
//! * **Context management**: Maintains context data across interactions, enabling stateful
//!   communication between components.
//!
//! * **Extensible plugin system**: Support for plugins that can extend the functionality
//!   of the MCP system.
//!
//! * **Tool management**: Framework for registering, discovering, and invoking tools
//!   across the system.
//!
//! * **Monitoring and metrics**: Built-in monitoring capabilities for tracking system
//!   health and performance.
//!
//! ## Architecture
//!
//! The MCP system is designed with the following architectural components:
//!
//! * **Protocol Layer**: Defines the message format and communication protocol
//!   (in the `protocol` module).
//!
//! * **Security Layer**: Handles authentication, authorization, and encryption
//!   (in the `security` module).
//!
//! * **Context Layer**: Manages state and context information
//!   (in the `context_manager` module).
//!
//! * **Tool Layer**: Provides mechanisms for tool registration and execution
//!   (in the `tool` module).
//!
//! * **Plugin Layer**: Enables extension of functionality through plugins
//!   (in the `plugins` module).
//!
//! * **Persistence Layer**: Manages data storage and retrieval
//!   (in the `persistence` module).
//!
//! * **Monitoring Layer**: Tracks system health and performance
//!   (in the `monitoring` module).
//!
//! ## Usage Examples
//!
//! // Examples have been temporarily removed for troubleshooting
//!
//! ## Module Organization
//!
//! This crate is organized into several key modules, each responsible for a specific
//! aspect of the MCP system:

// Temporarily allow warnings during cleanup
// Following the MCP_CLIPPY_CLEANUP_PLAN.md strategy
#![allow(clippy::all)]
#![allow(missing_docs)]
#![allow(missing_debug_implementations)]
#![allow(clippy::unused_async)]
#![allow(clippy::needless_pass_by_ref_mut)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::redundant_pub_crate)]
#![allow(clippy::missing_errors_doc)] // TODO: Add error documentation
#![allow(clippy::missing_panics_doc)] // TODO: Add panic documentation

#![allow(dead_code)] // Temporarily allow dead code during migration

/// MCP context manager for maintaining state across interactions.
///
/// The context manager provides mechanisms to create, retrieve, update,
/// and manage contexts that store state information for different sessions.
pub mod context_manager;

/// Error types and error handling capabilities for the MCP system.
///
/// This module defines the error types used throughout the MCP system,
/// along with utilities for error handling and propagation.
pub mod error;

/// Protocol implementation for message-based communication.
///
/// The protocol module defines the structure of messages, the flow of
/// communication, and mechanisms for handling different message types.
///
/// Example of using the Protocol trait:
///
/// ```no_run
/// use squirrel_mcp::protocol::types::MessageId;
/// use squirrel_mcp::protocol::types::MessageType;
/// use squirrel_mcp::protocol::types::MCPMessage;
/// use squirrel_mcp::security::types::SecurityMetadata;
/// use squirrel_mcp::protocol::types::ProtocolVersion;
/// use chrono::Utc;
/// use serde_json::json;
///
/// let message = MCPMessage {
///     id: MessageId::new(),
///     type_: MessageType::Command,
///     timestamp: Utc::now(),
///     payload: json!({"command": "status"}),
///     metadata: None,
///     security: SecurityMetadata::default(),
///     version: ProtocolVersion::new(1, 0),
///     trace_id: None,
/// };
///
/// assert!(!message.id.0.is_empty());
/// assert_eq!(message.type_, MessageType::Command);
/// assert!(!message.payload.is_null());
/// ```
pub mod protocol;

/// Tool management system for registering and executing tools.
///
/// This module provides a framework for defining, registering, discovering,
/// and invoking tools through the MCP system.
pub mod tool;

/// Monitoring and metrics collection for system health tracking.
///
/// The monitoring module provides mechanisms to collect metrics, track
/// system health, and generate alerts when issues are detected.
pub mod monitoring;

/// Security and authentication features for the MCP system.
///
/// This module handles authentication, authorization, encryption, and
/// role-based access control (RBAC) for the MCP system.
pub mod security;

/// Persistence layer for data storage and retrieval.
///
/// The persistence module provides interfaces and implementations for
/// storing and retrieving data in various storage systems.
pub mod persistence;

/// Synchronization primitives for concurrent operations.
///
/// This module provides utilities for safe concurrent access to shared
/// resources within the MCP system.
pub mod sync;

/// Common types used throughout the MCP system.
///
/// This module defines the core data structures and enumerations that
/// are used across different parts of the MCP system.
///
/// ```
/// use squirrel_mcp::protocol::types::{MCPMessage, MessageId, MessageType};
/// use squirrel_mcp::protocol::types::ProtocolVersion;
/// use squirrel_mcp::security::types::SecurityMetadata;
/// use serde_json::json;
/// use chrono::Utc;
/// 
/// // Create a message
/// let message = MCPMessage {
///     id: MessageId::new(),
///     type_: MessageType::Command,
///     payload: json!({"command": "status"}),
///     metadata: Some(json!({})),
///     security: SecurityMetadata::default(),
///     timestamp: Utc::now(),
///     version: ProtocolVersion::new(1, 0),
///     trace_id: Some("trace-1".to_string()),
/// };
/// 
/// // Verify properties
/// assert!(!message.id.0.is_empty());
/// assert_eq!(message.type_, MessageType::Command);
/// assert!(message.timestamp <= Utc::now());
/// ```
pub mod types;

/// Configuration management for the MCP system.
///
/// This module defines structures and utilities for configuring the
/// behavior of the MCP system and its components.
pub mod config;

/// Plugin system for extending MCP functionality.
///
/// This module provides mechanisms for defining, registering, discovering,
/// and loading plugins that extend the capabilities of the MCP system.
pub mod plugins;

/// MCP factory for creating and configuring MCP instances.
///
/// This module provides factory functions and types for creating
/// MCP instances with specific configurations.
pub mod factory;

/// Re-export common types from the error module for easier access.
pub use error::{ErrorContext};
pub use error::MCPError;
pub use error::Result;

pub use context_manager::Context;
/// Re-export commonly used security types.
pub use security::{SecurityManager, SecurityManagerImpl};
pub use types::{AccountId};
pub use protocol::types::{MCPMessage, MessageType, ProtocolVersion};
pub use security::types::{AuthCredentials, SecurityLevel, UserId};
pub use security::types::{EncryptionFormat};
pub use security::token::{Token, SessionToken, AuthToken};
// Comment out the Permission import for now until we resolve the circular dependencies
// pub use protocol::security::auth::Permission;

/// Adapter for MCP operations with dependency injection support.
///
/// ```no_run
/// use squirrel_mcp::{adapter::MCPInterface, MCPAdapter, MCPConfig};
/// use std::sync::Arc;
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a configuration
///     let config = MCPConfig::default();
///     
///     // Create an adapter
///     let adapter = MCPAdapter::new(config);
///     
///     // Initialize the adapter
///     adapter.initialize()?;
///     
///     // Check if initialized
///     assert!(adapter.is_initialized());
///     
///     Ok(())
/// }
/// ```
pub mod adapter;
pub use adapter::{MCPAdapter, MCPInterface};

/// Re-export plugin interfaces for easier access.
pub use plugins::interfaces::{Plugin, PluginMetadata, PluginStatus, McpPlugin, PluginManagerInterface};
pub use plugins::lifecycle::{PluginLifecycleHook, CompositePluginLifecycleHook};
pub use plugins::adapter::{ToolPluginAdapter, ToolPluginFactory};
pub use plugins::discovery::{PluginProxyExecutor, PluginDiscoveryManager};

/// Re-export the main configuration type.
pub use config::McpConfig;
/// Alias for backward compatibility 
pub use config::McpConfig as MCPConfig;

/// Re-export factory functions for creating MCP instances.
pub use factory::{create_mcp, create_mcp_factory, MCPFactory};

/// Transport module for the MCP system
///
/// This module provides the Transport trait and implementations for different
/// transport mechanisms, including TCP, WebSocket, stdio, and in-memory transports.
pub mod transport;

/// Registry module for the MCP system.
///
/// This module provides mechanisms for registering and discovering
/// components within the MCP system.
pub mod registry;

/// Session management module for the MCP system.
///
/// This module handles session creation, tracking, and cleanup
/// for MCP communication sessions.
pub mod session;

/// Port management module for the MCP system.
///
/// This module provides interfaces and implementations for managing
/// communication ports and connections.
pub mod port;

/// Integration module for the MCP system.
///
/// This module provides utilities for integrating the MCP system
/// with other components and systems.
pub mod integration;

/// Resilience module for the MCP system.
///
/// This module provides mechanisms for handling failures, retries,
/// and circuit breaking within the MCP system.
pub mod resilience;

/// Compression module for the MCP system.
pub mod compression;

/// Message format and builder for the MCP protocol
pub mod message;

/// Message router and handler functionality
pub mod message_router;

/// Frame handling for the MCP protocol.
///
/// Example of using Frame for the MCP protocol:
///
/// ```
/// use bytes::BytesMut;
/// use squirrel_mcp::frame::Frame;
///
/// // Create a frame
/// let message_bytes = b"Hello, MCP!".to_vec();
/// let frame = Frame::from_vec(message_bytes);
/// 
/// // Get the payload
/// let payload = frame.payload();
/// assert_eq!(payload.len(), b"Hello, MCP!".len());
/// ```
pub mod frame;

/// Client API for the MCP protocol
///
/// The client module provides the ability to establish connections to MCP servers,
/// send commands, receive responses, and handle events.
///
/// # Example
///
/// ```no_run
/// use squirrel_mcp::client::{MCPClient, ClientConfig};
/// use squirrel_mcp::message::{Message, MessageType};
/// use serde_json::json;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a client with default configuration
///     let mut client = MCPClient::new(ClientConfig::default());
///     
///     // Connect to the server
///     client.connect().await?;
///     
///     // Create a command message
///     let command = Message::request(
///         "status".to_string(),
///         "client".to_string(), 
///         "server".to_string()
///     );
///     
///     // Send the command and get a response
///     let response = client.send_command(&command).await?;
///     println!("Response: {}", response.content);
///     
///     // Disconnect when done
///     client.disconnect().await?;
///     
///     Ok(())
/// # }
/// ```
///
/// ## Event subscription example
///
/// ```no_run
/// use squirrel_mcp::client::{MCPClient, ClientConfig};
/// use squirrel_mcp::message::{Message, MessageType};
/// use serde_json::json;
/// use tokio::sync::broadcast;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create client with default configuration
///     let mut client = MCPClient::new(ClientConfig::default());
///     
///     // Connect to server
///     client.connect().await?;
///     
///     // Subscribe to events
///     let mut events = client.subscribe_to_events().await;
///     
///     // Handle events in a separate task
///     tokio::spawn(async move {
///         loop {
///             match events.recv().await {
///                 Ok(Some(message)) => {
///                     println!("Received event: {:?}", message);
///                 },
///                 Ok(None) => {
///                     break;
///                 },
///                 Err(tokio::sync::broadcast::error::RecvError::Closed) => {
///                     println!("Channel closed");
///                     break;
///                 },
///                 Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
///                     println!("Lagged behind");
///                     continue;
///                 }
///             }
///         }
///     });
///     
///     // Publish an event
///     client.send_event_with_content(
///         "status_changed",
///         json!({
///             "new_status": "running"
///         })
///     ).await?;
///     
///     // Disconnect when done
///     client.disconnect().await?;
///     
///     Ok(())
/// # }
/// ```
pub mod client;

/// Server API for the MCP protocol
///
/// Example of implementing a server with a command handler:
///
/// ```no_run
/// use squirrel_mcp::server::{MCPServer, ServerConfig, CommandHandler};
/// use squirrel_mcp::message::{Message, MessageType};
/// use squirrel_mcp::error::Result;
/// use std::sync::Arc;
///
/// #[derive(Debug, Clone)]
/// struct StatusCommandHandler;
///
/// impl CommandHandler for StatusCommandHandler {
///     fn handle_command<'a>(
///         &'a self, 
///         message: &'a Message
///     ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<Message>>> + Send + 'a>> {
///         Box::pin(async move {
///             // Handle the command and return a response
///             let response = Message::new(
///                 MessageType::Response,
///                 "Status: OK".to_string(),
///                 message.destination.clone(),
///                 message.source.clone()
///             );
///             
///             Ok(Some(response))
///         })
///     }
///
///     fn supported_commands(&self) -> Vec<String> {
///         vec!["status".to_string()]
///     }
///
///     fn clone_box(&self) -> Box<dyn CommandHandler> {
///         Box::new(self.clone())
///     }
/// }
///
/// async fn start_server() -> Result<()> {
///     let config = ServerConfig::default();
///     let mut server = MCPServer::new(config);
///     
///     // Register command handler
///     let handler = Box::new(StatusCommandHandler);
///     server.register_command_handler(handler).await?;
///     
///     // Start the server
///     server.start().await?;
///     Ok(())
/// }
/// ```
pub mod server;

/// Logging module for the MCP system.
///
/// This module provides structured logging capabilities for MCP components,
/// with support for various log levels and contextual information.
pub mod logging;

/// Metrics module for the MCP system.
///
/// This module provides metrics collection and reporting capabilities
/// for monitoring MCP component performance and behavior.
pub mod metrics;

/// Debug trait implementations for MCP types.
///
/// This module provides implementations of the Debug trait for types
/// in the MCP codebase that don't derive Debug automatically.
pub mod debug_impl;

/// MCP context manager for maintaining state across interactions.
///
/// The context manager provides mechanisms to create, retrieve, update,
/// and manage contexts that store state information for different sessions.
pub use context_manager::ContextManager;

pub use crate::protocol::{MCPProtocol};
pub use crate::integration::adapter::CoreMCPAdapter;
pub use crate::config::McpConfig as CoreAdapterConfig;

// Only export core types once - these were previously duplicated
pub use types::{MCPResponse, ResponseStatus}; // Re-export core types
