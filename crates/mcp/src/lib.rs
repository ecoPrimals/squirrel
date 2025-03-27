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
//! ### Basic Initialization
//!
//! ```
//! use mcp::{create_mcp, MCPConfig};
//!
//! // Create an MCP instance with default configuration
//! let mcp = create_mcp();
//!
//! // Initialize the MCP system
//! mcp.initialize().expect("Failed to initialize MCP");
//!
//! // Now you can use the MCP system
//! let context = mcp.create_context("session-123");
//! ```
//!
//! ### Configuring Security
//!
//! ```
//! use mcp::{MCPConfig, SecurityLevel, factory::MCPFactory};
//!
//! // Create a custom configuration with high security
//! let mut config = MCPConfig::default();
//! config.encryption_enabled = true;
//! config.security_level = SecurityLevel::High;
//!
//! // Create an MCP instance with the custom configuration
//! let factory = MCPFactory::with_config(config);
//! let mcp = factory.create_mcp();
//!
//! // Use the secure MCP instance
//! let security_manager = mcp.get_security_manager();
//! ```
//!
//! ### Working with Protocol Messages
//!
//! ```
//! use mcp::prelude::*;
//! use mcp::types::{MCPMessage, MessageId, MessageType};
//! use serde_json::json;
//!
//! // Creating and sending a command
//! let message = MCPMessage {
//!     id: MessageId("cmd-123".to_string()),
//!     message_type: MessageType::Command,
//!     payload: json!({
//!         "command": "read_file",
//!         "path": "/path/to/file.txt"
//!     }),
//! };
//!
//! // Using a protocol adapter to send the message
//! // (Assuming you have a protocol adapter instance)
//! // protocol_adapter.send_message(message).await.expect("Failed to send message");
//! ```
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
pub use error::{MCPError, Result};

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

/// Prelude module providing commonly used types and traits.
///
/// Import this module to quickly access the most commonly used types
/// and traits without having to import them individually.
pub mod prelude {
    // Core types and interfaces
    pub use crate::context_manager::{Context, ContextManager};
    
    // Protocol types
    pub use crate::types::MessageType;
    pub use crate::types::ProtocolState;
    pub use crate::protocol::MCPProtocolBase; 
    pub use crate::protocol::MCPProtocol;
    pub use crate::protocol::adapter::MCPProtocolAdapter;

    // Security features
    pub use crate::types::SecurityLevel;
    pub use crate::security::SecurityManager;
    
    // Tool management
    pub use crate::tool::{Tool, ToolManager, ToolState};
    pub use crate::tool::lifecycle::{LifecycleEvent, BasicLifecycleHook};
    
    // Monitoring system
    pub use crate::monitoring::MetricsCollector;
    pub use crate::monitoring::AlertManager;
    pub use crate::monitoring::MonitoringSystem;
    pub use crate::monitoring::alerts::{Alert, AlertSeverity, AlertState};
    
    // Error handling
    pub use crate::error::{MCPError, Result};
    
    // RBAC system
    pub use crate::security::rbac::{
        RBACManager, 
        ValidationResult, 
        ValidationRule, 
        InheritanceType,
        ValidationAuditRecord
    };
    pub use crate::security::{Action, Permission, PermissionContext, PermissionScope, Role};

    // Plugin integration
    pub use crate::plugins::{ToolPluginAdapter, ToolPluginFactory, PluginDiscoveryManager, PluginProxyExecutor};
    pub use crate::plugins::lifecycle::{PluginLifecycleHook, CompositePluginLifecycleHook};
    // Plugin interfaces
    pub use crate::plugins::interfaces::{Plugin, PluginMetadata, PluginStatus, McpPlugin, PluginManagerInterface};
}

pub mod transport;
pub mod transport_old;
pub mod registry;
pub mod session;
pub mod port;
pub mod integration;
pub mod resilience;

#[cfg(test)]
mod tests;
