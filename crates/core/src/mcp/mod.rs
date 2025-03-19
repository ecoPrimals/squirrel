//! # MCP (Machine Context Protocol) Module
//!
//! This module implements the core functionality for the Machine Context Protocol,
//! providing message handling, state synchronization, and security features.
//!
//! ## Core Functionality
//!
//! - **Message Handling**: Process and route protocol messages between components
//! - **State Synchronization**: Maintain consistent state across distributed systems
//! - **Security**: Implement encryption, authentication, and authorization
//! - **Session Management**: Handle client sessions and connection state
//! - **Persistence**: Store and retrieve protocol state
//!
//! ## Architecture
//!
//! The MCP module follows a layered architecture:
//!
//! 1. **Transport Layer**: Handles raw message transmission
//! 2. **Protocol Layer**: Implements the MCP protocol specification
//! 3. **Security Layer**: Manages encryption and authentication
//! 4. **Context Layer**: Connects to the application context system
//!
//! ## Dependencies
//!
//! MCP relies on the context system for state management and integrates with
//! the monitoring system for performance metrics and health status.

use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};

pub mod types;
pub mod security;
pub mod protocol;
/// Persistence for MCP state and operations
pub mod persistence;
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
    ProtocolVersion,
};

// Re-export error types
pub use error::types::{SecurityError, ProtocolError};
pub use error::types::MCPError;
pub use error::Result;

// Re-export protocols
pub use protocol::{MCPProtocol, MCPProtocolAdapter};

// Re-export security
pub use security::{SecurityConfig, SecurityManager, Credentials};

// Re-export context adapter
pub use context_adapter::{MCPContextAdapter, create_mcp_context_adapter};

// Re-export factory
pub use factory::{MCPFactory, create_mcp_factory, create_mcp};

// Re-export session for backward compatibility
pub use session::Session as MCPSession;

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
    pub initialized: AtomicBool,
    /// The configuration of the MCP system
    pub config: MCPConfig,
}

impl MCPState {
    /// Create a new `MCPState` with the given configuration
    #[must_use] pub fn new(config: MCPConfig) -> Self {
        Self {
            initialized: AtomicBool::new(false),
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
    #[must_use] pub fn new(config: MCPConfig) -> Self {
        Self {
            state: RwLock::new(MCPState::new(config)),
        }
    }

    /// Initialize the MCP system
    ///
    /// # Errors
    /// Returns an error if the MCP system is already initialized or if the RwLock is poisoned
    ///
    /// # Panics
    /// This function no longer panics and properly handles all error cases
    pub fn initialize(&self) -> Result<()> {
        let state = self.state.write()
            .map_err(|e| MCPError::State(format!("RwLock poisoned: {}", e)))?;
            
        if state.initialized.load(Ordering::SeqCst) {
            return Err(MCPError::AlreadyInitialized("MCP already initialized".into()));
        }
        
        // Perform initialization tasks
        state.initialized.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Check if the MCP system is initialized
    ///
    /// # Returns
    /// `true` if the MCP system is initialized, `false` otherwise
    /// 
    /// # Errors
    /// This function now returns a Result to handle RwLock poisoning errors properly
    pub fn is_initialized(&self) -> Result<bool> {
        self.state.read()
            .map_err(|e| MCPError::State(format!("RwLock poisoned: {}", e)))
            .map(|state| state.initialized.load(Ordering::SeqCst))
    }
    
    /// Get the MCP configuration
    ///
    /// # Returns
    /// The current MCP configuration
    ///
    /// # Errors
    /// Returns an error if the MCP system is not initialized or if the RwLock is poisoned
    pub fn get_config(&self) -> Result<MCPConfig> {
        let state = self.state.read()
            .map_err(|e| MCPError::State(format!("RwLock poisoned: {}", e)))?;
            
        if !state.initialized.load(Ordering::SeqCst) {
            return Err(MCPError::NotInitialized("MCP not initialized".into()));
        }
        
        Ok(state.config.clone())
    }

    /// Send a message through the MCP system
    ///
    /// # Arguments
    /// * `message` - The message to send
    ///
    /// # Returns
    /// A `Result` containing the response message
    ///
    /// # Errors
    /// Returns an error if the MCP system is not initialized or if the RwLock is poisoned
    pub fn send_message(&self, message: &str) -> Result<String> {
        let state = self.state.read()
            .map_err(|e| MCPError::State(format!("RwLock poisoned: {}", e)))?;
            
        if !state.initialized.load(Ordering::SeqCst) {
            return Err(MCPError::NotInitialized("MCP not initialized".into()));
        }
        
        // In a real implementation, we would process the message here
        // For now, we just echo it back
        Ok(format!("Processed: {message}"))
    }
} 
