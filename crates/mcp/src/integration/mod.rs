//! Integration adapters for connecting MCP with other components
//! 
//! This module contains adapter implementations that connect MCP with various other
//! components in the system. These adapters follow the adapter pattern described in
//! the MCP Integration Guide.

/// Core system adapter for MCP integration
///
/// Provides integration between the MCP protocol and core system components.
/// This adapter handles message routing, state management, authentication,
/// and other core integration points between MCP and the main application.
pub mod core_adapter;

pub use core_adapter::CoreMCPAdapter;

/// Re-exports of key integration traits and types
pub mod prelude {
    pub use crate::protocol::MCPProtocol;
    pub use crate::integration::core_adapter::MessageHandler;
    pub use crate::types::{MCPMessage, MCPResponse};
    pub use crate::security::{Credentials, Permission, Action};
    pub use crate::error::{MCPError, Result as MCPResult};
    
    pub use super::core_adapter::CoreMCPAdapter;
} 