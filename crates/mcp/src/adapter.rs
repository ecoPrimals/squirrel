//! MCP Adapter module for dependency injection

use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use squirrel_core::error::SquirrelError;
use crate::config::McpConfig as MCPConfig;

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

/// Adapter for the MCP system
pub struct MCPAdapter {
    /// The inner MCP instance
    mcp: Arc<RwLock<Option<Arc<dyn MCPInterface + Send + Sync>>>>,
    /// Mutex to ensure thread-safe initialization
    init_mutex: RwLock<()>,
}

impl MCPAdapter {
    /// Create a new MCPAdapter
    pub fn new(config: MCPConfig) -> Self {
        Self {
            mcp: Arc::new(RwLock::new(None)),
            init_mutex: RwLock::new(()),
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
        // Since this needs to be synchronous, we'll use block_in_place to execute the async code
        tokio::task::block_in_place(|| {
            let runtime = tokio::runtime::Handle::current();
            runtime.block_on(async {
                let _lock = self.init_mutex.read().await;
                
                // Check if an implementation already exists
                let impl_exists = {
                    let reader = self.mcp.read().await;
                    reader.is_some()
                };
                
                if impl_exists {
                    return Ok(());
                }
                
                // Initialize MCP implementation code here...
                Ok(())
            })
        })
    }
    
    fn is_initialized(&self) -> bool {
        tokio::task::block_in_place(|| {
            let runtime = tokio::runtime::Handle::current();
            runtime.block_on(async {
                let reader = self.mcp.read().await;
                reader.is_some()
            })
        })
    }
    
    fn get_config(&self) -> Result<MCPConfig, SquirrelError> {
        tokio::task::block_in_place(|| {
            let runtime = tokio::runtime::Handle::current();
            runtime.block_on(async {
                let reader = self.mcp.read().await;
                match &*reader {
                    Some(mcp) => Ok(mcp.get_config().clone()),
                    None => Err(SquirrelError::mcp(
                        "MCP is not initialized. Call initialize() first."
                    ))
                }
            })
        })
    }
    
    fn send_message(&self, message: &str) -> Result<String, SquirrelError> {
        tokio::task::block_in_place(|| {
            let runtime = tokio::runtime::Handle::current();
            runtime.block_on(async {
                let reader = self.mcp.read().await;
                match &*reader {
                    Some(mcp) => mcp.send_message(message).map_err(|e| e.into()),
                    None => Err(SquirrelError::mcp(
                        "MCP is not initialized. Call initialize() first."
                    ))
                }
            })
        })
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