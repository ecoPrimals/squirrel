//! MCP (Machine Context Protocol) module for Squirrel
//!
//! This module implements the core functionality for the Machine Context Protocol,
//! providing message handling, state synchronization, and security features.

use std::sync::{Arc, Mutex, RwLock};
use crate::error::{AppInitializationError, AppOperationError, SquirrelError};

pub mod types;
pub mod security;
pub mod protocol;
/// State synchronization for MCP
pub mod sync;
/// Context manager for storing and retrieving context data
pub mod context_manager;
/// Context adapter for connecting to the general context system
pub mod context_adapter;
/// Error handling and management for MCP operations
#[allow(missing_docs, unused_imports, unused_variables, dead_code)]
pub mod error;
/// Monitoring for MCP operations
pub mod monitoring;
/// Factory for creating MCP components
pub mod factory;
/// Persistence for MCP state data
pub mod persistence;
/// Session management for MCP
pub mod session;

#[cfg(test)]
pub mod tests;

#[cfg(feature = "di-tests")]
/// MCP adapter for dependency injection
pub mod adapter;

#[cfg(feature = "di-tests")]
pub use adapter::MCPAdapter;

// Re-export common types for easy access
pub use types::{
    SecurityLevel,
    EncryptionFormat,
    MessageType,
    CompressionFormat,
    MessageMetadata,
    ResponseStatus,
    MCPMessage,
    MCPResponse,
    MCPCommand,
};

// Re-export error types
pub use error::types::{MCPError, SecurityError, ProtocolError};
pub use error::Result;

// Re-export protocols
pub use protocol::{MCPProtocol, MCPProtocolAdapter};

// Re-export security
pub use security::{SecurityConfig, SecurityManager, Credentials};

// Re-export context adapter
pub use context_adapter::{MCPContextAdapter, create_mcp_context_adapter};

// Re-export factory
pub use factory::{MCPFactory, create_mcp_factory, create_mcp};

/// Configuration for the MCP system
#[derive(Debug, Clone)]
pub struct MCPConfig {
    /// The protocol version
    pub version: String,
    /// Maximum message size in bytes
    pub max_message_size: u64,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
    /// Enable encryption
    pub encryption_enabled: bool,
}

impl Default for MCPConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            max_message_size: 1024 * 1024, // 1MB
            timeout_ms: 5000, // 5 seconds
            encryption_enabled: true,
        }
    }
}

/// Core MCP state
#[derive(Debug)]
pub struct MCPState {
    /// Whether the MCP system is initialized
    pub initialized: bool,
    /// The configuration of the MCP system
    pub config: MCPConfig,
}

impl MCPState {
    /// Create a new MCPState with the given configuration
    pub fn new(config: MCPConfig) -> Self {
        Self {
            initialized: false,
            config,
        }
    }
}

/// The main MCP system controller
pub struct MCP {
    /// The MCP state
    state: RwLock<MCPState>,
}

impl MCP {
    /// Create a new MCP with the given configuration
    pub fn new(config: MCPConfig) -> Self {
        Self {
            state: RwLock::new(MCPState::new(config)),
        }
    }

    /// Initialize the MCP system
    pub fn initialize(&self) -> Result<(), AppInitializationError> {
        let mut state = self.state.write().unwrap();
        if state.initialized {
            return Err(AppInitializationError::AlreadyInitialized);
        }
        
        // Perform initialization tasks
        state.initialized = true;
        Ok(())
    }

    /// Check if the MCP system is initialized
    pub fn is_initialized(&self) -> bool {
        self.state.read().unwrap().initialized
    }
    
    /// Get the MCP configuration
    pub fn get_config(&self) -> Result<MCPConfig, AppOperationError> {
        let state = self.state.read().unwrap();
        if !state.initialized {
            return Err(AppOperationError::NotInitialized);
        }
        
        Ok(state.config.clone())
    }

    /// Send a message through the MCP system
    pub fn send_message(&self, message: &str) -> Result<String, AppOperationError> {
        let state = self.state.read().unwrap();
        if !state.initialized {
            return Err(AppOperationError::NotInitialized);
        }
        
        // In a real implementation, we would process the message here
        // For now, we just echo it back
        Ok(format!("Processed: {}", message))
    }
} 