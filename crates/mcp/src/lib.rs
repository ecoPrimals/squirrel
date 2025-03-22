//! Machine Context Protocol (MCP) implementation for Squirrel
//!
//! This crate provides the implementation of the Machine Context Protocol,
//! a system for secure communication and context management between systems.

#![allow(dead_code)] // Temporarily allow dead code during migration

/// MCP context manager
pub mod context_manager;

/// Error types and error handling
pub mod error;

/// Protocol-related functionality
pub mod protocol;

/// Tool management system
pub mod tool;

/// Monitoring and metrics
pub mod monitoring;

/// Security and authentication
pub mod security;

/// Persistence layer
pub mod persistence;

/// Synchronization
pub mod sync;

/// Common types
pub mod types;

/// Configuration module
pub mod config;

/// Re-export common types from the error module
pub use error::{MCPError, Result};

pub use context_manager::Context;
/// Re-export commonly used types
pub use protocol::ProtocolConfig;
pub use security::{Credentials, SecurityManager, Session};
pub use types::{EncryptionFormat, SecurityLevel};

/// Adapter for MCP operations
pub mod adapter;
pub use adapter::{MCPAdapter, MCPInterface};

/// Re-export the configuration type
pub use config::McpConfig as MCPConfig;

/// Re-export commonly used types from modules for convenience
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
    pub use crate::security::rbac::{RBACManager, Role, Permission, PermissionScope, Action};
}

#[cfg(test)]
mod tests;
