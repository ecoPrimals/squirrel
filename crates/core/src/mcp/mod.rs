//! MCP (Machine Context Protocol) module
//!
//! This module implements the core functionality for the Machine Context Protocol.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

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
pub mod tests {
    // Enable adapter tests
    pub mod adapter;
    // Temporarily comment out test modules until we fix them
    // pub mod refactored_test;
}

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConfig {
    /// The protocol version
    pub version: String,
    /// Maximum message size in bytes
    pub max_message_size: u64,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
}

/// The main MCP system controller
#[derive(Debug)]
pub struct MCP {
    /// The MCP configuration, wrapped in a thread-safe read-write lock
    config: Arc<RwLock<MCPConfig>>,
    /// Protocol adapter for handling MCP messages
    protocol: Arc<protocol::MCPProtocolAdapter>,
    /// Context adapter for interacting with the context system
    context_adapter: Arc<context_adapter::MCPContextAdapter>,
    /// Sync component for state synchronization
    sync: sync::MCPSync,
    /// Flag to track initialization state
    initialized: bool,
}

impl MCP {
    /// Creates a new MCP instance with the specified configuration
    #[must_use]
    pub fn new(config: MCPConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            protocol: Arc::new(protocol::MCPProtocolAdapter::new()),
            context_adapter: Arc::new(context_adapter::MCPContextAdapter::new()),
            sync: sync::MCPSync::default(),
            initialized: false,
        }
    }

    /// Creates a new MCP instance with the specified components
    #[must_use]
    pub fn new_with_components(
        protocol: Arc<protocol::MCPProtocolAdapter>,
        context_adapter: Arc<context_adapter::MCPContextAdapter>,
        sync: sync::MCPSync,
    ) -> Self {
        Self {
            config: Arc::new(RwLock::new(MCPConfig::default())),
            protocol,
            context_adapter,
            sync,
            initialized: true,
        }
    }

    /// Initialize the MCP instance
    ///
    /// # Errors
    ///
    /// Returns an error if initialization of any component fails
    pub async fn init(&mut self) -> error::Result<()> {
        if self.initialized {
            return Ok(());
        }

        // Initialize protocol adapter
        if !self.protocol.is_initialized() {
            self.protocol.initialize().await?;
        }

        // Initialize context adapter
        if !self.context_adapter.is_initialized() {
            self.context_adapter.initialize()?;
        }

        // Initialize sync (if not already initialized)
        self.sync.init().await?;

        self.initialized = true;
        Ok(())
    }

    /// Gets the current configuration
    ///
    /// # Errors
    ///
    /// Returns a `MCPError::LockError` if the configuration lock cannot be acquired
    pub async fn get_config(&self) -> error::Result<MCPConfig> {
        if !self.initialized {
            return Err(MCPError::NotInitialized("MCP not initialized".into()));
        }
        
        let config = self.config.read().await;
        Ok(config.clone())
    }

    /// Gets a reference to the protocol adapter
    ///
    /// # Errors
    ///
    /// Returns a `MCPError::NotInitialized` if the MCP instance is not initialized
    pub fn get_protocol_adapter(&self) -> error::Result<Arc<protocol::MCPProtocolAdapter>> {
        if !self.initialized {
            return Err(MCPError::NotInitialized("MCP not initialized".into()));
        }
        
        Ok(self.protocol.clone())
    }

    /// Gets a reference to the context adapter
    ///
    /// # Errors
    ///
    /// Returns a `MCPError::NotInitialized` if the MCP instance is not initialized
    pub fn get_context_adapter(&self) -> error::Result<Arc<context_adapter::MCPContextAdapter>> {
        if !self.initialized {
            return Err(MCPError::NotInitialized("MCP not initialized".into()));
        }
        
        Ok(self.context_adapter.clone())
    }

    /// Gets a reference to the sync component
    ///
    /// # Errors
    ///
    /// Returns a `MCPError::NotInitialized` if the MCP instance is not initialized
    pub fn get_sync(&self) -> error::Result<&sync::MCPSync> {
        if !self.initialized {
            return Err(MCPError::NotInitialized("MCP not initialized".into()));
        }
        
        Ok(&self.sync)
    }
}

impl Default for MCPConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            max_message_size: 1024 * 1024 * 10, // 10MB
            timeout_ms: 30000, // 30 seconds
        }
    }
}

impl Default for MCP {
    fn default() -> Self {
        Self::new(MCPConfig::default())
    }
} 