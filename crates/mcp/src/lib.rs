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
pub use error::types::MCPError;
pub use error::Result;

pub use context_manager::Context;
/// Re-export commonly used security types.
pub use security::{SecurityManager, SecurityManagerImpl};
pub use types::{EncryptionFormat, SecurityLevel};

/// Adapter for MCP operations with dependency injection support.
pub mod adapter;
pub use adapter::{MCPAdapter, MCPInterface};

/// Re-export plugin interfaces for easier access.
pub use plugins::interfaces::{Plugin, PluginMetadata, PluginStatus, McpPlugin, PluginManagerInterface};
pub use plugins::lifecycle::{PluginLifecycleHook, CompositePluginLifecycleHook};
pub use plugins::adapter::{ToolPluginAdapter, ToolPluginFactory};
pub use plugins::discovery::{PluginProxyExecutor, PluginDiscoveryManager};

/// Re-export the main configuration type.
pub use config::McpConfig as MCPConfig;

/// Re-export factory functions for creating MCP instances.
pub use factory::{create_mcp, create_mcp_factory, MCPFactory};

/// Transport module for the MCP system
///
/// This module provides the Transport trait and implementations for different
/// transport mechanisms, including TCP, WebSocket, stdio, and in-memory transports.
pub mod transport;

/// Legacy transport module for the MCP system
///
/// This module contains the old implementation of transport mechanisms.
/// 
/// **Note**: This module is deprecated and will be removed in a future release.
/// Please use the new `transport` module instead.
///
/// This module is conditionally compiled with the `legacy-transport` feature,
/// which is enabled by default. To disable this module and remove all deprecated
/// code, disable the `legacy-transport` feature in your Cargo.toml:
///
/// ```toml
/// [dependencies]
/// mcp = { version = "0.2.0", default-features = false }
/// ```
#[cfg(feature = "legacy-transport")]
#[deprecated(
    since = "0.2.0",
    note = "Use the new transport module instead. Will be removed in a future release."
)]
pub mod transport_old;

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

/// Frame encoding/decoding for message transport
pub mod frame;

/// Client API for the MCP protocol
pub mod client;

/// Server API for the MCP protocol
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

#[cfg(test)]
mod tests;
