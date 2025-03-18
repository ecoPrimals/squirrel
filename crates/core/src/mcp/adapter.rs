//! MCP Adapter module for dependency injection

use std::sync::{Arc, Mutex};
use crate::error::SquirrelError;
use crate::mcp::{MCP, MCPConfig};

/// Interface for the MCP system
pub trait MCPInterface {
    /// Initialize the MCP system
    fn initialize(&self) -> Result<(), SquirrelError>;
    
    /// Check if the MCP system is initialized
    fn is_initialized(&self) -> bool;
    
    /// Get the MCP configuration
    fn get_config(&self) -> Result<MCPConfig, SquirrelError>;
    
    /// Send a message through the MCP system
    fn send_message(&self, message: &str) -> Result<String, SquirrelError>;
}

/// Adapter for the MCP struct
pub struct MCPAdapter {
    /// The inner MCP instance
    mcp: Arc<MCP>,
    /// Mutex to ensure thread-safe initialization
    init_mutex: Mutex<()>,
}

impl MCPAdapter {
    /// Create a new MCPAdapter
    pub fn new(config: MCPConfig) -> Self {
        Self {
            mcp: Arc::new(MCP::new(config)),
            init_mutex: Mutex::new(()),
        }
    }
    
    /// Create a new MCPAdapter that is already initialized
    pub fn new_initialized(config: MCPConfig) -> Result<Self, SquirrelError> {
        let adapter = Self::new(config);
        adapter.initialize()?;
        Ok(adapter)
    }
}

impl MCPInterface for MCPAdapter {
    fn initialize(&self) -> Result<(), SquirrelError> {
        let _lock = self.init_mutex.lock().unwrap();
        self.mcp.initialize().map_err(Into::into)
    }
    
    fn is_initialized(&self) -> bool {
        self.mcp.is_initialized()
    }
    
    fn get_config(&self) -> Result<MCPConfig, SquirrelError> {
        self.mcp.get_config().map_err(Into::into)
    }
    
    fn send_message(&self, message: &str) -> Result<String, SquirrelError> {
        self.mcp.send_message(message).map_err(Into::into)
    }
}

/// Create a new MCPAdapter
pub fn create_mcp_adapter(config: MCPConfig) -> MCPAdapter {
    MCPAdapter::new(config)
}

/// Create a new MCPAdapter with default configuration
pub fn create_default_mcp_adapter() -> MCPAdapter {
    MCPAdapter::new(MCPConfig::default())
}

/// Create a new MCPAdapter that is already initialized
pub fn create_initialized_mcp_adapter(config: MCPConfig) -> Result<MCPAdapter, SquirrelError> {
    MCPAdapter::new_initialized(config)
}

/// Create a new MCPAdapter with default configuration that is already initialized
pub fn create_default_initialized_mcp_adapter() -> Result<MCPAdapter, SquirrelError> {
    MCPAdapter::new_initialized(MCPConfig::default())
} 